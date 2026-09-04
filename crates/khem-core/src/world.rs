//! World state and core data types: `AtomId`, `BondId`, `ElementId`,
//! `AtomState`, `BondState`, `WorldState`, `Grid2D`, `BoundaryType`.
//!
//! Design rules (runtime spec section 4): flat arrays indexed by
//! integer ID - no per-atom heap allocation, no pointers,
//! cache-friendly, partitionable by region for V2/V3 scaling.
//!
//! Spec: docs/specs/runtime-spec.md, section 4 (core data
//! structures).

use std::sync::Arc;

use crate::config::PhysicsConfig;
use crate::elements::{ELEMENTS, ElementProperties};
use crate::energy::EnergySource;
use crate::observer::Event;
use crate::rng::Rng;
use crate::spatial::SpatialIndex;

/// Maximum bonds any element can hold. Covers every element's
/// `max_bonds` (runtime spec section 4.4) and is the fixed width of
/// [`AtomState::bonds`].
pub const MAX_BONDS: usize = 6;

/// Identifies an atom: an index into [`WorldState::atoms`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtomId(pub u32);

/// Identifies a bond: an index into [`WorldState::bonds`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BondId(pub u32);

/// Identifies an element: an index into the element table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(pub u8);

/// One atom. Fixed size, no heap allocation (runtime spec 4.4).
///
/// Dead atoms are flagged (`alive = false`) and compacted periodically
/// by the tick loop, never removed immediately - removal would
/// invalidate IDs.
///
/// The bond slots are `Option` rather than the spec's raw
/// `[BondId; 6]`: an empty slot must be representable and a sentinel
/// id would be unidiomatic. Revise the spec wording at phase-1
/// review if this sticks.
#[derive(Debug, Clone)]
pub struct AtomState {
    pub id: AtomId,
    pub element: ElementId,
    /// Position, angstroms.
    pub x: f32,
    pub y: f32,
    /// Velocity, angstroms per tick.
    pub vx: f32,
    pub vy: f32,
    /// Currently held bonds; only the first `bond_count` slots are
    /// valid.
    pub bonds: [Option<BondId>; MAX_BONDS],
    pub bond_count: u8,
    pub alive: bool,
}

impl AtomState {
    /// A new live atom at rest.
    pub fn new(id: AtomId, element: ElementId, x: f32, y: f32) -> Self {
        Self {
            id,
            element,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            bonds: [None; MAX_BONDS],
            bond_count: 0,
            alive: true,
        }
    }

    /// Adds a bond slot. Returns false (changing nothing) if the atom
    /// already holds [`MAX_BONDS`] bonds; callers enforce the tighter
    /// per-element `max_bonds` limit (see [`WorldState::form_bond`]).
    pub fn push_bond(&mut self, bond: BondId) -> bool {
        if self.bond_count as usize >= MAX_BONDS {
            return false;
        }
        self.bonds[self.bond_count as usize] = Some(bond);
        self.bond_count += 1;
        true
    }

    /// Removes a bond slot, keeping the valid slots contiguous.
    /// Returns whether the atom held it.
    pub fn remove_bond(&mut self, bond: BondId) -> bool {
        let Some(i) = self.bonds.iter().position(|b| *b == Some(bond)) else {
            return false;
        };
        let count = self.bond_count as usize;
        self.bonds.copy_within(i + 1..count, i);
        self.bonds[count - 1] = None;
        self.bond_count -= 1;
        true
    }
}

/// One bond (runtime spec 4.5).
#[derive(Debug, Clone)]
pub struct BondState {
    pub id: BondId,
    pub atom_a: AtomId,
    pub atom_b: AtomId,
    /// Bond order: 1 single, 2 double, 3 triple.
    pub order: u8,
    pub alive: bool,
    /// Stored chemical energy, kJ/mol.
    pub energy: f32,
}

impl BondState {
    pub fn new(id: BondId, atom_a: AtomId, atom_b: AtomId, order: u8, energy: f32) -> Self {
        Self {
            id,
            atom_a,
            atom_b,
            order,
            alive: true,
            energy,
        }
    }
}

