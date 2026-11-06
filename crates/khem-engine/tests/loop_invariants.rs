//! Loop invariants that must hold under ANY constants -
//! determinism (G02), bond bookkeeping (G04), event stream
//! integrity. Always on: part of `mise run check`. Constants are
//! the gates' business, not this test's.
//!
//! Lineage: the phase-1 kernel's always-on smoke; the gate
//! ladder lives in plans/gates/stability.md.

mod common;

use common::*;
use khem_engine::Sim;
use khem_forcefield::ForceField;
use khem_world::Event;
use khem_world::WorldState;
use khem_worlds::pond;

#[test]
fn loop_invariants_hold() {
    // Invariants that must hold under any constants: the full loop
    // runs, is deterministic per seed, respects G04, and produces a
    // well-formed event stream.
    let config = ForceField::default();

    fn dump(world: &WorldState) -> Vec<(f32, f32, f32, u8)> {
        world
            .atoms
            .iter()
            .map(|a| (a.x, a.y, a.z, a.bond_count))
            .collect()
    }

    let mut a = pond::primordial_pond(7, config);
    let mut sim_a = Sim::new(config, observer(7, 5));
    let mut events_a = vec![sim_a.start(&a)];
    for _ in 0..25 {
        events_a.extend(sim_a.tick(&mut a));
    }
    let mut b = pond::primordial_pond(7, config);
    let mut sim_b = Sim::new(config, observer(7, 5));
    let mut events_b = vec![sim_b.start(&b)];
    for _ in 0..25 {
        events_b.extend(sim_b.tick(&mut b));
    }

    assert_eq!(dump(&a), dump(&b), "per-seed determinism through the loop");
    // Event stream: START first, ticks at interval; the timing
    // fields differ (wall clock) so compare structure only.
    assert!(matches!(events_a.first(), Some(Event::Start { .. })));
    let tick_events = events_a
        .iter()
        .filter(|e| matches!(e, Event::Tick { .. }))
        .count();
    assert_eq!(tick_events, 5, "tick_interval 5 over 25 ticks");
    // G04 held throughout.
    for atom in &a.atoms {
        let max = a.element(atom.element).max_bonds;
        assert!(atom.bond_count <= max);
    }
    // The emitted stream is single-line JSON objects.
    for event in &events_a {
        let line = khem_engine::ndjson::emit(event);
        assert!(line.starts_with("{\"v\":2,") && line.ends_with('}'));
        assert!(!line.contains('\n'));
    }
}
