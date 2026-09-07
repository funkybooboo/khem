//! The physics system: the Langevin thermostat (velocities relax
//! toward the local field temperature, spec 6.1), temperature
//! diffusion, bond spring forces with a bounded soft core,
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
//! - Constants are the spec 11 tuned set (2026-09-05, findings
//!   F1-F9 in docs/research/abstraction-notes.md); the thermostat
//!   (F8) and the bounded soft core (F9) are measured fixes, and
//!   the K1 harness is the judge of every further retune.
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
        // Setpoint relaxation (spec 6.2, K1.1): the environment
        // reservoir. Cells with a declared setpoint (> 0) relax
        // toward it - the pond's heat sink; a vented Wrap world
        // with no sink only heats.
        let rate = self.config.field_relax_rate;
        if rate > 0.0 {
            let field = &mut world.temp_field;
            for i in 0..field.data.len() {
                let set = world.setpoint_field.data[i];
                if set > 0.0 {
                    field.data[i] += (set - field.data[i]) * rate;
                }
            }
        }
    }

    /// Spec 6.1: the Langevin thermostat (finding F8's fix).
    ///
    /// `v_new = v * (1 - damping) + sqrt(damping*(2-damping)) *
    /// sigma(T) * normal(0,1)` per component, sigma(T) =
    /// sqrt(thermal_kick_scale * T / mass) - so velocities relax
    /// to the local field temperature with the correct stationary
    /// variance instead of random-walking upward forever. T <= 0
    /// clamps to zero (negative field: sigma 0, pure damping).
    ///
    /// Fluctuation-dissipation bookkeeping keeps the field the
    /// honest energy ledger: the damping DEPOSITS the kinetic
    /// energy it removed into the atom's cell (fast atoms heat
    /// their surroundings - the F6 fix), and the noise DRAINS its
    /// expected injection back out (`mass * s^2`, both components).
    /// Net zero at equilibrium; off equilibrium energy flows
    /// field <-> atoms both ways. Draws: exactly two normals per
    /// LIVE atom per tick, unchanged from before.
    fn thermal_bath(&mut self, world: &mut WorldState) {
        let kb = self.config.thermal_kick_scale;
        let gamma = self.config.thermostat_damping;
        let noise = (gamma * (2.0 - gamma)).sqrt();
        let ke_scale = self.config.ke_field_scale;
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
            let s = noise * (kb * t / mass).sqrt();
            let vx_new = atom.vx * (1.0 - gamma) + (world.rng.normal(0.0, s as f64) as f32);
            let vy_new = atom.vy * (1.0 - gamma) + (world.rng.normal(0.0, s as f64) as f32);
            // The signed KE delta is the COMPLETE exchange: the
            // noise energy is already inside vx_new (the field pays
            // for it there), the damping removal likewise. A
            // separate "injected" term double-counts and bleeds the
            // field - found while diagnosing the furnace; the old
            // drift test's tolerance masked the bleed.
            let delta_ke = 0.5
                * mass
                * (atom.vx * atom.vx + atom.vy * atom.vy - vx_new * vx_new - vy_new * vy_new);
            world.temp_field.add(atom.x, atom.y, delta_ke * ke_scale);
            self.kx.push(vx_new);
            self.ky.push(vy_new);
        }
    }

    /// Spec 6.3 (revised 2026-09-05): ONE smooth Hooke law, both
    /// directions - `F = bond.energy * spring_energy_scale *
    /// (r - r_eq)` toward `r_eq = radius_a + radius_b`, applied to
    /// both atoms along the bond axis, equal and opposite.
    /// Attractive stretched, repulsive compressed, bounded at
    /// `k * r_eq` near coincidence. The founding spec's separate
    /// hard core (`-strong_repulsion / r^2` below `0.5 * r_eq`) was
    /// a force DISCONTINUITY that symplectic Euler pumped into
    /// runaway oscillation every time a thermal kick carried an
    /// atom through it - the measured K1 furnace ignition (F9,
    /// revised: not a cannon to cap, a core to remove). Coincident
    /// atoms (r ~ 0) have no defined axis; skipped, and the next
    /// kick separates them.
    fn spring_forces(&mut self, world: &mut WorldState) {
        let scale = self.config.spring_energy_scale;
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
            let f = bond.energy * scale * (r - r_eq);
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
    /// Non-bonded excluded volume (finding F4's fix, spec 6.5):
    /// every UNBONDED live pair closer than its cutoff
    /// `(radius_a + radius_b) * non_bonded_margin` is pushed apart
    /// with `F = non_bonded_repulsion * (cutoff - r)` along the
    /// minimum-image axis, equal and opposite. Bonded pairs are
    /// excluded (springs own them); same-molecule unbonded pairs
    /// (1,3 and beyond) are NOT excluded - real sterics, mild by
    /// construction since VSEPR ideals keep most beyond cutoff.
    /// Coincident unbonded atoms are skipped (no defined axis;
    /// kicks separate them). Pairs are visited once (lower AtomId
    /// iterates), candidates via the wrap-aware index (F11).
    fn non_bonded_forces(&mut self, world: &mut WorldState) {
        let strength = self.config.non_bonded_repulsion;
        let margin = self.config.non_bonded_margin;
        let max_radius = world
            .element_table
            .iter()
            .map(|e| e.radius)
            .fold(0.0f32, f32::max);
        for i in 0..world.atoms.len() {
            let (a_id, a_el, ax, ay, a_alive) = {
                let a = &world.atoms[i];
                (a.id, a.element, a.x, a.y, a.alive)
            };
            if !a_alive {
                continue;
            }
            let r_a = world.element(a_el).radius;
            let query = (r_a + max_radius) * margin;
            let candidates = world.spatial_index.neighbors(ax, ay, query);
            for b_id in candidates {
                if b_id.0 <= a_id.0 {
                    continue;
                }
                let (b_el, bx, by, b_alive) = {
                    let b = world.atom(b_id);
                    (b.element, b.x, b.y, b.alive)
                };
                if !b_alive || world.is_bonded(a_id, b_id) {
                    continue;
                }
                let (dx, dy) = world.delta(ax, ay, bx, by);
                let r2 = dx * dx + dy * dy;
                if r2 < f32::EPSILON {
                    continue;
                }
                let r = r2.sqrt();
                let cutoff = (r_a + world.element(b_el).radius) * margin;
                if r >= cutoff {
                    continue;
                }
                let f = -strength * (cutoff - r);
                let (ux, uy) = (dx / r, dy / r);
                self.fx[i] += f * ux;
                self.fy[i] += f * uy;
                self.fx[b_id.0 as usize] -= f * ux;
                self.fy[b_id.0 as usize] -= f * uy;
            }
        }
    }
}

