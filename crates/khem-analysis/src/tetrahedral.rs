//! The tetrahedral order parameter q (Chau, Hardwick, Bhatt):
//! for each center atom with its four nearest neighbors,
//!
//! ```text
//! q = 1 - (3/8) * sum over the 6 neighbor pairs i<j of
//!     (cos(theta_ij) + 1/3)^2
//! ```
//!
//! q = 1.0 is a perfect tetrahedron (ice-like); q = 0 is an
//! uncorrelated/uniform arrangement; liquid water sits at
//! q ~ 0.7 (the number khem's real-water substrate sitting
//! targets: MEASURABLE tetrahedral order, ADR-0014's checkpoint
//! contract). The four nearest neighbors are the definition -
//! no shells, no molecule filtering (in water the four nearest
//! oxygens around an oxygen ARE the first shell, hydrogens
//! closer than any oxygen sit at the intramolecular distances
//! and are filtered by element choice).
//!
//! This is the mW/Mercedes-Benz lineage made measurable: if the
//! directional hydrogen-bond term builds the network, q rises
//! from the uncorrelated value toward liquid water's own.

use khem_forcefield::ElementId;
use khem_world::WorldState;

/// Mean tetrahedral order over every alive atom of `center`
/// whose four nearest `neighbor`-element atoms exist within
/// `within` (angstroms). Atoms without four such neighbors are
/// skipped (q needs exactly four corners). Returns 0.0 when no
/// atom qualifies.
pub fn tetrahedral_order(
    world: &WorldState,
    center: ElementId,
    neighbor: ElementId,
    within: f32,
) -> f64 {
    let mut q_sum = 0.0f64;
    let mut n = 0u64;
    for atom in world
        .atoms
        .iter()
        .filter(|a| a.alive && a.element == center)
    {
        // The four nearest neighbor-element atoms by true
        // (minimum-image) distance.
        let near = world
            .spatial_index
            .neighbors(atom.x, atom.y, atom.z, within);
        let mut dists: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(8);
        for id in near {
            let b = world.atom(id);
            if !b.alive || b.element != neighbor || id == atom.id {
                continue;
            }
            let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            if r <= within {
                dists.push((r, dx, dy, dz));
            }
        }
        if dists.len() < 4 {
            continue;
        }
        dists.sort_by(|a, b| a.0.total_cmp(&b.0));
        let corners = &dists[..4];
        let mut q = 1.0f64;
        for i in 0..4 {
            for j in (i + 1)..4 {
                // cos(theta) between corner i and corner j as seen
                // from the center.
                let dot = (corners[i].1 as f64 * corners[j].1 as f64
                    + corners[i].2 as f64 * corners[j].2 as f64
                    + corners[i].3 as f64 * corners[j].3 as f64)
                    / (corners[i].0 as f64 * corners[j].0 as f64);
                let cos = dot.clamp(-1.0, 1.0);
                q -= 0.375 * (cos + 1.0 / 3.0) * (cos + 1.0 / 3.0);
            }
        }
        q_sum += q;
        n += 1;
    }
    if n == 0 { 0.0 } else { q_sum / n as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use khem_forcefield::{ForceField, element_id};
    use khem_world::{BoundaryType, WorldState};

    /// A center at the origin with four neighbors at the perfect
    /// tetrahedron directions (109.47 degrees between all pairs).
    #[test]
    fn perfect_tetrahedron_scores_one() {
        let mut w = WorldState::new(
            40.0,
            40.0,
            40.0,
            BoundaryType::Wrap,
            1,
            ForceField::default(),
        );
        let o = element_id("O").unwrap();
        let s = 2.8_f32; // first-shell scale
        let dirs: [(f32, f32, f32); 4] = [
            (1.0, 1.0, 1.0),
            (1.0, -1.0, -1.0),
            (-1.0, 1.0, -1.0),
            (-1.0, -1.0, 1.0),
        ];
        let _ = w.spawn_atom(o, 20.0, 20.0, 20.0);
        for (dx, dy, dz) in dirs {
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            w.spawn_atom(o, 20.0 + dx / d * s, 20.0 + dy / d * s, 20.0 + dz / d * s);
        }
        w.spatial_index.rebuild(&w.atoms);
        let q = tetrahedral_order(&w, o, o, 3.5);
        assert!(q > 0.999, "perfect tetrahedron q = {q}");
    }

    /// A square-planar arrangement (90 degree pairs) scores well
    /// below 1 and above the anti-correlated value.
    #[test]
    fn square_planar_scores_below_tetrahedral() {
        let mut w = WorldState::new(
            40.0,
            40.0,
            40.0,
            BoundaryType::Wrap,
            1,
            ForceField::default(),
        );
        let o = element_id("O").unwrap();
        let _ = w.spawn_atom(o, 20.0, 20.0, 20.0);
        for (dx, dy) in [(1.0f32, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            w.spawn_atom(o, 20.0 + dx * 2.8, 20.0 + dy * 2.8, 20.0);
        }
        w.spatial_index.rebuild(&w.atoms);
        let q = tetrahedral_order(&w, o, o, 3.5);
        // cos(90) = 0: q = 1 - (3/8)*6*(1/9) = 0.75 exactly, but
        // the planar 180-degree diagonals are among the 6 pairs:
        // cos(180) = -1, term (-1+1/3)^2 = 4/9. Two such pairs:
        // q = 1 - (3/8)*(4*(1/9) + 2*(4/9)) = 1 - (3/8)*(12/9)
        //   = 0.5.
        assert!((q - 0.5).abs() < 1e-6, "square planar q = {q}");
    }
}
