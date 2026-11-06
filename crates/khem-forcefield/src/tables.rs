//! The bond-energy lookup table (spec 7.3) - force-field DATA, the
//! same family as the element table. `bond_energy` is the pure
//! lookup the chemistry laws and the spring laws both read; the
//! table is private, the function is the contract.
//!
//! Unknown (pair, order) rows resolve by scaling the single-bond
//! reference (the elements' own radii-derived references, or the
//! table's single row) - documented, deterministic, cheap.

use crate::ElementId;

/// Bond energies, kJ/mol (spec 7.3). Rows are (a, b, order, energy)
/// with a <= b. Element indices: H=0 C=1 N=2 O=3 P=4 S=5 Si=6 Fe=7
/// Na=8 Cl=9 (the canonical ELEMENTS order). Fe-O is ~390, variable
/// in reality (spec notes this).
pub const BOND_ENERGY_TABLE: [(u8, u8, u8, f32); 29] = [
    (0, 0, 1, 436.0), // H-H
    (0, 1, 1, 413.0), // H-C
    (0, 2, 1, 391.0), // H-N
    (0, 3, 1, 463.0), // H-O
    (0, 5, 1, 363.0), // H-S
    (1, 1, 1, 346.0), // C-C
    (1, 1, 2, 614.0), // C=C
    (1, 1, 3, 839.0), // C#C
    (1, 2, 1, 305.0), // C-N
    (1, 2, 2, 615.0), // C=N
    (1, 2, 3, 891.0), // C#N
    (1, 3, 1, 358.0), // C-O
    (1, 3, 2, 799.0), // C=O
    (1, 4, 1, 264.0), // C-P
    (1, 5, 1, 272.0), // C-S
    (2, 2, 1, 163.0), // N-N
    (2, 2, 2, 418.0), // N=N
    (2, 2, 3, 945.0), // N#N
    (2, 3, 1, 201.0), // N-O
    (2, 3, 2, 607.0), // N=O
    (3, 3, 1, 146.0), // O-O
    (3, 3, 2, 498.0), // O=O
    (3, 4, 1, 335.0), // O-P
    (3, 5, 1, 265.0), // O-S
    (3, 6, 1, 452.0), // Si-O
    (3, 7, 1, 390.0), // Fe-O (approximate)
    (4, 4, 1, 201.0), // P-P
    (5, 5, 1, 266.0), // S-S
    (6, 6, 1, 222.0), // Si-Si
];

/// Per-element single-bond reference energy, kJ/mol, for the
/// unknown-pair geometric-mean fallback: the element's own single
/// bond where the table has one (N-N 163 is genuinely weak; note the
/// fallback makes unknown N pairs weak too - a documented v0
/// behavior), standard values for Na and Cl, which have no table
/// entries.
pub const SINGLE_REFERENCE: [f32; 10] = [
    436.0, // H
    346.0, // C
    163.0, // N
    146.0, // O
    201.0, // P
    266.0, // S
    222.0, // Si
    390.0, // Fe
    77.0,  // Na
    243.0, // Cl
];

/// Bond energy, kJ/mol (spec 7.3 with the fallback rules above).
pub fn bond_energy(a: ElementId, b: ElementId, order: u8) -> f32 {
    let (lo, hi) = if a.0 <= b.0 { (a.0, b.0) } else { (b.0, a.0) };
    for (x, y, o, e) in BOND_ENERGY_TABLE {
        if x == lo && y == hi && o == order {
            return e;
        }
    }
    // Unknown (pair, order): known single scaled by order, else the
    // geometric mean of the elements' references, scaled by order.
    let single = if lo == hi {
        SINGLE_REFERENCE[lo as usize]
    } else if let Some(e) = table_single(lo, hi) {
        e
    } else {
        (SINGLE_REFERENCE[lo as usize] * SINGLE_REFERENCE[hi as usize]).sqrt()
    };
    single * (order.max(1) as f32)
}

fn table_single(lo: u8, hi: u8) -> Option<f32> {
    for (x, y, o, e) in BOND_ENERGY_TABLE {
        if x == lo && y == hi && o == 1 {
            return Some(e);
        }
    }
    None
}
