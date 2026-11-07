//! The water-likeness probe - the real-water substrate sitting's
//! evidence tool (ADR-0014). Runs the pond at a candidate
//! geometry (the wall margin, the hydrogen-bond scale, the
//! cohesion depth) and measures the owner's checkpoint contract,
//! THE SIX BARS:
//!
//! 1. MOLECULAR DENSITY: alive waters per volume against real
//!    water's ~0.033 molecules/A^3, held UNIFORM (no blobs: the
//!    8 A density grid's max/mean ratio ~ 1, not >> 2).
//! 2. O-O FIRST SHELL: the O-O g(r) peak position ~2.8 A.
//! 3. O COORDINATION: mean O within the first shell ~4.
//! 4. TETRAHEDRAL ORDER: q over O centers measurable (real liquid
//!    water ~0.7; uncorrelated ~0; the bar is MEASURABLE
//!    structure, not a number).
//! 5. WATERS INTACT: the molecule census (the seeded waters
//!    survive the new geometry).
//! 6. TICK COST: measured by the run's own timing (against the
//!    dormancy baseline).
//!
//! Not a gate: the evidence tool whose chosen values, and the
//! re-validation they force, land in the sitting's record. Run
//! (release, ~12k ticks per candidate):
//!   cargo test --release -p khem-engine
//!     --test water_likeness_probe -- --ignored --nocapture

mod common;

use common::observer;
use khem_analysis::{Rdf, mean_coordination, molecule_sizes, tetrahedral_order};
use khem_engine::Sim;
use khem_forcefield::{ExcludedVolumeParams, ForceField, HbondParams, O};
use khem_worlds::pond;

/// One candidate's six bars.
struct Bars {
    /// Alive atoms at the tail (waters + free).
    alive: u64,
    /// Intact waters (H2O molecules) at the tail.
    waters: u64,
    /// Molecular density, molecules/A^3.
    density: f64,
    /// The 8 A density grid's max/mean (blob detector: uniform
    /// liquid ~1.0-1.3; phase separation >> 2).
    uniformity: f64,
    /// The O-O g(r): (peak bin position, peak height).
    oo_peak: (f64, f64),
    /// Mean O within the first-shell probe (3.4 A).
    oo_coordination: f64,
    /// Tetrahedral order q over O centers (O neighbors, 3.4 A).
    q: f64,
    /// Seconds per 1000 ticks at the tail (cost bar).
    sec_per_kilo: f64,
}

fn measure(config: ForceField, t_end: u64, t_sample_start: u64, seed: u64) -> Bars {
    let mut world = pond::primordial_pond(seed, config);
    let mut sim = Sim::new(config, observer(seed, 10_000));
    let _ = sim.start(&world);

    // The O-O rdf and the O-O coordination, accumulated over the
    // steady-tail samples; q and the blob grid measured per
    // sample and averaged.
    let mut rdf = Rdf::new(6.0, 0.1);
    let mut coord_sum = 0.0f64;
    let mut q_sum = 0.0f64;
    let mut uniformity_sum = 0.0f64;
    let mut samples = 0u64;
    let mut alive = 0u64;
    let mut waters = 0u64;
    let mut t_last = t_sample_start;
    let mut time_last = std::time::Instant::now();
    let mut sec_per_kilo = 0.0f64;

    // The blob grid: 8 A cells over the slab, live-atom counts.
    let (gw, gh, gd) = (
        (world.width / 8.0).ceil() as usize,
        (world.height / 8.0).ceil() as usize,
        (world.depth / 8.0).ceil() as usize,
    );

    for t in 1..=t_end {
        sim.tick(&mut world);
        if t < t_sample_start || !(t - t_sample_start).is_multiple_of(250) {
            continue;
        }
        // The cost bar: wall clock per 250 ticks at the tail
        // (both anchors move with each sample).
        let now = std::time::Instant::now();
        sec_per_kilo = (now - time_last).as_secs_f64() / (t - t_last) as f64 * 1000.0;
        t_last = t;
        time_last = now;

        alive = world.atoms.iter().filter(|a| a.alive).count() as u64;
        // Molecule census: intact waters are size-3 molecules of
        // H2O composition - count size-3 components whose members
        // are two H and one O.
        let sizes = molecule_sizes(&world);
        waters = sizes.iter().filter(|&&s| s == 3).count() as u64;

        // O-O rdf: all alive O pairs (the analysis convention:
        // each unordered pair once).
        let oxygens: Vec<(f32, f32, f32)> = world
            .atoms
            .iter()
            .filter(|a| a.alive && a.element == O)
            .map(|a| (a.x, a.y, a.z))
            .collect();
        for (i, &(ax, ay, az)) in oxygens.iter().enumerate() {
            let near = world.spatial_index.neighbors(ax, ay, az, 6.0);
            for id in near {
                if id.0 as usize <= i {
                    continue;
                }
                let b = world.atom(id);
                if !b.alive || b.element != O {
                    continue;
                }
                let (dx, dy, dz) = world.delta(ax, ay, az, b.x, b.y, b.z);
                rdf.add((dx * dx + dy * dy + dz * dz).sqrt());
            }
        }
        // The other bars.
        coord_sum += mean_coordination(&world, O, O, 3.4);
        q_sum += tetrahedral_order(&world, O, O, 3.4);
        let mut grid = vec![0u32; gw * gh * gd];
        for atom in world.atoms.iter().filter(|a| a.alive) {
            let (cx, cy, cz) = (
                (atom.x / 8.0) as usize,
                (atom.y / 8.0) as usize,
                (atom.z / 8.0) as usize,
            );
            grid[cy * gw * gd + cz * gw + cx] += 1;
        }
        let mean = grid.iter().map(|&c| c as f64).sum::<f64>() / grid.len() as f64;
        let max = grid.iter().map(|&c| c as f64).fold(0.0, f64::max);
        uniformity_sum += max / mean.max(1.0);
        samples += 1;
    }

    let volume = world.width as f64 * world.height as f64 * world.depth as f64;
    // The O-O g(r) over the samples.
    let n_o = world
        .atoms
        .iter()
        .filter(|a| a.alive && a.element == O)
        .count() as u64;
    let g = rdf.normalize(n_o, volume, samples.max(1));
    let (peak_bin, peak) = g
        .iter()
        .enumerate()
        .skip(2)
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, &v)| (i as f64 * 0.1, v))
        .unwrap_or((0.0, 0.0));

    Bars {
        alive,
        waters,
        density: waters as f64 / volume,
        uniformity: uniformity_sum / samples.max(1) as f64,
        oo_peak: (peak_bin, peak),
        oo_coordination: coord_sum / samples.max(1) as f64,
        q: q_sum / samples.max(1) as f64,
        sec_per_kilo,
    }
}

