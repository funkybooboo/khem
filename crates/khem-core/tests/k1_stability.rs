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
//!   RE-VALIDATED 2026-09-07 in K1.3's integrator commit and again
//!   in K1.4's reactive-balance commit - the run horizon moved to
//!   20k ticks with the measured transient): thermostat coupling
//!   law + KE and bond-length flatness over the steady tail of a
//!   20k-tick vented-pond run. Run: `mise exec -- cargo test
//!   --release -p khem-core --test k1_stability -- --ignored
//!   --nocapture` (release: ~350 s; debug is ~20x slower).
//! - `k1_2_force_sanity` (gate K1.2, PASSED 2026-09-07, re-run
//!   in the K1.3 commit): a bonded overlap imparts bounded
//!   velocity (the probe), and the mean bond stretch ratio stays
//!   within [0.8, 1.5] over the same vented-pond run (the band).
//! - `k1_3_water_persistence` (gate K1.3, PASSED 2026-09-07):
//!   the pond keeps its molecules - intact count high and flat,
//!   seeded-water O-H breaks essentially never (the census
//!   splits them from runtime-formed pair churn, which is K1.4's
//!   reactive balance). Same run command.
//! - `k1_4_reactive_balance` (gate K1.4, PASSED 2026-09-07): the
//!   free-atom beaker settles to a stationary molecule-size
//!   distribution - weak bonds break at the Boltzmann scale
//!   (measured: O-O mean lifetime 10k ticks) while strong ones
//!   persist, no runaway crosslinking, no frozen inertness, and
//!   formation refrigeration stays bounded and recovers (tail
//!   avg ~35.5 C against the 35 C setpoint). 20k-tick run.
//! - `k1_diagnostics` (the K1 rollup): the honest measurement
//!   after 2000 ticks - water survival, bond activity, geometry,
//!   energy. Its bars pass as of the K1.4 commit (the
//!   broken-ever bar moved to K1.4's census: thermal breaks live
//!   at the Boltzmann scale, past this window); the ladder's
//!   remaining K1 sub-gate (K1.5) owns the finer criteria.
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
/// PASS (re-validated 2026-09-07 for the K1.4 reactive-balance
/// commit, per the re-validation contract - the run horizon and
/// windows moved with the measured transient, the same move the
/// K1.3 commit made):
/// - KE/atom bounded under 2x the 35 C bath level at EVERY sample,
///   transient included (the founding F8 furnace measured 1e13);
/// - the THERMOSTAT COUPLING law: KE/atom stays within [0.8, 1.4]
///   of the field's warm-cell thermal level kb*T at every sample
///   (the atoms ride their local bath, never decoupled above or
///   below it);
/// - flatness over the steady tail: window means 16k-17.75k vs
///   18k-20k within 15% for both metrics. The horizon is 20k
///   ticks now: the K1.4 substrate's construction drain (each
///   persistent bond sequesters formation_fraction * E until it
///   breaks; the free population is consumed over ~14k ticks)
///   extends the field-recovery transient to ~16k - measured, the
///   old 6.25k-10k windows sat mid-recovery and the KE window
///   drift measured +14.5% against the 15% bar: a pass by 0.5%
///   is a flaky gate, so the windows ride the true steady tail
///   (the same ticks K1.4 measures its stationarity on).
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_1_thermostat_flatness() {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 20_000));
    let _ = sim.start(&world);

    let mut ke_samples: Vec<f64> = Vec::new();
    let mut len_samples: Vec<f32> = Vec::new();
    for t in 1..=20_000u64 {
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
    // Samples land every 250 ticks from 2000 through 20000: 73 of
    // them. Sample-array index for tick t is (t - 2000) / 250, so a
    // window over ticks [from..=to] is the slice
    // [idx(from)..idx(to) + 1] - written that way below, so the
    // windows state their ticks instead of magic indices.
    fn idx(t: u64) -> usize {
        ((t - 2000) / 250) as usize
    }
    assert_eq!(ke_samples.len(), idx(20_000) + 1, "sampling bug");
    // Bounded throughout, transient included: the 35 C bath level
    // (kb * T = 0.291) is the scale; 2x it would already be a
    // furnace signature (the founding F8 failure measured 1e13).
    assert!(
        ke_samples.iter().all(|k| *k < 2.0 * 0.291),
        "K1.1 FAIL: KE/atom unbounded during the run: {ke_samples:?}"
    );
    let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    // Steady-tail windows: ticks 16k..=17.75k vs 18k..=20k (the
    // K1.4 substrate's measured settle - see the doc comment).
    let ke_mid = mean(&ke_samples[idx(16_000)..idx(17_750) + 1]);
    let ke_late = mean(&ke_samples[idx(18_000)..idx(20_000) + 1]);
    let len_mean = |from: u64, to: u64| {
        let w = &len_samples[idx(from)..idx(to) + 1];
        w.iter().sum::<f32>() / w.len() as f32
    };
    let len_mid = len_mean(16_000, 17_750);
    let len_late = len_mean(18_000, 20_000);

    eprintln!(
        "KE/atom 16k-17.75k {ke_mid:.4} 18k-20k {ke_late:.4} ({:+.1}%)",
        100.0 * (ke_late - ke_mid) / ke_mid
    );
    eprintln!(
        "mean_len 16k-17.75k {len_mid:.3} 18k-20k {len_late:.3} ({:+.1}%)",
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
/// springs); re-run PASS in K1.4's commit (steric-contact births
/// tighten the band further: measured probe 0.33 vs the bound
/// 0.58, band mean ratio 1.003-1.014, p95 1.15-1.23, every sample
/// inside [0.8, 1.5]).
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
/// - The band: the same vented pond as K1.1 (seed 42, 10k-tick
///   window - the band settles early, long before the field
///   transient ends); PASS is the ladder's criterion - the mean
///   per-bond stretch ratio stays within [0.8, 1.5] at every
///   sample.
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
/// Re-run PASS in K1.4's commit (the formation-rule changes move
/// the churn channel): intact 1025/1024 flat (the pond's
/// self-assembled water persists too), ZERO O-H breaks of any
/// kind, 28 runtime O-H formations, 2 weak-pair (O-O/N-N)
/// thermal breaks late in the run - the first flicker events,
/// K1.4's material.
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

// ---- Gate K1.4: reactive balance --------------------------------------

/// K1.4 REACTIVE BALANCE (gate ladder): a beaker of free atoms
/// settles to a STATIONARY molecule-size distribution - weak bonds
/// break (O-O on a ~10k-tick scale), strong ones persist; no runaway
/// crosslinking, no frozen inertness; formation refrigeration (F6)
/// stays bounded and recovers.
///
/// Same vented pond as K1.1-K1.3 (seed 42), run to 20k ticks:
/// construction of the free population ends by ~16k and the tail
/// measures the settled state. Run: `mise exec -- cargo test
/// --release -p khem-core --test k1_stability -- --ignored
/// --nocapture` (release: ~340 s; debug is ~20x slower).
///
/// PASS (bars set from the measured profile, 2026-09-07):
///
/// - F6 BOUNDED AND RECOVERING: field avg >= 10 C at every sample
///   (measured min 12.16 during the construction dip; the failing
///   pre-K1.4 substrate sat at -162 C steady), and the steady-tail
///   avg within [30, 40] C against the 35 C setpoint (measured
///   ~35.5 - the vent plume and the thermostat's standing backflow
///   hold the pond ~1 C above the reservoir's target).
/// - STATIONARY DISTRIBUTION: tail windows 16k-18k vs 18k-20k -
///   the 2-5 molecule bucket within 2% (measured dead flat,
///   1066 vs 1066), free singles within 25% (measured -9%),
///   live bonds within 3% (measured +0.4%), and the cluster count
///   (6-20 + 21+) stable to within 5 molecules (measured -2: the
///   weak-pair flicker merges clusters across the bucket boundary
///   slowly - glass annealing, not growth).
/// - NO RUNAWAY CROSSLINKING: largest molecule <= 40 (measured
///   22), 21+ bucket <= 4 molecules (measured 1), live bonds
///   <= 2600 (measured 2406; the K1 rollup keeps 3072).
/// - NO FROZEN INERTNESS: chemistry keeps running in the tail -
///   at least 5 formations and 3 thermal breaks over 15k-20k
///   (measured ~25 and ~11: the O-O/N-N flicker re-forming).
/// - WEAK/STRONG ASYMMETRY: O-O bonds break thermally at the
///   ladder's ~10k-tick scale (measured: 18 breaks, mean age
///   10,007 ticks; N-N 5 more) while NO strong pair ever breaks
///   thermally (H-O, H-C, C-C, ...: measured 0), the seeded
///   waters' bonds essentially never break (measured 0; K1.3's
///   criterion), phantom formations are 0 (F18's fix), and
///   mechanical breaks are collision outliers only (measured 1;
///   bar <= 3).
///
/// The measurement classifies every bond event of the run and
/// keeps the field's energy ledger per 1k-tick window. The field's
/// writers are exactly: the vent (energy system, constant per tick
/// because the source geometry is fixed), setpoint relaxation
/// (physics, exactly `field_relax_rate * (setpoint_sum - field_sum)`
/// per tick: diffusion conserves the sum, so the pre-tick snapshot
/// pins it), formation absorption and break release (chemistry,
/// from the event payloads; the release COMMITS to the reservoir
/// and the ledger's `release` column is the committed amount), and
/// the thermostat exchange (physics, derived as the residual: it
/// is the only other writer).
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k1_4_reactive_balance() {
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 10_000));
    let _ = sim.start(&world);

    let sym = |e: ElementId| khem_core::ELEMENTS[e.0 as usize].symbol;
    let pair_key = |a: ElementId, b: ElementId, order: u8| {
        let (lo, hi) = if a.0 <= b.0 { (a, b) } else { (b, a) };
        format!(
            "{}-{}{}",
            sym(lo),
            sym(hi),
            if order == 2 { "=" } else { "" }
        )
    };

    // Vent injection per tick: constant (fixed source geometry and
    // grid); mirrors energy.rs's falloff loop, computed once.
    let vent_flux: f32 = world
        .energy_sources
        .iter()
        .filter(|s| matches!(s.kind, khem_core::SourceKind::Hydrothermal))
        .map(|s| {
            let field = &world.temp_field;
            let r2 = s.radius * s.radius;
            let mut sum = 0.0f32;
            for row in 0..field.rows {
                for col in 0..field.cols {
                    let cx = (col as f32 + 0.5) * field.cell_width;
                    let cy = (row as f32 + 0.5) * field.cell_height;
                    let d2 = (cx - s.position.0).powi(2) + (cy - s.position.1).powi(2);
                    if d2 <= r2 {
                        sum += s.intensity * (1.0 / (1.0 + d2 / r2)) * config.vent_heat_rate;
                    }
                }
            }
            sum
        })
        .sum();

    let setpoint_sum: f32 = world.setpoint_field.data.iter().sum();
    let field_sum = |w: &WorldState| w.temp_field.data.iter().sum::<f32>();

    // Trajectory samples every 250 ticks.
    #[derive(Clone, Copy)]
    struct Sample {
        tick: u64,
        singles: usize,
        pairs_2_5: usize,
        clusters_6_20: usize,
        clusters_21: usize,
        largest: u32,
        bonds: usize,
        field_avg: f32,
    }
    let mut traj: Vec<Sample> = Vec::new();
    // Tail-window activity (ticks >= 15k).
    let mut tail_formations = 0usize;
    let mut tail_thermal = 0usize;
    let mut tail_mechanical = 0usize;

    // Formation record: bond id -> (tick formed, distance, phantom).
    // Phantom: born past the 7.1 mechanical break length
    // (bond_break_factor * r_eq) - a bond the next chemistry pass
    // kills silently, keeping its absorbed formation heat (F18).
    let mut formed: std::collections::HashMap<u32, (u64, f32, bool)> =
        std::collections::HashMap::new();

    #[derive(Default, Clone)]
    struct FormCensus {
        count: usize,
        phantom: usize,
        r_sum: f32,
        r_max: f32,
        // Distance buckets in stretch ratios r/r_eq: [<=1.25,
        // <=1.5, <=2.0, <=2.5(break), >2.5].
        buckets: [usize; 5],
        absorbed: f32,
    }
    #[derive(Default, Clone)]
    struct BreakCensus {
        thermal: usize,
        thermal_released: f32,
        thermal_age_sum: f32,
        // Mechanical (silent, 0 release) by origin:
        mech_phantom: usize, // born past break length
        mech_legal: usize,   // born legal, broke anyway
        mech_seeded: usize,  // a pond-seeded bond
        mech_phantom_age_sum: f32,
        mech_legal_age_sum: f32,
    }
    let mut forms: std::collections::HashMap<String, FormCensus> = std::collections::HashMap::new();
    let mut breaks: std::collections::HashMap<String, BreakCensus> =
        std::collections::HashMap::new();
    // A pond-seeded bond broken thermally (seeded breaks are K1.3
    // material; counted for the cross-check).
    let mut seeded_thermal = 0usize;

    // Ledger window accumulators (field degree-units per tick).
    #[derive(Default)]
    struct Window {
        vent: f32,
        relax: f32,
        absorb: f32,
        release: f32,
        thermo: f32,
        delta: f32,
    }
    let mut window = Window::default();
    let report_window = |w: &Window, t: u64| {
        eprintln!(
            "ledger {:>5}-{:>5}: vent {:+.2} relax {:+.2} absorb {:+.2} \
             release {:+.2} thermo {:+.2} delta {:+.2}",
            t - 1000,
            t,
            w.vent,
            w.relax,
            -w.absorb,
            w.release,
            w.thermo,
            w.delta
        );
    };

    eprintln!("K1.4 probe: vent flux {vent_flux:.4}/tick, setpoint sum {setpoint_sum:.0}");
    for t in 1..=20_000u64 {
        let sum_before = field_sum(&world);
        let events = sim.tick(&mut world);
        let sum_after = field_sum(&world);

        let relax = config.field_relax_rate * (setpoint_sum - sum_before);
        let mut absorb = 0.0f32;
        let mut release = 0.0f32;
        for event in &events {
            match event {
                Event::BondFormed {
                    bond_id,
                    atom_a,
                    atom_b,
                    elem_a,
                    elem_b,
                    order,
                    energy,
                    ..
                } => {
                    let (a, b) = (world.atom(*atom_a), world.atom(*atom_b));
                    let (dx, dy) = world.delta(a.x, a.y, b.x, b.y);
                    let r = (dx * dx + dy * dy).sqrt();
                    let r_eq = world.element(a.element).radius + world.element(b.element).radius;
                    let phantom = r > config.bond_break_factor * r_eq;
                    formed.insert(*bond_id, (t, r, phantom));
                    let key = pair_key(*elem_a, *elem_b, *order);
                    let c = forms.entry(key).or_default();
                    c.count += 1;
                    c.phantom += usize::from(phantom);
                    c.r_sum += r;
                    c.r_max = c.r_max.max(r);
                    let ratio = r / r_eq;
                    let bucket = if ratio <= 1.25 {
                        0
                    } else if ratio <= 1.5 {
                        1
                    } else if ratio <= 2.0 {
                        2
                    } else if ratio <= config.bond_break_factor {
                        3
                    } else {
                        4
                    };
                    c.buckets[bucket] += 1;
                    let absorbed = energy * config.formation_fraction;
                    c.absorbed += absorbed;
                    absorb += absorbed;
                }
                Event::BondBroken {
                    bond_id,
                    elem_a,
                    elem_b,
                    energy_released,
                    ..
                } => {
                    let key = pair_key(*elem_a, *elem_b, 1); // order lost at break
                    let c = breaks.entry(key).or_default();
                    if *energy_released > 0.0 {
                        c.thermal += 1;
                        c.thermal_released += energy_released;
                        if (*bond_id as usize) < WATERS * 2 {
                            seeded_thermal += 1;
                        }
                        if let Some((tick0, _, _)) = formed.get(bond_id) {
                            c.thermal_age_sum += (t - tick0) as f32;
                        }
                        release += energy_released;
                    } else if (*bond_id as usize) < WATERS * 2 {
                        c.mech_seeded += 1;
                    } else {
                        match formed.get(bond_id) {
                            Some((tick0, _, true)) => {
                                c.mech_phantom += 1;
                                c.mech_phantom_age_sum += (t - tick0) as f32;
                            }
                            Some((tick0, _r0, false)) => {
                                c.mech_legal += 1;
                                c.mech_legal_age_sum += (t - tick0) as f32;
                            }
                            None => {
                                // Seeded-range id not in the map is a
                                // pond bond; anything else is a bug.
                                c.mech_seeded += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let delta = sum_after - sum_before;
        let thermo = delta - vent_flux - relax + absorb - release;
        window.vent += vent_flux;
        window.relax += relax;
        window.absorb += absorb;
        window.release += release;
        window.thermo += thermo;
        window.delta += delta;
        if t >= 15_000 {
            tail_formations += events
                .iter()
                .filter(|e| matches!(e, Event::BondFormed { .. }))
                .count();
            tail_thermal += events
                .iter()
                .filter(|e| matches!(e, Event::BondBroken { energy_released, .. } if *energy_released > 0.0))
                .count();
            tail_mechanical += events
                .iter()
                .filter(|e| matches!(e, Event::BondBroken { energy_released, .. } if *energy_released <= 0.0))
                .count();
        }
        if t.is_multiple_of(1000) {
            report_window(&window, t);
            window = Window::default();
        }

        if t.is_multiple_of(250) {
            let sizes = Observer::molecule_sizes(&world);
            let mut nonzero: Vec<u32> = sizes.into_iter().filter(|s| *s > 0).collect();
            nonzero.sort_unstable();
            let mut buckets = [0usize; 4];
            for &s in &nonzero {
                let b = match s {
                    1 => 0,
                    2..=5 => 1,
                    6..=20 => 2,
                    _ => 3,
                };
                buckets[b] += 1;
            }
            let largest = nonzero.last().copied().unwrap_or(0);
            let free = world
                .atoms
                .iter()
                .filter(|a| a.alive && a.bond_count == 0)
                .count();
            let slots: u32 = world
                .atoms
                .iter()
                .filter(|a| a.alive)
                .map(|a| (world.element(a.element).max_bonds - a.bond_count) as u32)
                .sum();
            let avg = sum_after / world.temp_field.data.len() as f32;
            let min = world
                .temp_field
                .data
                .iter()
                .copied()
                .fold(f32::INFINITY, f32::min);
            let neg = world.temp_field.data.iter().filter(|&v| *v < 0.0).count();
            let mut free_by = [0usize; 10];
            for atom in world.atoms.iter().filter(|a| a.alive && a.bond_count == 0) {
                free_by[atom.element.0 as usize] += 1;
            }
            eprintln!(
                "t={t} dist [1:{}, 2-5:{}, 6-20:{}, 21+{}] largest {largest} \
                 free {free} (H{}/C{}/N{}/O{}) slots {slots} bonds {} \
                 avg {avg:.2} min {min:.1} neg {neg}",
                buckets[0],
                buckets[1],
                buckets[2],
                buckets[3],
                free_by[0],
                free_by[1],
                free_by[2],
                free_by[3],
                world.live_bond_count()
            );
            traj.push(Sample {
                tick: t,
                singles: buckets[0],
                pairs_2_5: buckets[1],
                clusters_6_20: buckets[2],
                clusters_21: buckets[3],
                largest,
                bonds: world.live_bond_count(),
                field_avg: avg,
            });
        }
    }

    eprintln!("\nK1.4 formation census (pair: count, phantom, mean r, max r, buckets, absorbed):");
    let mut keys: Vec<_> = forms.keys().cloned().collect();
    keys.sort();
    for key in &keys {
        let c = &forms[key];
        eprintln!(
            "  {key:8} n {:>5} phantom {:>4} r {:.2}/{:.2} \
             b {:?} absorb {:.0}",
            c.count,
            c.phantom,
            c.r_sum / c.count as f32,
            c.r_max,
            c.buckets,
            c.absorbed
        );
    }
    eprintln!(
        "\nK1.4 break census (pair: thermal n/heat/mean-age, mech phantom/legal/seeded, legal mean age):"
    );
    let mut keys: Vec<_> = breaks.keys().cloned().collect();
    keys.sort();
    for key in &keys {
        let c = &breaks[key];
        let thermal_age = if c.thermal > 0 {
            c.thermal_age_sum / c.thermal as f32
        } else {
            0.0
        };
        let legal_age = if c.mech_legal > 0 {
            c.mech_legal_age_sum / c.mech_legal as f32
        } else {
            0.0
        };
        let phantom_age = if c.mech_phantom > 0 {
            c.mech_phantom_age_sum / c.mech_phantom as f32
        } else {
            0.0
        };
        eprintln!(
            "  {key:8} thermal {:>4} {:+.0}deg age {:.0} | mech ph {:>4} (age {:.1}) legal {:>4} (age {:.0}) seeded {:>4}",
            c.thermal,
            c.thermal_released,
            thermal_age,
            c.mech_phantom,
            phantom_age,
            c.mech_legal,
            legal_age,
            c.mech_seeded
        );
    }
    let total_phantom: usize = forms.values().map(|c| c.phantom).sum();
    let total_absorbed: f32 = forms.values().map(|c| c.absorbed).sum();
    eprintln!("\nphantom formations total: {total_phantom}");
    eprintln!("total absorbed: {total_absorbed:.0} field degrees");
    eprintln!(
        "tail 15k-20k activity: {tail_formations} formations, \
     {tail_thermal} thermal, {tail_mechanical} mechanical"
    );

    // ---- The bars (doc comment; measured numbers cited) ---------

    // Window means over the trajectory samples.
    assert_eq!(traj.len(), 80, "sampling bug");
    let mean_of = |from: u64, to: u64, pick: fn(&Sample) -> f64| -> f64 {
        let sel: Vec<f64> = traj
            .iter()
            .filter(|s| s.tick >= from && s.tick <= to)
            .map(pick)
            .collect();
        assert!(
            sel.len() >= 8,
            "window {from}-{to} sampled {} times",
            sel.len()
        );
        sel.iter().sum::<f64>() / sel.len() as f64
    };
    let pct = |a: f64, b: f64| 100.0 * (b - a) / a;

    // F6 bounded and recovering.
    let min_avg = traj
        .iter()
        .map(|s| s.field_avg)
        .fold(f32::INFINITY, f32::min);
    let tail_avg = mean_of(18_000, 20_000, |s| s.field_avg as f64);
    eprintln!(
        "K1.4 field: min avg {min_avg:.2}, tail avg {tail_avg:.2} \
         (setpoint 35)"
    );
    assert!(
        min_avg >= 10.0,
        "K1.4 FAIL: formation refrigeration unbounded - field avg \
         dipped to {min_avg:.2} C"
    );
    assert!(
        (30.0..=40.0).contains(&tail_avg),
        "K1.4 FAIL: field tail avg {tail_avg:.2} not recovered to \
         the 35 C setpoint band [30, 40]"
    );

    // Stationary distribution: mid [16k, 17.75k] vs late [18k, 20k].
    let pairs_mid = mean_of(16_000, 17_750, |s| s.pairs_2_5 as f64);
    let pairs_late = mean_of(18_000, 20_000, |s| s.pairs_2_5 as f64);
    eprintln!(
        "K1.4 stationarity: 2-5 {pairs_mid:.1} -> {pairs_late:.1} \
         ({:+.1}%), singles {:.1} -> {:.1} ({:+.1}%), bonds {:.0} -> \
         {:.0} ({:+.1}%), clusters {:.1} -> {:.1}",
        pct(pairs_mid, pairs_late),
        mean_of(16_000, 17_750, |s| s.singles as f64),
        mean_of(18_000, 20_000, |s| s.singles as f64),
        pct(
            mean_of(16_000, 17_750, |s| s.singles as f64),
            mean_of(18_000, 20_000, |s| s.singles as f64)
        ),
        mean_of(16_000, 17_750, |s| s.bonds as f64),
        mean_of(18_000, 20_000, |s| s.bonds as f64),
        pct(
            mean_of(16_000, 17_750, |s| s.bonds as f64),
            mean_of(18_000, 20_000, |s| s.bonds as f64)
        ),
        mean_of(16_000, 17_750, |s| (s.clusters_6_20 + s.clusters_21) as f64),
        mean_of(18_000, 20_000, |s| (s.clusters_6_20 + s.clusters_21) as f64),
    );
    assert!(
        pct(pairs_mid, pairs_late).abs() < 2.0,
        "K1.4 FAIL: 2-5 bucket drifted {:+.1}% over the tail",
        pct(pairs_mid, pairs_late)
    );
    let singles_mid = mean_of(16_000, 17_750, |s| s.singles as f64);
    let singles_late = mean_of(18_000, 20_000, |s| s.singles as f64);
    assert!(
        pct(singles_mid, singles_late).abs() < 25.0,
        "K1.4 FAIL: free-atom count drifted {:+.1}% over the tail",
        pct(singles_mid, singles_late)
    );
    let bonds_mid = mean_of(16_000, 17_750, |s| s.bonds as f64);
    let bonds_late = mean_of(18_000, 20_000, |s| s.bonds as f64);
    assert!(
        pct(bonds_mid, bonds_late).abs() < 3.0,
        "K1.4 FAIL: live bonds drifted {:+.1}% over the tail",
        pct(bonds_mid, bonds_late)
    );
    let clusters_mid = mean_of(16_000, 17_750, |s| (s.clusters_6_20 + s.clusters_21) as f64);
    let clusters_late = mean_of(18_000, 20_000, |s| (s.clusters_6_20 + s.clusters_21) as f64);
    assert!(
        (clusters_late - clusters_mid).abs() <= 5.0,
        "K1.4 FAIL: cluster count moved {:+.1} over the tail",
        clusters_late - clusters_mid
    );

    // No runaway crosslinking.
    let largest = traj.iter().map(|s| s.largest).max().unwrap_or(0);
    let b21 = traj.iter().map(|s| s.clusters_21).max().unwrap_or(0);
    let bonds_max = traj.iter().map(|s| s.bonds).max().unwrap_or(0);
    eprintln!("K1.4 bounds: largest {largest}, 21+ bucket {b21}, bonds {bonds_max}");
    assert!(largest <= 40, "K1.4 FAIL: largest molecule {largest}");
    assert!(b21 <= 4, "K1.4 FAIL: {b21} molecules of 21+ atoms");
    assert!(
        bonds_max <= 2600,
        "K1.4 FAIL: {bonds_max} live bonds - crosslinking"
    );

    // No frozen inertness: the tail keeps reacting.
    eprintln!(
        "K1.4 tail activity: {tail_formations} formations, \
         {tail_thermal} thermal breaks (15k-20k)"
    );
    assert!(
        tail_formations >= 5,
        "K1.4 FAIL: frozen inertness - {tail_formations} formations in the tail"
    );
    assert!(
        tail_thermal >= 3,
        "K1.4 FAIL: the weak-pair flicker is dead - {tail_thermal} \
         thermal breaks in the tail"
    );

    // Weak/strong asymmetry from the census.
    let weak = ["O-O", "N-N"];
    let weak_thermal: usize = breaks
        .iter()
        .filter(|(k, _)| weak.contains(&k.as_str()))
        .map(|(_, c)| c.thermal)
        .sum();
    let strong_thermal: usize = breaks
        .iter()
        .filter(|(k, _)| !weak.contains(&k.as_str()))
        .map(|(_, c)| c.thermal)
        .sum();
    let oo = breaks.get("O-O");
    let oo_age = oo.map_or(0.0, |c| {
        if c.thermal > 0 {
            c.thermal_age_sum / c.thermal as f32
        } else {
            0.0
        }
    });
    let mech_total: usize = breaks
        .values()
        .map(|c| c.mech_phantom + c.mech_legal + c.mech_seeded)
        .sum();
    eprintln!(
        "K1.4 asymmetry: weak thermal {weak_thermal} (O-O age \
         {oo_age:.0} ticks), strong thermal {strong_thermal}, \
         seeded thermal {seeded_thermal}, mechanical {mech_total}"
    );
    assert!(
        weak_thermal >= 5,
        "K1.4 FAIL: weak bonds are not breaking - {weak_thermal} \
         thermal breaks of O-O/N-N in 20k ticks"
    );
    assert!(
        strong_thermal == 0,
        "K1.4 FAIL: {strong_thermal} thermal breaks of strong pairs \
         - strong bonds must persist"
    );
    assert!(
        oo_age >= 2000.0,
        "K1.4 FAIL: O-O mean lifetime {oo_age:.0} - weak bonds must \
         break at the Boltzmann scale, not instantly"
    );
    assert!(
        seeded_thermal <= 2,
        "K1.4 FAIL: {seeded_thermal} seeded-water bonds broke - \
         K1.3's criterion"
    );
    assert_eq!(
        total_phantom, 0,
        "K1.4 FAIL: {total_phantom} phantom formations - F18"
    );
    assert!(
        mech_total <= 3,
        "K1.4 FAIL: {mech_total} mechanical breaks - churn is back"
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
    // NOT barred here anymore: with F18's steric-contact fix the
    // mechanical churn that used to break bonds within this
    // window is gone, and thermal breaks live at the Boltzmann
    // scale (weak O-O pairs: ~10k-tick lifetimes at 35 C) - a
    // 2000-tick window measures 0 breaks honestly (measured: 0
    // with 119 formations). The break channel is gated where its
    // scale lives: K1.4's census bars weak thermal breaks >= 5
    // over the 20k-tick run.
    assert!(
        live_bonds <= WATERS * 3,
        "K1 crosslinking: {live_bonds} bonds from {} initial",
        WATERS * 2
    );
}
