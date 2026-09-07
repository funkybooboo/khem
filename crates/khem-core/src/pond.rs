//! The hardcoded primordial pond: phase-1 world construction
//! (docs/plans/phase-1-kernel.md: "a hardcoded primordial pond").
//! Phase 4 replaces this with
//! .kem world declarations parsed by khem-lang; every constant here
//! is provisional and gets revisited when the language arrives.
//!
//! Scale decision (2026-09-05, owner-approved): the language-spec
//! pond is ~70k atoms in 200x200 A - ~20x liquid-water density and
//! 7x the v0.1 performance target. The phase-1 pond is
//! ~1,000 water molecules plus a few hundred free atoms: a dilute
//! 2D monolayer, sized so the K1 signal (do molecules persist?)
//! reads cleanly.
//!
//! The 3D port (2026-09-08, ADR-0013): the monolayer became a
//! shallow SLAB - 60x60x15 A, waters on the same 3.75 A lattice
//! constant extended to a cubic 16x16x4 lattice (1024 waters, the
//! same atom budget as the 2D pond: ~3.4k atoms keeps the K1
//! harnesses' runtimes comparable through the port's ~3x pair-scan
//! cost). The shape is PROVISIONAL per the port plan (cube versus
//! slab is decided from the K1.1/K1.4 re-climb evidence, not
//! taste): the slab keeps the lateral extent/depth ratio that made
//! the 2D pond's K1 signal readable while restoring real 3D
//! geometry (chains can pass, tetrahedra exist). The vertical axis
//! is z: the vent sits at the floor, the surface is the top layer
//! (spec 8.1/8.2), and convection lifts +z.
//!
//! The vented pond (gate K1.1): the vent at the floor injects
//! heat, the 35 C setpoint reservoir drains it - a real steady
//! state to be stable against, with chemistry perturbing locally.
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
use crate::elements::{self, element_id};
use crate::energy::EnergySource;
use crate::world::{BoundaryType, WorldState};

/// Pond dimensions, angstroms. The slab: wide in x/y, shallow in
/// z (the port plan's provisional shape; see the module doc).
pub const POND_WIDTH: f32 = 60.0;
pub const POND_HEIGHT: f32 = 60.0;
pub const POND_DEPTH: f32 = 15.0;
/// Uniform starting temperature, celsius (ocean-region value from
/// the language-spec pond).
pub const POND_TEMP: f32 = 35.0;
/// Water lattice spacing, angstroms (the 2D pond's constant,
/// extended to a cubic lattice by the port; re-derived with the
/// K1.4 re-tune if the re-climb evidence demands it).
const WATER_SPACING: f32 = 3.75;
/// Lattice dimensions in molecules (16x16x4 = 1024).
const WATER_COLS: i32 = (POND_WIDTH / WATER_SPACING) as i32;
const WATER_ROWS: i32 = (POND_HEIGHT / WATER_SPACING) as i32;
const WATER_LAYERS: i32 = (POND_DEPTH / WATER_SPACING) as i32;
/// Seeded water molecules in the pond - one number the K1
/// harness and the pond tests share, derived from the same consts
/// the builder loops over (no hand-copied 32 * 32 to desync).
pub const POND_WATERS: usize = (WATER_COLS * WATER_ROWS * WATER_LAYERS) as usize;
/// Free-atom sprinkle: (symbol, count). Rebalanced 2026-09-05:
/// the founding H-dominated mix could only form strong bonds
/// (H-H 436, H-O 463 - p_break ~ exp(-22) at pond temperature), so
/// the measured harness showed zero breaks ever: frozen inertness
/// by composition, violating K1's "bonds form AND break". O/N-rich
/// gives weak flickering pairs (O-O 146, N-N 163 - p ~ 1e-3/1e-4
/// per tick at 45 C) alongside the strong ones.
const FREE_ATOMS: [(&str, u32); 4] = [("H", 100), ("C", 80), ("N", 80), ("O", 100)];

/// Lattice jitter: +-30% of the spacing on every axis, drawn from
/// the world RNG (draw order is part of the pond's determinism).
fn lattice_jitter(w: &mut WorldState) -> f32 {
    ((w.rng.f01() - 0.5) * (WATER_SPACING * 0.6) as f64) as f32
}

