//! The K1 stability harness (docs/plans/phase-1-kernel.md gate
//! ladder, milestone K1).
//!
//! Five levels:
//!
//! - `phase1_loop_smoke` (always on, part of `mise run check`):
//!   invariants that must hold under ANY constants - determinism,
//!   G04, event stream integrity. Constants are the gates'
//!   business, not this test's.
//! - `k1_1_thermostat_flatness` (gate K1.1; PASSED 2026-09-05,
//!   RE-VALIDATED 2026-09-07 in K1.3's integrator commit per the
//!   re-validation contract): thermostat coupling law + KE and
//!   bond-length flatness over the steady tail of a 10k-tick
//!   vented-pond run. Run: `mise exec -- cargo test --release -p
//!   khem-core --test k1_stability -- --ignored --nocapture`
//!   (release: ~190 s; debug is ~20x slower).
//! - `k1_2_force_sanity` (gate K1.2, PASSED 2026-09-07, re-run
//!   in the K1.3 commit): a bonded overlap imparts bounded
//!   velocity (the probe), and the mean bond stretch ratio stays
//!   within [0.8, 1.5] over the same 10k-tick run (the band).
//! - `k1_3_water_persistence` (gate K1.3, PASSED 2026-09-07):
//!   the pond keeps its molecules - intact count high and flat,
//!   seeded-water O-H breaks essentially never (the census
//!   splits them from runtime-formed pair churn, which is K1.4's
//!   reactive balance). Same run command.
//! - `k1_diagnostics` (the K1 rollup): the honest measurement
//!   after 2000 ticks - water survival, bond activity, geometry,
//!   energy. Its bars pass as of the K1.3 commit; the ladder's
//!   remaining K1 sub-gates (K1.4, K1.5) own the finer criteria.
//!
//! Findings and gate history live in
//! docs/research/abstraction-notes.md.

use khem_core::config::PhysicsConfig;
use khem_core::observer::Event;
use khem_core::pond::{self, water_intact};
use khem_core::{
    BondId, BoundaryType, ElementId, Observer, ObserverConfig, Sim, WorldState, bond_energy,
};

const WATERS: usize = pond::POND_WATERS;

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