/// Field values (temperature, pressure, UV) on a grid coarser than
/// atom positions (runtime spec 4.8). Flat index = col + row * cols.
///
/// Indices wrap, like the Wrap boundary: the tick order normalizes
/// positions (boundary step) before any field access, so wrapping is
/// safe for every boundary type.
#[derive(Debug, Clone)]
pub struct Grid2D {
    pub data: Vec<f32>,
    pub cols: u32,
    pub rows: u32,
    /// Cell size, angstroms.
    pub cell_width: f32,
    pub cell_height: f32,
}

impl Grid2D {
    /// A zeroed grid covering `width` x `height` angstroms.
    ///
    /// # Panics
    /// Panics if `cell_size` is not positive.
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "cell size must be positive");
        let cols = Self::cells_across(width, cell_size);
        let rows = Self::cells_across(height, cell_size);
        Self {
            data: vec![0.0; (cols * rows) as usize],
            cols,
            rows,
            cell_width: cell_size,
            cell_height: cell_size,
        }
    }

    /// Cell coordinates for a world position, wrapping at the grid
    /// edges.
    pub fn cell(&self, x: f32, y: f32) -> (u32, u32) {
        let cx = (x / self.cell_width).floor().rem_euclid(self.cols as f32) as u32;
        let cy = (y / self.cell_height).floor().rem_euclid(self.rows as f32) as u32;
        (cx, cy)
    }

    /// Flat index for a world position.
    pub fn index(&self, x: f32, y: f32) -> usize {
        let (c, r) = self.cell(x, y);
        c as usize + r as usize * self.cols as usize
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        self.data[self.index(x, y)]
    }

    pub fn set(&mut self, x: f32, y: f32, value: f32) {
        let i = self.index(x, y);
        self.data[i] = value;
    }

    /// Adds energy to a cell. Bond events release and absorb heat
    /// through this (runtime spec 7.1, 7.2).
    pub fn add(&mut self, x: f32, y: f32, amount: f32) {
        let i = self.index(x, y);
        self.data[i] += amount;
    }

    fn cells_across(length: f32, cell: f32) -> u32 {
        ((length / cell).ceil().max(1.0)) as u32
    }
}

/// World edge behavior (runtime spec 4.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    /// Torus: exits re-enter on the opposite side.
    Wrap,
    /// Reflective walls.
    Wall,
    /// Atoms leaving are flagged dead; their bonds break first.
    Open,
}

/// The complete mutable simulation state (runtime spec 4.7).
#[derive(Debug)]
pub struct WorldState {
    pub tick: u64,
    pub atoms: Vec<AtomState>,
    pub bonds: Vec<BondState>,
    /// World dimensions, angstroms.
    pub width: f32,
    pub height: f32,
    pub boundary: BoundaryType,
    pub temp_field: Grid2D,
    pub pressure_field: Grid2D,
    pub uv_field: Grid2D,
    pub energy_sources: Vec<EnergySource>,
    pub element_table: Arc<Vec<ElementProperties>>,
    pub spatial_index: SpatialIndex,
    pub rng: Rng,
    /// Filled by the observer, flushed to stdout after each tick.
    pub event_queue: Vec<Event>,
}

impl WorldState {
    /// An empty world sized in angstroms, with the standard element
    /// table and the given physics constants.
    pub fn new(
        width: f32,
        height: f32,
        boundary: BoundaryType,
        seed: u64,
        config: PhysicsConfig,
    ) -> Self {
        Self {
            tick: 0,
            atoms: Vec::new(),
            bonds: Vec::new(),
            width,
            height,
            boundary,
            temp_field: Grid2D::new(width, height, config.field_cell_size),
            pressure_field: Grid2D::new(width, height, config.field_cell_size),
            uv_field: Grid2D::new(width, height, config.field_cell_size),
            energy_sources: Vec::new(),
            element_table: Arc::new(ELEMENTS.to_vec()),
            spatial_index: SpatialIndex::new(config.spatial_cell_size),
            rng: Rng::new(seed),
            event_queue: Vec::new(),
        }
    }

