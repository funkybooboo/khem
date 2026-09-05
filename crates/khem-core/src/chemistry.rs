//! The chemistry system: bond breaking and formation by
//! Boltzmann/Arrhenius probabilities, the real bond-energy lookup
//! table, VSEPR geometry factors, valence bookkeeping.
//!
//! Phase-1 placement decision, to fold into the spec at revision
//! (ADR-0006): UV bond breaking (spec 8.2) executes inside
//! [`ChemistrySystem::break_bonds`] alongside the thermal Boltzmann
//! roll, not in the energy system. The tick order (spec 5.1) gives
//! chemistry the only bond-mutating steps, and one place that
//! breaks bonds keeps the RNG discipline legible; the energy system
//! only writes fields.
//!
//! The implementation - including the bond-energy table (spec 7.3)
//! and the VSEPR angle table (spec 7.4), whose unknown-pair fallback
//! semantics get decided with real phase-1 context - is the plan's
//! phase 1 work, gates K1-K4.
//!
//! Spec: docs/specs/runtime-spec.md, section 7 (Chemistry System).

use crate::world::WorldState;

/// The chemistry system interface, decomposed per the tick order
/// (runtime spec 5.1: break_bonds, form_bonds). v0.1 compiles exactly
/// one implementation; the trait exists so future plugin loading
/// does not require restructuring (runtime spec 10.4).
pub trait ChemistrySystem {
    /// Thermal Boltzmann breaking (spec 7.1) and UV breaking (spec
    /// 8.2, reading the UV field the energy system set this tick).
    fn break_bonds(&mut self, world: &mut WorldState);
    /// Probabilistic bond formation (spec 7.2).
    fn form_bonds(&mut self, world: &mut WorldState);
}
