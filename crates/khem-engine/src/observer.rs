//! The observer system: read-only world sampling (guarantee G03,
//! enforced by `&WorldState` in every method signature), molecule
//! detection via union-find on the bond graph, and event composition
//! for the NDJSON output contract.
//!
//! The NDJSON stream is a public, versioned contract (schema v:1)
//! consumed by khem-view and any other tool; serialization lives in
//! [`crate::ndjson`] and tools target the stream, not these types
//! (ADR-0004, docs/build/architecture.md).
//!
//! Phase-1 scope: START, TICK, BOND_FORMED, BOND_BROKEN, END per
//! runtime spec 3.3. NOTABLE (watch conditions) and SAVE (state
//! persistence) arrive with phase 3; the enum grows additively when
//! they do.
//!
//! Spec: docs/build/specs/runtime-spec.md, sections 3 and 9.
//!
//! The event model ([`Event`], [`Timing`], [`WorldStats`]) lives
//! in khem-world (the systems that mutate the world produce the
//! events); the molecule census lives in khem-analysis (this
//! observer's buckets are its summary). This module is the
//! observer BEHAVIOR: read-only sampling and run-event
//! composition.

use khem_world::{Event, Timing, WorldState, WorldStats};

/// The observer system interface (runtime spec 10.4; tick order
/// steps 8 and the bookends). Read-only over the world (G03). v0.1
/// compiles exactly one implementation.
pub trait ObserverSystem {
    /// The START event from initial state.
    fn start(&mut self, world: &WorldState) -> Event;
    /// Events for this tick: a TICK event on `tick_interval`
    /// boundaries, plus any watch-condition NOTABLEs (phase 3).
    fn sample(&mut self, world: &WorldState, timing: Timing) -> Vec<Event>;
    /// The END event with the run's reason.
    fn end(&mut self, world: &WorldState, timing: Timing) -> Event;
}

/// Run metadata the observer needs for the START event. Phase 3
/// reads these from the run declaration; phase 1 hardcodes them.
#[derive(Debug, Clone)]
pub struct ObserverConfig {
    pub khem_version: &'static str,
    pub run_name: String,
    pub world_name: String,
    pub seed: u64,
    pub tick_interval: u64,
}

/// The v0.1 observer implementation (runtime spec 10.4: exactly one).
pub struct Observer {
    config: ObserverConfig,
}

impl Observer {
    pub fn new(config: ObserverConfig) -> Self {
        Self { config }
    }

    /// World statistics for a TICK event: live counts, field
    /// min/max/avg, free atoms per element, molecule size buckets.
    fn stats(world: &WorldState) -> WorldStats {
        let atom_count = world.live_atom_count() as u32;
        let bond_count = world.live_bond_count() as u32;
        let mut free_atoms = [0u32; 10];
        for atom in world.atoms.iter().filter(|a| a.alive) {
            if atom.bond_count == 0 {
                free_atoms[atom.element.0 as usize] += 1;
            }
        }
        let (temp_min, temp_max, temp_avg) = field_stats(&world.temp_field.data);
        let (pressure_min, pressure_max, pressure_avg) = field_stats(&world.pressure_field.data);
        let mut mol_size_dist = [0u32; 4];
        for size in khem_analysis::molecule_sizes(world) {
            let bucket = match size {
                1 => 0,
                2..=5 => 1,
                6..=20 => 2,
                _ => 3,
            };
            mol_size_dist[bucket] += 1;
        }
        WorldStats {
            atom_count,
            bond_count,
            temp_min,
            temp_max,
            temp_avg,
            pressure_min,
            pressure_max,
            pressure_avg,
            free_atoms,
            mol_size_dist,
        }
    }
}

fn field_stats(data: &[f32]) -> (f32, f32, f32) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut sum = 0.0f32;
    for v in data {
        min = min.min(*v);
        max = max.max(*v);
        sum += *v;
    }
    (min, max, sum / data.len() as f32)
}

impl ObserverSystem for Observer {
    fn start(&mut self, world: &WorldState) -> Event {
        let atom_count = world.live_atom_count() as u32;
        let bond_count = world.live_bond_count() as u32;
        Event::Start {
            khem_version: self.config.khem_version,
            run_name: self.config.run_name.clone(),
            world_name: self.config.world_name.clone(),
            seed: self.config.seed,
            atom_count,
            bond_count,
            world_width: world.width,
            world_height: world.height,
            world_depth: world.depth,
        }
    }

    fn sample(&mut self, world: &WorldState, timing: Timing) -> Vec<Event> {
        if world.tick == 0 || !world.tick.is_multiple_of(self.config.tick_interval) {
            return Vec::new();
        }
        vec![Event::Tick {
            tick: world.tick,
            timing,
            stats: Self::stats(world),
        }]
    }

