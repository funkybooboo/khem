//! The hardcoded primordial pond: phase-1 world construction
//! (PLAN: "a hardcoded primordial pond"). Phase 3 replaces this with
//! .kem world declarations parsed by khem-lang; every constant here
//! is provisional and gets revisited when the language arrives.
//!
//! Scale decision (2026-09-05, owner-approved): the language-spec
//! pond is ~70k atoms in 200x200 A - ~20x liquid-water density and
//! 7x the v0.1 performance target. The phase-1 pond is
//! ~1,000 water molecules plus a few hundred free atoms in a
//! 120x120 A world: a dilute 2D monolayer, sized so the K1 signal
//! (do molecules persist?) reads cleanly, with no energy sources -
//! stability is isolated from convection.
//!
//! Every placement draw comes from the world's seeded RNG, so the
//! same seed builds the identical pond (G02 from tick 0).
//!
//! Saturation note: intact water is chemically inert under the
//! phase-1 rules - both O slots and both H slots are full, so
//! crosslinking onto water is impossible until something breaks a
//! bond first. The reactive population is the free atoms.

use crate::chemistry::bond_energy;
use crate::config::PhysicsConfig;
use crate::elements::element_id;
use crate::world::{BoundaryType, WorldState};

/// Pond dimensions, angstroms.
pub const POND_WIDTH: f32 = 120.0;
pub const POND_HEIGHT: f32 = 120.0;
/// Uniform starting temperature, celsius (ocean-region value from
/// the language-spec pond).
pub const POND_TEMP: f32 = 35.0;
/// Water lattice spacing, angstroms (32x32 grid = 1024 molecules).
const WATER_SPACING: f32 = 3.75;
/// Free-atom sprinkle: (symbol, count).
const FREE_ATOMS: [(&str, u32); 4] = [("H", 150), ("C", 80), ("N", 60), ("O", 60)];

/// Builds the hardcoded primordial pond. The seed also seeds the
/// world RNG; the same seed yields the identical pond and run.
pub fn primordial_pond(seed: u64, config: PhysicsConfig) -> WorldState {
    let mut w = WorldState::new(POND_WIDTH, POND_HEIGHT, BoundaryType::Wrap, seed, config);
    // Uniform starting temperature; nothing heats or cools it until
    // chemistry moves energy (G06).
    for v in w.temp_field.data.iter_mut() {
        *v = POND_TEMP;
    }

    // Water: O plus two H at the bent geometry (104.5 degrees), bond
    // length = covalent radius sum (physics 6.3 equilibrium), on a
    // jittered lattice with a random orientation per molecule.
    let h_el = element_id("H").expect("H in table");
    let o_el = element_id("O").expect("O in table");
    let oh_energy = bond_energy(o_el, h_el, 1);
    let bond_len = 0.66 + 0.53; // covalent radii of O and H
    let half_angle = (104.5f32 / 2.0).to_radians();
    let cols = (POND_WIDTH / WATER_SPACING) as i32;
    let rows = (POND_HEIGHT / WATER_SPACING) as i32;
    for gy in 0..rows {
        for gx in 0..cols {
            let jitter = |w: &mut WorldState| (w.rng.f01() - 0.5) * (WATER_SPACING * 0.6) as f64;
            let x = (gx as f32 + 0.5) * WATER_SPACING + jitter(&mut w) as f32;
            let y = (gy as f32 + 0.5) * WATER_SPACING + jitter(&mut w) as f32;
            let orientation = w.rng.f01() as f32 * std::f32::consts::TAU;
            let o = w.spawn_atom(o_el, x, y);
            let a1 = orientation - half_angle;
            let a2 = orientation + half_angle;
            let h1 = w.spawn_atom(h_el, x + bond_len * a1.cos(), y + bond_len * a1.sin());
            let h2 = w.spawn_atom(h_el, x + bond_len * a2.cos(), y + bond_len * a2.sin());
            w.form_bond(o, h1, 1, oh_energy);
            w.form_bond(o, h2, 1, oh_energy);
        }
    }

    // Free atoms: uniform sprinkle with a small margin.
    let margin = 2.0;
    for (symbol, count) in FREE_ATOMS {
        let el = element_id(symbol).expect("free-atom element in table");
        for _ in 0..count {
            let x = (w.rng.f01() * ((POND_WIDTH - 2.0 * margin) as f64)) as f32 + margin;
            let y = (w.rng.f01() * ((POND_HEIGHT - 2.0 * margin) as f64)) as f32 + margin;
            w.spawn_atom(el, x, y);
        }
    }
    w
}

/// Whether a spawned water molecule is still intact (both O-H bonds
/// alive). K1 harness metric: the fraction of intact waters is the
/// persistence signal. Recognized structurally: an alive O holding
/// exactly two bonds, both to H.
pub fn water_intact(world: &WorldState) -> usize {
    let h_el = element_id("H").expect("H in table");
    let o_el = element_id("O").expect("O in table");
    world
        .atoms
        .iter()
        .filter(|a| {
            if a.element != o_el || !a.alive || a.bond_count != 2 {
                return false;
            }
            a.bonds.iter().flatten().all(|id| {
                let bond = world.bond(*id);
                bond.alive
                    && (world.atom(bond.atom_a).element == h_el
                        || world.atom(bond.atom_b).element == h_el)
            })
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pond_shape_and_content() {
        let w = primordial_pond(42, PhysicsConfig::default());
        // 1024 waters (3 atoms each) + 350 free atoms.
        let waters = 32 * 32;
        let free: u32 = FREE_ATOMS.iter().map(|(_, n)| n).sum();
        assert_eq!(w.atoms.len(), waters * 3 + free as usize);
        assert_eq!(w.bonds.len(), waters * 2);
        // Every bond alive; every water O full.
        let o_el = element_id("O").unwrap();
        let intact = w
            .atoms
            .iter()
            .filter(|a| a.element == o_el && a.bond_count == 2)
            .count();
        assert_eq!(intact, waters);
        // Uniform temperature everywhere.
        assert!(
            w.temp_field
                .data
                .iter()
                .all(|t| (*t - POND_TEMP).abs() < 1e-4)
        );
        assert_eq!(
            w.energy_sources.len(),
            0,
            "no sources: stability is isolated"
        );
    }

    #[test]
    fn pond_is_deterministic_per_seed() {
        let a = primordial_pond(7, PhysicsConfig::default());
        let b = primordial_pond(7, PhysicsConfig::default());
        assert_eq!(a.atoms.len(), b.atoms.len());
        for i in 0..a.atoms.len() {
            assert_eq!(a.atoms[i].x, b.atoms[i].x, "atom {i} x");
            assert_eq!(a.atoms[i].element, b.atoms[i].element, "atom {i} element");
        }
        let c = primordial_pond(8, PhysicsConfig::default());
        assert_ne!(a.atoms[0].x, c.atoms[0].x, "different seed, different pond");
    }

    #[test]
    fn water_geometry_is_bent() {
        let w = primordial_pond(42, PhysicsConfig::default());
        // Check the first water molecule's H-O-H angle.
        let o = &w.atoms[0];
        let h1 = w.atom(w.bond(o.bonds[0].unwrap()).atom_b);
        let h2 = w.atom(w.bond(o.bonds[1].unwrap()).atom_b);
        let a1 = (h1.y - o.y).atan2(h1.x - o.x);
        let a2 = (h2.y - o.y).atan2(h2.x - o.x);
        let d = (a1 - a2).abs().to_degrees();
        let d = d.min(360.0 - d);
        assert!((d - 104.5).abs() < 1.0, "H-O-H angle {d}");
    }
}