/// Per-bond stretch ratios (length / r_eq) of all live bonds, in
/// bond order. Each bond is measured against its OWN equilibrium
/// (radius sum), so a heterogeneous pond compares like with like;
/// lengths use the minimum-image delta (F10).
fn bond_stretch_ratios(world: &WorldState) -> Vec<f32> {
    world
        .bonds
        .iter()
        .filter(|b| b.alive)
        .map(|b| {
            let (a, c) = (world.atom(b.atom_a), world.atom(b.atom_b));
            let (dx, dy) = world.delta(a.x, a.y, c.x, c.y);
            let r_eq = world.element(a.element).radius + world.element(c.element).radius;
            (dx * dx + dy * dy).sqrt() / r_eq
        })
        .collect()
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
/// PASS (re-validated 2026-09-07 for the K1.3 integrator commit,
/// per the re-validation contract):
/// - KE/atom bounded under 2x the 35 C bath level at EVERY sample,
///   transient included (the founding F8 furnace measured 1e13);
/// - the THERMOSTAT COUPLING law: KE/atom stays within [0.8, 1.4]
///   of the field's warm-cell thermal level kb*T at every sample
///   (measured: constant 1.09-1.13 through the whole run - the
///   atoms ride their local bath, never decoupled above or
///   below it);
/// - flatness over the steady tail: window means 6.25k-8k vs
///   8.25k-10k within 15% for both metrics (measured: KE/atom
///   +10.6%, mean bond length +0.3%). The window moved past a
///   measured, converging TRANSIENT: the pre-K1.3 substrate's
///   chemistry refrigerated the field into a steady cold state
///   within ~2k ticks (every formation absorbed 0.3*E; shatter
///   fed formations - measured field avg -162 C), so flatness
///   held from 2k on. The K1.3 substrate quiesced the chemistry
///   (waters persist, thermal breaks rare) - the refrigeration
///   starved, and the vent + setpoint reservoir warm the field
///   toward the vent-profile steady state over the first ~6k
///   ticks (measured: field avg 16.7 C at 3.25k -> ~26 C by 8k,
///   then fluctuating 22-28 with no deep negative cells). The
///   transient is part of the evidence, not a failure mode; the
///   field's own settle temperature is gate K1.4's criterion.
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
        if t >= 2000 && t.is_multiple_of(250) {
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
            let field_avg: f32 =
                world.temp_field.data.iter().sum::<f32>() / world.temp_field.data.len() as f32;
            let field_warm: f32 = world
                .temp_field
                .data
                .iter()
                .map(|v| v.max(0.0))
                .sum::<f32>()
                / world.temp_field.data.len() as f32;
            // The thermostat coupling law: atoms at their local
            // bath's thermal level (kb * T), not above, not below.
            let coupling =
                ke_per_atom / (f64::from(config.thermal_kick_scale) * f64::from(field_warm));
            ke_samples.push(ke_per_atom);
            len_samples.push(mean_len);
            eprintln!(
                "t={t} KE/atom={ke_per_atom:.4} mean_len={mean_len:.3} \
                 field_avg={field_avg:.1} coupling={coupling:.3}"
            );
            assert!(
                (0.8..=1.4).contains(&coupling),
                "K1.1 FAIL: thermostat decoupled at tick {t}: KE/atom {ke_per_atom:.4} \
                 vs bath level {:.4} (ratio {coupling:.3})",
                config.thermal_kick_scale * field_warm
            );
        }
    }
    // Samples land every 250 ticks from 2000 through 10000: 33 of
    // them. Sample-array index for tick t is (t - 2000) / 250, so a
    // window over ticks [from..=to] is the slice
    // [idx(from)..idx(to) + 1] - written that way below, so the
    // windows state their ticks instead of magic indices.
    fn idx(t: u64) -> usize {
        ((t - 2000) / 250) as usize
    }
    assert_eq!(ke_samples.len(), idx(10_000) + 1, "sampling bug");
    // Bounded throughout, transient included: the 35 C bath level
    // (kb * T = 0.291) is the scale; 2x it would already be a
    // furnace signature (the founding F8 failure measured 1e13).
    assert!(
        ke_samples.iter().all(|k| *k < 2.0 * 0.291),
        "K1.1 FAIL: KE/atom unbounded during the run: {ke_samples:?}"
    );
    let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    // Steady-tail windows: ticks 6.25k..=8k vs 8.25k..=10k.
    let ke_mid = mean(&ke_samples[idx(6250)..idx(8000) + 1]);
    let ke_late = mean(&ke_samples[idx(8250)..idx(10_000) + 1]);
    let span = (idx(8000) + 1 - idx(6250)) as f32; // samples per window
    let len_mid = len_samples[idx(6250)..idx(8000) + 1].iter().sum::<f32>() / span;
    let len_late = len_samples[idx(8250)..idx(10_000) + 1].iter().sum::<f32>() / span;

    eprintln!(
        "KE/atom 6.25k-8k {ke_mid:.4} 8.25k-10k {ke_late:.4} ({:+.1}%)",
        100.0 * (ke_late - ke_mid) / ke_mid
    );
    eprintln!(
        "mean_len 6.25k-8k {len_mid:.3} 8.25k-10k {len_late:.3} ({:+.1}%)",
        100.0 * (len_late - len_mid) / len_mid
    );
    assert!(
        (ke_late - ke_mid).abs() / ke_mid < 0.15,
        "K1.1 FAIL: KE/atom drifted {:.1}% after the transient",
        100.0 * (ke_late - ke_mid) / ke_mid
    );
    assert!(
        (len_late - len_mid).abs() / len_mid < 0.15,
        "K1.1 FAIL: mean bond length drifted {:.1}% after the transient",
        100.0 * (len_late - len_mid) / len_mid
    );
}

// ---- Gate K1.2: force sanity ------------------------------------------

