//! khem-core: the khem simulation engine.
//!
//! Everything the runtime knows lives in this crate: WorldState,
//! atoms, bonds, fields, energy, the tick loop, and the observer.
//! Nothing here knows about the .kem language; world definition and
//! parsing arrive in phase 3 as the khem-lang crate. The crate
//! boundary enforces runtime guarantee G01: no concept above the
//! atom/bond level exists in the engine (see ARCHITECTURE.md and
//! docs/specs/05-runtime-spec.md).
//!
//! Module decomposition follows the runtime spec's systems. Phase 1
//! work (PLAN.md, K1-K5 gates) lands in these modules.

pub mod chemistry;
pub mod elements;
pub mod energy;
pub mod observer;
pub mod physics;
pub mod rng;
pub mod sim;
pub mod spatial;
pub mod world;