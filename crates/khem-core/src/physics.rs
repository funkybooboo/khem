//! The physics system: the Langevin thermostat (velocities relax
//! toward the local field temperature, spec 6.1), the bounded-rate
//! drain of the thermal-release reservoir and temperature diffusion
//! and setpoint relaxation, smooth two-way Hooke bond springs,
//! non-bonded excluded volume, pressure-gradient forces, sub-stepped
//! integration, boundary conditions.
//!
//! Phase-1 placement decisions, to fold into the spec at revision
//! (ADR-0006: specs are drafts until validated):
//!
//! - The release reservoir drain (F19's fix, spec 6.2) runs at the
//!   top of [`PhysicsSystem::apply_bath`], before diffusion: bond-break
//!   heat committed by chemistry enters the temperature field at
//!   `release_rate_cap` per cell per tick instead of as a delta
//!   function (the instantaneous dump was a bomb - see
//!   [`crate::world::WorldState::release_field`]).
//! - Temperature diffusion (spec 6.2) runs at the top of
//!   [`PhysicsSystem::apply_bath`], before the kicks sample the
//!   field; spec 5.1 names no slot for it.
//! - The tick is dt = 1 for chemistry, the thermostat, and the
//!   fields, but the force/integration pair is SUB-STEPPED (spec
//!   6.5, gate K1.3): `integration_substeps` passes per tick at
//!   dt_sub = 1/n. This is the proper fix for the tunneling mint
//!   (F13) - an atom crossing the non-bonded zone in several
//!   sub-steps samples the force symmetrically - so the velocity
//!   clamp is gone, and the spring stability bound is evaluated
//!   at dt_sub, which deepened the bond wells ~n^2: the O-H
//!   mechanical well is ~80 kT (real water's own ratio), and a
//!   thermal kick essentially never stretches a bond to the 7.1
//!   break point.
//! - Every per-atom effect is computed in read-only passes that
//!   accumulate into scratch arrays, then applied in one mutable
//!   pass per sub-step. The compute/apply split is the shape a V2
//!   region-parallel implementation needs, and it keeps each pass
//!   free of tangled borrows (ADR-0005, runtime spec 10.2).
//! - Constants are the spec 11 tuned set (2026-09-07, findings
//!   F1-F18 in docs/research/abstraction-notes.md): every retune
//!   is a measured fix, the passed gates are re-validated in the
//!   commit that moves their operating assumptions, and the
//!   harness is the judge of every further retune.
//!
//! RNG discipline (ADR-0005): this system is the first RNG consumer
//! in the tick (the energy system, step 1, draws nothing). Exactly
//! two `normal` draws per LIVE atom per tick, in AtomId order,
//! once per tick inside [`PhysicsSystem::apply_bath`]; dead atoms
//! draw nothing. The integration sub-steps draw nothing. Every
//! other physics step is deterministic arithmetic with no RNG
//! access.
//!
//! Spec: docs/specs/runtime-spec.md, section 6 (Physics System).

use crate::config::PhysicsConfig;
use crate::observer::Event;
use crate::world::{AtomId, AtomState, BoundaryType, WorldState};

/// The physics system interface, decomposed per the tick order
/// (runtime spec 5.1: apply_bath once per tick, then the
/// update_velocities / update_positions pair repeated
/// `integration_substeps` times, then apply_boundary). v0.1
/// compiles exactly one implementation; the trait exists so
/// future plugin loading does not require restructuring (runtime
/// spec 10.4).
pub trait PhysicsSystem {
    /// Temperature diffusion, setpoint relaxation, and the
    /// Langevin thermal bath (spec 6.1, 6.2) - once per tick,
    /// before the integration sub-steps. The tick's only physics
    /// RNG consumer.
    fn apply_bath(&mut self, world: &mut WorldState);
    /// One integration sub-step's forces (springs, excluded
    /// volume, pressure; spec 6.1, 6.3-6.5) applied to velocities
    /// at dt_sub (F = ma).
    fn update_velocities(&mut self, world: &mut WorldState);
    /// One integration sub-step's position update (spec 6.5) at
    /// dt_sub; the sub-steps sum to dt = 1 per tick.
    fn update_positions(&mut self, world: &mut WorldState);
    /// Boundary normalization (spec 6.7; guarantee G05). Open
    /// worlds break a leaving atom's bonds before it dies and emit
    /// BOND_BROKEN for each (energy_released 0 - no field
    /// exchange), keeping the event stream a complete record of
    /// bond liveness (spec 3.3).
    fn apply_boundary(&mut self, world: &mut WorldState);
}

