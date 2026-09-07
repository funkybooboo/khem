//! The K1 stability harness (PLAN.md phase 1, gate K1):
//!
//!     K1  STABILITY: water and small molecules persist at moderate
//!         temperature. Bonds form and break at plausible rates -
//!         no runaway crosslinking, no frozen inertness.
//!
//! Two levels:
//!
//! - `phase1_loop_smoke` (always on, part of the gate): a short run
//!   asserting invariants that must hold under ANY constants -
//!   determinism, G04, no panics, event stream integrity. Constants
//!   are the gate's business, not this test's.
//! - `k1_diagnostics` (run explicitly with `cargo test -- --ignored
//!   --nocapture`): the honest measurement - water survival, bond
//!   activity, kinetic energy drift, and finiteness after 2000
//!   ticks. This is the number-producing entry point the tuning
//!   commits cite; its assertions encode K1 itself and are expected
//!   to fail until the constants pass the gate.
//!
//! Post-tuning state (rounds 1-3 measured; constants are the
//! spec-11 tuned set, findings F6-F10 in
//! docs/research/abstraction-notes.md): springs are inside the
//! symplectic bound and thermal breaking has a sane temperature
//! profile, but the substrate still has NO dissipation channel,
//! so additive kicks random-walk energy upward forever (F8) and
//! strong_repulsion/r^2 fires cannon-shot impulses that never
//! dissipate (F9). Expected failure signature: mean bond length
//! far above equilibrium, KE huge, field refrigerated negative
//! by formation (F6). The thermostat proposal (abstraction-notes
//! section 10) is the pending owner decision that unblocks K1.

use khem_core::config::PhysicsConfig;
use khem_core::observer::{Event, Timing};
use khem_core::pond::{self, water_intact};
use khem_core::{Observer, ObserverConfig, Sim};

const WATERS: usize = 32 * 32;

fn observer(seed: u64, interval: u64) -> Observer {
    Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k1_harness".to_string(),
        world_name: "primordial_pond".to_string(),
        seed,
        tick_interval: interval,
    })
}

fn run_ticks(seed: u64, ticks: u64) -> khem_core::WorldState {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(seed, config);
    let mut sim = Sim::new(config, observer(seed, 100));
    let _ = sim.start(&world);
    for _ in 0..ticks {
        sim.tick(&mut world);
    }
    world
}

fn kinetic_energy(world: &khem_core::WorldState) -> f64 {
    world
        .atoms
        .iter()
        .filter(|a| a.alive)
        .map(|a| {
            let m = world.element(a.element).mass as f64;
            0.5 * m * (a.vx as f64 * a.vx as f64 + a.vy as f64 * a.vy as f64)
        })
        .sum()
}