impl PhysicsSystem for Physics {
    fn update_velocities(&mut self, world: &mut WorldState) {
        let n = world.atoms.len();
        self.diffuse_temperature(world);
        self.thermal_bath(world);
        self.fx.clear();
        self.fy.clear();
        self.fx.resize(n, 0.0);
        self.fy.resize(n, 0.0);
        self.spring_forces(world);
        self.non_bonded_forces(world);
        self.pressure_forces(world);
        // Mass lookup without borrowing world against the mutable
        // atom loop (Arc clone, cheap).
        let table = world.element_table.clone();
        let vmax = self.config.max_atom_speed;
        for (i, atom) in world.atoms.iter_mut().enumerate() {
            // Dead atoms are frozen: nothing in this system touches
            // them (kx/ky carry 0 for them; writing it would zero
            // their state).
            if !atom.alive {
                continue;
            }
            // kx/ky carry the COMPLETE post-bath velocity (the
            // thermostat consumes the old velocity); fx/fy are
            // FORCES, divided by mass here (F = ma). Set, not add -
            // adding would compound the velocity every tick. The
            // founding spec applied forces without /mass; with
            // unit-mass atoms hidden in H-H tests, that minted
            // ~(0.5*m - 1) * F^2 energy per heavy-atom interaction
            // (the one-water probe isolated it: m=16 O, ~8*F^2 per
            // spring-tick - the pond furnace).
            let mass = table[atom.element.0 as usize].mass;
            let (vx, vy) = (
                self.kx[i] + self.fx[i] / mass,
                self.ky[i] + self.fy[i] / mass,
            );
            // Numerical guard (config doc): unresolved fast passes
            // through the short-range forces mint energy; clamp the
            // speed and deposit what the clamp removed so the
            // ledger stays exact.
            let speed2 = vx * vx + vy * vy;
            let (vx, vy) = if speed2 > vmax * vmax {
                let scale = vmax / speed2.sqrt();
                let removed = 0.5 * mass * (speed2 - vmax * vmax);
                world
                    .temp_field
                    .add(atom.x, atom.y, removed * self.config.ke_field_scale);
                (vx * scale, vy * scale)
            } else {
                (vx, vy)
            };
            atom.vx = vx;
            atom.vy = vy;
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
    fn unbonded_atoms_push_apart_within_cutoff() {
        // F4 law: excluded volume. Two free H 0.5 A apart (cutoff
        // 1.59) push apart; the force is the soft linear core, not
        // the bonded hard core.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 50.5, 50.0);
        // Outside the tick loop the index must be built by hand; in
        // the loop, update_velocities consumes the previous tick's
        // rebuild (read-previous-tick state).
        w.spatial_index.rebuild(&w.atoms);
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx < 0.0, "a pushed away");
        assert!(w.atom(b).vx > 0.0, "b pushed away");
        // Magnitude: scale * (cutoff - r) = 1.0 * (1.59 - 0.5) ~ 1.09.
        assert!(
            (w.atom(b).vx - 1.09).abs() < 0.15,
            "soft-core magnitude {}",
            w.atom(b).vx
        );

        // Beyond cutoff: no force.
        let mut w = world(1, BoundaryType::Wrap);
        let c = w.spawn_atom(ElementId(0), 50.0, 50.0);
        w.spawn_atom(ElementId(0), 55.0, 50.0);
        physics().update_velocities(&mut w);
        assert_eq!(w.atom(c).vx, 0.0);
        let _ = b;
    }

    #[test]
    fn bonded_pairs_are_exempt_from_excluded_volume() {
        // A bonded pair AT equilibrium distance feels no spring and
        // must feel no non-bonded push either, even though the
        // distance is inside the non-bonded cutoff (1.06 < 1.59).
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 51.06, 50.0); // r_eq
        w.form_bond(a, b, 1, 436.0);
        w.spatial_index.rebuild(&w.atoms);
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx.abs() < 1e-3, "no net force at r_eq");
        assert!(w.atom(b).vx.abs() < 1e-3);
    }

    #[test]
    fn free_atoms_cannot_penetrate_an_intact_water() {
        // The foundation behavior K2 builds on: a free atom landing
        // inside a molecule's excluded volume is pushed out, while
        // the water's own geometry is untouched (its H-O pairs are
        // bonded, exempt).
        let mut w = world(1, BoundaryType::Wrap);
        let o = w.spawn_atom(ElementId(3), 50.0, 50.0);
        let h1 = w.spawn_atom(ElementId(0), 48.81, 50.0);
        let h2 = w.spawn_atom(ElementId(0), 51.19, 50.0);
        w.form_bond(o, h1, 1, 463.0);
        w.form_bond(o, h2, 1, 463.0);
        // A free O squeezed between the hydrogens.
        let probe = w.spawn_atom(ElementId(3), 50.0, 51.0);
        w.spatial_index.rebuild(&w.atoms);
        physics().update_velocities(&mut w);
        let pushed = w.atom(probe).vy;
        assert!(
            pushed > 0.0,
            "probe pushed away from the water, got {pushed}"
        );
        // Equal and opposite: the water O reacts downward (its own
        // H-O pairs are bonded and exempt; only the probe pushes
        // it).
        assert!(
            w.atom(o).vy < 0.0,
            "water O reacts opposite, got {}",
            w.atom(o).vy
        );
    }

    #[test]
    fn unbonded_seam_pair_pushes_through_the_wrap() {
        // F4 + F11 together: two free atoms 0.5 A apart ACROSS the
        // seam are found by the wrapped index and pushed apart the
        // short way.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 0.2, 50.0);
        let b = w.spawn_atom(ElementId(0), 99.7, 50.0);
        w.spatial_index.rebuild(&w.atoms);
        let mut sys = physics();
        sys.update_velocities(&mut w);
        // In a's frame b sits 0.5 A to the LEFT (across the seam),
        // so a is pushed +x and b -x: apart the short way.
        assert!(
            w.atom(a).vx > 0.0,
            "seam a pushed short way, got {}",
            w.atom(a).vx
        );
        assert!(
            w.atom(b).vx < 0.0,
            "seam b pushed short way, got {}",
            w.atom(b).vx
        );
    }

    #[test]
    fn thermostat_relaxes_velocity_to_field_temperature() {
        // F8 law: a hot atom cools to the thermal scale of its
        // cell; the velocity does not random-walk upward forever.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(3), 50.0, 50.0);
        w.atom_mut(a).vx = 50.0;
        w.temp_field.set(50.0, 50.0, 35.0);
        let mut sys = physics();
        for _ in 0..500 {
            sys.update_velocities(&mut w);
        }
        let speed = w.atom(a).vx.hypot(w.atom(a).vy);
        // O at 35 C: sigma = 0.135; thermal speed ~ sqrt(2) * sigma
        // ~ 0.19. Band 2.0 leaves generous statistical room.
        assert!(speed < 2.0, "relaxed speed {speed} should be thermal");
    }

    #[test]
    fn thermostat_moves_kinetic_energy_into_the_field() {
        // The deposit half of fluctuation-dissipation (F6's fix): a
        // moving atom in a frozen cell heats it; the atom slows.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(3), 50.0, 50.0);
        w.atom_mut(a).vx = 20.0;
        let mut sys = physics();
        for _ in 0..50 {
            sys.update_velocities(&mut w);
        }
        let cell = w.temp_field.get(50.0, 50.0);
        assert!(cell > 0.0, "atom KE must warm the frozen cell, got {cell}");
        assert!(w.atom(a).vx < 20.0, "damping must slow the atom");
    }

    #[test]
    fn thermostat_exchanges_conserve_field_plus_kinetic_energy() {
        // The exact bookkeeping law: field_sum + KE / ke_field_scale
        // is invariant through the bath (springs and non-bonded
        // forces off, so KE is the only other reservoir). The
        // previous tolerance-based drift test masked the
        // double-count bleed; this form cannot.
        let config = PhysicsConfig {
            non_bonded_repulsion: 0.0,
            ..PhysicsConfig::default()
        };
        let mut w = world(2, BoundaryType::Wrap);
        for v in w.temp_field.data.iter_mut() {
            *v = 35.0;
        }
        let a = w.spawn_atom(ElementId(3), 50.0, 50.0);
        w.atom_mut(a).vx = 17.0; // far from equilibrium: exchanges flow
        // The exchange rate: depositing X KE units raises the field
        // X * ke_field_scale degrees, so the invariant is
        // field_degrees + KE * ke_field_scale (the KE * ke_scale
        // form; dividing is the inverted-scale mistake).
        let total = |w: &WorldState| -> f64 {
            let field: f32 = w.temp_field.data.iter().sum();
            let ke: f64 = w
                .atoms
                .iter()
                .filter(|at| at.alive)
                .map(|at| {
                    let m = w.element(at.element).mass as f64;
                    0.5 * m * (at.vx as f64).powi(2) + 0.5 * m * (at.vy as f64).powi(2)
                })
                .sum();
            field as f64 + ke * config.ke_field_scale as f64
        };
        let before = total(&w);
        let mut sys = Physics::new(config);
        for _ in 0..500 {
            sys.update_velocities(&mut w);
        }
        let after = total(&w);
        // Tolerance is f32 ledger accumulation over 500 ticks
        // (~2e-4 per add on degree-magnitude sums), plus the
        // unbooked micro-work of the pressure mean field; the law
        // itself is exact in the exchange terms.
        assert!(
            (after - before).abs() < 0.1,
            "field + KE*ke_scale changed by {}",
            after - before
        );
    }

    #[test]
    fn bonded_compression_is_smooth_and_bounded() {
        // F9 law (revised): compression repels through the same
        // smooth Hooke term, bounded by k * r_eq - no hard core, no
        // discontinuity, no cannon.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 50.1, 50.0);
        w.form_bond(a, b, 1, 436.0); // k = 0.872; F at r=0.1 ~ -0.86
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx < 0.0, "compressed bond pushes a away");
        assert!(w.atom(b).vx > 0.0, "compressed bond pushes b away");
        let bound = 436.0 * PhysicsConfig::default().spring_energy_scale * 1.06; // k * r_eq
        assert!(
            w.atom(b).vx <= bound,
            "compression force {} exceeds k*r_eq {}",
            w.atom(b).vx,
            bound
        );
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