fn report(label: &str, config: ForceField, t_end: u64, t_start: u64) {
    eprintln!(
        "== {label}: margin {:.2}, hbond {:.2}, cohesion {:.2}",
        config.excluded_volume.non_bonded_margin,
        config.hbond.hbond_scale,
        config.cohesion.cohesion_scale,
    );
    let b = measure(config, t_end, t_start, 42);
    eprintln!(
        "   alive {} | intact H2O {} | density {:.4} mol/A^3 \
         | uniformity max/mean {:.2} | O-O peak {:.2} A at g {:.2} \
         | O coord {:.2} | q {:.3} | {:.2} s/kilotick",
        b.alive,
        b.waters,
        b.density,
        b.uniformity,
        b.oo_peak.0,
        b.oo_peak.1,
        b.oo_coordination,
        b.q,
        b.sec_per_kilo,
    );
}

/// The sitting's sweep: candidate geometries against the six
/// bars. Each run is 12k ticks with the tail sampled from 10k
/// (the pre-settle window is fine for a SHAPE comparison; the
/// final config's official gate re-runs carry the chosen value).
#[test]
#[ignore] // explicit: cargo test --release -p khem-engine --test water_likeness_probe -- --ignored --nocapture
fn real_geometry_candidates() {
    // Round two: the RE-BRACKETED well. The first sweep's finding
    // (why q never moved at safe scales): the bracket [1.5, 2.6]
    // puts the well's restoring zero at H...O = 2.05 A, but the
    // liquid's actual contact geometry is O...O 2.90 with the
    // O-H bond at 1.19 - a collinear bond puts H...O at 1.71,
    // INSIDE the old midpoint, where the well's inner slope
    // repels the very contact it should stabilize. The bracket
    // moves to [1.3, 2.1]: midpoint 1.7 A = the first shell
    // (2.90) minus the bond (1.19), the geometry the liquid
    // adopts. Scales inside the mechanically-safe bracket (the
    // first sweep measured 1.5 shattering every water; 0.5-1.0
    // safe).
    for scale in [0.5f32, 1.0] {
        report(
            &format!("wall 2.2, hbond {scale}, bracket [1.3, 2.1]"),
            ForceField {
                excluded_volume: ExcludedVolumeParams {
                    non_bonded_margin: 2.2,
                    ..ForceField::default().excluded_volume
                },
                hbond: HbondParams {
                    hbond_scale: scale,
                    hbond_contact: 1.3,
                    hbond_range: 2.1,
                },
                ..ForceField::default()
            },
            12_000,
            10_000,
        );
    }
}
