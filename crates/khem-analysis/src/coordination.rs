//! Coordination measurements: the intermolecular contact census
//! (the K2.2 gate's "contact-dense matter" bar) and the
//! per-element-pair coordination number (the real-water
//! sitting's ~4 bar: how many of element B sit within `cutoff`
//! of each atom of element A).
//!
//! Both are read-only over the world (G03) and use the spatial
//! index + minimum-image delta exactly as the harnesses do.

use crate::molecule_roots;
use khem_forcefield::ElementId;
use khem_world::WorldState;

/// The K2.2 census: the fraction of alive atoms holding at
/// least one INTERMOLECULAR neighbor (different molecule) of
/// any element within `cutoff` (the gate's 3 A; bar 0.5,
/// measured 0.978 on the pond). This is "contact-dense matter":
/// the substrate's atoms sit in a condensed medium, not a gas.
pub fn intermolecular_contact_census(world: &WorldState, cutoff: f32) -> f64 {
    let roots = molecule_roots(world);
    let alive = world.atoms.iter().filter(|a| a.alive).count();
    if alive == 0 {
        return 0.0;
    }
    let mut within = 0u64;
    for atom in world.atoms.iter().filter(|a| a.alive) {
        let near = world
            .spatial_index
            .neighbors(atom.x, atom.y, atom.z, cutoff);
        for b_id in near {
            let b = world.atom(b_id);
            if !b.alive || b_id == atom.id || roots[b_id.0 as usize] == roots[atom.id.0 as usize] {
                continue; // self or same molecule: intramolecular
            }
            let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
            if (dx * dx + dy * dy + dz * dz) <= cutoff * cutoff {
                within += 1;
                break;
            }
        }
    }
    within as f64 / alive as f64
}

/// Mean coordination: for each alive atom of element `a`, the
/// number of `b`-element atoms within `cutoff` (angstroms),
/// averaged over all such `a` atoms. Molecules are NOT filtered
/// (callers choose the element pair - e.g. O around O in water
/// counts only other waters' oxygens once the intramolecular
/// O-H distance exceeds nothing relevant; O-O intramolecular
/// contacts do not exist in unbonded waters).
pub fn mean_coordination(world: &WorldState, a: ElementId, b: ElementId, cutoff: f32) -> f64 {
    let centers: Vec<&khem_world::AtomState> = world
        .atoms
        .iter()
        .filter(|x| x.alive && x.element == a)
        .collect();
    if centers.is_empty() {
        return 0.0;
    }
    let mut total = 0u64;
    for atom in &centers {
        let near = world
            .spatial_index
            .neighbors(atom.x, atom.y, atom.z, cutoff);
        for b_id in near {
            let o = world.atom(b_id);
            if !o.alive || o.element != b || b_id == atom.id {
                continue;
            }
            let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, o.x, o.y, o.z);
            if (dx * dx + dy * dy + dz * dz) <= cutoff * cutoff {
                total += 1;
            }
        }
    }
    total as f64 / centers.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use khem_forcefield::{ForceField, element_id};
    use khem_world::BoundaryType;

    /// Two bonded atoms (one molecule) plus one far free atom:
    /// census counts only the free atom if the pair's contact is
    /// intramolecular and the free one sits outside cutoff.
    #[test]
    fn intramolecular_contacts_do_not_count() {
        let mut w = WorldState::new(
            50.0,
            50.0,
            50.0,
            BoundaryType::Wrap,
            3,
            ForceField::default(),
        );
        let h = element_id("H").unwrap();
        let a = w.spawn_atom(h, 10.0, 10.0, 10.0);
        let b = w.spawn_atom(h, 10.7, 10.0, 10.0);
        w.spawn_atom(h, 30.0, 30.0, 30.0);
        w.form_bond(a, b, 1, 436.0);
        // cutoff 1.0 A: the bonded pair touches intramolecularly
        // (excluded), the free atom is far: census 0.
        w.spatial_index.rebuild(&w.atoms);
        assert!((intermolecular_contact_census(&w, 1.0) - 0.0).abs() < 1e-9);
        // cutoff 40 A: everyone is in contact: census 1.0.
        assert!((intermolecular_contact_census(&w, 40.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn coordination_counts_the_right_element() {
        let mut w = WorldState::new(
            50.0,
            50.0,
            50.0,
            BoundaryType::Wrap,
            3,
            ForceField::default(),
        );
        let h = element_id("H").unwrap();
        let o = element_id("O").unwrap();
        let c = w.spawn_atom(h, 10.0, 10.0, 10.0);
        w.spawn_atom(o, 11.0, 10.0, 10.0);
        w.spawn_atom(o, 12.0, 10.0, 10.0);
        let _ = c;
        w.spatial_index.rebuild(&w.atoms);
        let coord = mean_coordination(&w, h, o, 5.0);
        assert!(
            (coord - 2.0).abs() < 1e-9,
            "both oxygens within 5 A: {coord}"
        );
    }
}
