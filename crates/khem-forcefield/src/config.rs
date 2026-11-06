//! The tunable constants (runtime spec section 11), grouped by the
//! law that reads them - a force-field specification, not a flat
//! bag. Phase 1 hardcodes [`ForceField::default`] everywhere;
//! loading physics.cfg from a project root arrives later. Tuning
//! affects behavior and stability - it never changes what
//! chemistry is possible.
//!
//! The leaf names (`non_bonded_margin`, `cohesion_scale`, ...) are
//! the project's historical knob names, kept stable across the
//! docs (spec 6/7/8/11, the gates' PASS records, the findings
//! log) exactly like the K-numbers: renames would break the
//! evidence chain.
//!
//! The grouped names are the phase-4 `.kem` surface in miniature:
//! a `physics` declaration in a world file is this struct's data.

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

/// Langevin thermostat coupling to the temperature field
/// (spec 6.1/6.2). Every group's `Default` carries the values the
/// gates certify - out-of-the-box defaults are the README's claim
/// (ADR-0014) - so struct-update syntax (`..Default::default()`)
/// at ANY level yields the shipped constants, never zeros.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermostatParams {
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
    /// Field relaxation toward declared setpoints (spec 6.2, K1.1):
    /// T += (setpoint - T) * rate per tick, where setpoint > 0.
    /// This is the environment reservoir - the pond's heat sink -
    /// without which a vented pond cannot hold a steady state (a
    /// vent injects continuously and a Wrap boundary leaks
    /// nothing). Region declarations (phase 4) are the
    /// setpoint source; the phase-1 pond declares 35 C.
    pub field_relax_rate: f32,
}

/// The sub-stepped integration (spec 6.5, gate K1.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratorParams {
    /// Integration sub-steps per tick: the force/integration pair
    /// runs this many times per tick with dt_sub = 1 /
    /// integration_substeps, so the tick stays dt = 1 for
    /// chemistry, the thermostat, and the fields while the
    /// short-range dynamics resolve at dt_sub. This is the proper
    /// fix for the tunneling mint (F13): an atom crossing the
    /// ~1.6-4 A non-bonded zone in several sub-steps samples the
    /// force symmetrically - no energy mint - which is why the
    /// velocity clamp could be REMOVED with this change (the
    /// re-validation contract required removing it before the
    /// E-gates). 4 gives dt_sub = 0.25: sub-step displacement of
    /// the fastest thermal atoms (~2-3 A/tick tail) stays under
    /// half the narrowest interaction zone.
    pub integration_substeps: u32,
}

/// The Pauli wall: unbonded pairs do not overlap (finding F4's
/// fix; gate K2.1's lever).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExcludedVolumeParams {
    /// Non-bonded soft-core strength: every UNBONDED pair closer
    /// than its cutoff is pushed apart with force
    /// `non_bonded_repulsion * (cutoff - r)`. Without excluded
    /// volume, atoms pass through each other and every lipid model
    /// in the literature fails to self-assemble (abstraction-notes
    /// section 4). Retuned 2026-11-04 from 1.0 with gate K2.1: the
    /// scattering census measured the strength-1 core failing the
    /// gate's own letter at the pond's thermal tail - an H-H pair
    /// at the p99 closing speed (2.28 A/tick at 35 C) passed clean
    /// through (min approach 0.015 of r_sum, 8/8 crossings) and a
    /// 2-sigma pair penetrated to 0.48 against the 0.8 bar. At 8.0
    /// the symplectic bound sits at dt_sub * sqrt(S/mu) = 1.00 for
    /// the worst pair (H-H; the stiffest spring's own bound is
    /// 1.375), every bin then holds the 0.8 bar with 20% margin
    /// (worst 0.959), and the bounce stays energy-honest (pair KE
    /// at exit 1.00-1.01 x launch). 32.0 was measured and rejected:
    /// the bound hits 1.99 and the discretized wall mints 2.0-3.6x
    /// pair KE. Evidence: the excluded-volume gate + sweep
    /// (khem-engine tests, gate K2.1).
    pub non_bonded_repulsion: f32,
    /// Non-bonded cutoff multiplier: cutoff = (radius_a + radius_b)
    /// * margin.
    pub non_bonded_margin: f32,
}

