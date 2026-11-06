//! khem-forcefield: the physics parameters as DATA.
//!
//! The MD convention (GROMACS, LAMMPS, OpenMM): the force field is
//! the parameter set that drives generic force laws, separate from
//! the engine that evaluates them. This crate owns:
//!
//! - [`elements`]: the 10-element table with real valences, masses,
//!   electronegativities, radii - and [`ElementId`], the table's
//!   index type (khem-world re-exports it; the atom is world state,
//!   the element is force-field data)
//! - [`tables`]: the bond-energy lookup table (spec 7.1) and the
//!   single-bond reference radii
//! - [`config`]: [`ForceField`] - every tunable constant from
//!   runtime spec section 11, grouped by the law that reads it.
//!   A `.kem` `physics` declaration (phase 4) is this struct's data.
//!
//! What exists here is data and pure lookups; no world, no motion.

pub mod config;
pub mod elements;
pub mod tables;

pub use config::{
    BondParams, ChemistryParams, CohesionParams, ExcludedVolumeParams, FieldParams, ForceField,
    IntegratorParams, LayoutParams, ThermostatParams, UvSensitivity,
};
pub use elements::{ELEMENTS, ElementId, ElementProperties, H, O, element, element_id};
pub use tables::{SINGLE_REFERENCE, bond_energy};
