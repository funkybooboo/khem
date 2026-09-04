//! The observer system: read-only world sampling (guarantee G03),
//! molecule detection via union-find on the bond graph, watch
//! conditions, NDJSON event emission to stdout.
//!
//! Setup scope: the event data model only. The observer system
//! (sampling, union-find, serialization to the NDJSON contract) is
//! the plan's phase 1/2 work.
//!
//! The NDJSON stream is a public, versioned contract consumed by
//! khem-view and any other tool; tools target the contract, not
//! shared types (see ARCHITECTURE.md).
//!
//! Spec: docs/specs/runtime-spec.md, sections 3 and 9.

/// Event kinds, runtime spec section 3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// First line of output, emitted once.
    Start,
    /// Every `tick_interval` ticks.
    Tick,
    /// When `output.bond_events` is true.
    BondFormed,
    BondBroken,
    /// Watch condition triggered; always emitted.
    Notable,
    /// State saved.
    Save,
    /// Last line of output, emitted once.
    End,
}

/// One queued output event. The full per-kind field schemas (runtime
/// spec 3.3) and NDJSON serialization land with the observer system;
/// the queue exists now because [`crate::world::WorldState`] owns
/// it.
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub kind: EventKind,
    pub tick: u64,
}
