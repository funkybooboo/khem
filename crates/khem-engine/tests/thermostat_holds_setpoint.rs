//! The thermostat holds the pond at its setpoint: KE per atom and
//! mean bond length both go FLAT over a vented-pond run
//! (steady-tail windows), and the coupling law (KE vs the field's
//! own equipartition level) holds at every sample.
//!
//! Gate K1.1 of the stability ladder (PASS records:
//! plans/gates/stability.md). Run (release, ~6 min):
//!   cargo test --release -p khem-engine
//!     --test thermostat_holds_setpoint -- --ignored --nocapture
//! Re-validation bindings: K1.4's settled-temperature moves and
//! any integrator change re-run this gate (the contract).

mod common;

use common::*;
use khem_engine::Sim;
use khem_forcefield::ForceField;
use khem_worlds::pond::{self};

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
///
/// RE-VALIDATED 2026-09-07 in K1.5's commit (F20's min-image
/// anchor fix moves the pond trajectory through its
/// seam-crossing anchored formation draws): PASS - coupling
/// 1.055-1.158 at every sample, steady-tail KE +1.7%, bond
/// length +0.3%.
///
/// 3D RE-CLIMB PASS (2026-09-08, the port's re-validation
/// contract; measured on the 25k-tick 3D run):
/// - coupling 1.037-1.121 at EVERY sample (the ratio to the
///   dimensionally-correct 3/2 * kb * T level; the port commit
///   first mis-scaled the bar to [1.2, 2.1] - the first sample
///   measured 1.042 and falsified that scaling in one read; the
///   ratio is dimension-normalized by construction, [0.8, 1.4]
///   carries unchanged);
/// - KE bounded throughout: max 0.527 against the 2x bar 0.873
///   (the 3/2-scaled absolute bound; the raw KE level is the
///   dimensional quantity);
/// - steady-tail flatness: 21k-22.75k vs 23k-25k, KE/atom
///   +4.5%, mean bond length +0.4% (the 15% bar);
/// - the tail field avg measures ~36.1 C (2D: 35.6) with single
///   samples swinging 33-38.6 on the formation/break churn
///   (the K1.4 ledger material) - the 2D windows (16k-17.75k vs
///   18k-20k) sat mid-recovery in 3D and the horizon moved to
///   25k to ride the measured settle. The pond rides ~1 C above
///   its 35 C setpoint: the vent/relaxation/thermostat-backflow
///   balance of the slab's 72-cell field (the 2D pond's 144),
///   honest G06 physics (the environment is a reservoir, the
///   vent an additional input), recorded for K1.4's ledger.
///
/// 3D PORT NOTE (2026-09-08): the coupling law is dimensional in
/// the ABSOLUTE KE level - the Langevin bath draws three
/// components, each with stationary variance kb*T/m, so KE/atom
/// settles at 3/2 * kb * T where the 2D substrate settled at
/// 1 * kb * T (the bounded-KE bar scales by 3/2). The coupling
/// RATIO divides by exactly that dimensionally-correct level, so
/// the [0.8, 1.4] bar carries unchanged. Everything else in this
/// gate - the steady-tail flatness windows - is dimension-agnostic
/// and re-measures at the re-climb.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn thermostat_holds_setpoint() {
    let config = ForceField::default();
    let mut world = pond::primordial_pond(42, config);
    let mut sim = Sim::new(config, observer(42, 20_000));
    let _ = sim.start(&world);

    let mut ke_samples: Vec<f64> = Vec::new();
    let mut len_samples: Vec<f32> = Vec::new();
    // 3D re-climb: the horizon rides the measured steady tail
    // (~39.8 C asymptote, ~500-tick relaxation constant; the 2D
    // run's 20k ended mid-recovery at 38.3 C).
    for t in 1..=25_000u64 {
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
            // bath's thermal level (3/2 * kb * T - three Langevin
            // components since the 3D port), not above, not below.
            let coupling = ke_per_atom
                / (1.5 * f64::from(config.thermostat.thermal_kick_scale) * f64::from(field_warm));
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
                1.5 * config.thermostat.thermal_kick_scale * field_warm
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
    assert_eq!(ke_samples.len(), idx(25_000) + 1, "sampling bug");
    // Bounded throughout, transient included: the 35 C bath level
    // (3/2 * kb * T = 0.4365 in 3D) is the scale; 2x it would
    // already be a furnace signature (the founding F8 failure
    // measured 1e13).
    assert!(
        ke_samples.iter().all(|k| *k < 2.0 * 0.4365),
        "K1.1 FAIL: KE/atom unbounded during the run: {ke_samples:?}"
    );
    let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    // Steady-tail windows: ticks 21k..=22.75k vs 23k..=25k (the
    // 3D pond's measured settle - see the doc comment; the 2D
    // windows 16k-17.75k vs 18k-20k sat mid-recovery in 3D).
    let ke_mid = mean(&ke_samples[idx(21_000)..idx(22_750) + 1]);
    let ke_late = mean(&ke_samples[idx(23_000)..idx(25_000) + 1]);
    let len_mean = |from: u64, to: u64| {
        let w = &len_samples[idx(from)..idx(to) + 1];
        w.iter().sum::<f32>() / w.len() as f32
    };
    let len_mid = len_mean(21_000, 22_750);
    let len_late = len_mean(23_000, 25_000);

    eprintln!(
        "KE/atom 21k-22.75k {ke_mid:.4} 23k-25k {ke_late:.4} ({:+.1}%)",
        100.0 * (ke_late - ke_mid) / ke_mid
    );
    eprintln!(
        "mean_len 21k-22.75k {len_mid:.3} 23k-25k {len_late:.3} ({:+.1}%)",
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
