//! The cohesion depth sweep - the evidence tool behind the
//! cohesion channel's DORMANT default (the measured finding:
//! every shell-building depth condenses the seeded pond; the
//! structural cause is in the config's cohesion_scale doc).
//! The real-water substrate sitting (ADR-0014) owns the
//! re-derivation at the new wall geometry.
//!
//! Run (release, ~2 x 12k ticks): cargo test --release
//!   -p khem-engine --test cohesion_sweep -- --ignored --nocapture

mod common;

use common::*;
use khem_forcefield::{CohesionParams, ForceField};

/// The K2.3 cohesion lever curve: the pond's structure across
/// cohesion_scale values (12k-tick runs, sampling 10k-12k every
/// 50 ticks - the pre-settle window is fine for a SHAPE
/// comparison across values; the official K2.2 re-run at 25k
/// carries the chosen value). Not a gate: the evidence tool
/// whose value, and the re-validation it owes, live in the K2.3
/// records.
///
/// THE MEASURED FINDING (2026-11-04, why the channel ships
/// DORMANT): every useful depth condenses the seeded pond into
/// a droplet - depth 0.25: first shell 3.5 with the g(r)
/// plateauing at ~2.4-3.0x the box average (local density ~2.4x;
/// the first shell reads 1.18 RELATIVE TO THE LOCAL density);
/// depth 0.5: similar; depth 1.0: g ~ 8.8-11.4 everywhere (a
/// ~10x clump). A cohesive liquid seeded at the pond's 0.0636
/// atoms/A^3 (0.63x real water's atomic density) contracts
/// toward liquid density, exactly as real water would - the
/// pond RE-DENSITY the port plan pre-authorized on evidence is
/// the named blocker for activation; see the memo's revision
/// log.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn cohesion_depth_sweep() {
    for cohesion in [0.5f32, 1.0] {
        let config = ForceField {
            cohesion: CohesionParams {
                cohesion_scale: cohesion,
                ..Default::default()
            },
            ..Default::default()
        };
        eprintln!("K2.3 cohesion sweep: cohesion_scale {cohesion}");
        let (census_mean, census_min, gr, samples, alive) =
            pond_structure(config, 12_000, 10_000, 50, 42);
        let peak = gr.iter().cloned().fold(0.0f64, f64::max);
        let peak_bin = gr
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap_or(0);
        eprintln!(
            "  census {census_mean:.3} (min {census_min:.3}), peak g \
             {peak:.3} at [{:.2},{:.2}), {samples} samples, {alive} atoms",
            peak_bin as f64 * GR_BIN as f64,
            peak_bin as f64 * GR_BIN as f64 + GR_BIN as f64
        );
        eprintln!(
            "  g(r): {}",
            gr.iter()
                .map(|g| format!("{g:.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}
