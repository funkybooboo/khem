//! The K1 stability harness (PLAN.md gate ladder, milestone K1).
//!
//! Three levels:
//!
//! - `phase1_loop_smoke` (always on, part of `mise run check`):
//!   invariants that must hold under ANY constants - determinism,
//!   G04, event stream integrity. Constants are the gates'
//!   business, not this test's.
//! - `k1_1_thermostat_flatness` (gate K1.1, PASSED 2026-09-05):
//!   KE per atom and mean bond length both flat over a 10k-tick
//!   vented-pond run. Run: `mise exec -- cargo test --release -p
//!   khem-core --test k1_stability -- --ignored --nocapture`
//!   (release: ~80 s; debug is ~20x slower).
//! - `k1_diagnostics` (the K1 rollup): the honest measurement
//!   after 2000 ticks - water survival, bond activity, geometry,
//!   energy. Expected to fail until the remaining sub-gates
//!   (K1.2-K1.5) pass; the ladder entries own their criteria.
//!
//! Findings and gate history live in
//! docs/research/abstraction-notes.md.

use khem_core::config::PhysicsConfig;
use khem_core::observer::Event;
use khem_core::pond::{self, water_intact};
use khem_core::{Observer, ObserverConfig, Sim, WorldState};

const WATERS: usize = 32 * 32;

// ---- Shared helpers ----------------------------------------------------

fn observer(seed: u64, interval: u64) -> Observer {
    Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k1_harness".to_string(),
        world_name: "primordial_pond".to_string(),
        seed,
        tick_interval: interval,
    })
}

fn run_ticks(seed: u64, ticks: u64) -> WorldState {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(seed, config);
    let mut sim = Sim::new(config, observer(seed, 100));
    let _ = sim.start(&world);
    for _ in 0..ticks {
        sim.tick(&mut world);
    }
    world
}

/// Mean of the finite live-bond lengths (NaN lengths are a finding;
/// the caller counts them separately via `bond_lengths`).
fn mean_bond_length(world: &WorldState) -> f32 {
    let lens = world.bond_lengths();
    let finite: Vec<f32> = lens.iter().copied().filter(|l| l.is_finite()).collect();
    finite.iter().sum::<f32>() / finite.len().max(1) as f32
}

/// All atom state finite (no NaN/inf positions or velocities).
fn all_state_finite(world: &WorldState) -> bool {
    world
        .atoms
        .iter()
        .all(|a| a.x.is_finite() && a.y.is_finite() && a.vx.is_finite() && a.vy.is_finite())
}

// ---- Always-on invariants ---------------------------------------------

#[test]
fn phase1_loop_smoke() {
    // Invariants that must hold under any constants: the full loop
    // runs, is deterministic per seed, respects G04, and produces a
    // well-formed event stream.
    let config = PhysicsConfig::default();

    fn dump(world: &WorldState) -> Vec<(f32, f32, u8)> {
        world
            .atoms
            .iter()
            .map(|a| (a.x, a.y, a.bond_count))
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
        let line = khem_core::ndjson::emit(event);
        assert!(line.starts_with("{\"v\":1,") && line.ends_with('}'));
        assert!(!line.contains('\n'));
    }
}

// ---- Gate K1.1: thermostat flatness ------------------------------------

/// K1.1 THERMOSTAT (gate ladder): KE per atom AND mean bond length
/// both go FLAT over a 10k-tick vented-pond run.
///
/// The pond is the canonical vented one (vent at the floor, 35 C
/// setpoint reservoir): heat in from the vent, heat out through
/// setpoint relaxation - a genuine steady state to be flat
/// against. The thermostat must hold atoms at the local field
/// temperature (no random-walk heating, no furnace, no
/// dissociation cascade) and bond geometry must settle.
///
/// PASS: comparing window means, ticks 2k-6k vs 6k-10k, both
/// metrics within 15% (threshold is a starting point; the ladder
/// lets evidence move it), all values finite throughout.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_1_thermostat_flatness() {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 10_000));
    let _ = sim.start(&world);

    let mut ke_samples: Vec<f64> = Vec::new();
    let mut len_samples: Vec<f32> = Vec::new();
    for t in 1..=10_000u64 {
        sim.tick(&mut world);
        if t >= 2000 && t.is_multiple_of(1000) {
            assert!(
                all_state_finite(&world),
                "K1.1: non-finite state at tick {t}"
            );
            let nan_lens = world
                .bond_lengths()
                .iter()
                .filter(|l| !l.is_finite())
                .count();
            assert_eq!(nan_lens, 0, "K1.1: NaN bond lengths at tick {t}");
            let ke_per_atom = world.kinetic_energy() / world.live_atom_count() as f64;
            let mean_len = mean_bond_length(&world);
            ke_samples.push(ke_per_atom);
            len_samples.push(mean_len);
            eprintln!("t={t} KE/atom={ke_per_atom:.4} mean_len={mean_len:.3}");
        }
    }
    assert!(ke_samples.len() >= 8, "sampling bug");

    let ke_early = ke_samples[0..4].iter().sum::<f64>() / 4.0; // ticks 2k..6k
    let ke_late = ke_samples[4..8].iter().sum::<f64>() / 4.0; // ticks 6k..10k
    let len_early = len_samples[0..4].iter().sum::<f32>() / 4.0;
    let len_late = len_samples[4..8].iter().sum::<f32>() / 4.0;

    eprintln!(
        "KE/atom early {ke_early:.4} late {ke_late:.4} ({:+.1}%)",
        100.0 * (ke_late - ke_early) / ke_early
    );
    eprintln!(
        "mean_len early {len_early:.3} late {len_late:.3} ({:+.1}%)",
        100.0 * (len_late - len_early) / len_early
    );
    assert!(
        (ke_late - ke_early).abs() / ke_early < 0.15,
        "K1.1 FAIL: KE/atom drifted {:.1}%",
        100.0 * (ke_late - ke_early) / ke_early
    );
    assert!(
        (len_late - len_early).abs() / len_early < 0.15,
        "K1.1 FAIL: mean bond length drifted {:.1}%",
        100.0 * (len_late - len_early) / len_early
    );
}

