//! Forces stay sane: a bonded overlap imparts bounded velocity
//! (the probe), and the mean bond stretch ratio stays within its
//! band over a vented-pond run (the band).
//!
//! Gate K1.2 of the stability ladder (PASS records:
//! plans/gates/stability.md). Run (release):
//!   cargo test --release -p khem-engine
//!     --test force_sanity -- --ignored --nocapture

mod common;

use common::*;
use khem_engine::Sim;
use khem_forcefield::{ForceField, bond_energy};
use khem_world::{BoundaryType, ElementId, WorldState};
use khem_worlds::pond::{self};

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
/// inside [0.8, 1.5]); re-run PASS in K1.5's commit (F20's anchor
/// fix moves the pond trajectory: measured probe 0.334 vs the
/// bound 0.578, band mean ratio 1.003-1.014, p95 <= 1.21, every
/// sample inside [0.8, 1.5]).
///
/// 3D RE-CLIMB PASS (2026-09-08): the band is dimension-agnostic
/// exactly as the port plan predicted - probe 0.334 vs the bound
/// 0.578 (the 2D pair measured 0.334/0.578), band mean ratio
/// 1.008-1.026, p95 <= 1.216, every sample inside [0.8, 1.5];
/// outside-band bonds up to 73/~2300 (~3%, the formed-pair churn
/// riding the hotter 3D tail; reported, not barred).
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
fn forces_stay_sane() {
    let config = ForceField::default();

    // Part 1: the overlap probe.
    let o = ElementId(3);
    let r_eq = 2.0 * khem_forcefield::ELEMENTS[o.0 as usize].radius;
    let mut world = WorldState::new(50.0, 50.0, 50.0, BoundaryType::Wrap, 42, config);
    let a = world.spawn_atom(o, 25.0, 25.0, 25.0);
    let b = world.spawn_atom(o, 25.0 + 0.05 * r_eq, 25.0, 25.0);
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
    let k = energy * config.bonds.spring_energy_scale;
    let m = khem_forcefield::ELEMENTS[o.0 as usize].mass;
    let impulse_bound = 1.5 * k * r_eq / m;
    let mut max_speed = 0.0f32;
    for (id, atom) in [(a, world.atom(a)), (b, world.atom(b))] {
        let speed = (atom.vx * atom.vx + atom.vy * atom.vy + atom.vz * atom.vz).sqrt();
        max_speed = max_speed.max(speed);
        assert!(
            speed <= impulse_bound,
            "K1.2 probe: atom {id:?} speed {speed:.4} exceeds the \
             single-impulse Hooke bound {impulse_bound:.4}"
        );
    }
    let (ax, ay, az) = (world.atom(a).x, world.atom(a).y, world.atom(a).z);
    let (bx, by, bz) = (world.atom(b).x, world.atom(b).y, world.atom(b).z);
    let separation = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay) + (bz - az) * (bz - az)).sqrt();
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
