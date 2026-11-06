//! The NDJSON event model (runtime spec 3.3) - the public,
//! versioned contract (schema v:2, ADR-0004).
//!
//! The types live in khem-world because the systems that mutate
//! the world (physics, chemistry) produce the system events into
//! the world's own queue; the observer (khem-engine) composes the
//! run events and the emitter (ndjson) serializes. Tools target
//! the stream, not these types.
//!
//! Timing fields (elapsed_ms, ticks_per_sec) are wall-clock and
//! therefore excluded from G02's byte-identical guarantee by the
//! documented carve-out (spec revision note).

use crate::world::{AtomId, ElementId};

/// Wall-clock timing for TICK and END events, measured by the tick
/// loop (the only wall-clock consumer). Excluded from G02.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Timing {
    pub elapsed_ms: u64,
    pub ticks_per_sec: f64,
}

/// Per-tick world statistics for TICK events (runtime spec 3.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldStats {
    pub atom_count: u32,
    pub bond_count: u32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub temp_avg: f32,
    pub pressure_min: f32,
    pub pressure_max: f32,
    pub pressure_avg: f32,
    /// Unbonded live atoms per canonical element index.
    pub free_atoms: [u32; 10],
    /// Molecule size distribution buckets: [0] size 1, [1] 2-5,
    /// [2] 6-20, [3] 21+ (schema keys "1", "2_5", "6_20",
    /// "21plus").
    pub mol_size_dist: [u32; 4],
}

/// One output event (runtime spec 3.3), payloads carrying exactly
/// the schema fields. `elem_*` fields are element indexes into the
/// canonical table (phase 4 revisits if custom element tables
/// arrive).
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// First line of output, emitted once before tick 1.
    Start {
        khem_version: &'static str,
        run_name: String,
        world_name: String,
        seed: u64,
        atom_count: u32,
        bond_count: u32,
        world_width: f32,
        world_height: f32,
        world_depth: f32,
    },
    /// Every `tick_interval` ticks.
    Tick {
        tick: u64,
        timing: Timing,
        stats: WorldStats,
    },
    BondFormed {
        tick: u64,
        bond_id: u32,
        atom_a: AtomId,
        atom_b: AtomId,
        elem_a: ElementId,
        elem_b: ElementId,
        order: u8,
        energy: f32,
        x: f32,
        y: f32,
        z: f32,
    },
    BondBroken {
        tick: u64,
        bond_id: u32,
        elem_a: ElementId,
        elem_b: ElementId,
        energy_released: f32,
        x: f32,
        y: f32,
        z: f32,
    },
    /// Last line of output, emitted once. Reason vocabulary per
    /// spec 3.3: max_ticks_reached, user_interrupt, extinction,
    /// runtime_error.
    End {
        tick: u64,
        timing: Timing,
        reason: &'static str,
    },
}
