//! khem-engine: the tick loop and the output stream.
//!
//! [`Sim`] drives the runtime spec 5.1 step order exactly
//! (energy -> bath -> sub-stepped forces -> boundary -> breaking ->
//! formation -> observe -> flush); [`Observer`] samples the world
//! read-only and composes the run events; [`ndjson`] emits the
//! versioned public contract (schema v:2, ADR-0004).
//!
//! The engine knows no specific world: the pond (and any other
//! world) live in [`khem_worlds`], passed in as data. [`Sim`]
//! produces events and never touches streams; the khem bin
//! serializes - which keeps G10 (stdout is only NDJSON)
//! enforceable in one place.

pub mod ndjson;
pub mod observer;
pub mod sim;

pub use ndjson::emit;
pub use observer::{Observer, ObserverConfig, ObserverSystem};
pub use sim::{Sim, TICK_ORDER};

// The event model lives in khem-world (the systems that mutate the
// world produce the events); it is re-exported here because it is
// the observer/emitter surface.
pub use khem_world::{Event, Timing, WorldStats};
