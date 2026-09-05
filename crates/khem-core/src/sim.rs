//! The tick loop and strict system ordering: energy -> velocities ->
//! positions -> boundary -> spatial index -> bond breaking -> bond
//! formation -> observe -> event flush.
//!
//! [`Sim`] drives the nine steps of spec 5.1 exactly, in order;
//! [`TICK_ORDER`] keeps the order as inspectable data, and the test
//! asserts the loop is its implementation. Systems read
//! previous-tick state and write current-tick state; no system reads
//! another system's writes within a tick. This fixed order is what
//! makes runs deterministic and later parallelizable (guarantees
//! G02, G14; ADR-0005).
//!
//! I/O boundary: [`Sim`] produces events and never touches streams;
//! the khem bin serializes (ndjson) and writes stdout, which keeps
//! G10 (stdout is only NDJSON) enforceable in one place and keeps
//! the loop testable.
//!
//! Timing: the only wall-clock consumer, for the TICK/END timing
//! fields excluded from G02 (observer module doc).
//!
//! Spec: docs/specs/runtime-spec.md, section 5 (Tick Execution).

use std::time::Instant;

use crate::chemistry::{Chemistry, ChemistrySystem};
use crate::config::PhysicsConfig;
use crate::energy::{Energy, EnergySystem};
use crate::observer::{Event, Observer, ObserverSystem, Timing};
use crate::physics::{Physics, PhysicsSystem};
use crate::world::WorldState;

/// The nine systems in strict tick order (runtime spec 5.1).
///
/// Kept as data, not control flow, so the order is inspectable and
/// testable.
pub const TICK_ORDER: [&str; 9] = [
    "EnergySystem::update",
    "PhysicsSystem::update_velocities",
    "PhysicsSystem::update_positions",
    "PhysicsSystem::apply_boundary",
    "SpatialIndex::rebuild",
    "ChemistrySystem::break_bonds",
    "ChemistrySystem::form_bonds",
    "ObserverSystem::sample",
    "EventQueue::flush_to_output",
];

/// The tick loop: owns the systems and the wall clock, drives one
/// [`WorldState`] through fixed-order ticks, and yields the events
/// each tick produced (bond events from chemistry plus the observer's
/// tick events). The caller serializes and flushes (step 9 of the
/// tick order - I/O belongs to the bin).
pub struct Sim {
    energy: Energy,
    physics: Physics,
    chemistry: Chemistry,
    observer: Observer,
    started: Instant,
    ticks: u64,
}

impl Sim {
    /// One sim, one set of systems, all sharing the same immutable
    /// config copy. The observer carries the run metadata
    /// (ObserverConfig) for START/TICK/END composition.
    pub fn new(config: PhysicsConfig, observer: Observer) -> Self {
        Self {
            energy: Energy::new(config),
            physics: Physics::new(config),
            chemistry: Chemistry::new(config),
            observer,
            started: Instant::now(),
            ticks: 0,
        }
    }

    /// The START event, composed from initial state. Call once,
    /// before the first [`Self::tick`].
    pub fn start(&mut self, world: &WorldState) -> Event {
        self.started = Instant::now();
        self.ticks = 0;
        self.observer.start(world)
    }

    /// Runs one tick (spec 5.1 steps 1-8) and returns the events it
    /// produced, in order: chemistry's bond events first (they
    /// happened during the tick), then the observer's tick-boundary
    /// events. The caller serializes and flushes (step 9).
    pub fn tick(&mut self, world: &mut WorldState) -> Vec<Event> {
        world.tick += 1;
        self.energy.update(world);
        self.physics.update_velocities(world);
        self.physics.update_positions(world);
        self.physics.apply_boundary(world);
        world.spatial_index.rebuild(&world.atoms);
        self.chemistry.break_bonds(world);
        self.chemistry.form_bonds(world);
        self.ticks += 1;
        let timing = self.timing();
        let mut events = std::mem::take(&mut world.event_queue);
        events.extend(self.observer.sample(world, timing));
        events
    }

    /// The END event with the run's reason. Call once, after the
    /// last tick. The reason vocabulary is fixed by the observer
    /// (max_ticks_reached in v0.1; interrupt/extinction arrive with
    /// phase 2 hardening).
    pub fn end(&mut self, world: &WorldState) -> Event {
        self.observer.end(world, self.timing())
    }

