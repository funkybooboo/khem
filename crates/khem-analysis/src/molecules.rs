//! The molecule census (spec 9.2): connected components of live
//! atoms over live bonds, union-find with path halving, bonds
//! visited in BondId order for determinism.
//!
//! Extracted from the observer so the gate harnesses and the
//! future khem-view tools share ONE census with the runtime
//! stats (the observer's buckets are this census's summary).

use khem_world::WorldState;

/// Molecule detection (spec 9.2): connected components of live
/// atoms over live bonds, union-find with path halving, bonds
/// visited in BondId order for determinism. Returns component
/// sizes, one per live atom. Public: the spec 9.2 diagnostic
/// behind `stats`'s buckets - the K1 harness reads the full
/// distribution (largest molecule, growth trend) that the four
/// buckets flatten away.
pub fn molecule_roots(world: &WorldState) -> Vec<u32> {
    let n = world.atoms.len();
    let mut parent: Vec<u32> = (0..n as u32).collect();
    for bond in &world.bonds {
        if !bond.alive {
            continue;
        }
        let mut ra = find(&mut parent, bond.atom_a.0);
        let mut rb = find(&mut parent, bond.atom_b.0);
        if ra != rb {
            if ra > rb {
                std::mem::swap(&mut ra, &mut rb);
            }
            parent[rb as usize] = ra;
        }
    }
    // Flatten: every atom reads its final root directly.
    for i in 0..n {
        let r = find(&mut parent, i as u32);
        parent[i] = r;
    }
    parent
}

/// The census as sizes: one entry per live molecule, sorted -
/// the spec 9.2 diagnostic behind `stats`'s buckets, and the
/// K1 harness's full distribution (largest molecule, growth
/// trend) that the four buckets flatten away.
pub fn molecule_sizes(world: &WorldState) -> Vec<u32> {
    let roots = molecule_roots(world);
    let mut sizes = std::collections::HashMap::new();
    for atom in &world.atoms {
        if atom.alive {
            *sizes.entry(roots[atom.id.0 as usize]).or_insert(0u32) += 1;
        }
    }
    let mut sizes: Vec<u32> = sizes.into_values().collect();
    sizes.sort();
    sizes
}

/// Union-find find with path halving, iterative.
fn find(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        parent[i as usize] = parent[parent[i as usize] as usize];
        i = parent[i as usize];
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use khem_forcefield::{ForceField, element_id};
    use khem_world::{AtomId, BoundaryType, WorldState};

    #[test]
    fn chain_and_lone_atoms() {
        let mut w = WorldState::new(
            50.0,
            50.0,
            50.0,
            BoundaryType::Wrap,
            42,
            ForceField::default(),
        );
        let h = element_id("H").unwrap();
        let a = w.spawn_atom(h, 10.0, 10.0, 10.0);
        let b = w.spawn_atom(h, 11.0, 10.0, 10.0);
        w.spawn_atom(h, 20.0, 10.0, 10.0);
        w.form_bond(a, b, 1, 436.0);
        let sizes = molecule_sizes(&w);
        assert_eq!(sizes.iter().sum::<u32>(), 3);
        assert_eq!(sizes.iter().filter(|&&s| s == 2).count(), 1);
        assert_eq!(sizes.iter().filter(|&&s| s == 1).count(), 1);
    }

    #[test]
    fn ring_is_one_molecule() {
        let mut w = WorldState::new(
            50.0,
            50.0,
            50.0,
            BoundaryType::Wrap,
            42,
            ForceField::default(),
        );
        let c = element_id("C").unwrap();
        let ring: Vec<AtomId> = (0..6)
            .map(|i| w.spawn_atom(c, 10.0 + i as f32, 10.0, 10.0))
            .collect();
        for i in 0..6 {
            w.form_bond(ring[i], ring[(i + 1) % 6], 1, 346.0);
        }
        let sizes = molecule_sizes(&w);
        let nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        assert_eq!(nonzero, vec![6], "the ring is a single 6-molecule");
    }
}
