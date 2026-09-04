//! World state and core data types: AtomState, BondState, WorldState,
//! Grid2D field grids, BoundaryType.
//!
//! Design rules (spec section 4): flat arrays indexed by AtomId and
//! BondId - no per-atom heap allocation, no pointers, cache-friendly,
//! partitionable by region for V2/V3 scaling.
//!
//! Spec: docs/specs/runtime-spec.md, section 4 (core data
//! structures).

// Phase 1 implementation lands here.