#[test]
fn phase1_loop_smoke() {
    // Invariants that must hold under any constants: the full loop
    // runs, is deterministic per seed, respects G04, and produces a
    // well-formed event stream.
    let config = PhysicsConfig::default();

    fn dump(world: &khem_core::WorldState) -> Vec<(f32, f32, u8)> {
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
    // Event stream: START first, END-able, ticks at interval; the
    // timing fields differ (wall clock) so compare structure only.
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
    // The emitted stream parses as single-line JSON objects.
    for event in &events_a {
        let line = khem_core::ndjson::emit(event);
        assert!(line.starts_with("{\"v\":1,") && line.ends_with('}'));
        assert!(!line.contains('\n'));
    }
    let _ = Timing::default();
}

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
/// RUN: `mise exec -- cargo test --release -p khem-core --test
/// k1_stability -- --ignored --nocapture` (release: ~10k ticks in
/// well under a minute; debug is ~20x slower).
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_1_thermostat_flatness() {
    let config = khem_core::PhysicsConfig::default();
    let mut world = khem_core::pond::primordial_pond(42, config);
    let observer = Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k1_1".to_string(),
        world_name: "primordial_pond".to_string(),
        seed: 42,
        tick_interval: 10_000,
    });
    let mut sim = khem_core::Sim::new(config, observer);
    let _ = sim.start(&world);

    let mut ke_samples: Vec<f64> = Vec::new();
    let mut len_samples: Vec<f32> = Vec::new();
    for t in 1..=10_000u64 {
        sim.tick(&mut world);
        if t >= 2000 && t.is_multiple_of(1000) {
            let alive = world.atoms.iter().filter(|a| a.alive).count();
            let ke: f64 = world
                .atoms
                .iter()
                .filter(|a| a.alive)
                .map(|a| {
                    let m = world.element(a.element).mass as f64;
                    0.5 * m * (a.vx as f64).powi(2) + 0.5 * m * (a.vy as f64).powi(2)
                })
                .sum();
            let ke_per_atom = ke / alive as f64;
            let mut lens: Vec<f32> = world
                .bonds
                .iter()
                .filter(|b| b.alive)
                .map(|b| {
                    let a = world.atom(b.atom_a);
                    let c = world.atom(b.atom_b);
                    let (dx, dy) = world.delta(a.x, a.y, c.x, c.y);
                    (dx * dx + dy * dy).sqrt()
                })
                .collect();
            lens.sort_unstable_by(f32::total_cmp);
            let nan = lens.iter().filter(|l| l.is_nan()).count();
            let mean_len = lens.iter().filter(|l| l.is_finite()).sum::<f32>()
                / lens.iter().filter(|l| l.is_finite()).count().max(1) as f32;
            assert_eq!(nan, 0, "K1.1: NaN bond lengths at tick {t}");
            assert!(
                world
                    .atoms
                    .iter()
                    .all(|a| a.x.is_finite() && a.vx.is_finite()),
                "K1.1: non-finite atom state at tick {t}"
            );
            ke_samples.push(ke_per_atom);
            len_samples.push(mean_len);
            eprintln!("t={t} KE/atom={ke_per_atom:.4} mean_len={mean_len:.3}");
        }
    }
    assert!(ke_samples.len() >= 8, "sampling bug");

    let window = |xs: &[f64], from: usize, to: usize| -> f64 {
        xs[from..to].iter().sum::<f64>() / (to - from) as f64
    };
    let ke_early = window(&ke_samples, 0, 4); // ticks 2k..6k
    let ke_late = window(&ke_samples, 4, 8); // ticks 6k..10k
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

#[test]
#[ignore] // explicit: cargo test -- --ignored --nocapture
fn k1_diagnostics() {
    let world = run_ticks(42, 2000);
    let intact = water_intact(&world);
    let alive = world.atoms.iter().filter(|a| a.alive).count();
    let live_bonds = world.bonds.iter().filter(|b| b.alive).count();
    let formed: usize = world
        .bonds
        .iter()
        .filter(|b| b.id.0 as usize >= WATERS * 2)
        .filter(|b| b.alive)
        .count();
    let ke = kinetic_energy(&world);
    let finite = world
        .atoms
        .iter()
        .all(|a| a.x.is_finite() && a.y.is_finite() && a.vx.is_finite() && a.vy.is_finite());
    let temp_avg: f32 =
        world.temp_field.data.iter().sum::<f32>() / world.temp_field.data.len() as f32;
    // Bond geometry health: the random-walk probe. With no
    // dissipation channel, additive thermal kicks pump oscillator
    // energy forever; bond lengths balloon even while bonds never
    // break.
    let mut lens: Vec<f32> = world
        .bonds
        .iter()
        .filter(|b| b.alive)
        .map(|b| {
            let a = world.atom(b.atom_a);
            let c = world.atom(b.atom_b);
            ((a.x - c.x).powi(2) + (a.y - c.y).powi(2)).sqrt()
        })
        .collect();
    // total_cmp puts NaN at the end; count them explicitly rather
    // than panicking in the sort (a NaN bond length is a finding,
    // not a test-harness crash).
    lens.sort_unstable_by(f32::total_cmp);
    let nan_lens = lens.iter().filter(|l| l.is_nan()).count();
    let mean_len = lens.iter().filter(|l| l.is_finite()).sum::<f32>()
        / lens.iter().filter(|l| l.is_finite()).count().max(1) as f32;
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

    // K1 itself. Expected to fail until the structural findings
    // (F8/F9) are resolved; the failure message states the gate,
    // not a bug.
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
    // Bonds ever formed minus those still alive = bonds that
    // broke (dead bonds stay in the vec; ids are never reused).
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
