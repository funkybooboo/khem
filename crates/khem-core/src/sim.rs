//! The tick loop and strict system ordering: energy -> velocities ->
//! positions -> boundary -> spatial index -> bond breaking -> bond
//! formation -> observer -> event flush.
//!
//! Systems read previous-tick state and write current-tick state; no
//! system reads another system's writes within a tick. This fixed
//! order is what makes runs deterministic and later parallelizable
//! (guarantees G02, G14).
//!
//! Spec: docs/specs/runtime-spec.md, section 5 (Tick Execution).

// Phase 1 implementation lands here.