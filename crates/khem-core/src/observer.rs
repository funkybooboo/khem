//! The observer system: read-only world sampling (guarantee G03),
//! molecule detection via union-find on the bond graph, watch
//! conditions, NDJSON event emission to stdout.
//!
//! The NDJSON stream is a public, versioned contract consumed by
//! khem-view and any other tool; tools target the contract, not
//! shared types (see ARCHITECTURE.md).
//!
//! Spec: docs/specs/runtime-spec.md, sections 3 and 9 (NDJSON
//! output, Observer System).

// Phase 1 implementation lands here.