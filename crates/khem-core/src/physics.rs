//! The physics system: thermal velocity perturbation
//! (Maxwell-Boltzmann), temperature diffusion, bond spring forces,
//! pressure-gradient forces, position updates, boundary conditions.
//!
//! Phase-1 placement decisions, to fold into the spec at revision
//! (ADR-0006: specs are drafts until validated):
//!
//! - Temperature diffusion (spec 6.2) runs at the top of
//!   [`PhysicsSystem::update_velocities`], before the kicks sample
//!   the field; spec 5.1 names no slot for it.
//! - Every per-atom effect is computed in read-only passes that
//!   accumulate into scratch arrays, then applied in one mutable
//!   pass. The compute/apply split is the shape a V2 region-parallel
//!   implementation needs, and it keeps each pass free of tangled
//!   borrows (ADR-0005, runtime spec 10.2).
//! - All constants are used exactly as spec 11 gives them. At least
//!   one is analytically unstable: explicit Euler with dt = 1
//!   diverges when sqrt(spring_k) > 2, and every bond in the table
//!   gives sqrt(energy * 0.01) > 2 (H-H: 2.09). The K1 harness
//!   measures that honestly; retuned constants get their own commit
//!   against evidence, not silently.
//!
//! RNG discipline (ADR-0005): this system is the first RNG consumer
//! in the tick (the energy system, step 1, draws nothing). Exactly
//! two `normal` draws per LIVE atom per tick, in AtomId order; dead
//! atoms draw nothing. Every other physics step is deterministic
//! arithmetic with no RNG access.
//!
//! Spec: docs/specs/runtime-spec.md, section 6 (Physics System).

use crate::config::PhysicsConfig;
use crate::world::{AtomId, BoundaryType, WorldState};

/// The physics system interface, decomposed per the tick order
/// (runtime spec 5.1: update_velocities, update_positions,
/// apply_boundary). v0.1 compiles exactly one implementation; the
/// trait exists so future plugin loading does not require
/// restructuring (runtime spec 10.4).
pub trait PhysicsSystem {
    /// Temperature diffusion, thermal kicks, bond spring forces,
    /// pressure-gradient forces (spec 6.1-6.4).
    fn update_velocities(&mut self, world: &mut WorldState);
    /// Position integration, dt = 1.0 (spec 6.5).
    fn update_positions(&mut self, world: &mut WorldState);
    /// Boundary normalization (spec 6.6; guarantee G05).
    fn apply_boundary(&mut self, world: &mut WorldState);
}

/// The v0.1 physics implementation (runtime spec 10.4: exactly one).
///
/// Holds only immutable constants plus reusable scratch space; it
/// keeps no state between ticks, so the same instance is safe for
/// every tick of a run.
pub struct Physics {
    config: PhysicsConfig,
    /// Per-atom thermal kicks for this tick (RNG output).
    kx: Vec<f32>,
    ky: Vec<f32>,
    /// Per-atom force accumulators for this tick (springs, pressure).
    fx: Vec<f32>,
    fy: Vec<f32>,
    /// Per-cell scratch for temperature diffusion.
    diffused: Vec<f32>,
}

