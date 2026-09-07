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
    /// Langevin thermostat damping (the F8 fix): velocities relax
    /// toward the local field temperature at this rate per tick;
    /// the noise amplitude carries the matching fluctuation term
    /// sqrt(damping * (2 - damping)) so the stationary velocity
    /// variance is thermal_kick_scale * T / mass.
    pub thermostat_damping: f32,
    /// Field degrees per unit of kinetic energy exchanged by the
    /// thermostat. Damped KE deposits into the local cell; the
    /// expected noise injection drains back out - net zero at
    /// equilibrium, transfer off equilibrium.
    pub ke_field_scale: f32,
    pub diffusion_rate: f32,
    pub pressure_sensitivity: f32,
    /// Spring constant scale: k = bond.energy * scale (spec 6.3).
    /// Retuned 2026-09-05 from the founding 0.01 (F2: the
    /// integrator is symplectic Euler; stability is
    /// dt * sqrt(k / reduced_mass) < 2, and 0.01 put light-pair
    /// bonds far over the bound). 0.002 was stable but floppy:
    /// measured water O-H mean 2.46 A vs 1.19 equilibrium (thermal
    /// kicks vs spring stiffness). 0.004 doubles rigidity with the
    /// worst tabulated case (H-H, reduced mass 0.5) at
    /// sqrt(436 * 0.004 / 0.5) = 1.87, inside the bound; the
    /// bond_table_symplectic_stability test pins this law for
    /// every row.
    pub spring_energy_scale: f32,
    /// Non-bonded soft-core strength (finding F4's fix): every
    /// UNBONDED pair closer than its cutoff is pushed apart with
    /// force `non_bonded_repulsion * (cutoff - r)`. Without excluded
    /// volume, atoms pass through each other and every lipid model
    /// in the literature fails to self-assemble (abstraction-notes
    /// section 4).
    pub non_bonded_repulsion: f32,
    /// Non-bonded cutoff multiplier: cutoff = (radius_a + radius_b)
    /// * margin.
    pub non_bonded_margin: f32,
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
    /// Field relaxation toward declared setpoints (spec 6.2, K1.1):
    /// T += (setpoint - T) * rate per tick, where setpoint > 0.
    /// This is the environment reservoir - the pond's heat sink -
    /// without which a vented pond cannot hold a steady state (a
    /// vent injects continuously and a Wrap boundary leaks
    /// nothing). Region declarations (phase 3) are the
    /// setpoint source; the phase-1 pond declares 35 C.
    pub field_relax_rate: f32,
    /// Velocity clamp (numerical guard, not physics): the
    /// non-bonded interaction range is ~1.6-5 A while fast atoms
    /// move 3-18 A per tick; unresolved passes sample the force
    /// asymmetrically and mint energy (measured: collision cascade
    /// to max_v ~18 and a 2200 C field). Atoms above this speed are
    /// clamped and the removed KE is deposited into the local
    /// field, keeping the ledger exact. Default 2.0 A/tick (~3.7
    /// sigma for H at pond temperature - thermal dynamics are
    /// untouched). 3.0 re-opened the tunneling mint (measured:
    /// full dissociation, field to 2000 C); 2.0 sits below the
    /// mint threshold, and the over-stretched bonds it cannot
    /// reel in are handled by mechanical dissociation
    /// (bond_break_factor) - which releases no heat, so breaks
    /// cannot cascade. Proper collision sub-stepping is phase 2.
    pub max_atom_speed: f32,
    /// Formation capture gate: pairs with relative speed above
    /// this do not bond (spec 7.2, v0 fill). A bond forming between
    /// atoms flying past each other cannot absorb their relative
    /// kinetic energy and becomes a stretched comet (measured: mean
    /// bond length 5.8 A, tail at ~80 A without the gate); real
    /// capture requires slow relative motion. Default 1.5 A/tick
    /// (~3x hydrogen's thermal speed at pond temperature).
    pub max_form_speed: f32,
    /// Maximum bond extension: bonds longer than
    /// bond_break_factor * r_eq break (mechanical dissociation,
    /// spec 7.1 v0 addition). Real bonds cannot be stretched to
    /// multiples of their length; without this rule the measured
    /// substrate let collision-shoved bonds random-walk to 30-80 A
    /// while alive. Standard practice in coarse-grained MD (FENE
    /// R_max). Deterministic: no RNG roll.
    pub bond_break_factor: f32,
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
            thermostat_damping: 0.1,
            ke_field_scale: 0.01,
            diffusion_rate: 0.1,
            pressure_sensitivity: 0.01,
            spring_energy_scale: 0.004,
            non_bonded_repulsion: 1.0,
            non_bonded_margin: 1.5,
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
            max_form_speed: 1.5,
            bond_break_factor: 2.5,
            max_atom_speed: 2.0,
            field_relax_rate: 0.002,
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
        assert_eq!(c.thermostat_damping, 0.1);
        assert_eq!(c.ke_field_scale, 0.01);
        assert_eq!(c.non_bonded_repulsion, 1.0);
        assert_eq!(c.non_bonded_margin, 1.5);
        assert_eq!(c.spring_energy_scale, 0.004);
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
        assert_eq!(c.max_form_speed, 1.5);
        assert_eq!(c.bond_break_factor, 2.5);
        assert_eq!(c.max_atom_speed, 2.0);
        assert_eq!(c.field_relax_rate, 0.002);
        assert_eq!(c.uv_sensitivity.of_order(2), 0.0003);
        assert_eq!(c.uv_sensitivity.of_order(0), 0.0);
    }
}
