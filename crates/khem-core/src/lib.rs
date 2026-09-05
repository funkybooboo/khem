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
//! What exists today (phase 1 kernel, mid-build):
//!
//! - [`world`]: the complete data model - `AtomId`, `BondId`,
//!   `ElementId`, `AtomState`, `BondState`, `WorldState` with
//!   guarantee-G04 bond bookkeeping, `Grid2D`, `BoundaryType`
//! - [`elements`]: `ElementProperties` and the 10-element table with
//!   real values
//! - [`config`]: `PhysicsConfig`, the tunable constants from runtime
//!   spec section 11 plus the v0 fills the spec's formulas reference
//!   but never define
//! - [`rng`]: the deterministic RNG that guarantee G02 stands on
//! - [`spatial`]: the spatial hash index rebuilt every tick
//! - [`physics`], [`chemistry`], [`energy`]: the systems (spec 6, 7,
//!   8) behind their traits (spec 10.4)
//! - [`observer`]: event model + read-only sampling with union-find
//!   molecule detection (spec 9); [`ndjson`]: the schema v:1 emitter
//! - [`sim`]: the tick loop driving all systems in spec 5.1 order

pub mod chemistry;
pub mod config;
pub mod elements;
pub mod energy;
pub mod ndjson;
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
pub use observer::{Event, Observer, ObserverConfig, ObserverSystem, Timing, WorldStats};
pub use physics::{Physics, PhysicsSystem};
pub use rng::Rng;
pub use sim::Sim;
pub use spatial::SpatialIndex;
pub use world::{
    AtomId, AtomState, BondId, BondState, BoundaryType, ElementId, Grid2D, MAX_BONDS, WorldState,
};
