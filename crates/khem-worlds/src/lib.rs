//! khem-worlds: world constructors.
//!
//! The phase-1 hardcoded worlds live here until the .kem language
//! (phase 4) replaces hardcoded with declared: the vented
//! primordial pond (the gate world) and, following the same
//! pattern, the beakers the harnesses need. A world constructor
//! takes a seed and a [`khem_forcefield::ForceField`] and returns
//! a fully seeded [`khem_world::WorldState`] - keep-apart
//! construction included: what leaves here is legal state.

pub mod pond;

pub use pond::primordial_pond;
