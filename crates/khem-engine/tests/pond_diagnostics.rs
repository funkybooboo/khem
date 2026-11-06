//! The pond's honest rollup after 2000 ticks: water survival,
//! bond activity, geometry, energy - the K1 ladder's diagnostic
//! summary. Its bars pass as of the K1.5 commit; the sub-gates
//! own the finer criteria.
//!
//! Run (release): cargo test --release -p khem-engine
//!   --test pond_diagnostics -- --ignored --nocapture

mod common;

use common::*;
use khem_worlds::pond::water_intact;

#[test]
#[ignore] // explicit: cargo test -- --ignored --nocapture
fn pond_rollup_diagnostics() {
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
