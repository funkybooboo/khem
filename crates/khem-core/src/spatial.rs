//! The spatial hash index for neighbor queries; rebuilt every tick
//! after position updates (runtime spec sections 4.9 and 5.1, tick
//! step 5).
//!
//! Queries do not wrap at world edges: the boundary step runs before
//! the rebuild in the tick order, so positions are already
//! normalized.

use std::collections::HashMap;

use crate::world::{AtomId, AtomState};

/// Spatial hash over live atom positions.
///
/// Cell size defaults to 5 angstroms
/// ([`crate::config::PhysicsConfig::spatial_cell_size`]), roughly
/// the bond search radius. Rebuild is O(n); queries are O(1)
/// average.
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    cells: HashMap<(i32, i32), Vec<AtomId>>,
    cell_size: f32,
}

impl SpatialIndex {
    /// An empty index. Rebuild before the first query.
    ///
    /// # Panics
    /// Panics if `cell_size` is not positive.
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "cell size must be positive");
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }

    /// Cell size, angstroms.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Rebuilds the index from live atoms. Dead atoms are skipped.
    pub fn rebuild(&mut self, atoms: &[AtomState]) {
        self.cells.clear();
        for atom in atoms {
            if !atom.alive {
                continue;
            }
            let (gx, gy) = self.cell_coords(atom.x, atom.y);
            self.cells.entry((gx, gy)).or_default().push(atom.id);
        }
    }

    /// Candidate atoms whose cells overlap the circle at (x, y) with
    /// the given radius. The caller filters by exact distance
    /// (runtime spec 8.3). Order is by cell, not by distance.
    pub fn neighbors(&self, x: f32, y: f32, radius: f32) -> Vec<AtomId> {
        let mut out = Vec::new();
        let (cx, cy) = self.cell_coords(x, y);
        let span = (radius / self.cell_size).ceil().max(1.0) as i32;
        for gx in cx - span..=cx + span {
            for gy in cy - span..=cy + span {
                if let Some(ids) = self.cells.get(&(gx, gy)) {
                    out.extend_from_slice(ids);
                }
            }
        }
        out
    }

    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::ElementId;

    fn atom(id: u32, x: f32, y: f32) -> AtomState {
        AtomState::new(AtomId(id), ElementId(0), x, y)
    }

    #[test]
    fn rebuild_and_query() {
        let atoms = vec![atom(0, 1.0, 1.0), atom(1, 50.0, 50.0), atom(2, 2.0, 2.0)];
        let mut idx = SpatialIndex::new(5.0);
        idx.rebuild(&atoms);
        let found = idx.neighbors(1.0, 1.0, 4.0);
        assert!(found.contains(&AtomId(0)));
        assert!(found.contains(&AtomId(2)));
        assert!(!found.contains(&AtomId(1)));
    }

    #[test]
    fn dead_atoms_skipped() {
        let mut a = atom(0, 1.0, 1.0);
        a.alive = false;
        let mut idx = SpatialIndex::new(5.0);
        idx.rebuild(&[a]);
        assert!(idx.neighbors(1.0, 1.0, 4.0).is_empty());
    }

    #[test]
    fn rebuild_replaces_previous() {
        let mut idx = SpatialIndex::new(5.0);
        idx.rebuild(&[atom(0, 1.0, 1.0)]);
        idx.rebuild(&[atom(5, 100.0, 100.0)]);
        assert!(!idx.neighbors(1.0, 1.0, 4.0).contains(&AtomId(0)));
    }
}
