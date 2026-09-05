//! Tunable constants (runtime spec section 11).
//!
//! Phase 1 hardcodes [`PhysicsConfig::default`] everywhere; loading
//! physics.cfg from the project root arrives later. Tuning affects
//! behavior and stability - it never changes what chemistry is
//! possible.

/// UV break probability per bond order (runtime spec 8.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvSensitivity {
    pub single: f32,
    pub double: f32,
    pub triple: f32,
}

impl UvSensitivity {
    /// Sensitivity for a bond order (1, 2, 3). Anything else is not a
    /// bond and cannot break: 0.0.
    pub fn of_order(self, order: u8) -> f32 {
        match order {
            1 => self.single,
            2 => self.double,
            3 => self.triple,
            _ => 0.0,
        }
    }
}

/// Every tunable constant from runtime spec section 11.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsConfig {
    /// Boltzmann constant for THERMAL BREAKING (spec 7.1), scaled to
    /// sim units. Retuned 2026-09-05 from the literal 0.008314: with
    /// the physical kB, p_break = exp(-E/(kB*T)) is ~1e-76 for every
    /// bond at pond temperatures - the measured K1 harness ran 2000
    /// ticks with zero breaks (frozen inertness, finding F1). At
    /// 0.45 the phenomenology straddles the table: weak bonds (O-O
    /// 146) break at 35 C every ~10k ticks, water's O-H (463)
    /// essentially never - which is real chemistry's answer too.
    /// Note the split from thermal_kick_scale: one physical kB set
    /// two incompatible sim scales (breaking rate and kick
    /// magnitude), so they decouple as of the tuning commit.
    pub kb_scaled: f32,
    /// Velocity-kick scale (spec 6.1: sigma = sqrt(k * T / mass)).
    /// Kept at the literal Boltzmann value when kb_scaled retuned:
    /// kicks must stay small against bond lengths, a different
    /// constraint than breaking rates. Tuning knob.
    pub thermal_kick_scale: f32,
    pub diffusion_rate: f32,
    pub pressure_sensitivity: f32,
    /// Spring constant scale: k = bond.energy * scale (spec 6.3).
    /// Retuned 2026-09-05 from 0.01: the tick integrator is
    /// semi-implicit Euler, stable for dt * sqrt(k) < 2; at 0.01
    /// every bond over 400 kJ/mol violated the bound (O-H 463:
    /// 2.15) and the measured K1 harness KE reached 4.4e13 in 2000
    /// ticks (finding F2). At 0.002 the strongest tabulated bond
    /// (N#N 945) gives 1.375, inside the bound with margin.
    pub spring_energy_scale: f32,
    pub strong_repulsion: f32,
    pub convection_rate: f32,
    /// Vent heat injection per tick (spec 8.1). Phase-1 addition:
    /// the spec's formula uses this constant but section 11 omits
    /// it; revision adds it here. Pure tuning knob.
    pub vent_heat_rate: f32,
    /// Bond-formation search radius, angstroms.
    pub bond_search_radius: f32,
    /// Per eligible pair per tick.
    pub base_formation_rate: f32,
    /// Fraction of bond energy released into the temperature field
    /// on breaking (spec 7.1). Set equal to formation_fraction in
    /// the 2026-09-05 tuning: the spec's 0.5/0.3 asymmetry created
    /// energy - a form+break cycle deposited 0.2 * E into the field
    /// from nowhere (finding F7). Equal fractions conserve through
    /// the cycle; the field still differs because kinetics differ.
    pub release_fraction: f32,
    pub formation_fraction: f32,
    pub en_bonus: f32,
    /// Angular tolerance of the VSEPR geometry factor, degrees. v0
    /// fill: spec 7.2 requires a geometry factor but names no
    /// tolerance. Tuning knob.
    pub geometry_sigma: f32,
    /// Temperature-factor peak scale: T_opt = t_opt_scale *
    /// bond_energy. v0 fill: spec 7.2 says "gaussian peaked at an
    /// optimal temperature for the element pair" but defines no
    /// optimum; stronger bonds tolerate hotter formation. Tuning
    /// knob.
    pub t_opt_scale: f32,
    /// Temperature-factor gaussian width, degrees. v0 fill as
    /// above. Tuning knob.
    pub t_width: f32,
    /// Spatial hash cell size, angstroms.
    pub spatial_cell_size: f32,
    /// Field grid cell size, angstroms.
    pub field_cell_size: f32,
    /// Ticks between dead-entity compactions.
    pub compaction_interval: u32,
    /// Fraction of world height counted as surface for solar UV.
    pub surface_threshold: f32,
    pub uv_sensitivity: UvSensitivity,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            kb_scaled: 0.45,
            thermal_kick_scale: 0.008314,
            diffusion_rate: 0.1,
            pressure_sensitivity: 0.01,
            spring_energy_scale: 0.002,
            strong_repulsion: 1000.0,
            convection_rate: 0.001,
            vent_heat_rate: 0.1,
            bond_search_radius: 4.0,
            base_formation_rate: 0.001,
            release_fraction: 0.3,
            formation_fraction: 0.3,
            en_bonus: 0.1,
            geometry_sigma: 30.0,
            t_opt_scale: 0.1,
            t_width: 20.0,
            spatial_cell_size: 5.0,
            field_cell_size: 10.0,
            compaction_interval: 10_000,
            surface_threshold: 0.9,
            uv_sensitivity: UvSensitivity {
                single: 0.0001,
                double: 0.0003,
                triple: 0.0002,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = PhysicsConfig::default();
        assert_eq!(c.kb_scaled, 0.45);
        assert_eq!(c.thermal_kick_scale, 0.008314);
        assert_eq!(c.spring_energy_scale, 0.002);
        assert_eq!(
            c.release_fraction, c.formation_fraction,
            "cycle conservation"
        );
        assert_eq!(c.diffusion_rate, 0.1);
        assert_eq!(c.bond_search_radius, 4.0);
        assert_eq!(c.formation_fraction, 0.3);
        assert_eq!(c.convection_rate, 0.001);
        assert_eq!(c.vent_heat_rate, 0.1);
        assert_eq!(c.compaction_interval, 10_000);
        assert_eq!(c.surface_threshold, 0.9);
        assert_eq!(c.geometry_sigma, 30.0);
        assert_eq!(c.t_opt_scale, 0.1);
        assert_eq!(c.t_width, 20.0);
        assert_eq!(c.uv_sensitivity.of_order(2), 0.0003);
        assert_eq!(c.uv_sensitivity.of_order(0), 0.0);
    }
}
