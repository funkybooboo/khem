//! A Wrap world's seam is not special: cross-seam formation is
//! symmetric with the bulk under the composition-conditioned
//! census, and the wrap-aware index is verified pair-for-pair
//! complete against brute force at every tick (the F11/F21
//! records).
//!
//! Gate K1.5 of the stability ladder - the K1 close (PASS
//! records: plans/gates/stability.md). Run (release):
//!   cargo test --release -p khem-engine
//!     --test seam_symmetry -- --ignored --nocapture

mod common;

use common::*;
use khem_engine::Sim;
use khem_forcefield::ForceField;
use khem_world::{AtomId, BoundaryType, Event, WorldState};

/// K1.5 SEAM CORRECTNESS (gate ladder, the K1 close): a Wrap
/// world's seam is not special - cross-seam formation is symmetric
/// with the bulk. Promoted from phase 3 because a Wrap world with
/// asymmetric formation cannot pass honest gates (K2-K5 all run
/// Wrap worlds; the 3D port re-climbs this gate on three seams).
///
/// 3D RE-CLIMB PASS (2026-09-08) - and the re-climb's audit
/// earned its keep again, exactly as the 2D one did (F20): the
/// gate's bars passed, but the per-axis census exposed a signal
/// the 2D pooled classes averaged away, and the probe chain
/// ended in a falsification that PROVED the substrate's seam
/// symmetry rather than merely failing to refute it. The
/// record (seeds 42+137 pooled, 30x30x30 beaker - in 3D a
/// 4 A seam band is a large surface fraction of any affordable
/// box, so the seam class is ~10% of eligible pairs where the
/// 2D beaker's was ~4%: better statistics, same conditioning):
///
/// - GATHERING COMPLETENESS: F11's law held pair-for-pair
///   against brute force at every tick of both runs, all three
///   seams - zero lost pairs.
/// - FORMATION SYMMETRY, CONDITIONED: pooled seam S=132
///   E=107.7 (z +2.34), bulk S=1025 E=1076.6 (z -1.57),
///   seam/bulk efficiency ratio 1.287 - inside the bar [0.6,
///   1.4], but ~3 sigma of its own noise, unlike the 2D pass's
///   0.935. The per-axis split showed the shape: axis 0 (x,
///   the OUTER candidate-scan loop) z +2.9 pooled, axis 1
///   z +0.5, axis 2 z +0.1 - an ordering matching the
///   lexicographic scan.
/// - THE PROBE CHAIN: (1) windowed S/E - the excess is NOT
///   contention-monotone (the 8-12k window measured 0.95/1.00
///   dead-symmetric; 12-16k swung back), refuting simple
///   construction-burst priority; (2) the DOUBLE-CENSUS -
///   E accumulated against both the pre-tick and post-tick
///   states agreed within 0.5% (seam 54.2 vs 53.9), refuting
///   the mid-pass-state timing artifact; (3) the SCAN-ORDER
///   REVERSAL - a probe build with the candidate iteration
///   reversed (the physics unchanged, per-pair Bernoulli
///   draws) moved the excess to axis 2 (z +0.97/+0.82, pooled
///   efficiency 1.248) and dropped axis 0 to neutral
///   (1.020), pooled ratio 1.287 -> 1.047. A REAL seam
///   asymmetry survives scan reversal; the measured excess
///   follows the scan. THE SUBSTRATE'S SEAMS ARE SYMMETRIC.
/// - The mechanism, identified and documented as F21
///   (RESOLVED, by falsification): mid-pass ANCHOR-STATE
///   evolution - bonds formed earlier in a pass change the
///   anchor's geometry factor and order preference for its
///   LATER candidates (the 2D gate's documented "+6%,
///   anchors/orders were freer before the pass filled them"
///   offset, now exposed per-axis because the 3D census splits
///   the seam class by scan position). The census conditions
///   on the post-tick state and cannot see it; the error's
///   sign follows candidate scan position. The gate's bar
///   holds under BOTH scan orders (1.287, 1.047), and the
///   reversal is the falsification that settles the claim.
///
/// The pond CANNOT supply this measurement: its free-atom
/// sprinkle carries a 2 A margin, so a free pair straddling the
/// seam starts 4 A apart - past every capture cap (steric contact
/// ~1.5-2.3 A) - and its saturated waters never form bonds at
/// all; the cross-seam population only builds after atoms have
/// diffused to the seam, too thin to compare rates on. The gate
/// runs a dedicated BEAKER instead: the pond's own free-atom mix
/// and monolayer density (760 atoms in 60x60 A = 0.21 atoms/A^2),
/// a uniform 55 C setpoint, and NO vent - the pond's vent plume
/// sits against its y-seam, and any temperature gradient would
/// confound the seam/bulk split with a t_factor bias, so a
/// uniform field is non-negotiable; in the beaker the seam is the
/// ONLY possible asymmetry source. 55 C because the weak-pair
/// break channel runs ~30x the pond's 35 C rate (O-O lifetimes
/// ~370 ticks vs ~10k), so the run accumulates construction
/// volume and real break/re-form cycling before the strong-bonded
/// population locks up. TWO seeds (42, 137), pooled.
///
/// Measurements:
///
/// - GATHERING COMPLETENESS (the law, G07's seam half, F11):
///   every tick of both runs, for every unordered live pair within
///   the minimum-image search radius, BOTH atoms' index queries
///   must find the partner - the index is a pure accelerator in
///   Wrap worlds, never a filter. Brute force is the reference.
/// - FORMATION SYMMETRY, COMPOSITION-CONDITIONED (the dynamics):
///   every eligible pair (spec 7.2's gates mirrored here: alive,
///   capacity on both sides, minimum-image distance inside the
///   search radius AND the steric-contact cap, capture speed, not
///   already bonded to each other) is classified SEAM (the
///   minimum-image delta differs from the raw delta: the pair
///   straddles a boundary) or BULK, and contributes its REAL
///   formation probability (Chemistry::pair_probability, the same
///   law chemistry draws against, the anchor passed the same way)
///   to the class's expected count E. Actual formations S come
///   from the events. The bar is the classes' formation efficiency
///   ratio (S/E seam vs bulk): the seam class's formations must
///   match the law's expectation under the SAME probability law
///   the bulk runs.
///
/// WHY CONDITIONED, not the raw rate ratio S/eligible: the classes'
/// eligible pools CO-EVOLVE with the run and CLUSTER on shared
/// atoms - the seam class is a thin slice (a handful of edge atoms
/// supply many of its pairs), so its composition drifts as those
/// atoms keep or lose slots, and an unconditioned rate ratio
/// carries noise far beyond Poisson (measured, this gate's probe:
/// pooled raw ratio 0.78 with per-seed draws 1.01 and 0.58, while
/// the expected-p-per-pair already differed by class the same
/// 0.77 - the pool's composition, dominated by low-probability
/// C-double keys at the run's sagged temperatures, was the whole
/// story; the formation path itself was class-symmetric
/// throughout). Conditioning on sum-p absorbs composition and
/// temperature drift exactly and leaves only the Bernoulli draws.
///
/// The measured profile (2026-09-07, seeds 42+137 pooled): seam
/// S=57 E=57.7 z=-0.09; bulk S=1443 E=1365 z=+2.1; the bar - the
/// seam/bulk formation EFFICIENCY RATIO (S/E per class) - measured
/// 0.935 against a 1-sigma of ~0.13: symmetric. Both classes'
/// z sit positive: the census conditions on the POST-TICK state
/// while chemistry drew against the MID-PASS state (the tick's
/// own formation absorption cooled cells, and anchors/orders were
/// freer before the pass filled them), so the census under-models
/// attempt-p by ~6% - class-symmetric by construction, which is
/// exactly why the gate bars the ratio, not the absolute z (a
/// +-4 z tripwire stays to catch a real census-vs-law divergence).
/// The F11 failure mode (a non-folding index) still reads as a
/// total deficit: S_seam ~ 0 against E_seam ~ 58 pooled.
///
/// The census runs on the post-tick state, which is exactly the
/// state chemistry saw in POSITIONS and VELOCITIES: chemistry is
/// the last mutating step of the tick and the observer only
/// reads (G03). The divergences are the tick's own mid-pass
/// effects: pairs whose atoms filled mid-pass leave the census
/// entirely (sub-percent, seam-symmetric), and the pairs that
/// stay are modeled on the POST-TICK field and bond states while
/// the draws happened against the MID-PASS ones - the ~6%
/// class-symmetric model offset below.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn seam_formations_match_the_bulk() {
    let config = ForceField::default();
    let chem = khem_chemistry::Chemistry::new(config);

    // One beaker run: returns the pooled classes plus the
    // per-axis seam census (a pair folding on two axes - a
    // corner pair - counts once per folded axis; the axis
    // classes overlap by construction and are reported as the
    // per-seam evidence, while the POOLED seam class - any
    // folded axis, each pair once - stays the gate's primary
    // bar). The gathering law is asserted every tick inside.
    #[derive(Default, Clone, Copy)]
    struct Axis {
        elig: u64,
        exp: f64,
        forms: u64,
    }
    let run = |seed: u64| -> (f64, f64, u64, u64, [Axis; 3]) {
        // 3D PORT (provisional sizing; see the module doc): the
        // 2D beaker's free mix in a 30 A cube. In 3D a 4 A seam
        // band is a large surface fraction of any affordable box,
        // so the seam/bulk volume split differs from 2D by
        // construction - the census conditions on sum-p (see the
        // doc comment), which is exactly what makes the comparison
        // honest under a different split. The re-climb commit owns
        // the per-axis split, the sizing, and the bars.
        let mut world = WorldState::new(30.0, 30.0, 30.0, BoundaryType::Wrap, seed, config);
        world.temp_field.data.fill(55.0);
        world.setpoint_field.data.fill(55.0);
        let mix = [("H", 200u32), ("C", 160), ("N", 160), ("O", 240)];
        for (symbol, count) in mix {
            let el = khem_forcefield::element_id(symbol).expect("element in table");
            for _ in 0..count {
                let x = world.rng.f01() as f32 * world.width;
                let y = world.rng.f01() as f32 * world.height;
                let z = world.rng.f01() as f32 * world.depth;
                world.spawn_atom(el, x, y, z);
            }
        }
        let mut sim = Sim::new(config, observer(seed, 100_000));
        let _ = sim.start(&world);

        let radius = config.chemistry.bond_search_radius;
        let radius2 = radius * radius;
        let mut elig_seam: u64 = 0;
        let mut elig_bulk: u64 = 0;
        let mut exp_seam: f64 = 0.0;
        let mut exp_bulk: f64 = 0.0;
        let mut form_seam: u64 = 0;
        let mut form_bulk: u64 = 0;
        let mut axes = [Axis::default(); 3];

        eprintln!("K1.5 beaker seed {seed}: 760 free atoms, 30x30x30 Wrap, 55 C");
        for t in 1..=20_000u64 {
            let events = sim.tick(&mut world);

            // Formations, classified by the pair's own geometry:
            // pooled (any folded axis) and per-axis.
            for event in &events {
                if let Event::BondFormed { atom_a, atom_b, .. } = event {
                    let (a, b) = (world.atom(*atom_a), world.atom(*atom_b));
                    let (raw_dx, raw_dy, raw_dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
                    let (dx, dy, dz) = world.delta(a.x, a.y, a.z, b.x, b.y, b.z);
                    let folds = [dx != raw_dx, dy != raw_dy, dz != raw_dz];
                    let any_fold = folds.iter().any(|f| *f);
                    if any_fold {
                        form_seam += 1;
                    } else {
                        form_bulk += 1;
                    }
                    for (ax, folded) in folds.iter().enumerate() {
                        if *folded {
                            axes[ax].forms += 1;
                        }
                    }
                }
            }

            // Census + the gathering law, on the post-tick state
            // (the state chemistry saw; see the doc comment).
            let n = world.atoms.len();
            let queries: Vec<Vec<AtomId>> = world
                .atoms
                .iter()
                .map(|a| world.spatial_index.neighbors(a.x, a.y, a.z, radius))
                .collect();
            for (i, a) in world.atoms.iter().enumerate() {
                if !a.alive {
                    continue;
                }
                let a_free = world.element(a.element).max_bonds as i32 - a.bond_count as i32;
                for j in (i + 1)..n {
                    let b = &world.atoms[j];
                    if !b.alive {
                        continue;
                    }
                    let (raw_dx, raw_dy, raw_dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
                    let (dx, dy, dz) = world.delta(a.x, a.y, a.z, b.x, b.y, b.z);
                    let d2 = dx * dx + dy * dy + dz * dz;
                    if d2 > radius2 {
                        continue;
                    }
                    // Gathering law: both sides must find the pair.
                    assert!(
                        queries[i].contains(&b.id) && queries[j].contains(&a.id),
                        "K1.5 FAIL: index lost a close pair at tick {t}: \
                         atoms ({}, {}) at ({:.2}, {:.2}) vs ({:.2}, {:.2}), \
                         minimum-image distance {:.2}",
                        a.id.0,
                        b.id.0,
                        a.x,
                        a.y,
                        b.x,
                        b.y,
                        d2.sqrt()
                    );
                    // Eligibility census (spec 7.2 gates, mirrored).
                    if a_free <= 0 {
                        continue;
                    }
                    let b_free = world.element(b.element).max_bonds as i32 - b.bond_count as i32;
                    if b_free <= 0 {
                        continue;
                    }
                    let r_eq = world.element(a.element).radius + world.element(b.element).radius;
                    let cap = config.chemistry.bond_form_factor * r_eq;
                    let (rvx, rvy, rvz) = (b.vx - a.vx, b.vy - a.vy, b.vz - a.vz);
                    if d2 <= cap * cap
                        && rvx * rvx + rvy * rvy + rvz * rvz
                            <= config.chemistry.max_form_speed * config.chemistry.max_form_speed
                        && !world.is_bonded(a.id, b.id)
                    {
                        // The anchor (lower AtomId) is passed first:
                        // chemistry attempts the pair from the
                        // lower id, and the geometry factor anchors
                        // on it (7.2).
                        let p = chem.pair_probability(&world, a.id, b.id).p;
                        let folds = [dx != raw_dx, dy != raw_dy, dz != raw_dz];
                        let any_fold = folds.iter().any(|f| *f);
                        if any_fold {
                            elig_seam += 1;
                            exp_seam += f64::from(p);
                        } else {
                            elig_bulk += 1;
                            exp_bulk += f64::from(p);
                        }
                        for (ax, folded) in folds.iter().enumerate() {
                            if *folded {
                                axes[ax].elig += 1;
                                axes[ax].exp += f64::from(p);
                            }
                        }
                    }
                }
            }

            if t.is_multiple_of(2000) {
                let free = world
                    .atoms
                    .iter()
                    .filter(|a| a.alive && a.bond_count == 0)
                    .count();
                let avg: f32 =
                    world.temp_field.data.iter().sum::<f32>() / world.temp_field.data.len() as f32;
                eprintln!(
                    "seed {seed} t={t} elig seam/bulk {elig_seam}/{} \
                     form seam/bulk {form_seam}/{} free {free} bonds {} \
                     field {avg:.1}",
                    elig_bulk,
                    form_bulk,
                    world.live_bond_count()
                );
            }
        }
        (exp_seam, exp_bulk, form_seam, form_bulk, axes)
    };

    let mut exp = (0.0f64, 0.0f64);
    let mut forms = (0u64, 0u64);
    let mut axis_total = [Axis::default(); 3];
    for seed in [42u64, 137] {
        let (es, eb, fs, fb, axes) = run(seed);
        let z_seam = (fs as f64 - es) / es.sqrt();
        let z_bulk = (fb as f64 - eb) / eb.sqrt();
        eprintln!(
            "seed {seed}: seam S={fs} E={es:.1} z {z_seam:+.2}; \
             bulk S={fb} E={eb:.1} z {z_bulk:+.2}",
        );
        for ax in 0..3 {
            let a = &axes[ax];
            let z = if a.exp > 0.0 {
                (a.forms as f64 - a.exp) / a.exp.sqrt()
            } else {
                0.0
            };
            eprintln!(
                "  axis {ax}: S={} E={:.1} z {z:+.2} (elig {})",
                a.forms, a.exp, a.elig
            );
            axis_total[ax].elig += a.elig;
            axis_total[ax].exp += a.exp;
            axis_total[ax].forms += a.forms;
        }
        exp.0 += es;
        exp.1 += eb;
        forms.0 += fs;
        forms.1 += fb;
    }
    let (exp_seam, exp_bulk, form_seam, form_bulk) = (exp.0, exp.1, forms.0, forms.1);
    // The per-seam record: each axis's efficiency ratio against
    // the bulk (the classes overlap on corner pairs by
    // construction; the POOLED ratio below is the gate's bar).
    for (ax, a) in axis_total.iter().enumerate() {
        let ratio = (a.forms as f64 / a.exp) / (form_bulk as f64 / exp_bulk);
        eprintln!(
            "axis {ax} pooled: S={} E={:.1} efficiency vs bulk {ratio:.3}",
            a.forms, a.exp
        );
    }
    let z_seam = (form_seam as f64 - exp_seam) / exp_seam.sqrt();
    let z_bulk = (form_bulk as f64 - exp_bulk) / exp_bulk.sqrt();
    // THE seam-symmetry statistic: the classes' formation
    // efficiencies under the same law-model. The census-model
    // offset (see the doc comment: the census conditions on the
    // post-tick state, chemistry drew against the mid-pass state)
    // is class-symmetric by construction and cancels here.
    let ratio = (form_seam as f64 / exp_seam) / (form_bulk as f64 / exp_bulk);
    eprintln!(
        "\nK1.5 pooled: seam S={form_seam} E={exp_seam:.1} z {z_seam:+.2}; \
         bulk S={form_bulk} E={exp_bulk:.1} z {z_bulk:+.2}; \
         seam/bulk efficiency ratio {ratio:.3} (the bar)"
    );

    // PASS bars (set from the measured profile; see the commit):
    // the pooled expectation must carry enough signal that the
    // ratio means something (E_seam ~ 58 pooled makes |z| = 3 a
    // ~40% asymmetry - F11's total suppression is z ~ -7.5), the
    // per-class z is a DIVERGENCE tripwire at +-4 (the known
    // class-symmetric census-model offset measures ~+6% pooled,
    // z ~ +2: a real census-vs-law divergence sits far beyond
    // it), and the seam/bulk efficiency ratio - the gate's claim
    // - must sit inside the band. The ratio's Poisson noise at
    // the measured counts is ~0.13 (1 sigma), so [0.6, 1.4] is
    // ~3 sigma: F11 (ratio ~ 0) and any doubled asymmetry fail
    // decisively.
    assert!(
        exp_seam >= 40.0,
        "K1.5 FAIL: only {exp_seam:.1} expected cross-seam formations - \
         the beaker cannot see the seam"
    );
    assert!(
        form_seam >= 20,
        "K1.5 FAIL: only {form_seam} cross-seam formations - \
         the beaker cannot see the seam"
    );
    assert!(
        z_bulk.abs() <= 4.0 && z_seam.abs() <= 4.0,
        "K1.5 FAIL: census diverged from the law - seam z {z_seam:+.2}, \
         bulk z {z_bulk:+.2} (the model offset is ~+6%; see the doc comment)"
    );
    assert!(
        (0.6..=1.4).contains(&ratio),
        "K1.5 FAIL: seam/bulk formation efficiency ratio {ratio:.3} - \
         the seam is special"
    );
}
