//! The energy system: hydrothermal vents and solar UV, field updates,
//! convection. Guarantee G06: energy sources are the only energy
//! inputs to the world - without sources, no field ever gains energy.
//!
//! Phase-1 scope decisions, to fold into the spec at revision
//! (ADR-0006):
//!
//! - This system writes fields and atom velocities only. UV bond
//!   breaking (spec 8.2) executes in the chemistry system's
//!   break_bonds step; the tick order (5.1) gives chemistry the only
//!   bond-mutating steps.
//! - Radiation sources have no defined behavior in the v0.1 spec;
//!   they are ignored (not an error), so a world can carry them.
//! - Spec 8.3 (energy tracking: total kinetic, bond, and field
//!   energy) is a diagnostic, not part of the NDJSON v:1 schema
//!   (spec 3.3 has no such fields). The K1 harness computes the sums
//!   directly from WorldState; whether the schema grows them is a
//!   contract decision made when a consumer needs them.
//!
//! RNG discipline (ADR-0005): this system consumes no randomness.
//! The physics system is the tick's first RNG consumer.
//!
//! Spec: docs/specs/runtime-spec.md, section 8 (Energy System).

use crate::config::PhysicsConfig;
use crate::world::WorldState;

/// The three source kinds the world vocabulary allows (language spec
/// section 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Hydrothermal,
    SolarUv,
    Radiation,
}

/// One energy source (runtime spec 8.1, 8.2).
#[derive(Debug, Clone, Copy)]
pub struct EnergySource {
    pub kind: SourceKind,
    /// Position, angstroms; meaningful for point sources.
    pub position: (f32, f32),
    /// Normalized intensity, 0.0 - 1.0.
    pub intensity: f32,
    /// Radius, angstroms; point sources.
    pub radius: f32,
    /// Solar UV only; defaults to true (language spec V-WORLD-08).
    pub surface_only: bool,
}

impl EnergySource {
    pub fn hydrothermal(position: (f32, f32), intensity: f32, radius: f32) -> Self {
        Self {
            kind: SourceKind::Hydrothermal,
            position,
            intensity,
            radius,
            surface_only: false,
        }
    }

    pub fn solar_uv(intensity: f32, surface_only: bool) -> Self {
        Self {
            kind: SourceKind::SolarUv,
            position: (0.0, 0.0),
            intensity,
            radius: 0.0,
            surface_only,
        }
    }
}

/// The energy system interface (runtime spec 10.4; tick order step
/// 1). v0.1 compiles exactly one implementation.
pub trait EnergySystem {
    fn update(&mut self, world: &mut WorldState);
}

/// The v0.1 energy implementation (runtime spec 10.4: exactly one).
pub struct Energy {
    config: PhysicsConfig,
}

impl Energy {
    pub fn new(config: PhysicsConfig) -> Self {
        Self { config }
    }

    /// Spec 8.1: heat cells within the vent radius by
    /// `intensity * falloff * vent_heat_rate`, where
    /// `falloff = 1 / (1 + d^2 / radius^2)`; lift atoms in radius by
    /// `convection_rate * falloff`. Plain Euclidean distance; vent
    /// influence does not wrap at world edges.
    fn apply_hydrothermal(&self, world: &mut WorldState, source: EnergySource) {
        let (px, py) = source.position;
        let r2 = source.radius * source.radius;
        let field = &mut world.temp_field;
        let (cols, rows) = (field.cols as usize, field.rows as usize);
        for row in 0..rows {
            for col in 0..cols {
                let cx = (col as f32 + 0.5) * field.cell_width;
                let cy = (row as f32 + 0.5) * field.cell_height;
                let d2 = (cx - px) * (cx - px) + (cy - py) * (cy - py);
                if d2 > r2 {
                    continue;
                }
                let falloff = 1.0 / (1.0 + d2 / r2);
                field.data[col + row * cols] +=
                    source.intensity * falloff * self.config.vent_heat_rate;
            }
        }
        for atom in &mut world.atoms {
            if !atom.alive {
                continue;
            }
            let d2 = (atom.x - px) * (atom.x - px) + (atom.y - py) * (atom.y - py);
            if d2 > r2 {
                continue;
            }
            let falloff = 1.0 / (1.0 + d2 / r2);
            // +y is toward the surface (surface cells sit at high y),
            // so convection lifts.
            atom.vy += self.config.convection_rate * falloff;
        }
    }

    /// Spec 8.2: surface cells (center above
    /// `surface_threshold * height`) carry the UV intensity for this
    /// tick; all other cells are zero. The field is per-tick state,
    /// rebuilt every update, so stale values cannot persist.
    fn apply_solar_uv(&self, world: &mut WorldState, source: EnergySource) {
        let limit = self.config.surface_threshold * world.height;
        let field = &mut world.uv_field;
        let (cols, rows) = (field.cols as usize, field.rows as usize);
        for row in 0..rows {
            let cy = (row as f32 + 0.5) * field.cell_height;
            let value = if cy > limit { source.intensity } else { 0.0 };
            for col in 0..cols {
                field.data[col + row * cols] = value;
            }
        }
    }
}