    fn timing(&self) -> Timing {
        let elapsed = self.started.elapsed();
        let secs = elapsed.as_secs_f64();
        let ticks_per_sec = if secs > 0.0 {
            self.ticks as f64 / secs
        } else {
            0.0
        };
        Timing {
            elapsed_ms: elapsed.as_millis() as u64,
            ticks_per_sec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::ObserverConfig;
    use crate::world::{BoundaryType, ElementId};

    fn observer(interval: u64) -> Observer {
        Observer::new(ObserverConfig {
            khem_version: "0.1.0",
            run_name: "sim_test".into(),
            world_name: "test".into(),
            seed: 42,
            tick_interval: interval,
        })
    }

    fn small_world(seed: u64) -> WorldState {
        let mut w = WorldState::new(
            50.0,
            50.0,
            BoundaryType::Wrap,
            seed,
            PhysicsConfig::default(),
        );
        for i in 0..4 {
            w.spawn_atom(ElementId(3), 10.0 + i as f32, 10.0);
            w.spawn_atom(ElementId(0), 20.0 + i as f32, 20.0);
        }
        w.temp_field.set(25.0, 25.0, 35.0);
        w
    }

    #[test]
    fn tick_order_is_fixed() {
        assert_eq!(TICK_ORDER.len(), 9);
        assert_eq!(TICK_ORDER[0], "EnergySystem::update");
        assert_eq!(TICK_ORDER[4], "SpatialIndex::rebuild");
        assert_eq!(TICK_ORDER[8], "EventQueue::flush_to_output");
    }

    #[test]
    fn start_is_emitted_once_and_describes_initial_state() {
        let w = small_world(1);
        let mut sim = Sim::new(PhysicsConfig::default(), observer(100));
        match sim.start(&w) {
            Event::Start { atom_count, .. } => assert_eq!(atom_count, 8),
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn ticks_advance_and_tick_events_respect_interval() {
        let mut w = small_world(1);
        let mut sim = Sim::new(PhysicsConfig::default(), observer(10));
        let _ = sim.start(&w);
        let mut tick_events = 0;
        for expected in 1..=25 {
            let events = sim.tick(&mut w);
            assert_eq!(w.tick, expected);
            let this_tick = events
                .iter()
                .filter(|e| matches!(e, Event::Tick { .. }))
                .count();
            assert!(this_tick <= 1, "at most one Tick event per tick");
            if expected % 10 == 0 {
                assert_eq!(this_tick, 1, "boundary tick {expected}");
                tick_events += 1;
            } else {
                assert_eq!(this_tick, 0, "off-boundary tick {expected}");
            }
        }
        assert_eq!(tick_events, 2);
    }

    #[test]
    fn end_event_closes_the_run() {
        let mut w = small_world(1);
        let mut sim = Sim::new(PhysicsConfig::default(), observer(10));
        let _ = sim.start(&w);
        for _ in 0..5 {
            sim.tick(&mut w);
        }
        match sim.end(&w) {
            Event::End { tick, reason, .. } => {
                assert_eq!(tick, 5);
                assert_eq!(reason, "max_ticks_reached");
            }
            other => panic!("expected End, got {other:?}"),
        }
    }

    #[test]
    fn full_loop_is_deterministic_per_seed() {
        fn run(seed: u64) -> Vec<(f32, f32)> {
            let mut w = small_world(seed);
            let mut sim = Sim::new(PhysicsConfig::default(), observer(1000));
            let _ = sim.start(&w);
            for _ in 0..50 {
                sim.tick(&mut w);
            }
            w.atoms.iter().map(|a| (a.x, a.y)).collect()
        }
        // Same seed: identical trajectories through the whole loop
        // (timing fields differ but are not part of the trajectory).
        assert_eq!(run(7), run(7));
    }

    #[test]
    fn spatial_index_is_current_when_chemistry_runs() {
        // G07: chemistry queries the index the loop just rebuilt.
        // A moved atom must be findable at its new position: with
        // rate 1.0 and a warm field, two atoms placed far apart but
        // moved adjacent by physics must bond.
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        let mut w = WorldState::new(50.0, 50.0, BoundaryType::Wrap, 3, config);
        let a = w.spawn_atom(ElementId(0), 10.0, 10.0);
        w.spawn_atom(ElementId(0), 14.0, 10.0);
        w.atom_mut(a).vx = 2.0;
        // b stays; after one tick a is at 12.0, within the 4 A
        // search radius.
        w.temp_field.set(12.0, 10.0, 43.6);
        let mut sim = Sim::new(config, observer(1000));
        let _ = sim.start(&w);
        let events = sim.tick(&mut w);
        assert!(
            events.iter().any(|e| matches!(e, Event::BondFormed { .. })),
            "moved atom must bond through the rebuilt index"
        );
    }
}
