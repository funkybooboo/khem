//! The pond keeps its molecules: intact count high and flat,
//! seeded-water O-H breaks essentially never (the census splits
//! them from runtime-formed pair churn, which is the reactive
//! balance gate's business).
//!
//! Gate K1.3 of the stability ladder (PASS records:
//! plans/gates/stability.md). Run (release, ~6 min):
//!   cargo test --release -p khem-engine
//!     --test water_molecules_persist -- --ignored --nocapture

mod common;

use common::*;
use khem_engine::Sim;
use khem_forcefield::ForceField;
use khem_world::{BondId, ElementId, Event};
use khem_worlds::pond::{self};

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
/// K1.4's material. Re-run PASS in K1.5's commit (F20's anchor
/// fix moves the pond trajectory): intact 1024/1024 flat, ZERO
/// O-H breaks of any kind.
///
/// 3D RE-CLIMB PASS (2026-09-08): intact 1025/1024 flat (one
/// water self-assembled from free atoms, as the 2D substrate
/// also measured), ZERO O-H breaks of any kind (seeded or
/// churn), 14 runtime O-H formations, 1 weak-pair break - the
/// water-persistence law is dimension-agnostic; the ~80 kT
/// mechanical well and the exp(-29) thermal scale neither know
/// what a z axis is.
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
fn water_molecules_persist() {
    let config = ForceField::default();
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
                    z,
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
                        let rel =
                            ((a.vx - b.vx).powi(2) + (a.vy - b.vy).powi(2) + (a.vz - b.vz).powi(2))
                                .sqrt();
                        oh_details.push(format!(
                            "t={tick} {} {} local_T={:.1} rel_speed={rel:.2}",
                            if mechanical { "MECH" } else { "THERM" },
                            if seeded { "seeded" } else { "formed" },
                            world.temp_field.get(*x, *y, *z),
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
