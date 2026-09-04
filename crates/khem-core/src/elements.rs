//! The element table: valence, electronegativity, mass, covalent
//! radius, max bonds - real values for the 10 elements of the
//! founding design (H, C, N, O, P, S, Si, Fe, Na, Cl).
//!
//! Valence, electronegativity, and mass are the founding
//! conversation's values; so are the H and C radii. The remaining
//! radii are standard covalent radii, chosen so equilibrium bond
//! lengths (sum of radii, runtime spec 6.3) are plausible. All values
//! are phase-1 tuning knobs; phase 3 moves them into .kem
//! `elements` declarations loaded at runtime.
//!
//! Spec: docs/specs/runtime-spec.md, section 4.6.

use crate::world::ElementId;

/// Physical properties of one element. Immutable after load; shared
/// by reference inside [`crate::world::WorldState`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementProperties {
    /// One- or two-letter symbol, uppercase ASCII.
    pub symbol: &'static str,
    pub atomic_number: u8,
    /// Bonding electrons.
    pub valence: u8,
    /// Maximum simultaneous bonds.
    pub max_bonds: u8,
    /// Mass, daltons.
    pub mass: f32,
    /// Electronegativity, Pauling scale.
    pub electronegativity: f32,
    /// Covalent radius, angstroms.
    pub radius: f32,
}

impl ElementProperties {
    /// Table-building constructor. `const` so [`ELEMENTS`] is a true
    /// compile-time table.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        symbol: &'static str,
        atomic_number: u8,
        valence: u8,
        max_bonds: u8,
        mass: f32,
        electronegativity: f32,
        radius: f32,
    ) -> Self {
        Self {
            symbol,
            atomic_number,
            valence,
            max_bonds,
            mass,
            electronegativity,
            radius,
        }
    }
}

/// The 10 elements of the founding design, in canonical index order.
/// [`ElementId`] values index this table.
pub const ELEMENTS: [ElementProperties; 10] = [
    ElementProperties::new("H", 1, 1, 1, 1.008, 2.20, 0.53),
    ElementProperties::new("C", 6, 4, 4, 12.011, 2.55, 0.77),
    ElementProperties::new("N", 7, 3, 3, 14.007, 3.04, 0.71),
    ElementProperties::new("O", 8, 2, 2, 15.999, 3.44, 0.66),
    ElementProperties::new("P", 15, 5, 5, 30.974, 2.19, 1.07),
    // Sulfur and iron have variable coordination (runtime spec 7.4);
    // the max_bonds here is the phase-1 ceiling.
    ElementProperties::new("S", 16, 2, 6, 32.06, 2.58, 1.05),
    ElementProperties::new("Si", 14, 4, 4, 28.085, 1.90, 1.11),
    ElementProperties::new("Fe", 26, 2, 6, 55.845, 1.83, 1.32),
    ElementProperties::new("Na", 11, 1, 1, 22.990, 0.93, 1.66),
    ElementProperties::new("Cl", 17, 1, 1, 35.45, 3.16, 1.02),
];

/// Looks up an element by symbol.
pub fn element_id(symbol: &str) -> Option<ElementId> {
    ELEMENTS
        .iter()
        .position(|e| e.symbol == symbol)
        .map(|i| ElementId(i as u8))
}

/// Properties for an element id, straight from the const table. For
/// the world's table (phase 3: custom tables), use
/// [`crate::world::WorldState::element`].
///
/// # Panics
/// Panics if the id is out of range.
pub fn element(id: ElementId) -> &'static ElementProperties {
    &ELEMENTS[id.0 as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::MAX_BONDS;

    #[test]
    fn table_constraints_hold() {
        for (i, e) in ELEMENTS.iter().enumerate() {
            assert!(matches!(e.symbol.len(), 1 | 2));
            assert!((1..=118).contains(&e.atomic_number));
            assert!((1..=8).contains(&e.valence));
            assert!((0.0..=4.0).contains(&e.electronegativity));
            assert!(e.radius > 0.0);
            assert!(e.mass > 0.0);
            assert!(e.max_bonds >= e.valence);
            assert!(e.max_bonds as usize <= MAX_BONDS);
            assert_eq!(element_id(e.symbol), Some(ElementId(i as u8)));
        }
    }

    #[test]
    fn symbols_unique() {
        for (i, a) in ELEMENTS.iter().enumerate() {
            for b in &ELEMENTS[i + 1..] {
                assert_ne!(a.symbol, b.symbol);
            }
        }
    }

    #[test]
    fn lookup() {
        assert_eq!(element_id("O"), Some(ElementId(3)));
        assert_eq!(element_id("Au"), None);
        assert_eq!(element(ElementId(1)).symbol, "C");
    }
}