/// The v0.1 physics implementation (runtime spec 10.4: exactly one).
///
/// Holds only immutable constants plus reusable scratch space; it
/// keeps no state between ticks, so the same instance is safe for
/// every tick of a run.
pub struct Physics {
    config: PhysicsConfig,
    /// Integration time per sub-step: 1 / integration_substeps.
    /// The tick as a whole stays dt = 1 (spec 6.5).
    dt: f32,
    /// Per-atom force accumulators for this sub-step (springs,
    /// excluded volume, pressure).
    fx: Vec<f32>,
    fy: Vec<f32>,
    /// Per-cell scratch for temperature diffusion.
    diffused: Vec<f32>,
}

impl Physics {
    pub fn new(config: PhysicsConfig) -> Self {
        assert!(config.integration_substeps >= 1, "at least one sub-step");
        Self {
            dt: 1.0 / config.integration_substeps as f32,
            config,
            fx: Vec::new(),
            fy: Vec::new(),
            diffused: Vec::new(),
        }
    }

    /// Drain of the thermal-release reservoir (spec 6.2, F19's
    /// fix): each cell moves at most `release_rate_cap` degrees of
    /// committed break-heat into its temperature cell per tick -
    /// the finite thermalization rate that keeps one O-H release
    /// (~139 degrees) from spiking a water-lattice cell into the
    /// p_break runaway (see WorldState::release_field). Runs before
    /// diffusion so the drained heat spreads the same tick.
    fn drain_release_reservoir(&self, world: &mut WorldState) {
        let cap = self.config.release_rate_cap;
        for i in 0..world.release_field.data.len() {
            // Chemistry commits only positive releases; the reservoir
            // holds no other state.
            let drained = world.release_field.data[i].min(cap).max(0.0);
            if drained > 0.0 {
                world.release_field.data[i] -= drained;
                world.temp_field.data[i] += drained;
            }
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
        self.relax_toward_setpoints(world);
    }

    /// Setpoint relaxation (spec 6.2, K1.1): the environment
    /// reservoir. Cells with a declared setpoint (> 0) relax toward
    /// it - the pond's heat sink; a vented Wrap world with no sink
    /// only heats.
    fn relax_toward_setpoints(&self, world: &mut WorldState) {
        let rate = self.config.field_relax_rate;
        if rate <= 0.0 {
            return;
        }
        for i in 0..world.temp_field.data.len() {
            let set = world.setpoint_field.data[i];
            if set > 0.0 {
                world.temp_field.data[i] += (set - world.temp_field.data[i]) * rate;
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
    /// honest energy ledger: the SIGNED kinetic-energy delta
    /// (old minus new) is deposited into the atom's cell. That one
    /// term is the complete exchange - the noise's energy is
    /// inside the new velocity (the field pays for it there), the
    /// damping's removal likewise. A separate "injected" term
    /// double-counts and bleeds the field (found diagnosing the
    /// K1.1 furnace). Net zero at equilibrium; off equilibrium
    /// energy flows field <-> atoms both ways. Draws: exactly two
    /// normals per LIVE atom per tick, unchanged from before.
    fn thermal_bath(&mut self, world: &mut WorldState) {
        let kb = self.config.thermal_kick_scale;
        let gamma = self.config.thermostat_damping;
        let noise = (gamma * (2.0 - gamma)).sqrt();
        let ke_scale = self.config.ke_field_scale;
        for i in 0..world.atoms.len() {
            // Index-based loop: the write below borrows atoms[i]
            // mutably while rng and temp_field are borrowed through
            // their own fields (disjoint borrows). All fields Copy:
            // the destructure copies out; mass comes from the table.
            let AtomState {
                x,
                y,
                vx,
                vy,
                element,
                alive,
                ..
            } = world.atoms[i];
            if !alive {
                continue;
            }
            let mass = world.element(element).mass;
            let t = world.temp_field.get(x, y).max(0.0);
            let s = noise * (kb * t / mass).sqrt();
            let vx_new = vx * (1.0 - gamma) + (world.rng.normal(0.0, s as f64) as f32);
            let vy_new = vy * (1.0 - gamma) + (world.rng.normal(0.0, s as f64) as f32);
            // Signed KE delta = the complete exchange (function doc
            // above); a separate "injected" term double-counts.
            let delta_ke = 0.5 * mass * (vx * vx + vy * vy - vx_new * vx_new - vy_new * vy_new);
            world.temp_field.add(x, y, delta_ke * ke_scale);
            let atom = &mut world.atoms[i];
            atom.vx = vx_new;
            atom.vy = vy_new;
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

    /// Non-bonded excluded volume (finding F4's fix, spec 6.6):
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
            // All fields Copy: the destructure copies out; the
            // force accumulators below mutate while these are held.
            let AtomState {
                id: a_id,
                element: a_el,
                x: ax,
                y: ay,
                alive: a_alive,
                ..
            } = world.atoms[i];
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

impl Physics {
    /// The apply half of the compute/apply split: adds one
    /// sub-step's accumulated forces to each live atom's velocity,
    /// divided by mass (F = ma), scaled by dt_sub.
    ///
    /// The founding spec applied forces without /mass; with
    /// unit-mass atoms hidden in H-H tests, that minted
    /// ~(0.5*m - 1) * F^2 energy per heavy-atom interaction (the
    /// one-water probe isolated it: m=16 O, ~8*F^2 per spring-tick
    /// - the pond furnace). Dead atoms are frozen: nothing in
    ///   this system touches them.
    ///
    /// No velocity clamp: the integrator sub-steps resolve the
    /// fast passes the clamp guarded (F13; spec 6.1) - an atom
    /// crossing the non-bonded zone in several sub-steps samples
    /// the force symmetrically, so no tunneling mint - and the
    /// re-validation contract required the clamp's removal before
    /// the E-gates run their long horizons.
    fn apply_forces(&mut self, world: &mut WorldState) {
        // Split field borrows: the table is read-only while atoms
        // are mutated - disjoint fields of one struct, no Arc clone
        // needed (the clone-per-sub-step was dodging a borrow that
        // does not exist).
        let table = &world.element_table;
        let dt = self.dt;
        for (i, atom) in world.atoms.iter_mut().enumerate() {
            if !atom.alive {
                continue;
            }
            let mass = table[atom.element.0 as usize].mass;
            atom.vx += self.fx[i] / mass * dt;
            atom.vy += self.fy[i] / mass * dt;
        }
    }
}

impl PhysicsSystem for Physics {
    fn apply_bath(&mut self, world: &mut WorldState) {
        self.drain_release_reservoir(world);
        self.diffuse_temperature(world);
        self.thermal_bath(world);
    }

    fn update_velocities(&mut self, world: &mut WorldState) {
        let n = world.atoms.len();
        self.fx.clear();
        self.fy.clear();
        self.fx.resize(n, 0.0);
        self.fy.resize(n, 0.0);
        self.spring_forces(world);
        self.non_bonded_forces(world);
        self.pressure_forces(world);
        self.apply_forces(world);
    }

    fn update_positions(&mut self, world: &mut WorldState) {
        let dt = self.dt;
        for atom in &mut world.atoms {
            if !atom.alive {
                continue;
            }
            // Sub-step position update (spec 6.5); the sub-steps
            // sum to dt = 1 per tick (one tick = one femtosecond at
            // default scale).
            atom.x += atom.vx * dt;
            atom.y += atom.vy * dt;
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
                // Bonds break before the atom dies (spec 6.7);
                // break_bond updates both sides. Each break emits
                // BOND_BROKEN with energy_released 0: the boundary
                // removes the bond with no field exchange (sinks
                // cannot cascade - the same rule as the 7.1
                // mechanical channel), so a stream consumer never
                // carries a ghost bond that died at the edge.
                for id in leaving {
                    let slots = world.atom(id).bonds;
                    for bond in slots.into_iter().flatten() {
                        let (a, b) = {
                            let bond_state = world.bond(bond);
                            (bond_state.atom_a, bond_state.atom_b)
                        };
                        // Midpoint via delta (spec 4.9 rule): raw
                        // displacement in an Open world, so a bond to
                        // a leaving atom reports a midpoint outside
                        // [0, w) - informational, like chemistry's.
                        let (ax, ay) = (world.atom(a).x, world.atom(a).y);
                        let (dx, dy) = world.delta(ax, ay, world.atom(b).x, world.atom(b).y);
                        let (mx, my) = (ax + dx * 0.5, ay + dy * 0.5);
                        if world.break_bond(bond) {
                            world.event_queue.push(Event::BondBroken {
                                tick: world.tick,
                                bond_id: bond.0,
                                elem_a: world.atom(a).element,
                                elem_b: world.atom(b).element,
                                energy_released: 0.0,
                                x: mx,
                                y: my,
                            });
                        }
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
    #[should_panic(expected = "at least one sub-step")]
    fn physics_rejects_zero_substeps() {
        let config = PhysicsConfig {
            integration_substeps: 0,
            ..PhysicsConfig::default()
        };
        let _ = Physics::new(config);
    }

    /// The integration sub-step, dt_sub = 1/integration_substeps,
    /// from the configured constants (the same value Physics::new
    /// derives into its private `dt` field).
    fn dt_sub() -> f32 {
        1.0 / PhysicsConfig::default().integration_substeps as f32
    }

    #[test]
    fn thermal_noise_scales_with_temperature_and_inverse_mass() {
        // Zero temperature: sigma = 0, the atom stays at rest.
        let mut cold = world(1, BoundaryType::Wrap);
        let a = cold.spawn_atom(ElementId(3), 50.0, 50.0); // O
        physics().apply_bath(&mut cold);
        let (vx, vy) = (cold.atom(a).vx, cold.atom(a).vy);
        assert_eq!((vx, vy), (0.0, 0.0));

        // Hot cell: O gets real kicks.
        let mut hot = world(1, BoundaryType::Wrap);
        let b = hot.spawn_atom(ElementId(3), 50.0, 50.0);
        hot.temp_field.set(50.0, 50.0, 400.0);
        physics().apply_bath(&mut hot);
        let dv = hot.atom(b).vx.hypot(hot.atom(b).vy);
        assert!(dv > 0.1, "hot oxygen kick magnitude {dv}");

        // Same temperature, lighter atom (H) -> larger sigma. Same
        // seed, same draw sequence shape.
        let mut light = world(1, BoundaryType::Wrap);
        let c = light.spawn_atom(ElementId(0), 50.0, 50.0);
        light.temp_field.set(50.0, 50.0, 400.0);
        physics().apply_bath(&mut light);
        let dv_light = light.atom(c).vx.hypot(light.atom(c).vy);
        assert!(dv_light > dv, "H kick {dv_light} should exceed O kick {dv}");
    }

    #[test]
    fn stretched_bond_pulls_together() {
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0); // H
        let b = w.spawn_atom(ElementId(0), 52.0, 50.0); // H, r_eq = 1.06
        w.form_bond(a, b, 1, 1.0); // weak spring so the pair does not fly
        // One full tick of integration sub-steps (bath skipped:
        // zero field, no kicks; positions move only under forces).
        let mut sys = physics();
        for _ in 0..PhysicsConfig::default().integration_substeps {
            sys.update_velocities(&mut w);
            sys.update_positions(&mut w);
        }
        assert!(w.atom(a).vx > 0.0, "a should move toward b");
        assert!(w.atom(b).vx < 0.0, "b should move toward a");
        let d = w.atom(b).x - w.atom(a).x;
        assert!(d < 1.99, "distance {d} should shrink");
    }

    #[test]
    fn unbonded_atoms_push_apart_within_cutoff() {
        // F4 law: excluded volume. Two free H 0.5 A apart (cutoff
        // 1.59) push apart; the force is the soft linear core, not
        // the bonded hard core. One sub-step applies F/m * dt_sub:
        // scale * (cutoff - r) * dt_sub = 1.09 * 0.25 ~ 0.27.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 50.5, 50.0);
        // Outside the tick loop the index must be built by hand; in
        // the loop, each sub-step after the first rebuilds it.
        w.spatial_index.rebuild(&w.atoms);
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx < 0.0, "a pushed away");
        assert!(w.atom(b).vx > 0.0, "b pushed away");
        // Magnitude: the sub-step impulse law, F/m * dt_sub.
        let dt = dt_sub();
        assert!(
            (w.atom(b).vx - 1.09 * dt).abs() < 0.05,
            "soft-core sub-step impulse {}",
            w.atom(b).vx
        );

        // Beyond cutoff: no force. The pair sits 1.7 A apart (cutoff
        // 1.59 for H-H) with the index REBUILT, so the candidate
        // query (radius 3.3 A) actually finds the pair and the
        // distance filter rejects it. The previous probe never
        // rebuilt the index, so it passed vacuously - verified by
        // deleting the cutoff filter and watching it stay green.
        let mut w = world(1, BoundaryType::Wrap);
        let c = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let d = w.spawn_atom(ElementId(0), 51.7, 50.0);
        w.spatial_index.rebuild(&w.atoms);
        physics().update_velocities(&mut w);
        assert_eq!(w.atom(c).vx, 0.0, "beyond-cutoff pair must feel nothing");
        assert_eq!(w.atom(d).vx, 0.0);
        assert_eq!(
            (w.atom(c).bond_count, w.atom(d).bond_count),
            (0, 0),
            "distant pair: no forces, no bonds"
        );
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
    fn release_reservoir_drains_at_a_bounded_rate() {
        // F19 law: break heat enters the field through the bounded
        // drain, never as a cell spike. An empty world (no atoms, no
        // sources): the bath's other steps conserve the field sum
        // (diffusion) or touch nothing (no atoms, no setpoints), so
        // the drain is the only mover.
        let config = PhysicsConfig::default();
        let mut w = world(1, BoundaryType::Wrap);
        let committed = 139.0; // one O-H release at fraction 0.3
        w.release_field.set(50.0, 50.0, committed);
        let mut sys = physics();
        sys.apply_bath(&mut w);
        // Exactly the cap moved in one tick (diffusion then spreads
        // the installment - the field SUM is what the drain moved);
        // the rest waits in the reservoir.
        let moved: f32 = w.temp_field.data.iter().sum();
        assert!((moved - config.release_rate_cap).abs() < 1e-4);
        // The installment bounds the cell's excursion: no cell rose
        // more than the cap in one tick (a delta-function dump would
        // have put all 139 in at once). Sustained accumulation in
        // this sink-less test world is honest diffusion physics;
        // the pond's boundedness regression is the harness's.
        let max_cell = w.temp_field.data.iter().copied().fold(0.0f32, f32::max);
        assert!(
            max_cell <= config.release_rate_cap + 1e-4,
            "cell rose to {max_cell} in one tick"
        );
        assert_eq!(
            w.release_field.get(50.0, 50.0),
            committed - config.release_rate_cap
        );
        // The full amount lands after committed/cap ticks; no cell
        // ever spiked - each 2-degree installment diffuses away
        // before the next arrives.
        let ticks = (committed / config.release_rate_cap).ceil() as usize;
        for _ in 1..ticks {
            sys.apply_bath(&mut w);
        }
        let residue: f32 = w.release_field.data.iter().sum();
        assert!(residue.abs() < 1e-3, "residue {residue}");
        let gained: f32 = w.temp_field.data.iter().sum();
        assert!((gained - committed).abs() < 1e-2, "gained {gained}");
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
            sys.apply_bath(&mut w);
        }
        let speed = w.atom(a).vx.hypot(w.atom(a).vy);
        // O at 35 C: sigma = 0.135; thermal speed ~ sqrt(2) * sigma
        // ~ 0.19. No velocity clamp exists anymore (sub-stepping
        // replaced it): 1.0 is a real 7-sigma tail bound, not a
        // clamp value.
        assert!(speed < 1.0, "relaxed speed {speed} should be thermal");
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
            sys.apply_bath(&mut w);
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
        w.temp_field.data.fill(35.0);
        let a = w.spawn_atom(ElementId(3), 50.0, 50.0);
        w.atom_mut(a).vx = 17.0; // far from equilibrium: exchanges flow
        // The exchange rate: depositing X KE units raises the field
        // X * ke_field_scale degrees, so the invariant is
        // field_degrees + KE * ke_field_scale (the KE * ke_scale
        // form; dividing is the inverted-scale mistake).
        let total = |w: &WorldState| -> f64 {
            let field: f32 = w.temp_field.data.iter().sum();
            field as f64 + w.kinetic_energy() * config.ke_field_scale as f64
        };
        let before = total(&w);
        let mut sys = Physics::new(config);
        for _ in 0..500 {
            sys.apply_bath(&mut w);
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
        // discontinuity, no cannon. One sub-step applies at most
        // the analytic single-SUB-step impulse k * r_eq * dt_sub
        // per atom (the K1.2 harness probe pins the per-tick
        // bound).
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        let b = w.spawn_atom(ElementId(0), 50.1, 50.0);
        w.form_bond(a, b, 1, 436.0); // k = 13.95; F at r=0.1 ~ -13.4
        physics().update_velocities(&mut w);
        assert!(w.atom(a).vx < 0.0, "compressed bond pushes a away");
        assert!(w.atom(b).vx > 0.0, "compressed bond pushes b away");
        let bound = 436.0 * PhysicsConfig::default().spring_energy_scale * 1.06 * dt_sub(); // k*r_eq*dt
        assert!(
            w.atom(b).vx <= bound,
            "compression impulse {} exceeds k*r_eq*dt_sub {}",
            w.atom(b).vx,
            bound
        );
    }

    #[test]
    fn bonded_pair_across_wrap_seam_is_not_shredded() {
        // Minimum-image convention: two atoms 1 A apart ACROSS the
        // wrap seam must feel the spring for 1 A, not 99 A. The
        // pair sits slightly compressed (1.00 < r_eq 1.06), so the
        // honest law is a mild REPULSION apart the short way - a
        // broken min-image reads the raw 99 A delta as a huge
        // stretch and shreds the pair (measured pre-F10: +-103
        // A/tick). The pond starts with seam-straddling waters, so
        // this law is load-bearing for K1, not a corner case.
        let mut w = world(1, BoundaryType::Wrap);
        let a = w.spawn_atom(ElementId(0), 0.5, 50.0);
        let b = w.spawn_atom(ElementId(0), 99.5, 50.0); // 1 A apart across the seam
        w.form_bond(a, b, 1, 436.0);
        physics().update_velocities(&mut w);
        let (va, vb) = (w.atom(a).vx, w.atom(b).vx);
        // Compressed: b sits 1 A to a's LEFT across the seam, so
        // the repulsion pushes a +x and b -x - apart the short way.
        assert!(
            va > 0.0 && vb < 0.0,
            "seam pair must push apart the short way, got {va} / {vb}"
        );
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
        physics().apply_bath(&mut w);
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
        // One sub-step moves v * dt_sub; a full tick of sub-steps
        // sums back to dt = 1 (spec 6.5).
        let mut sys = physics();
        sys.update_positions(&mut w);
        let dt = dt_sub();
        assert_eq!(
            (w.atom(a).x, w.atom(a).y),
            (50.0 + 1.5 * dt, 50.0 - 0.5 * dt)
        );
        let substeps = PhysicsConfig::default().integration_substeps;
        for _ in 1..substeps {
            sys.update_positions(&mut w);
        }
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
        // Spec 3.3: BOND_BROKEN for every break, boundary removals
        // included - and with no field exchange.
        match w.event_queue.first() {
            Some(Event::BondBroken {
                bond_id,
                energy_released,
                ..
            }) => {
                assert_eq!(*bond_id, bond.0);
                assert_eq!(*energy_released, 0.0);
            }
            other => panic!("expected BondBroken, got {other:?}"),
        }
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
                // The tick shape: bath once, sub-steps after.
                sys.apply_bath(&mut w);
                for _ in 0..PhysicsConfig::default().integration_substeps {
                    sys.update_velocities(&mut w);
                    sys.update_positions(&mut w);
                }
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
        // Forces only (the bath is a separate step; the frozen field
        // would kick nothing anyway).
        physics().update_velocities(&mut w);
        // No panic, and the live atom was not flung by a force
        // against a dead partner.
        assert_eq!(w.atom(b).vx, 0.0);
        assert_eq!(w.bond(BondId(0)).atom_b, b, "the bond still points at b");
    }
}
