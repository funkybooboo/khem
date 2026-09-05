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
    lens.sort_unstable_by(|x, y| x.partial_cmp(y).unwrap());
    let mean_len = lens.iter().sum::<f32>() / lens.len() as f32;
    let p95_len = lens[lens.len() * 95 / 100];

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
    eprintln!("  all finite:         {finite}");

    // K1 itself. Expected to fail until the structural findings
    // (F8/F9) are resolved; the failure message states the gate,
    // not a bug.
    assert!(
        intact as f64 / WATERS as f64 > 0.9,
        "K1 persistence: only {intact}/{WATERS} waters intact"
    );
    assert!(finite, "K1 stability: positions/velocities not finite");
    assert!(
        mean_len < 2.0,
        "K1 geometry: mean bond length {mean_len:.3} A vs ~1.2 equilibrium"
    );
    assert!(
        formed > 0,
        "K1 activity: no new bonds formed (frozen inertness)"
    );
    assert!(
        live_bonds <= WATERS * 3,
        "K1 crosslinking: {live_bonds} bonds from {} initial",
        WATERS * 2
    );
}