    /// Properties for an element id.
    ///
    /// # Panics
    /// Panics if the id is out of range; element ids only ever come
    /// from the table or from validated definitions.
    pub fn element(&self, id: ElementId) -> &ElementProperties {
        &self.element_table[id.0 as usize]
    }

    /// Adds a live atom at rest. Its id is its index; compaction
    /// remaps ids later (runtime spec 5.2).
    pub fn spawn_atom(&mut self, element: ElementId, x: f32, y: f32) -> AtomId {
        let id = AtomId(self.atoms.len() as u32);
        self.atoms.push(AtomState::new(id, element, x, y));
        id
    }

    /// The atom with the given id.
    ///
    /// # Panics
    /// Panics if the id is out of range; ids are indexes handed out
    /// by this world.
    pub fn atom(&self, id: AtomId) -> &AtomState {
        &self.atoms[id.0 as usize]
    }

    /// The atom with the given id, mutably.
    ///
    /// # Panics
    /// Panics if the id is out of range.
    pub fn atom_mut(&mut self, id: AtomId) -> &mut AtomState {
        &mut self.atoms[id.0 as usize]
    }

    /// The bond with the given id.
    ///
    /// # Panics
    /// Panics if the id is out of range.
    pub fn bond(&self, id: BondId) -> &BondState {
        &self.bonds[id.0 as usize]
    }

    /// The bond with the given id, mutably.
    ///
    /// # Panics
    /// Panics if the id is out of range.
    pub fn bond_mut(&mut self, id: BondId) -> &mut BondState {
        &mut self.bonds[id.0 as usize]
    }

    /// Whether two atoms share a live bond.
    pub fn is_bonded(&self, a: AtomId, b: AtomId) -> bool {
        let atom = self.atom(a);
        atom.bonds[..atom.bond_count as usize]
            .iter()
            .filter_map(|slot| *slot)
            .any(|id| {
                let bond = self.bond(id);
                bond.alive && (bond.atom_a == b || bond.atom_b == b)
            })
    }

    /// Forms a bond between two atoms, enforcing guarantee G04 (bond
    /// formation never exceeds an element's `max_bonds`) and
    /// rejecting self-bonds, dead atoms, and duplicates. Returns the
    /// new bond id, or `None` if the bond was not formed.
    ///
    /// # Panics
    /// Panics if `order` is not 1, 2, or 3.
    pub fn form_bond(&mut self, a: AtomId, b: AtomId, order: u8, energy: f32) -> Option<BondId> {
        assert!((1..=3).contains(&order), "bond order must be 1, 2, or 3");
        if a == b || !self.atom(a).alive || !self.atom(b).alive || self.is_bonded(a, b) {
            return None;
        }
        if self.atom(a).bond_count >= self.element(self.atom(a).element).max_bonds
            || self.atom(b).bond_count >= self.element(self.atom(b).element).max_bonds
        {
            return None;
        }
        let id = BondId(self.bonds.len() as u32);
        self.bonds.push(BondState::new(id, a, b, order, energy));
        let ok_a = self.atom_mut(a).push_bond(id);
        let ok_b = self.atom_mut(b).push_bond(id);
        debug_assert!(ok_a && ok_b, "capacity checked above");
        Some(id)
    }

