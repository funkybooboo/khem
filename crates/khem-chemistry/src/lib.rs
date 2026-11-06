//! khem-chemistry: bonds form and break by law (spec 7).
//!
//! [`Chemistry`] gathers steric-contact candidates through the
//! spatial index, scores them with the VSEPR geometry factor and
//! the Boltzmann/Arrhenius rate tables, and breaks bonds
//! thermally (spec 7.1), mechanically (the FENE-style extension
//! limit), and by UV (spec 8.2) - pushing the system events into
//! the world's queue. The element and bond-energy data live in
//! [`khem_forcefield`]; the laws live here.

pub mod chemistry;

pub use chemistry::{Chemistry, ChemistrySystem, PairOutcome};
