//! The chemistry system: bond breaking and formation by
//! Boltzmann/Arrhenius probabilities, the real bond-energy lookup
//! table, VSEPR geometry factors, valence bookkeeping.
//!
//! Setup scope: the system trait (runtime spec 10.4). The
//! implementation - including the bond-energy table (spec 7.3) and
//! the VSEPR angle table (spec 7.4), whose unknown-pair fallback
//! semantics get decided with real phase-1 context - is the plan's
//! phase 1 work, gates K1-K4.
//!
//! Spec: docs/specs/runtime-spec.md, section 7 (Chemistry System).

use crate::world::WorldState;

/// The chemistry system interface. v0.1 compiles exactly one
/// implementation; the trait exists so future plugin loading does
/// not require restructuring (runtime spec 10.4).
pub trait ChemistrySystem {
    fn update(&mut self, world: &mut WorldState);
}