    /// Breaks a bond: flags it dead and updates both atoms. Returns
    /// whether the bond was live. Energy accounting (releasing
    /// `bond.energy * release_fraction` into the temperature field)
    /// belongs to the chemistry system, not here.
    pub fn break_bond(&mut self, id: BondId) -> bool {
        let (was_alive, a, b) = {
            let bond = self.bond(id);
            (bond.alive, bond.atom_a, bond.atom_b)
        };
        if !was_alive {
            return false;
        }
        self.bond_mut(id).alive = false;
        self.atom_mut(a).remove_bond(id);
        self.atom_mut(b).remove_bond(id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_bond_slots() {
        let mut a = AtomState::new(AtomId(0), ElementId(0), 0.0, 0.0);
        assert_eq!(a.bond_count, 0);
        assert!(a.push_bond(BondId(0)));
        assert!(a.push_bond(BondId(1)));
        assert_eq!(a.bond_count, 2);
        assert!(a.remove_bond(BondId(0)));
        assert_eq!(a.bond_count, 1);
        assert_eq!(a.bonds[0], Some(BondId(1)));
        assert!(!a.remove_bond(BondId(7)));
    }

    #[test]
    fn atom_bond_capacity() {
        let mut a = AtomState::new(AtomId(0), ElementId(0), 0.0, 0.0);
        for i in 0..MAX_BONDS as u32 {
            assert!(a.push_bond(BondId(i)));
        }
        assert!(!a.push_bond(BondId(99)));
        assert_eq!(a.bond_count, MAX_BONDS as u8);
    }

    #[test]
    fn grid_shape_and_wrap() {
        let g = Grid2D::new(200.0, 100.0, 10.0);
        assert_eq!((g.cols, g.rows), (20, 10));
        assert_eq!(g.index(0.0, 0.0), 0);
        // wrap: -5 A is equivalent to 195 A -> col 19
        assert_eq!(g.index(-5.0, 0.0), 19);
        assert_eq!(g.index(0.0, -5.0), 9 * 20);
        let mut g = Grid2D::new(100.0, 100.0, 10.0);
        g.set(12.0, 15.0, 5.0);
        g.add(12.0, 15.0, 1.5);
        assert_eq!(g.get(12.0, 15.0), 6.5);
    }

    #[test]
    fn world_state_shape() {
        let w = WorldState::new(
            200.0,
            100.0,
            BoundaryType::Wrap,
            42,
            PhysicsConfig::default(),
        );
        assert_eq!(w.tick, 0);
        assert_eq!(w.element_table.len(), 10);
        assert_eq!(w.temp_field.cols, 20);
        assert_eq!(w.spatial_index.cell_size(), 5.0);
        assert_eq!(w.element(ElementId(1)).symbol, "C");
    }

    #[test]
    fn spawn_ids_are_indexes() {
        let mut w = WorldState::new(
            200.0,
            200.0,
            BoundaryType::Wrap,
            1,
            PhysicsConfig::default(),
        );
        let a = w.spawn_atom(ElementId(0), 0.0, 0.0);
        let b = w.spawn_atom(ElementId(1), 1.0, 0.0);
        assert_eq!(a, AtomId(0));
        assert_eq!(b, AtomId(1));
        assert_eq!(w.atom(b).element, ElementId(1));
    }

    #[test]
    fn g04_max_bonds_enforced() {
        let mut w = WorldState::new(
            200.0,
            200.0,
            BoundaryType::Wrap,
            1,
            PhysicsConfig::default(),
        );
        // Hydrogen: max_bonds 1
        let a = w.spawn_atom(ElementId(0), 0.0, 0.0);
        let b = w.spawn_atom(ElementId(0), 1.0, 0.0);
        let c = w.spawn_atom(ElementId(0), 2.0, 0.0);
        assert!(w.form_bond(a, b, 1, 436.0).is_some());
        // a and b are both full now (max_bonds 1)
        assert!(w.form_bond(a, c, 1, 436.0).is_none());
        assert!(w.form_bond(b, c, 1, 436.0).is_none());
        // self-bond rejected
        assert!(w.form_bond(c, c, 1, 0.0).is_none());
        // breaking frees the slot
        assert!(w.break_bond(BondId(0)));
        assert!(!w.break_bond(BondId(0)));
        assert!(w.form_bond(a, b, 1, 436.0).is_some());
    }

    #[test]
    fn duplicate_bonds_rejected() {
        let mut w = WorldState::new(
            200.0,
            200.0,
            BoundaryType::Wrap,
            1,
            PhysicsConfig::default(),
        );
        // Carbon: max_bonds 4, so capacity is not the rejector here
        let a = w.spawn_atom(ElementId(1), 0.0, 0.0);
        let b = w.spawn_atom(ElementId(1), 1.0, 0.0);
        assert!(w.form_bond(a, b, 1, 346.0).is_some());
        // reversed atom order is still a duplicate
        assert!(w.form_bond(b, a, 1, 346.0).is_none());
        // same pair, different order, is still a duplicate
        assert!(w.form_bond(a, b, 2, 614.0).is_none());
    }
}
