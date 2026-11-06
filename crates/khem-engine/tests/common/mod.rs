#![allow(dead_code)]

//! The shared harness kit: run helpers, pond structure sampling,
//! and the always-on invariants every gate file reuses.
//! (Integration tests cannot import each other; each test file
//! declares `mod common;` and this file - the `common` directory,
//! not a test target - compiles into each test binary.)

use khem_engine::{Observer, ObserverConfig, Sim};
use khem_forcefield::ForceField;
use khem_world::WorldState;
use khem_worlds::pond;

pub const WATERS: usize = pond::POND_WATERS;

// ---- Shared helpers ----------------------------------------------------

pub fn observer(seed: u64, interval: u64) -> Observer {
    Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k1_harness".to_string(),
        world_name: "primordial_pond".to_string(),
        seed,
        tick_interval: interval,
    })
}

pub fn run_ticks(seed: u64, ticks: u64) -> WorldState {
    let config = ForceField::default();
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
pub fn mean_bond_length(world: &WorldState) -> f32 {
    let lens = world.bond_lengths();
    let finite: Vec<f32> = lens.iter().copied().filter(|l| l.is_finite()).collect();
    finite.iter().sum::<f32>() / finite.len().max(1) as f32
}

/// All atom state finite (no NaN/inf positions or velocities).
pub fn all_state_finite(world: &WorldState) -> bool {
    world.atoms.iter().all(|a| {
        a.x.is_finite()
            && a.y.is_finite()
            && a.z.is_finite()
            && a.vx.is_finite()
            && a.vy.is_finite()
            && a.vz.is_finite()
    })
}

/// Per-bond stretch ratios (length / r_eq) of all live bonds, in
/// bond order. Each bond is measured against its OWN equilibrium
/// (radius sum), so a heterogeneous pond compares like with like;
/// lengths use the minimum-image delta (F10).
pub fn bond_stretch_ratios(world: &WorldState) -> Vec<f32> {
    world
        .bonds
        .iter()
        .filter(|b| b.alive)
        .map(|b| {
            let (a, c) = (world.atom(b.atom_a), world.atom(b.atom_b));
            let (dx, dy, dz) = world.delta(a.x, a.y, a.z, c.x, c.y, c.z);
            let r_eq = world.element(a.element).radius + world.element(c.element).radius;
            (dx * dx + dy * dy + dz * dz).sqrt() / r_eq
        })
        .collect()
}

/// g(r) bin geometry, angstroems (the K2.2 gate's table: 0.25 A
/// bins out to 4.5 A - one shell past the cohesion range; the
/// maximum query sphere fits the pond slab's half-depth).
pub const GR_BIN: f32 = 0.25;
pub const GR_MAX: f32 = 4.5;

/// The pond's steady structure, sampled over [t_sample_start,
/// t_end] every `sample_every` ticks: (census mean, census min,
/// the g(r) table, samples, alive atoms). The K2.2 gate and the
/// cohesion sweep share it. Intermolecular pairs only (different
/// molecule by the bond graph's union-find).
pub fn pond_structure(
    config: ForceField,
    t_end: u64,
    t_sample_start: u64,
    sample_every: u64,
    seed: u64,
) -> (f64, f64, Vec<f64>, u64, u64) {
    // Runs the pond to t_end, samples the steady structure over
    // [t_sample_start, t_end] every sample_every ticks, and returns
    // (census mean, census min, the g(r) table, samples, alive
    // atoms). The K2.2 gate and the K2.3 cohesion sweep share it.
    let mut world = khem_worlds::pond::primordial_pond(seed, config);
    let observer = Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k2_harness".to_string(),
        world_name: "primordial_pond".to_string(),
        seed,
        tick_interval: 10_000,
    });
    let mut sim = Sim::new(config, observer);
    let _ = sim.start(&world);

    // Union-find over the alive bonds, per sample: the molecule
    // identity the census and the g(r) split on. Atom ids are
    // indexes (spawn order; no compaction in a Wrap world).
    let n = world.atoms.len();
    let mut parent = vec![0u32; n];
    let find = |parent: &mut Vec<u32>, mut x: u32| -> u32 {
        while parent[x as usize] != x {
            let grandparent = parent[parent[x as usize] as usize];
            parent[x as usize] = grandparent;
            x = grandparent;
        }
        x
    };

    let n_bins = (GR_MAX / GR_BIN) as usize;
    let mut bin_counts = vec![0u64; n_bins];
    let mut census_sum = 0.0f64;
    let mut census_min = f64::INFINITY;
    let mut samples = 0u64;
    let mut alive_atoms = 0u64;
    for t in 1..=t_end {
        sim.tick(&mut world);
        if t < t_sample_start || !t.is_multiple_of(sample_every) {
            continue;
        }
        for (i, slot) in parent.iter_mut().enumerate() {
            *slot = i as u32;
        }
        for bond in &world.bonds {
            if bond.alive {
                let ra = find(&mut parent, bond.atom_a.0);
                let rb = find(&mut parent, bond.atom_b.0);
                if ra != rb {
                    parent[ra as usize] = rb;
                }
            }
        }
        let alive = world.atoms.iter().filter(|a| a.alive).count();
        alive_atoms = alive as u64;
        let mut within = 0u64;
        for atom in &world.atoms {
            if !atom.alive {
                continue;
            }
            let near = world.spatial_index.neighbors(atom.x, atom.y, atom.z, 3.0);
            let root = find(&mut parent, atom.id.0);
            let mut nearest = f32::INFINITY;
            for b_id in near {
                let b = world.atom(b_id);
                if !b.alive || b_id == atom.id {
                    continue;
                }
                if find(&mut parent, b_id.0) == root {
                    continue; // same molecule: intramolecular
                }
                let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                if r < nearest {
                    nearest = r;
                }
            }
            if nearest <= 3.0 {
                within += 1;
            }
        }
        let fraction = within as f64 / alive as f64;
        census_sum += fraction;
        census_min = census_min.min(fraction);

        for atom in &world.atoms {
            if !atom.alive {
                continue;
            }
            let near = world
                .spatial_index
                .neighbors(atom.x, atom.y, atom.z, GR_MAX);
            let root = find(&mut parent, atom.id.0);
            for b_id in near {
                if b_id.0 <= atom.id.0 {
                    continue;
                }
                let b = world.atom(b_id);
                if !b.alive {
                    continue;
                }
                if find(&mut parent, b_id.0) == root {
                    continue;
                }
                let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                if r < GR_MAX {
                    bin_counts[(r / GR_BIN) as usize] += 1;
                }
            }
        }
        samples += 1;
    }
    let census_mean = census_sum / samples.max(1) as f64;
    let volume = world.width * world.height * world.depth;
    let rho = alive_atoms as f64 / volume as f64;
    let mut gr = vec![0.0f64; n_bins];
    for (b, count) in bin_counts.iter().enumerate() {
        let r0 = b as f64 * GR_BIN as f64;
        let r1 = r0 + GR_BIN as f64;
        let v_bin = 4.0 * std::f64::consts::PI / 3.0 * (r1.powi(3) - r0.powi(3));
        let expected = 0.5 * alive_atoms as f64 * rho * v_bin * samples as f64;
        gr[b] = *count as f64 / expected.max(1.0);
    }
    (census_mean, census_min, gr, samples, alive_atoms)
}