impl EnergySystem for Energy {
    fn update(&mut self, world: &mut WorldState) {
        // Per-tick fields: UV is rebuilt wholesale by solar sources,
        // so reset it first (no source leaves it zeroed).
        world.uv_field.data.fill(0.0);
        for i in 0..world.energy_sources.len() {
            let source = world.energy_sources[i];
            match source.kind {
                SourceKind::Hydrothermal => self.apply_hydrothermal(world, source),
                SourceKind::SolarUv => self.apply_solar_uv(world, source),
                // No defined behavior in the v0.1 spec; ignored.
                SourceKind::Radiation => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{BoundaryType, ElementId};

    fn world(seed: u64) -> WorldState {
        WorldState::new(
            100.0,
            100.0,
            BoundaryType::Wrap,
            seed,
            PhysicsConfig::default(),
        )
    }

    fn energy() -> Energy {
        Energy::new(PhysicsConfig::default())
    }

    #[test]
    fn constructors() {
        let v = EnergySource::hydrothermal((100.0, 0.0), 0.8, 15.0);
        assert_eq!(v.kind, SourceKind::Hydrothermal);
        assert!(!v.surface_only);
        let s = EnergySource::solar_uv(0.7, true);
        assert_eq!(s.kind, SourceKind::SolarUv);
        assert!(s.surface_only);
    }

    #[test]
    fn vent_heats_cells_within_radius_by_falloff() {
        let mut w = world(1);
        // 10 A cells; vent at (55, 55), a cell center, radius 15.
        w.energy_sources
            .push(EnergySource::hydrothermal((55.0, 55.0), 0.8, 15.0));
        energy().update(&mut w);
        // Sample by cell center (5 A offset in a 10 A grid).
        let center = w.temp_field.get(55.0, 55.0); // cell (5, 5): vent sits here
        let near = w.temp_field.get(65.0, 55.0); // cell (6, 5): d = 10
        let far = w.temp_field.get(25.0, 55.0); // cell (2, 5): d = 30 > radius
        assert!(center > near, "center {center} should exceed near {near}");
        assert!(near > 0.0, "in-radius cells must gain heat, got {near}");
        assert_eq!(far, 0.0, "out-of-radius cells must be untouched");
        // Exact value at the vent's own cell: intensity * 1.0 * vent_heat_rate.
        assert!((center - 0.08).abs() < 1e-6, "center {center}");
    }

    #[test]
    fn convection_lifts_atoms_within_radius() {
        let mut w = world(1);
        w.energy_sources
            .push(EnergySource::hydrothermal((50.0, 50.0), 0.8, 15.0));
        let lifted = w.spawn_atom(ElementId(0), 50.0, 52.0);
        let outside = w.spawn_atom(ElementId(0), 90.0, 90.0);
        energy().update(&mut w);
        assert!(w.atom(lifted).vy > 0.0, "atom in radius should rise");
        assert_eq!(w.atom(outside).vy, 0.0);
    }

    #[test]
    fn solar_uv_sets_surface_cells_only() {
        let mut w = world(1);
        w.energy_sources.push(EnergySource::solar_uv(0.7, true));
        energy().update(&mut w);
        // surface_threshold 0.9 of 100 A: cells centered above y = 90.
        let top = w.uv_field.get(50.0, 95.0);
        let bottom = w.uv_field.get(50.0, 15.0);
        assert_eq!(top, 0.7);
        assert_eq!(bottom, 0.0);
        // Only the last rows qualify: 10x10 grid, row 9 (center 95).
        let count = w.uv_field.data.iter().filter(|v| **v == 0.7).count();
        assert_eq!(count, 10, "exactly one row of cells is surface");
    }

    #[test]
    fn uv_field_is_per_tick_state() {
        let mut w = world(1);
        w.energy_sources.push(EnergySource::solar_uv(0.7, true));
        energy().update(&mut w);
        assert_eq!(w.uv_field.get(50.0, 95.0), 0.7);
        // Remove the source: next update must zero the field again.
        w.energy_sources.clear();
        energy().update(&mut w);
        assert_eq!(w.uv_field.get(50.0, 95.0), 0.0);
    }

    #[test]
    fn no_sources_no_energy_input() {
        // G06: without sources, no field gains energy.
        let mut w = world(1);
        let temp: Vec<f32> = w.temp_field.data.clone();
        let uv: Vec<f32> = w.uv_field.data.clone();
        energy().update(&mut w);
        assert_eq!(w.temp_field.data, temp);
        assert_eq!(w.uv_field.data, uv);
        assert!(w.uv_field.data.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn radiation_sources_are_ignored() {
        let mut w = world(1);
        w.energy_sources.push(EnergySource {
            kind: SourceKind::Radiation,
            position: (50.0, 50.0),
            intensity: 0.5,
            radius: 10.0,
            surface_only: false,
        });
        let a = w.spawn_atom(ElementId(0), 50.0, 50.0);
        energy().update(&mut w);
        assert_eq!(w.atom(a).vy, 0.0);
        assert!(w.temp_field.data.iter().all(|v| *v == 0.0));
    }
}