impl Physics {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            kx: Vec::new(),
            ky: Vec::new(),
            fx: Vec::new(),
            fy: Vec::new(),
            diffused: Vec::new(),
        }
    }

    /// Spec 6.2: `T_new = T * (1 - rate) + mean(4 neighbors) * rate`,
    /// 4-connected, wrapped at the grid edges (grids wrap, like the
    /// Wrap boundary: runtime spec 4.8).
    fn diffuse_temperature(&mut self, world: &mut WorldState) {
        let rate = self.config.diffusion_rate;
        debug_assert!((0.0..=1.0).contains(&rate), "diffusion_rate out of range");
        let field = &world.temp_field;
        let cols = field.cols as usize;
        let rows = field.rows as usize;
        let n = cols * rows;
        let data = &field.data;
        let buf = &mut self.diffused;
        if buf.len() != n {
            buf.clear();
            buf.resize(n, 0.0);
        }
        for row in 0..rows {
            for col in 0..cols {
                let i = col + row * cols;
                let left = data[(col + cols - 1) % cols + row * cols];
                let right = data[(col + 1) % cols + row * cols];
                let up = data[col + (row + rows - 1) % rows * cols];
                let down = data[col + (row + 1) % rows * cols];
                let mean = (left + right + up + down) * 0.25;
                buf[i] = data[i] * (1.0 - rate) + mean * rate;
            }
        }
        world.temp_field.data.copy_from_slice(buf);
    }

    /// Spec 6.1: Maxwell-Boltzmann perturbation.
    /// `sigma = sqrt(kB * T / mass)`, T from the temperature field at
    /// the atom's position. Negative field values clamp to zero (a
    /// negative absolute temperature is meaningless; sigma would be
    /// NaN).
    fn thermal_kicks(&mut self, world: &mut WorldState) {
        let kb = self.config.thermal_kick_scale;
        self.kx.clear();
        self.ky.clear();
        for atom in &world.atoms {
            if !atom.alive {
                self.kx.push(0.0);
                self.ky.push(0.0);
                continue;
            }
            let t = world.temp_field.get(atom.x, atom.y).max(0.0);
            let mass = world.element(atom.element).mass;
            let sigma = (kb * t / mass).sqrt() as f64;
            self.kx.push(world.rng.normal(0.0, sigma) as f32);
            self.ky.push(world.rng.normal(0.0, sigma) as f32);
        }
    }

    /// Spec 6.3: Hooke's law toward `r_eq = radius_a + radius_b`,
    /// `F = bond.energy * spring_energy_scale * (r - r_eq)` applied
    /// to both atoms along the bond axis, equal and opposite. When
    /// `r < 0.5 * r_eq`: `F = -strong_repulsion / r^2` (repulsion).
    /// Coincident atoms (r ~ 0) have no defined axis; they are
    /// skipped rather than given a random direction - the next
    /// thermal kick separates them deterministically enough.
    fn spring_forces(&mut self, world: &mut WorldState) {
        let scale = self.config.spring_energy_scale;
        let repulsion = self.config.strong_repulsion;
        for bond in &world.bonds {
            if !bond.alive {
                continue;
            }
            let (ia, ib) = (bond.atom_a.0 as usize, bond.atom_b.0 as usize);
            let (a, b) = (&world.atoms[ia], &world.atoms[ib]);
            // Unreachable by construction (form_bond rejects dead
            // atoms; the Open boundary breaks bonds before killing).
            // Skipped, not asserted: a physics pass must never panic.
            if !a.alive || !b.alive {
                continue;
            }
            let (dx, dy) = world.delta(a.x, a.y, b.x, b.y);
            let r2 = dx * dx + dy * dy;
            if r2 < f32::EPSILON {
                continue;
            }
            let r = r2.sqrt();
            let r_eq = world.element(a.element).radius + world.element(b.element).radius;
            let f = if r < 0.5 * r_eq {
                -repulsion / r2
            } else {
                bond.energy * scale * (r - r_eq)
            };
            let (ux, uy) = (dx / r, dy / r);
            self.fx[ia] += f * ux;
            self.fy[ia] += f * uy;
            self.fx[ib] -= f * ux;
            self.fy[ib] -= f * uy;
        }
    }

    /// Spec 6.4: `pressure[cell] = atom_count / cell_area`, rebuilt
    /// from scratch every tick, then a central-difference gradient
    /// force toward lower pressure, scaled by
    /// `pressure_sensitivity`.
    fn pressure_forces(&mut self, world: &mut WorldState) {
        let sensitivity = self.config.pressure_sensitivity;
        let field = &mut world.pressure_field;
        field.data.fill(0.0);
        for atom in &world.atoms {
            if atom.alive {
                let i = field.index(atom.x, atom.y);
                field.data[i] += 1.0;
            }
        }
        let area = field.cell_width * field.cell_height;
        for v in field.data.iter_mut() {
            *v /= area;
        }
        let (cols, rows) = (field.cols as usize, field.rows as usize);
        let field = &world.pressure_field;
        for (i, atom) in world.atoms.iter().enumerate() {
            if !atom.alive {
                continue;
            }
            let (col, row) = field.cell(atom.x, atom.y);
            let (col, row) = (col as usize, row as usize);
            let left = field.data[(col + cols - 1) % cols + row * cols];
            let right = field.data[(col + 1) % cols + row * cols];
            let up = field.data[col + (row + rows - 1) % rows * cols];
            let down = field.data[col + (row + 1) % rows * cols];
            let dpdx = (right - left) / (2.0 * field.cell_width);
            let dpdy = (down - up) / (2.0 * field.cell_height);
            self.fx[i] -= dpdx * sensitivity;
            self.fy[i] -= dpdy * sensitivity;
        }
    }
}