/// The like-dissolves-like cohesion well (spec 6.6's attraction;
/// the polarity-attraction memo's law).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohesionParams {
    /// Polarity-cohesion well depth scale. Each unbonded pair
    /// outside its excluded-volume cutoff and inside the flat
    /// `attraction_range` sits in a zero-net-force well of depth
    /// `cohesion_scale * (1 - |chi_a - chi_b| / chi_ref)`
    /// (like-dissolves-like over the table's Pauling
    /// electronegativities - water coheres to water, hydrocarbon
    /// to hydrocarbon, and mixed contacts sit 30-40% weaker at
    /// chi_ref 3.0: the hydrophobic gap from element properties
    /// alone).
    ///
    /// DEFAULT 0.0 - STILL DORMANT, BY TWO ROUNDS OF MEASUREMENT
    /// (the cohesion-sweep evidence). Round one (2026-11-04, the
    /// 0.0636 atoms/A^3 pond): every depth >= 0.25 condenses the
    /// seeded pond into a droplet. Round two (2026-11-06, the
    /// RE-DENSIFIED 0.1022 atoms/A^3 pond at real water's own
    /// atomic density): STILL phase-separated - D=0.5 built a
    /// first shell (peak g 2.12) with the medium collapsing into
    /// dense blobs over thin vapor by 12k ticks, and D=1.0 jammed
    /// pairs against the walls (peak 6.15). The structural cause
    /// is now measured: the model's atoms are ~2x smaller than
    /// real atoms' effective sizes (excluded-volume cutoffs
    /// 1.5-2 A vs real vdW contacts ~2.8-3.2), so the model's
    /// liquids live at ~2-3x real liquid density - and the water
    /// lattice cannot seed below ~3.2 A spacing (adjacent waters'
    /// hydrogens clash), so the pond can never sit at its own
    /// liquid's equilibrium. At any shell-building depth the pond
    /// condenses; that is the physics, not a bug. The real-water
    /// substrate sitting owns the activation (ADR-0014): the wall
    /// rescale plus the directional hydrogen-bond term re-derive
    /// this depth at the new geometry. At 0.0 the well is skipped
    /// entirely (and the non-bonded query radius stays at the
    /// excluded-volume cutoff).
    pub cohesion_scale: f32,
    /// Electronegativity discrimination scale for the well depth
    /// (spec 6.6): strength multiplier max(0, 1 - |chi_a - chi_b|
    /// / chi_ref). 3.0 puts water-water at 1.00, hydrocarbon pairs
    /// 0.88-1.00, and the mixed tail-water contacts at 0.59-0.70.
    pub chi_ref: f32,
    /// Well range, angstroms - FLAT, not r_sum-scaled: a molecular
    /// liquid's cohesion range is ~3-4 A regardless of atom size
    /// (real water's O...O first shell sits at ~2.8 A; dispersion
    /// contacts cluster at 3.4-3.7), so every unbonded pair outside
    /// its cutoff and inside this range sits in the well. The O-O
    /// well then spans 1.98-4.0 A with its minimum at 2.99 - real
    /// water's own first shell. The flat range also keeps the
    /// well's slopes gentle for the lightest pairs (stiffness ~
    /// depth/w^2; the H-H well is 2.41 A wide, so the sub-step
    /// bound holds at depths the r_sum-scaled draft could not),
    /// and pairs whose excluded volume swallows the whole shell
    /// (Na-Cl: cutoff 4.02) simply have no well - big atoms
    /// cohere by chemistry alone, a documented size
    /// discrimination off the K2-K3 path.
    pub attraction_range: f32,
}