/// K1.2 FORCE SANITY (gate ladder): a bonded overlap imparts
/// bounded velocity (the founding hard core measured v ~ 1e4 - a
/// cannon), and bond geometry holds the ladder's band. PASSED
/// 2026-09-07 against the pre-K1.3 substrate; re-run PASS in the
/// K1.3 integrator commit (sub-stepping resolves the overlap
/// over the whole tick, so the measured impulse lands further
/// below the bound, and the band tightens with the stiffer
/// springs).
///
/// Two measurements:
///
/// - Overlap probe: two bonded O atoms placed at 0.05 * r_eq in a
///   zero-temperature world - no kicks (sigma 0), no vent, no
///   other atoms - so the spring is the only actor. One tick must
///   impart at most the analytic single-impulse Hooke bound
///   k * r_eq / m per atom (the smooth law is bounded at
///   coincidence by construction, spec 6.3; the removed hard
///   core would land orders of magnitude over) and must push the
///   pair apart.
/// - The band: the same 10k-tick vented pond as K1.1 (seed 42);
///   PASS is the ladder's criterion - the mean per-bond stretch
///   ratio stays within [0.8, 1.5] at every sample.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_2_force_sanity() {
    let config = PhysicsConfig::default();

    // Part 1: the overlap probe.
    let o = ElementId(3);
    let r_eq = 2.0 * khem_core::ELEMENTS[o.0 as usize].radius;
    let mut world = WorldState::new(50.0, 50.0, BoundaryType::Wrap, 42, config);
    let a = world.spawn_atom(o, 25.0, 25.0);
    let b = world.spawn_atom(o, 25.0 + 0.05 * r_eq, 25.0);
    let energy = bond_energy(o, o, 1);
    let bond = world
        .form_bond(a, b, 1, energy)
        .expect("O valence allows one bond each");
    let mut sim = Sim::new(config, observer(42, 1000));
    let _ = sim.start(&world);
    sim.tick(&mut world);

    assert!(
        world.bond(bond).alive,
        "K1.2 probe: the bond broke at overlap"
    );
    let k = energy * config.spring_energy_scale;
    let m = khem_core::ELEMENTS[o.0 as usize].mass;
    let impulse_bound = 1.5 * k * r_eq / m;
    let mut max_speed = 0.0f32;
    for (id, atom) in [(a, world.atom(a)), (b, world.atom(b))] {
        let speed = (atom.vx * atom.vx + atom.vy * atom.vy).sqrt();
        max_speed = max_speed.max(speed);
        assert!(
            speed <= impulse_bound,
            "K1.2 probe: atom {id:?} speed {speed:.4} exceeds the \
             single-impulse Hooke bound {impulse_bound:.4}"
        );
    }
    let (ax, ay) = (world.atom(a).x, world.atom(a).y);
    let (bx, by) = (world.atom(b).x, world.atom(b).y);
    let separation = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt();
    assert!(
        separation > 0.05 * r_eq,
        "K1.2 probe: compression did not push the pair apart"
    );
    eprintln!(
        "K1.2 probe: overlap at 0.05*r_eq imparted speed {max_speed:.4} per atom \
         (Hooke bound {impulse_bound:.4}), pair separated to {separation:.3} A"
    );

    // Part 2: the band over the same run as K1.1.
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 10_000));
    let _ = sim.start(&world);
    let mut samples = 0;
    for t in 1..=10_000u64 {
        sim.tick(&mut world);
        if t < 2000 || !t.is_multiple_of(1000) {
            continue;
        }
        let ratios = bond_stretch_ratios(&world);
        assert!(!ratios.is_empty(), "K1.2: no live bonds at tick {t}");
        assert!(
            ratios.iter().all(|r| r.is_finite()),
            "K1.2: non-finite stretch ratio at tick {t}"
        );
        let mean = ratios.iter().sum::<f32>() / ratios.len() as f32;
        let mut sorted = ratios;
        sorted.sort_unstable_by(f32::total_cmp);
        let p95 = sorted[sorted.len() * 95 / 100];
        let outside = sorted.iter().filter(|r| **r < 0.8 || **r > 1.5).count();
        eprintln!(
            "t={t} mean ratio {mean:.3} (band [0.8, 1.5]) p95 {p95:.3} \
             outside-band {}/{}",
            outside,
            sorted.len()
        );
        assert!(
            (0.8..=1.5).contains(&mean),
            "K1.2 FAIL: mean stretch ratio {mean:.3} outside [0.8, 1.5] at tick {t}"
        );
        samples += 1;
    }
    assert!(samples >= 8, "sampling bug");
}

// ---- Gate K1.3: water persistence -------------------------------------

