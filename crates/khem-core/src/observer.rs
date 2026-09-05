//! The observer system: read-only world sampling (guarantee G03,
//! enforced by `&WorldState` in every method signature), molecule
//! detection via union-find on the bond graph, and event composition
//! for the NDJSON output contract.
//!
//! The NDJSON stream is a public, versioned contract (schema v:1)
//! consumed by khem-view and any other tool; serialization lives in
//! [`crate::ndjson`] and tools target the stream, not these types
//! (ADR-0004, ARCHITECTURE.md).
//!
//! Phase-1 scope: START, TICK, BOND_FORMED, BOND_BROKEN, END per
//! runtime spec 3.3. NOTABLE (watch conditions) and SAVE (state
//! persistence) arrive with phase 2; the enum grows additively when
//! they do.
//!
//! Timing fields (elapsed_ms, ticks_per_sec) are wall-clock and
//! therefore excluded from G02's byte-identical guarantee by the
//! documented carve-out: a run is byte-identical modulo those two
//! fields (spec revision note).
//!
//! Spec: docs/specs/runtime-spec.md, sections 3 and 9.

use crate::world::{AtomId, ElementId, WorldState};

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
/// canonical table (phase 3 revisits if custom element tables
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
    },
    BondBroken {
        tick: u64,
        bond_id: u32,
        elem_a: ElementId,
        elem_b: ElementId,
        energy_released: f32,
        x: f32,
        y: f32,
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

/// The observer system interface (runtime spec 10.4; tick order
/// steps 8 and the bookends). Read-only over the world (G03). v0.1
/// compiles exactly one implementation.
pub trait ObserverSystem {
    /// The START event from initial state.
    fn start(&mut self, world: &WorldState) -> Event;
    /// Events for this tick: a TICK event on `tick_interval`
    /// boundaries, plus any watch-condition NOTABLEs (phase 2).
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

    /// Molecule detection (spec 9.2): connected components of live
    /// atoms over live bonds, union-find with path halving, bonds
    /// visited in BondId order for determinism. Returns component
    /// sizes, one per live atom.
    fn molecule_sizes(world: &WorldState) -> Vec<u32> {
        let n = world.atoms.len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        for bond in &world.bonds {
            if !bond.alive {
                continue;
            }
            let mut ra = find(&mut parent, bond.atom_a.0);
            let mut rb = find(&mut parent, bond.atom_b.0);
            if ra != rb {
                if ra > rb {
                    std::mem::swap(&mut ra, &mut rb);
                }
                parent[rb as usize] = ra;
            }
        }
        let mut sizes = vec![0u32; n];
        for atom in &world.atoms {
            if atom.alive {
                let root = find(&mut parent, atom.id.0);
                sizes[root as usize] += 1;
            }
        }
        sizes
    }

    /// World statistics for a TICK event: live counts, field
    /// min/max/avg, free atoms per element, molecule size buckets.
    fn stats(world: &WorldState) -> WorldStats {
        let mut atom_count = 0;
        let mut bond_count = 0;
        let mut free_atoms = [0u32; 10];
        for atom in &world.atoms {
            if !atom.alive {
                continue;
            }
            atom_count += 1;
            if atom.bond_count == 0 {
                free_atoms[atom.element.0 as usize] += 1;
            }
        }
        for bond in &world.bonds {
            if bond.alive {
                bond_count += 1;
            }
        }
        let (temp_min, temp_max, temp_avg) = field_stats(&world.temp_field.data);
        let (pressure_min, pressure_max, pressure_avg) = field_stats(&world.pressure_field.data);
        let mut mol_size_dist = [0u32; 4];
        for size in Self::molecule_sizes(world) {
            if size == 0 {
                // Non-root member slots carry 0; only roots hold
                // component sizes.
                continue;
            }
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

/// Union-find find with path halving, iterative.
fn find(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        parent[i as usize] = parent[parent[i as usize] as usize];
        i = parent[i as usize];
    }
    i
}

impl ObserverSystem for Observer {
    fn start(&mut self, world: &WorldState) -> Event {
        let atom_count = world.atoms.iter().filter(|a| a.alive).count() as u32;
        let bond_count = world.bonds.iter().filter(|b| b.alive).count() as u32;
        Event::Start {
            khem_version: self.config.khem_version,
            run_name: self.config.run_name.clone(),
            world_name: self.config.world_name.clone(),
            seed: self.config.seed,
            atom_count,
            bond_count,
            world_width: world.width,
            world_height: world.height,
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
    use crate::config::PhysicsConfig;
    use crate::elements::ELEMENTS;
    use crate::world::{BoundaryType, ElementId};

    fn world(seed: u64) -> WorldState {
        WorldState::new(
            100.0,
            100.0,
            BoundaryType::Wrap,
            seed,
            PhysicsConfig::default(),
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
            .map(|i| w.spawn_atom(ElementId(1), i as f32, 0.0))
            .collect();
        w.form_bond(ids[0], ids[1], 1, 346.0);
        w.form_bond(ids[1], ids[2], 1, 346.0);
        w.form_bond(ids[2], ids[3], 1, 346.0);
        let sizes = Observer::molecule_sizes(&w);
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
            .map(|i| w.spawn_atom(ElementId(1), i as f32, 0.0))
            .collect();
        for i in 0..6 {
            w.form_bond(ring[i], ring[(i + 1) % 6], 1, 346.0);
        }
        let sizes = Observer::molecule_sizes(&w);
        let nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        assert_eq!(nonzero, vec![6], "the ring is a single 6-molecule");
    }

    #[test]
    fn molecule_sizes_ignores_dead() {
        let mut w = world(1);
        let a = w.spawn_atom(ElementId(1), 0.0, 0.0);
        let b = w.spawn_atom(ElementId(1), 1.0, 0.0);
        w.form_bond(a, b, 1, 346.0);
        w.atom_mut(b).alive = false;
        let sizes = Observer::molecule_sizes(&w);
        let nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
        assert_eq!(nonzero, vec![1], "dead atom not counted");
    }

    #[test]
    fn stats_count_free_atoms_and_buckets() {
        let mut w = world(1);
        // Two free H, one O-H pair, one chain of 6 C.
        w.spawn_atom(ElementId(0), 10.0, 10.0);
        w.spawn_atom(ElementId(0), 20.0, 10.0);
        let o = w.spawn_atom(ElementId(3), 30.0, 10.0);
        let h = w.spawn_atom(ElementId(0), 31.0, 10.0);
        w.form_bond(o, h, 1, 463.0);
        let chain: Vec<AtomId> = (0..6)
            .map(|i| w.spawn_atom(ElementId(1), 40.0 + i as f32, 10.0))
            .collect();
        for i in 0..5 {
            w.form_bond(chain[i], chain[i + 1], 1, 346.0);
        }
        w.temp_field.set(50.0, 50.0, 35.0);
        w.pressure_field.set(50.0, 50.0, 4.0);
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
        w.spawn_atom(ElementId(0), 50.0, 50.0);
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
        w.spawn_atom(ElementId(0), 50.0, 50.0);
        let mut obs = observer(100);
        match obs.start(&w) {
            Event::Start {
                khem_version,
                atom_count,
                world_width,
                seed,
                ..
            } => {
                assert_eq!(khem_version, "0.1.0");
                assert_eq!(atom_count, 1);
                assert_eq!(world_width, 100.0);
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