impl PhysicsSystem for Physics {
    fn update_velocities(&mut self, world: &mut WorldState) {
        let n = world.atoms.len();
        self.diffuse_temperature(world);
        self.thermal_kicks(world);
        self.fx.clear();
        self.fy.clear();
        self.fx.resize(n, 0.0);
        self.fy.resize(n, 0.0);
        self.spring_forces(world);
        self.pressure_forces(world);
        for (i, atom) in world.atoms.iter_mut().enumerate() {
            atom.vx += self.kx[i] + self.fx[i];
            atom.vy += self.ky[i] + self.fy[i];
        }
    }

    fn update_positions(&mut self, world: &mut WorldState) {
        for atom in &mut world.atoms {
            if !atom.alive {
                continue;
            }
            // dt = 1.0: one tick = one femtosecond at default scale
            // (spec 6.5).
            atom.x += atom.vx;
            atom.y += atom.vy;
        }
    }

    fn apply_boundary(&mut self, world: &mut WorldState) {
        let (w, h) = (world.width, world.height);
        match world.boundary {
            BoundaryType::Wrap => {
                for atom in &mut world.atoms {
                    if !atom.alive {
                        continue;
                    }
                    atom.x = atom.x.rem_euclid(w);
                    atom.y = atom.y.rem_euclid(h);
                }
            }
            BoundaryType::Wall => {
                for atom in &mut world.atoms {
                    if !atom.alive {
                        continue;
                    }
                    if atom.x < 0.0 {
                        atom.x = 0.0;
                        atom.vx = -atom.vx;
                    } else if atom.x > w {
                        atom.x = w;
                        atom.vx = -atom.vx;
                    }
                    if atom.y < 0.0 {
                        atom.y = 0.0;
                        atom.vy = -atom.vy;
                    } else if atom.y > h {
                        atom.y = h;
                        atom.vy = -atom.vy;
                    }
                }
            }
            BoundaryType::Open => {
                let leaving: Vec<AtomId> = world
                    .atoms
                    .iter()
                    .filter(|a| a.alive && (a.x < 0.0 || a.x >= w || a.y < 0.0 || a.y >= h))
                    .map(|a| a.id)
                    .collect();
                // Bonds break before the atom dies (spec 6.6);
                // break_bond updates both sides.
                for id in leaving {
                    let slots = world.atom(id).bonds;
                    for bond in slots.into_iter().flatten() {
                        world.break_bond(bond);
                    }
                    world.atom_mut(id).alive = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{BondId, ElementId};

    fn world(seed: u64, boundary: BoundaryType) -> WorldState {
        WorldState::new(100.0, 100.0, boundary, seed, PhysicsConfig::default())
    }

    fn physics() -> Physics {
        Physics::new(PhysicsConfig::default())
    }

    #[test]
    fn thermal_kicks_scale_with_temperature_and_inverse_mass() {
        // Zero temperature: sigma = 0, the atom stays at rest.
        let mut cold = world(1, BoundaryType::Wrap);
        let a = cold.spawn_atom(ElementId(3), 50.0, 50.0); // O
        physics().update_velocities(&mut cold);
        let (vx, vy) = (cold.atom(a).vx, cold.atom(a).vy);
        assert_eq!((vx, vy), (0.0, 0.0));

        // Hot cell: O gets real kicks.
        let mut hot = world(1, BoundaryType::Wrap);
        let b = hot.spawn_atom(ElementId(3), 50.0, 50.0);
        hot.temp_field.set(50.0, 50.0, 400.0);
        physics().update_velocities(&mut hot);
        let dv = hot.atom(b).vx.hypot(hot.atom(b).vy);
        assert!(dv > 0.1, "hot oxygen kick magnitude {dv}");

        // Same temperature, lighter atom (H) -> larger sigma. Same
        // seed, same draw sequence shape.
        let mut light = world(1, BoundaryType::Wrap);
        let c = light.spawn_atom(ElementId(0), 50.0, 50.0);
        light.temp_field.set(50.0, 50.0, 400.0);
        physics().update_velocities(&mut light);
        let dv_light = light.atom(c).vx.hypot(light.atom(c).vy);
        assert!(dv_light > dv, "H kick {dv_light} should exceed O kick {dv}");
    }

    #[test]
    fn stretched_bond_pulls_together() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0); // H
        let b = w.spawn_atom(ElementId(0), 52.0, 50.0); // H, r_eq = 1.06
        w.form_bond(a, b, 1, 1.0); // weak spring so dt=1 does not overshoot
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx > 0.0, "a should move toward b");
        assert!(w.atom(b).vx < 0.0, "b should move toward a");
        physics().update_positions(&mut w);
        let d = w.atom(b).x - w.atom(a).x;
        assert!(d < 2.0, "distance {d} should shrink");
    }

    #[test]
    fn bonded_pair_across_wrap_seam_is_not_shredded() {
        // Minimum-image convention: two atoms 1 A apart ACROSS the
        // wrap seam must feel the spring for 1 A, not 119 A. The
        // pond starts with seam-straddling waters (lattice at 0.75
        // A plus 1.19 A bond length), so this law is load-bearing
        // for K1, not a corner case.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 0.3, 50.0);
        let b = w.spawn_atom(ElementId(0), 99.7, 50.0); // 1 A apart across the seam
        w.form_bond(a, b, 1, 436.0);
        physics().update_velocities(&mut w);
        let (va, vb) = (w.atom(a).vx, w.atom(b).vx);
        assert!(
            va.abs() < 1.0 && vb.abs() < 1.0,
            "seam pair must feel a 1 A spring, got velocities {va} / {vb}"
        );
    }

    #[test]
    fn overlapped_bond_pushes_apart() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 50.3, 50.0); // r = 0.3 < 0.53
        w.form_bond(a, b, 1, 1.0);
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx < 0.0, "a should be pushed away");
        assert!(w.atom(b).vx > 0.0, "b should be pushed away");
    }

    #[test]
    fn pressure_pushes_from_dense_to_sparse() {
        let mut w = world(1, BoundaryType::Wrap);
        // 20 atoms crowd the cell at col 1, row 1 (10 A cells);
        // the test atom sits one cell to their right.
        for i in 0..20 {
            w.spawn_atom(ElementId(0), 15.0, 15.0 + i as f32 * 0.01);
        }
        let probe = w.spawn_atom(ElementId(0), 25.0, 15.0);
        physics().update_velocities(&mut w);
        let pushed = w.atom(probe).vx;
        assert!(
            pushed > 0.0,
            "probe should move away from the crowd, got {pushed}"
        );
    }

    #[test]
    fn temperature_diffusion_spreads_and_conserves() {
        let mut w = world(1, BoundaryType::Wrap);
        // 10x10 cells; heat the center cell only.
        w.temp_field.set(50.0, 50.0, 100.0);
        let before: f32 = w.temp_field.data.iter().sum();
        physics().update_velocities(&mut w);
        let center = w.temp_field.get(50.0, 50.0);
        let side = w.temp_field.get(45.0, 50.0);
        let after: f32 = w.temp_field.data.iter().sum();
        assert!((center - 90.0).abs() < 1e-3, "center {center}");
        assert!((side - 2.5).abs() < 1e-3, "neighbor {side}");
        assert!(
            (before - after).abs() < 1e-2,
            "heat sum {before} -> {after}"
        );
    }

    #[test]
    fn positions_integrate_velocity() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        w.atom_mut(a).vx = 1.5;
        w.atom_mut(a).vy = -0.5;
        physics().update_positions(&mut w);
        assert_eq!((w.atom(a).x, w.atom(a).y), (51.5, 49.5));
    }

    #[test]
    fn dead_atoms_are_frozen() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        w.atom_mut(a).vx = 10.0;
        w.atom_mut(a).alive = false;
        physics().update_velocities(&mut w);
        physics().update_positions(&mut w);
        assert_eq!((w.atom(a).x, w.atom(a).vx), (50.0, 10.0));
    }

    #[test]
    fn wrap_boundary_normalizes() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), -1.0, 101.0);
        physics().apply_boundary(&mut w);
        assert_eq!((w.atom(a).x, w.atom(a).y), (99.0, 1.0));
    }

    #[test]
    fn wall_boundary_clamps_and_reflects() {
        let mut w = world(1, BoundaryType::Wall);
        let a = w.spawn_atom(ElementId(0), -1.0, 50.0);
        w.atom_mut(a).vx = -2.0;
        physics().apply_boundary(&mut w);
        assert_eq!(w.atom(a).x, 0.0);
        assert_eq!(w.atom(a).vx, 2.0);

        let mut w = world(1, BoundaryType::Wall);
        let b = w.spawn_atom(ElementId(0), 101.0, 50.0);
        physics().apply_boundary(&mut w);
        assert_eq!((w.atom(b).x, w.atom(b).vy), (100.0, 0.0));
    }

    #[test]
    fn open_boundary_kills_and_breaks_bonds_first() {
        let mut w = world(1, BoundaryType::Open);
        let inside = w.spawn_atom(ElementId(1), 50.0, 50.0); // C
        let outside = w.spawn_atom(ElementId(0), -1.0, 50.0); // H
        let bond = w.form_bond(inside, outside, 1, 413.0).unwrap();
        assert_eq!(w.atom(inside).bond_count, 1);
        physics().apply_boundary(&mut w);
        assert!(!w.atom(outside).alive);
        assert!(!w.bond(bond).alive);
        assert!(w.atom(inside).alive);
        assert_eq!(w.atom(inside).bond_count, 0, "inside atom loses the bond");
        // The dead atom's bonds array is also cleared by break_bond.
        assert_eq!(w.atom(outside).bond_count, 0);
    }

    #[test]
    fn same_seed_same_trajectory() {
        fn build_and_run() -> WorldState {
            let mut w = world(7, BoundaryType::Wrap);
            w.temp_field.set(50.0, 50.0, 300.0);
            let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
            let b = w.spawn_atom(ElementId(1), 55.0, 55.0);
            w.form_bond(a, b, 1, 413.0);
            let mut sys = physics();
            for _ in 0..25 {
                sys.update_velocities(&mut w);
                sys.update_positions(&mut w);
                sys.apply_boundary(&mut w);
            }
            w
        }
        let a = build_and_run();
        let b = build_and_run();
        for i in 0..a.atoms.len() {
            assert_eq!(a.atoms[i].x, b.atoms[i].x, "atom {i} x");
            assert_eq!(a.atoms[i].y, b.atoms[i].y, "atom {i} y");
        }
    }

    #[test]
    fn live_bond_on_dead_atom_is_skipped() {
        // Only reachable by constructing an inconsistent state
        // directly (kill an atom without breaking its bond).
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 52.0, 50.0);
        w.form_bond(a, b, 1, 1.0);
        w.atom_mut(a).alive = false;
        physics().update_velocities(&mut w);
        // No panic, and the live atom was not flung by a force
        // against a dead partner (zero temperature: kicks are zero).
        assert_eq!(w.atom(b).vx, 0.0);
        let _ = BondId(0);
    }
}
