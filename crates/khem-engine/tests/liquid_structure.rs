//! The condensed medium: the steady-tail pond is contact-dense
//! matter - the intermolecular contact census holds its bar, and
//! the g(r) shows the excluded-volume hole with the
//! preregistered no-first-shell baseline (g_max 1.002 beyond
//! 3.0 A: repulsion cannot build cohesion; the attraction
//! channel must beat that number).
//!
//! Gate K2.2 of the self-assembly ladder (PASS records:
//! plans/gates/self-assembly.md). Run (release, the long gate):
//!   cargo test --release -p khem-engine
//!     --test liquid_structure -- --ignored --nocapture

mod common;

use common::*;
use khem_forcefield::ForceField;

#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn the_medium_is_condensed_matter() {
    let config = ForceField::default();
    eprintln!(
        "K2.2: the pond, 25k ticks, sampling the steady tail \
         (21k-25k every 100 ticks)"
    );
    let (census_mean, census_min, gr, samples, alive) =
        pond_structure(config, 25_000, 21_000, 100, 42);
    eprintln!(
        "K2.2 census: mean {census_mean:.3} (bar >= 0.5), min {census_min:.3}, \
         over {samples} samples ({alive} atoms)"
    );
    eprintln!("K2.2 g(r) vs uniform:");
    for (b, g) in gr.iter().enumerate() {
        let r0 = b as f64 * GR_BIN as f64;
        let r1 = r0 + GR_BIN as f64;
        eprintln!("  r [{r0:.2},{r1:.2}): g {g:.3}");
    }

    // ---- The bars ----------------------------------------------------
    assert!(
        census_mean >= 0.5,
        "K2.2 FAIL: census mean {census_mean:.3} < 0.5 - the pond \
         is not holding most atoms in intermolecular contact"
    );
    let hole_core_max = gr
        .iter()
        .take((1.25f32 / GR_BIN) as usize)
        .cloned()
        .fold(0.0f64, f64::max);
    assert!(
        hole_core_max <= 0.05,
        "K2.2 FAIL: the excluded-volume core is not clean - g \
         {hole_core_max:.3} below 1.25 A (bar 0.05)"
    );
    let hole_extent_max = gr
        .iter()
        .take((3.0f32 / GR_BIN) as usize)
        .cloned()
        .fold(0.0f64, f64::max);
    assert!(
        hole_extent_max < 1.0,
        "K2.2 FAIL: the hole does not extend - g {hole_extent_max:.3} \
         at or above uniform below 3.0 A"
    );
    // The preregistered baseline: repulsion-only cannot build a
    // first shell (spec 6.6); the attraction channel's commit
    // re-runs this gate and replaces this bar with a peak bar.
    let baseline_max = gr
        .iter()
        .skip((3.0f32 / GR_BIN) as usize)
        .cloned()
        .fold(0.0f64, f64::max);
    assert!(
        baseline_max <= 1.05,
        "K2.2: the repulsion-only baseline moved - g {baseline_max:.3} \
         beyond 3.0 A (baseline bar 1.05); if the attraction \
         channel landed, replace this bar with a peak bar and \
         re-run per the contract"
    );
    eprintln!(
        "K2.2 PASS: census {census_mean:.3} (bar 0.5), hole core \
         {hole_core_max:.3} (<= 0.05), hole extent max \
         {hole_extent_max:.3} (< 1.0 below 3 A), baseline \
         {baseline_max:.3} (<= 1.05 - no shell, as designed)"
    );
}