/// The mechanical bond: the spring law and its breaking limit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BondParams {
    /// Spring constant scale: k = bond.energy * scale (spec 6.3).
    /// Retuned 2026-09-07 from 0.004 (the dt=1 stability bound
    /// dt * sqrt(k / reduced_mass) < 2 capped it there; at 0.004
    /// the O-H mechanical well - the stretch energy at the 7.1
    /// break point - was only ~10 kT, and a thermal-speed hydrogen
    /// or one non-bonded shove carried enough to shatter water:
    /// gate K1.3 measured 1482 mechanical O-H breaks in 10k ticks,
    /// 75% of the pond). With the integrator sub-stepping n times
    /// per tick the bound is evaluated at dt_sub = 1/n, and 0.032
    /// puts the worst formable pair (H-H order 1) at
    /// dt_sub * sqrt(k/mu) = 1.32 < 2 while deepening the O-H well
    /// to ~80 kT - real water's own ratio - so the thermostat at
    /// 35 C essentially never stretches a bond to the break point
    /// (needs ~7 A/tick relative, ~13 sigma). Pinned per row by
    /// the bond_table_symplectic_stability test.
    pub spring_energy_scale: f32,
    /// Maximum bond extension: bonds longer than
    /// bond_break_factor * r_eq break (mechanical dissociation,
    /// spec 7.1 v0 addition). Real bonds cannot be stretched to
    /// multiples of their length; without this rule the measured
    /// substrate let collision-shoved bonds random-walk to 30-80 A
    /// while alive. Standard practice in coarse-grained MD (FENE
    /// R_max). Deterministic: no RNG roll.
    pub bond_break_factor: f32,
}

/// The chemistry laws: formation capture and thermal/UV breaking
/// (spec 7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChemistryParams {
    /// Boltzmann constant for THERMAL BREAKING (spec 7.1), scaled
    /// to sim units. Retuned 2026-09-05 from the literal 0.008314:
    /// with the physical kB, p_break = exp(-E/(kB*T)) is ~1e-76 for
    /// every bond at pond temperatures - the measured K1 harness
    /// ran 2000 ticks with zero breaks (frozen inertness, finding
    /// F1). At 0.45 the phenomenology straddles the table: weak
    /// bonds (O-O 146) break at 35 C every ~10k ticks, water's O-H
    /// (463) essentially never - which is real chemistry's answer
    /// too. Note the split from thermal_kick_scale: one physical
    /// kB set two incompatible sim scales (breaking rate and kick
    /// magnitude), so they decouple as of the tuning commit.
    pub kb_scaled: f32,
    /// Bond-formation search radius, angstroms. The candidate
    /// GATHERING radius only: a pair must additionally pass the
    /// per-pair capture contact (`bond_form_factor`), so this caps
    /// the spatial query, not the chemistry.
    pub bond_search_radius: f32,
    /// Per-pair formation cap: candidate pairs must sit within
    /// `bond_form_factor * r_eq` - bonds form at steric contact,
    /// never at multiples of their own length. Added 2026-09-07
    /// with gate K1.4 (finding F18): pairs formed inside the
    /// search radius but past the 7.1 break length are phantom
    /// captures - the mechanical rule kills them on the next
    /// chemistry pass, and each cycle silently keeps the absorbed
    /// formation_fraction * E (measured: 123 of 373 formations in
    /// the 10k-tick K1.4 probe, a standing refrigeration machine);
    /// wide-but-legal captures mint the birth-stretch spring PE
    /// that flings the pair into a mechanical break within ticks
    /// (measured: 40 more). 1.5 puts capture exactly at the
    /// excluded-volume standoff (non_bonded_margin * r_eq): a bond
    /// forms where the atoms touch.
    pub bond_form_factor: f32,
    /// Per eligible pair per tick, at steric contact (7.2).
    /// Retuned 2026-09-07 from 0.001 with gate K1.4: the original
    /// value was tuned when "eligible" meant anywhere inside the
    /// 4 A search disc, and the contact capture (bond_form_factor,
    /// F18's fix) cut the formation count 2.7x - construction of
    /// the free population was still unfinished after 10k ticks
    /// (131 free atoms, size distribution not stationary). At 0.01
    /// the free population is consumed within ~4k ticks and the
    /// weak-pair flicker (O-O form/thermal-break cycling in the
    /// vent plume) is measurable in the stationary tail; the
    /// measured profile lives in the K1.4 harness evidence.
    pub base_formation_rate: f32,
    /// Formation capture gate: pairs with relative speed above
    /// this do not bond (spec 7.2, v0 fill). A bond forming between
    /// atoms flying past each other cannot absorb their relative
    /// kinetic energy and becomes a stretched comet (measured:
    /// mean bond length 5.8 A, tail at ~80 A without the gate);
    /// real capture requires slow relative motion. Default 1.5
    /// A/tick (~3x hydrogen's thermal speed at pond temperature).
    pub max_form_speed: f32,
    /// Fraction of bond energy absorbed from the temperature field
    /// on forming (spec 7.2).
    pub formation_fraction: f32,
    /// Fraction of bond energy released into the temperature field
    /// on breaking (spec 7.1). Set equal to formation_fraction in
    /// the 2026-09-05 tuning: the spec's 0.5/0.3 asymmetry created
    /// energy - a form+break cycle deposited 0.2 * E into the field
    /// from nowhere (finding F7). Equal fractions conserve through
    /// the cycle; the field still differs because kinetics differ.
    pub release_fraction: f32,
    /// Thermalization rate of the bond-break heat reservoir
    /// (finding F19's fix, spec 6.2): each cell drains at most this
    /// many degrees into its temperature cell per tick. The
    /// thermal break's release (release_fraction * E) used to land
    /// in the cell as a delta function - one O-H release is ~139
    /// degrees, and the spike broke the neighboring waters' O-H
    /// bonds (p_break is exponential in T), each secondary break
    /// re-spiking the cell: measured, the pond reached its
    /// steady-state 35 C and vaporized within ~500 ticks (every
    /// bond, field at 1771 C). The bounded drain holds a cell at
    /// ~+20 C above its neighbors even under a full-cap stream and
    /// cuts the cascade to ~1e-4 expected secondaries per break,
    /// closing the feedback loop while conserving the full release
    /// into the field (F7's cycle law, just spread over
    /// E/0.3/cap ticks).
    pub release_rate_cap: f32,
    /// Electronegativity formation bonus (spec 7.2): pairs that
    /// polarize (|chi_a - chi_b| > 0.4) form at an increased rate.
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
}

