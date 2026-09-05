//! khem-core: the khem simulation engine.
//!
//! Everything the runtime knows lives in this crate: `WorldState`,
//! atoms, bonds, fields, energy, the tick loop, and the observer.
//! Nothing here knows about the .kem language; world definition and
//! parsing arrive in phase 3 as the khem-lang crate. The crate
//! boundary enforces runtime guarantee G01: no concept above the
//! atom/bond level exists in the engine (see ARCHITECTURE.md and
//! docs/specs/runtime-spec.md).
//!
//! What exists today (crate setup; the plan's phase 1 lands the
//! systems):
//!
//! - [`world`]: the complete data model - `AtomId`, `BondId`,
//!   `ElementId`, `AtomState`, `BondState`, `WorldState` with
//!   guarantee-G04 bond bookkeeping, `Grid2D`, `BoundaryType`
//! - [`elements`]: `ElementProperties` and the 10-element table with
//!   real values
//! - [`config`]: `PhysicsConfig`, the tunable constants from runtime
//!   spec section 11
//! - [`rng`]: the deterministic RNG that guarantee G02 stands on
//! - [`spatial`]: the spatial hash index rebuilt every tick
//! - [`energy`], [`observer`]: data types only; the systems are
//!   phase 1
//! - [`physics`], [`chemistry`]: the system traits (runtime spec
//!   10.4); implementations are phase 1
//! - [`sim`]: the fixed tick order, as data

pub mod chemistry;
pub mod config;
pub mod elements;
pub mod energy;
pub mod observer;
pub mod physics;
pub mod rng;
pub mod sim;
pub mod spatial;
pub mod world;

pub use chemistry::{Chemistry, ChemistrySystem, bond_energy};
pub use config::{PhysicsConfig, UvSensitivity};
pub use elements::{ELEMENTS, ElementProperties};
pub use energy::{Energy, EnergySource, EnergySystem, SourceKind};
pub use observer::{Event, EventKind};
pub use physics::{Physics, PhysicsSystem};
pub use rng::Rng;
pub use spatial::SpatialIndex;
pub use world::{
    AtomId, AtomState, BondId, BondState, BoundaryType, ElementId, Grid2D, MAX_BONDS, WorldState,
};