// ---- K1 rollup diagnostics ---------------------------------------------

#[test]
#[ignore] // explicit: cargo test -- --ignored --nocapture
fn k1_diagnostics() {
    let world = run_ticks(42, 2000);
    let intact = water_intact(&world);
    let alive = world.live_atom_count();
    let live_bonds = world.live_bond_count();
    // Bonds with ids past the seeded ones are the pond's own
    // chemistry; still-alive among them = formed-and-alive.
    let formed: usize = world
        .bonds
        .iter()
        .filter(|b| b.id.0 as usize >= WATERS * 2)
        .filter(|b| b.alive)
        .count();
    let ke = world.kinetic_energy();
    let finite = all_state_finite(&world);
    let temp_avg: f32 =
        world.temp_field.data.iter().sum::<f32>() / world.temp_field.data.len() as f32;
    let mut lens = world.bond_lengths();
    // total_cmp puts NaN at the end; count them explicitly rather
    // than panicking in the sort (a NaN bond length is a finding,
    // not a test-harness crash).
    lens.sort_unstable_by(f32::total_cmp);
    let nan_lens = lens.iter().filter(|l| !l.is_finite()).count();
    let mean_len = mean_bond_length(&world);
    let p95_len = if lens.is_empty() {
        0.0
    } else {
        lens[lens.len() * 95 / 100]
    };

    eprintln!("K1 diagnostics after 2000 ticks (seed 42):");
    eprintln!("  alive atoms:        {alive}");
    eprintln!("  intact waters:      {intact} / {WATERS}");
    eprintln!(
        "  live bonds:         {live_bonds} (initial {})",
        WATERS * 2
    );
    eprintln!("  formed-and-alive:   {formed}");
    eprintln!("  kinetic energy:     {ke:.3e}");
    eprintln!("  avg temperature:    {temp_avg:.3}");
    eprintln!("  bond len mean/p95:  {mean_len:.3} / {p95_len:.3} A (equilibrium ~1.2)");
    eprintln!("  nan bond lengths:   {nan_lens}");
    eprintln!("  all finite:         {finite}");

    // K1's own bars. The sub-gates (K1.2-K1.5) own the finer
    // criteria; these are the milestone's rollup.
    assert!(
        intact as f64 / WATERS as f64 > 0.9,
        "K1 persistence: only {intact}/{WATERS} waters intact"
    );
    assert!(finite, "K1 stability: positions/velocities not finite");
    // Geometry standard: molecules stay COHERENT (no shards, no
    // comets), at the flop level this substrate's dt=1 symplectic
    // bound allows. Real water's bond-PE/kT ratio is ~80; the
    // stability bound caps this substrate near ~3, so mean stretch
    // around equilibrium is inherent until collision sub-stepping
    // (phase 2) permits stiffer springs. The numbers below detect
    // the failure modes that existed: 80 A comets (mean >> p95)
    // and full shard blowups.
    assert!(
        mean_len < 3.5,
        "K1 geometry: mean bond length {mean_len:.3} A beyond coherent flop"
    );
    assert!(
        p95_len < 5.0,
        "K1 geometry: p95 bond length {p95_len:.3} A - shard tail"
    );
    assert!(
        formed > 0,
        "K1 activity: no new bonds formed (frozen inertness)"
    );
    // Bonds ever formed minus those still alive = bonds that broke
    // (dead bonds stay in the vec; ids are never reused).
    let broken = world.bonds.len() - WATERS * 2 - formed;
    eprintln!("  broken-ever:        {broken}");
    assert!(
        broken > 0,
        "K1 activity: no bond ever broke (frozen breaking)"
    );
    assert!(
        live_bonds <= WATERS * 3,
        "K1 crosslinking: {live_bonds} bonds from {} initial",
        WATERS * 2
    );
}