/// The energy environment: fields, convection, vents, sunlight
/// (spec 8).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldParams {
    /// Temperature-field diffusion rate (spec 6.2).
    pub diffusion_rate: f32,
    /// Pressure-field sensitivity (spec 6.2).
    pub pressure_sensitivity: f32,
    /// Convection rate (spec 8.3): the temperature field drifts
    /// upward along the vertical (z) axis.
    pub convection_rate: f32,
    /// Vent heat injection per tick (spec 8.1). Phase-1 addition:
    /// the spec's formula uses this constant but section 11 omits
    /// it; revision adds it here. Pure tuning knob.
    pub vent_heat_rate: f32,
    /// Fraction of world depth counted as surface for solar UV (z,
    /// the vertical axis since the 3D port; was the height/y
    /// fraction in 2D).
    pub surface_threshold: f32,
    /// UV break probability per bond order (runtime spec 8.2).
    pub uv_sensitivity: UvSensitivity,
}

/// World layout: grid geometry (not a force law, but part of the
/// same declared-physics surface a `.kem` world carries).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutParams {
    /// Spatial hash cell size, angstroms.
    pub spatial_cell_size: f32,
    /// Field grid cell size, angstroms.
    pub field_cell_size: f32,
    /// Ticks between dead-entity compactions (spec 5.2). NOT YET
    /// READ by any system: compaction is phase-3 work, so this only
    /// pins the spec 11 value until the compactor lands.
    pub compaction_interval: u32,
}

/// Every tunable constant from runtime spec section 11, grouped
/// by law. This IS the force field: the data the generic laws read
/// (the MD convention - GROMACS, LAMMPS, OpenMM - where the
/// parameters and the engine are separate concerns).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ForceField {
    /// Langevin thermostat coupling to the temperature field
    /// (spec 6.1/6.2).
    pub thermostat: ThermostatParams,
    /// The sub-stepped integrator (spec 6.5).
    pub integrator: IntegratorParams,
    /// The Pauli wall - unbonded pairs do not overlap (spec 6.6).
    pub excluded_volume: ExcludedVolumeParams,
    /// Like-dissolves-like cohesion well (spec 6.6, dormant).
    pub cohesion: CohesionParams,
    /// The mechanical bond spring and its breaking limit
    /// (spec 6.3, 7.1).
    pub bonds: BondParams,
    /// Formation and breaking laws (spec 7).
    pub chemistry: ChemistryParams,
    /// The energy environment (spec 8).
    pub fields: FieldParams,
    /// Grid geometry.
    pub layout: LayoutParams,
}