    fn end(&mut self, world: &WorldState, timing: Timing) -> Event {
        Event::End {
            tick: world.tick,
            timing,
            reason: "max_ticks_reached",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use khem_forcefield::ELEMENTS;
    use khem_forcefield::ForceField;
    use khem_world::AtomId;
    use khem_world::{BoundaryType, ElementId};

    fn world(seed: u64) -> WorldState {
        WorldState::new(
            100.0,
            100.0,
            100.0,
            BoundaryType::Wrap,
            seed,
            ForceField::default(),
        )
    }

    fn observer(interval: u64) -> Observer {
        Observer::new(ObserverConfig {
            khem_version: "0.1.0",
            run_name: "test_run".into(),
            world_name: "test_world".into(),
            seed: 42,
            tick_interval: interval,
        })
    }

    fn timing() -> Timing {
        Timing {
            elapsed_ms: 1000,
            ticks_per_sec: 500.0,
        }
    }

    #[test]
    fn molecule_sizes_chain_and_lone_atoms() {
        let mut w = world(1);
        // A chain a-b-c-d (3 bonds) plus two lone atoms.
        let ids: Vec<AtomId> = (0..6)
            .map(|i| w.spawn_atom(ElementId(1), i as f32, 0.0, 0.0))
            .collect();
        w.form_bond(ids[0], ids[1], 1, 346.0);
        w.form_bond(ids[1], ids[2], 1, 346.0);
        w.form_bond(ids[2], ids[3], 1, 346.0);
        let sizes = khem_analysis::molecule_sizes(&w);
        let mut nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        nonzero.sort_unstable();
        assert_eq!(nonzero, vec![1, 1, 4], "chain of 4 plus two lones");
    }

    #[test]
    fn molecule_sizes_handles_cycles() {
        // A ring is one molecule: union-find must not double-count
        // or split when bonds close a cycle.
        let mut w = world(1);
        let ring: Vec<AtomId> = (0..6)
            .map(|i| w.spawn_atom(ElementId(1), i as f32, 0.0, 0.0))
            .collect();
        for i in 0..6 {
            w.form_bond(ring[i], ring[(i + 1) % 6], 1, 346.0);
        }
        let sizes = khem_analysis::molecule_sizes(&w);
        let nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        assert_eq!(nonzero, vec![6], "the ring is a single 6-molecule");
    }

    #[test]
    fn molecule_sizes_ignores_dead() {
        let mut w = world(1);
        let a = w.spawn_atom(ElementId(1), 0.0, 0.0, 0.0);
        let b = w.spawn_atom(ElementId(1), 1.0, 0.0, 0.0);
        w.form_bond(a, b, 1, 346.0);
        w.atom_mut(b).alive = false;
        let sizes = khem_analysis::molecule_sizes(&w);
        let nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        assert_eq!(nonzero, vec![1], "dead atom not counted");
    }

    #[test]
    fn stats_count_free_atoms_and_buckets() {
        let mut w = world(1);
        // Two free H, one O-H pair, one chain of 6 C.
        w.spawn_atom(ElementId(0), 10.0, 10.0, 10.0);
        w.spawn_atom(ElementId(0), 20.0, 10.0, 10.0);
        let o = w.spawn_atom(ElementId(3), 30.0, 10.0, 10.0);
        let h = w.spawn_atom(ElementId(0), 31.0, 10.0, 10.0);
        w.form_bond(o, h, 1, 463.0);
        let chain: Vec<AtomId> = (0..6)
            .map(|i| w.spawn_atom(ElementId(1), 40.0 + i as f32, 10.0, 10.0))
            .collect();
        for i in 0..5 {
            w.form_bond(chain[i], chain[i + 1], 1, 346.0);
        }
        w.temp_field.set(50.0, 50.0, 50.0, 35.0);
        w.pressure_field.set(50.0, 50.0, 50.0, 4.0);
        let s = Observer::stats(&w);
        assert_eq!(s.atom_count, 10);
        assert_eq!(s.bond_count, 6);
        assert_eq!(s.free_atoms[0], 2, "the two unbonded H are free");
        // Molecules: 2 free H (bucket "1"), 1 pair ("2_5"),
        // 1 chain of 6 ("6_20"), none 21+.
        assert_eq!(s.mol_size_dist, [2, 1, 1, 0]);
        assert_eq!(s.temp_max, 35.0);
        assert_eq!(s.pressure_max, 4.0);
    }

    #[test]
    fn tick_events_respect_interval() {
        let mut w = world(1);
        w.spawn_atom(ElementId(0), 50.0, 50.0, 50.0);
        let mut obs = observer(1000);
        for t in 1..=1500 {
            w.tick = t;
            let events = obs.sample(&w, timing());
            let expect = (t % 1000 == 0) as usize;
            assert_eq!(events.len(), expect, "tick {t}");
        }
        // Tick 0 (before the first tick) never samples.
        w.tick = 0;
        assert!(obs.sample(&w, timing()).is_empty());
    }

    #[test]
    fn start_and_end_events_match_schema_fields() {
        let mut w = world(7);
        w.spawn_atom(ElementId(0), 50.0, 50.0, 50.0);
        let mut obs = observer(100);
        match obs.start(&w) {
            Event::Start {
                khem_version,
                atom_count,
                world_width,
                world_depth,
                seed,
                ..
            } => {
                assert_eq!(khem_version, "0.1.0");
                assert_eq!(atom_count, 1);
                assert_eq!(world_width, 100.0);
                assert_eq!(world_depth, 100.0);
                assert_eq!(seed, 42);
            }
            other => panic!("expected Start, got {other:?}"),
        }
        match obs.end(&w, timing()) {
            Event::End { tick, reason, .. } => {
                assert_eq!(tick, 0);
                assert_eq!(reason, "max_ticks_reached");
            }
            other => panic!("expected End, got {other:?}"),
        }
    }

    #[test]
    fn element_table_is_canonical_ten() {
        // The observer's free_atoms array is sized for the canonical
        // table; guard the assumption.
        assert_eq!(ELEMENTS.len(), 10);
    }
}