/// A uniform random unit vector on the sphere (two draws: the
/// cosine of the polar angle and the azimuth).
fn uniform_direction(w: &mut WorldState) -> (f32, f32, f32) {
    let cos_theta = (w.rng.f01() * 2.0 - 1.0) as f32;
    let phi = (w.rng.f01() * std::f32::consts::TAU as f64) as f32;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    (sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

/// Builds the hardcoded primordial pond. The seed also seeds the
/// world RNG; the same seed yields the identical pond and run.
pub fn primordial_pond(seed: u64, config: PhysicsConfig) -> WorldState {
    let mut w = WorldState::new(
        POND_WIDTH,
        POND_HEIGHT,
        POND_DEPTH,
        BoundaryType::Wrap,
        seed,
        config,
    );
    // The field starts AT its setpoint; the vent and chemistry
    // perturb from there (G06 as amended: the declared environment
    // is a reservoir, sources are additional inputs).
    w.temp_field.data.fill(POND_TEMP);
    w.setpoint_field.data.fill(POND_TEMP);
    // The vent at the floor (the language-spec pond carries one
    // there; the vertical axis is z since the port).
    w.energy_sources.push(EnergySource::hydrothermal(
        (POND_WIDTH * 0.5, POND_HEIGHT * 0.5, 5.0),
        0.8,
        15.0,
    ));

    // Water: O plus two H at the bent geometry (104.5 degrees), bond
    // length = covalent radius sum (physics 6.3 equilibrium), on a
    // jittered cubic lattice with a uniformly random 3D
    // orientation per molecule: a bisector direction on the sphere
    // (two draws) plus an in-plane rotation about it (one draw).
    let (h_el, o_el) = (elements::H, elements::O);
    let oh_energy = bond_energy(o_el, h_el, 1);
    // Physics 6.3 equilibrium: the covalent radius sum, from the
    // same table the springs read.
    let bond_len = elements::element(o_el).radius + elements::element(h_el).radius;
    let half_angle = (104.5f32 / 2.0).to_radians();
    for gz in 0..WATER_LAYERS {
        for gy in 0..WATER_ROWS {
            for gx in 0..WATER_COLS {
                let x = (gx as f32 + 0.5) * WATER_SPACING + lattice_jitter(&mut w);
                let y = (gy as f32 + 0.5) * WATER_SPACING + lattice_jitter(&mut w);
                let z = (gz as f32 + 0.5) * WATER_SPACING + lattice_jitter(&mut w);
                // The H-O-H bisector: a uniform direction d; the
                // molecule's plane holds d and one perpendicular
                // direction e, rotated uniformly about d.
                let (dx, dy, dz) = uniform_direction(&mut w);
                // Any vector not parallel to d spans the
                // perpendicular plane with it.
                let (ux, uy, uz) = if dx.abs() < 0.9 {
                    (1.0, 0.0, 0.0)
                } else {
                    (0.0, 1.0, 0.0)
                };
                // e1 = normalize(cross(d, u)); e2 = cross(d, e1).
                let (cx, cy, cz) = (dy * uz - dz * uy, dz * ux - dx * uz, dx * uy - dy * ux);
                let clen = (cx * cx + cy * cy + cz * cz).sqrt();
                let (e1x, e1y, e1z) = (cx / clen, cy / clen, cz / clen);
                let (e2x, e2y, e2z) = (
                    dy * e1z - dz * e1y,
                    dz * e1x - dx * e1z,
                    dx * e1y - dy * e1x,
                );
                let a = (w.rng.f01() * std::f32::consts::TAU as f64) as f32;
                let (ex, ey, ez) = (
                    e1x * a.cos() + e2x * a.sin(),
                    e1y * a.cos() + e2y * a.sin(),
                    e1z * a.cos() + e2z * a.sin(),
                );
                // The two hydrogens sit at +-half_angle from the
                // bisector, in the molecule's plane.
                let (bx, by, bz) = (
                    dx * half_angle.cos(),
                    dy * half_angle.cos(),
                    dz * half_angle.cos(),
                );
                let (sx, sy, sz) = (
                    ex * half_angle.sin(),
                    ey * half_angle.sin(),
                    ez * half_angle.sin(),
                );
                let o = w.spawn_atom(o_el, x, y, z);
                let h1 = w.spawn_atom(
                    h_el,
                    x + bond_len * (bx + sx),
                    y + bond_len * (by + sy),
                    z + bond_len * (bz + sz),
                );
                let h2 = w.spawn_atom(
                    h_el,
                    x + bond_len * (bx - sx),
                    y + bond_len * (by - sy),
                    z + bond_len * (bz - sz),
                );
                w.form_bond(o, h1, 1, oh_energy);
                w.form_bond(o, h2, 1, oh_energy);
            }
        }
    }

    // Free atoms: uniform sprinkle in the volume with a small
    // margin on every axis.
    let margin = 2.0;
    for (symbol, count) in FREE_ATOMS {
        let el = element_id(symbol).expect("free-atom element in table");
        for _ in 0..count {
            let x = (w.rng.f01() * ((POND_WIDTH - 2.0 * margin) as f64)) as f32 + margin;
            let y = (w.rng.f01() * ((POND_HEIGHT - 2.0 * margin) as f64)) as f32 + margin;
            let z = (w.rng.f01() * ((POND_DEPTH - 2.0 * margin) as f64)) as f32 + margin;
            w.spawn_atom(el, x, y, z);
        }
    }
    w
}

/// Whether a spawned water molecule is still intact (both O-H bonds
/// alive). K1 harness metric: the fraction of intact waters is the
/// persistence signal. Recognized structurally: an alive O holding
/// exactly two bonds, both to H.
pub fn water_intact(world: &WorldState) -> usize {
    let (h_el, o_el) = (elements::H, elements::O);
    world
        .atoms
        .iter()
        .filter(|a| {
            if a.element != o_el || !a.alive || a.bond_count != 2 {
                return false;
            }
            a.bond_ids().all(|id| {
                let bond = world.bond(id);
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
        // 1024 waters (3 atoms each) + 360 free atoms.
        let waters = POND_WATERS;
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
            1,
            "the vented pond carries its vent"
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
        // Check the first water molecule's H-O-H angle in 3D (the
        // angle between the two O->H vectors), walking the O's
        // bonds through the canonical bond_ids() interface.
        let o = &w.atoms[0];
        let bond_ids: Vec<_> = o.bond_ids().collect();
        let h1 = w.atom(w.bond(bond_ids[0]).atom_b);
        let h2 = w.atom(w.bond(bond_ids[1]).atom_b);
        let (v1, v2) = (
            (h1.x - o.x, h1.y - o.y, h1.z - o.z),
            (h2.x - o.x, h2.y - o.y, h2.z - o.z),
        );
        let dot = v1.0 * v2.0 + v1.1 * v2.1 + v1.2 * v2.2;
        let (l1, l2) = (
            (v1.0 * v1.0 + v1.1 * v1.1 + v1.2 * v1.2).sqrt(),
            (v2.0 * v2.0 + v2.1 * v2.1 + v2.2 * v2.2).sqrt(),
        );
        let angle = (dot / (l1 * l2)).acos().to_degrees();
        assert!((angle - 104.5).abs() < 1.0, "H-O-H angle {angle}");
    }

    #[test]
    fn water_orientations_cover_the_sphere() {
        // The port's orientation law: the H-O-H bisectors are
        // uniformly distributed over the sphere - no axis is
        // privileged (a broken orientation would pile them into a
        // plane or pole).
        let w = primordial_pond(42, PhysicsConfig::default());
        let o_el = element_id("O").unwrap();
        let mut cos_z: Vec<f32> = Vec::new();
        for a in w
            .atoms
            .iter()
            .filter(|a| a.element == o_el && a.bond_count == 2)
        {
            let bond_ids: Vec<_> = a.bond_ids().collect();
            let h1 = w.atom(w.bond(bond_ids[0]).atom_b);
            let h2 = w.atom(w.bond(bond_ids[1]).atom_b);
            let bis = (
                h1.x + h2.x - 2.0 * a.x,
                h1.y + h2.y - 2.0 * a.y,
                h1.z + h2.z - 2.0 * a.z,
            );
            let len = (bis.0 * bis.0 + bis.1 * bis.1 + bis.2 * bis.2).sqrt();
            cos_z.push(bis.2 / len);
        }
        assert_eq!(cos_z.len(), POND_WATERS);
        // Uniform on the sphere: cos(polar) is uniform on [-1, 1].
        // Coarse buckets with wide tolerance (a planar pileup
        // would concentrate all mass in the middle buckets).
        let mut buckets = [0usize; 4];
        for c in &cos_z {
            let b = match c {
                c if *c < -0.5 => 0,
                c if *c < 0.0 => 1,
                c if *c < 0.5 => 2,
                _ => 3,
            };
            buckets[b] += 1;
        }
        for (i, count) in buckets.iter().enumerate() {
            assert!(
                *count > POND_WATERS / 8,
                "bisector cos_z bucket {i} holds only {count} - orientation not uniform"
            );
        }
    }
}
