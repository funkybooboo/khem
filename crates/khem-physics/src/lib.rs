//! khem-physics: the force laws, the integrator, the thermostat,
//! and the energy sources.
//!
//! [`Physics`] evaluates the bond springs, the excluded-volume
//! wall, and the cohesion well (spec 6), sub-steps the integrator
//! (spec 6.5), couples the Langevin thermostat to the temperature
//! field (spec 6.1/6.2), and [`Energy`] runs the environment
//! sources - diffusion, convection, vents, sunlight (spec 8).
//! Both implement the system traits (spec 10.4); both read the
//! world and the [`khem_forcefield::ForceField`] data, never a
//! special case.

pub mod energy;
pub mod physics;

pub use energy::{Energy, EnergySystem};
pub use khem_world::{EnergySource, SourceKind};
pub use physics::{PairGeometry, Physics, PhysicsSystem, hydrogen_bond_force};
