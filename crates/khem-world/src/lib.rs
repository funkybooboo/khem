//! khem-world: the world's data model and its machinery.
//!
//! Everything the runtime KNOWS lives here: [`WorldState`] with its
//! guarantee-G04 bond bookkeeping, [`AtomState`]/[`BondState`], the
//! temperature/pressure/UV [`Grid3D`] fields, the spatial hash
//! [`SpatialIndex`], the deterministic [`Rng`], the boundaries -
//! and the [`Event`] model of the public NDJSON contract (the
//! systems that mutate the world produce events into the world's
//! own queue; khem-engine's observer composes the rest).
//!
//! The crate boundary enforces runtime guarantee G01's substrate:
//! no concept above the atom/bond level exists here (see
//! docs/build/architecture.md and docs/build/specs/runtime-spec.md).

pub mod energy;
pub mod event;
pub mod rng;
pub mod spatial;
pub mod world;

pub use energy::{EnergySource, SourceKind};
pub use event::{Event, Timing, WorldStats};
pub use rng::Rng;
pub use spatial::SpatialIndex;
pub use world::{
    AtomId, AtomState, BondId, BondState, BoundaryType, ElementId, Grid3D, MAX_BONDS, WorldState,
};