impl Default for UvSensitivity {
    fn default() -> Self {
        Self {
            single: 0.0001,
            double: 0.0003,
            triple: 0.0002,
        }
    }
}

impl Default for ThermostatParams {
    fn default() -> Self {
        Self {
            thermal_kick_scale: 0.008314,
            thermostat_damping: 0.1,
            ke_field_scale: 0.01,
            field_relax_rate: 0.002,
        }
    }
}

impl Default for IntegratorParams {
    fn default() -> Self {
        Self {
            integration_substeps: 4,
        }
    }
}

impl Default for ExcludedVolumeParams {
    fn default() -> Self {
        Self {
            non_bonded_repulsion: 8.0,
            non_bonded_margin: 1.5,
        }
    }
}

impl Default for CohesionParams {
    fn default() -> Self {
        Self {
            cohesion_scale: 0.0,
            chi_ref: 3.0,
            attraction_range: 4.0,
        }
    }
}

impl Default for BondParams {
    fn default() -> Self {
        Self {
            spring_energy_scale: 0.032,
            bond_break_factor: 2.5,
        }
    }
}

impl Default for ChemistryParams {
    fn default() -> Self {
        Self {
            kb_scaled: 0.45,
            bond_search_radius: 4.0,
            bond_form_factor: 1.5,
            base_formation_rate: 0.01,
            max_form_speed: 1.5,
            formation_fraction: 0.3,
            release_fraction: 0.3,
            release_rate_cap: 2.0,
            en_bonus: 0.1,
            geometry_sigma: 30.0,
            t_opt_scale: 0.1,
            t_width: 20.0,
        }
    }
}

impl Default for FieldParams {
    fn default() -> Self {
        Self {
            diffusion_rate: 0.1,
            pressure_sensitivity: 0.01,
            convection_rate: 0.001,
            vent_heat_rate: 0.1,
            surface_threshold: 0.9,
            uv_sensitivity: UvSensitivity::default(),
        }
    }
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            spatial_cell_size: 5.0,
            field_cell_size: 10.0,
            compaction_interval: 10_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = ForceField::default();
        assert_eq!(c.chemistry.kb_scaled, 0.45);
        assert_eq!(c.thermostat.thermal_kick_scale, 0.008314);
        assert_eq!(c.thermostat.thermostat_damping, 0.1);
        assert_eq!(c.thermostat.ke_field_scale, 0.01);
        assert_eq!(c.excluded_volume.non_bonded_repulsion, 8.0);
        assert_eq!(c.excluded_volume.non_bonded_margin, 1.5);
        assert_eq!(c.cohesion.cohesion_scale, 0.0);
        assert_eq!(c.cohesion.chi_ref, 3.0);
        assert_eq!(c.cohesion.attraction_range, 4.0);
        assert_eq!(c.bonds.spring_energy_scale, 0.032);
        assert_eq!(c.integrator.integration_substeps, 4);
        assert_eq!(
            c.chemistry.release_fraction, c.chemistry.formation_fraction,
            "cycle conservation"
        );
        assert_eq!(c.chemistry.release_rate_cap, 2.0);
        assert_eq!(c.fields.diffusion_rate, 0.1);
        assert_eq!(c.chemistry.bond_search_radius, 4.0);
        assert_eq!(c.chemistry.bond_form_factor, 1.5);
        assert_eq!(c.chemistry.base_formation_rate, 0.01);
        assert_eq!(c.chemistry.formation_fraction, 0.3);
        assert_eq!(c.fields.convection_rate, 0.001);
        assert_eq!(c.fields.vent_heat_rate, 0.1);
        assert_eq!(c.layout.compaction_interval, 10_000);
        assert_eq!(c.fields.surface_threshold, 0.9);
        assert_eq!(c.chemistry.geometry_sigma, 30.0);
        assert_eq!(c.chemistry.t_opt_scale, 0.1);
        assert_eq!(c.chemistry.t_width, 20.0);
        assert_eq!(c.chemistry.max_form_speed, 1.5);
        assert_eq!(c.bonds.bond_break_factor, 2.5);
        assert_eq!(c.thermostat.field_relax_rate, 0.002);
        assert_eq!(c.fields.uv_sensitivity.of_order(2), 0.0003);
        assert_eq!(c.fields.uv_sensitivity.of_order(0), 0.0);
    }
}