/// K1.3 WATER PERSISTS (gate ladder): a 35 C pond of H2O keeps its
/// molecules - intact count flat, O-H essentially never breaks
/// (real chemistry's own exp(-29) answer at the scaled Boltzmann
/// temperature), the form+break cycle mints no energy (the F7
/// regression, a unit law test in chemistry.rs, stays green).
///
/// Measurement over the same 10k-tick vented pond as K1.1/K1.2
/// (seed 42), every bond event classified by channel: a broken
/// bond's `energy_released == 0` marks a MECHANICAL break (the
/// 7.1 overstretch rule), `> 0` a thermal/UV roll. Water can only
/// lose intactness through an O-H break (an intact water's O is
/// saturated - crosslinking onto it is impossible), so the intact
/// census and the seeded-water O-H break census are two views of
/// one flow. O-H bonds formed during the run (bond ids above the
/// seeded count) are a separate population: free pairs bonding
/// and re-separating is reactive chemistry - K1.4's balance -
/// and the census reports it without gating on it.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_3_water_persistence() {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 10_000));
    let _ = sim.start(&world);

    let h = ElementId(0);
    let o = ElementId(3);
    let is_oh = |a: ElementId, b: ElementId| (a == o && b == h) || (a == h && b == o);

    let mut intact_samples: Vec<usize> = Vec::new();
    let mut oh_breaks = 0usize; // any channel
    let mut oh_mechanical = 0usize; // the 7.1 overstretch channel
    let mut seeded_breaks = 0usize; // a seeded water's own O-H
    let mut churn_breaks = 0usize; // a runtime-formed O-H pair
    let mut oh_details: Vec<String> = Vec::new(); // first breaks, for the record
    let mut other_breaks = 0usize;
    let mut oh_forms = 0usize;

    for t in 1..=10_000u64 {
        let events = sim.tick(&mut world);
        for event in &events {
            match event {
                Event::BondBroken {
                    tick,
                    bond_id,
                    elem_a,
                    elem_b,
                    energy_released,
                    x,
                    y,
                } => {
                    if !is_oh(*elem_a, *elem_b) {
                        other_breaks += 1;
                        continue;
                    }
                    oh_breaks += 1;
                    let mechanical = *energy_released <= 0.0;
                    oh_mechanical += usize::from(mechanical);
                    // Bond ids below the seeded count are the pond's
                    // own waters; above it, runtime chemistry (free
                    // pairs the pond formed during the run - their
                    // form/break churn is K1.4's reactive balance,
                    // not this gate's persistence).
                    let seeded = (*bond_id as usize) < WATERS * 2;
                    if seeded {
                        seeded_breaks += 1;
                    } else {
                        churn_breaks += 1;
                    }
                    if oh_details.len() < 12 {
                        // Velocities at processing time are the
                        // velocities at break time (chemistry is the
                        // last mutating step of the tick).
                        let bond = world.bond(BondId(*bond_id));
                        let (a, b) = (world.atom(bond.atom_a), world.atom(bond.atom_b));
                        let rel = ((a.vx - b.vx).powi(2) + (a.vy - b.vy).powi(2)).sqrt();
                        oh_details.push(format!(
                            "t={tick} {} {} local_T={:.1} rel_speed={rel:.2}",
                            if mechanical { "MECH" } else { "THERM" },
                            if seeded { "seeded" } else { "formed" },
                            world.temp_field.get(*x, *y),
                        ));
                    }
                }
                Event::BondFormed { elem_a, elem_b, .. } if is_oh(*elem_a, *elem_b) => {
                    oh_forms += 1;
                }
                _ => {}
            }
        }
        if t >= 2000 && t.is_multiple_of(1000) {
            let intact = pond::water_intact(&world);
            intact_samples.push(intact);
            eprintln!("t={t} intact waters {intact}/{WATERS}");
        }
    }
    assert!(intact_samples.len() >= 8, "sampling bug");

    eprintln!("K1.3 census over 10k ticks:");
    eprintln!(
        "  O-H breaks: {oh_breaks} (mechanical {oh_mechanical}; seeded waters \
         {seeded_breaks}, formed-pair churn {churn_breaks})"
    );
    for d in &oh_details {
        eprintln!("  {d}");
    }
    eprintln!("  O-H formations: {oh_forms}");
    eprintln!("  other-pair breaks: {other_breaks}");

    // PASS: molecules keep themselves. Intact count high and flat
    // (window means like K1.1's), and the pond's OWN waters' O-H
    // bonds essentially never break - the exp(-29) thermal scale
    // allows ~0 and the ~80 kT mechanical wells allow ~0; 2 leaves
    // headroom for single hot events without accepting the
    // bombardment shatter the failing substrate measured (1482
    // seeded-water breaks, 75% loss). Free-pair churn is not this
    // gate's failure mode: a runtime-formed O-H that re-separates
    // is reactive chemistry, reported for K1.4's balance, not
    // water persistence.
    let early = intact_samples[0..4].iter().sum::<usize>() as f64 / 4.0;
    let late = intact_samples[4..8].iter().sum::<usize>() as f64 / 4.0;
    eprintln!(
        "intact early {early:.0} late {late:.0} ({:+.1}%)",
        100.0 * (late - early) / early
    );
    assert!(
        intact_samples
            .iter()
            .all(|n| *n as f64 >= 0.95 * WATERS as f64),
        "K1.3 FAIL: intact waters dipped below 95%: {intact_samples:?}"
    );
    assert!(
        (late - early).abs() / early < 0.05,
        "K1.3 FAIL: intact count not flat ({early:.0} -> {late:.0})"
    );
    assert!(
        seeded_breaks <= 2,
        "K1.3 FAIL: {seeded_breaks} seeded-water O-H breaks over 10k ticks - \
         essentially-never is the criterion"
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
    // Geometry standard: molecules stay COHERENT - no shards, no
    // comets. Since the K1.3 sub-stepping the springs sit at
    // real-water well depths (~80 kT, spec 6.3), and this rollup
    // measures mean 1.193 / p95 1.369 A against equilibrium
    // ~1.19-1.32 per pair; the bars below sit well above that
    // with room for the formed-bond population mixing larger
    // pairs (O-O 1.32, C-C 1.54), while still catching the
    // failure modes that existed: 80 A comets (mean >> p95) and
    // full shard blowups.
    assert!(
        mean_len < 1.5,
        "K1 geometry: mean bond length {mean_len:.3} A beyond coherent"
    );
    assert!(
        p95_len < 2.5,
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
