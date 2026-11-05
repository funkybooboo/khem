//! The K2 self-assembly harness (docs/plans/phase-1-kernel.md
//! gate ladder, milestone K2).
//!
//! Levels:
//!
//! - `k2_scatter_smoke` (always on, part of `mise run check`): a
//!   head-on scattering pass at the shipped defaults stays
//!   finite, never coincides, never passes through, and never
//!   mints a large velocity. Cheap regression at the tuned
//!   constants - the 0.8 * (r_a + r_b) bar is the GATE's
//!   business; constants are the gate's lever.
//! - `k2_1_excluded_volume` (gate K2.1; the F4 fix's own
//!   measurement): free atoms no longer pass through each
//!   other. Two phases:
//!   - PHASE A, controlled scattering: pairs launched head-on
//!     and grazing at closing speeds c x sigma_rel(T = 35 C),
//!     c in {1, 2, 3} - a common collision, a fast one, and the
//!     p99 tail a 10k-tick beaker samples routinely - across
//!     four element classes (H-H the worst tunneling case:
//!     lightest pair, narrowest cutoff; H-O, O-O, Na-Cl the
//!     heavy span). Physics-only worlds (Sim's exact sub-step
//!     structure: index rebuilt after the first sub-step,
//!     boundary per tick - but no bath, no chemistry, so the
//!     kinematics are exactly what the integrator produces and
//!     nothing else), minimum approach measured at EVERY
//!     sub-step boundary. BARS: min approach >= 0.8 * (r_a +
//!     r_b) for every pair (the gate's letter); no pass-through
//!     (the launch-axis projection never flips); no energy mint
//!     (pair kinetic energy at encounter exit <= 1.10 x launch -
//!     the F9/F13 law: a crossing resolved by sub-stepping
//!     samples the wall symmetrically and cannot mint. The pair
//!     TOTAL is the invariant: unequal-mass bounces legitimately
//!     trade speed between the atoms).
//!   - PHASE B, the operating soak: the K1.5 beaker mix (760
//!     free atoms, 30x30x30 Wrap) at the pond's 35 C operating
//!     point, FULL Sim (bath and chemistry live - bonds form and
//!     the census excludes them: a bonded pair is the spring's
//!     business, K1.2's band), 10k ticks. BAR: no unbonded pair
//!     ever samples below 0.8 * (r_a + r_b); the closest
//!     approach and the near-miss census are the reported
//!     margin. Phase B is what the pond actually feels - the
//!     substrate K2.2 (condensed medium) and K2.3 (amphiphile
//!     sorting) will run on.
//!     Run: `mise exec -- cargo test --release -p khem-core
//!     --test k2_self_assembly -- --ignored --nocapture`
//! - `k2_1_strength_sweep` (the lever probe, not a gate): the
//!   same phase A across non_bonded_repulsion in {1, 4, 8, 16,
//!   32} - the curve that picked the default (8.0; PASS record
//!   above) plus the symplectic bound column (dt_sub * sqrt(S /
//!   mu) < 2; the pond's spring bound sits at 1.375 for
//!   comparison). The K1 re-runs the retune forced (the
//!   re-validation contract: a force change moves every operating
//!   point) are summarized in the gate's PASS record.
//!
//! Design notes:
//! - The wall's honest failure mode is a HEAD-ON pair with all
//!   its relative kinetic energy on the line of centers: the
//!   turning point must sit above 0.8 * (r_a + r_b). Grazing
//!   bins (impact parameter 0.4 * cutoff) deflect early and
//!   stress the mint law instead.
//! - One pair per world: exact attribution, no cross-pair
//!   interference, and the 40 A box plus the per-bin tick
//!   budget keeps total relative displacement under the box
//!   width, so no Wrap image is ever met twice.
//! - sigma_rel is the per-component relative thermal scale the
//!   bath's stationary distribution implies for the pair:
//!   sqrt(thermal_kick_scale * T / mu) - the same law
//!   thermal_bath drives (physics.rs), evaluated at the pair's
//!   reduced mass, so the bins are the pond's own speeds by
//!   construction, not hand-picked numbers.
//! - The field's temperature units are the pond's own (the 35 C
//!   setpoint), matching how thermal_bath reads the field.

use khem_core::config::PhysicsConfig;
use khem_core::observer::{Observer, ObserverConfig};
use khem_core::{BoundaryType, ElementId, Physics, PhysicsSystem, Sim, WorldState};

// ---- Shared helpers ----------------------------------------------------

/// One scattering pass's verdict.
struct Sample {
    /// Minimum approach over the encounter, as a fraction of the
    /// pair's radius sum (the gate bar is 0.8).
    min_ratio: f32,
    /// The pair's TOTAL kinetic energy at encounter exit divided
    /// by its launch value. An honest elastic wall returns ~1.0:
    /// equal-mass bounces keep both speeds, unequal-mass bounces
    /// legitimately trade speed between the atoms (momentum does
    /// that - a head-on H into O exits at 2.76x launch speed with
    /// the pair's KE unchanged, which is why the invariant is the
    /// PAIR total, not per-atom speed). The bar is 1.10 (the
    /// F9/F13 mint law).
    mint: f32,
    /// The pair's separation along the launch axis flipped sign:
    /// they passed through each other.
    crossed: bool,
}

/// The per-component relative thermal speed scale of a pair at
/// field temperature `t` (see the module doc): the bath's own
/// stationary law, evaluated at the reduced mass.
fn sigma_rel(config: &PhysicsConfig, a: ElementId, b: ElementId, t: f32) -> f32 {
    let (ma, mb) = (
        khem_core::ELEMENTS[a.0 as usize].mass,
        khem_core::ELEMENTS[b.0 as usize].mass,
    );
    let mu = ma * mb / (ma + mb);
    (config.thermal_kick_scale * t / mu).sqrt()
}

/// One scattering pass: two free atoms launched toward each
/// other with closing speed `v_c` along a random axis, impact
/// parameter `imp` (perpendicular offset), in a 40 A Wrap world.
/// Physics-only at Sim's exact sub-step structure (index rebuilt
/// after the first sub-step, boundary per tick); no bath, no
/// chemistry - the kinematics are exactly the integrator's.
fn scatter(
    config: PhysicsConfig,
    a_el: ElementId,
    b_el: ElementId,
    v_c: f32,
    imp: f32,
    seed: u64,
) -> Sample {
    let mut world = WorldState::new(40.0, 40.0, 40.0, BoundaryType::Wrap, seed, config);
    // Launch axis: a random unit direction from the world RNG
    // (deterministic per seed); degenerate draws guarded.
    let mut ux = world.rng.normal(0.0, 1.0) as f32;
    let mut uy = world.rng.normal(0.0, 1.0) as f32;
    let mut uz = world.rng.normal(0.0, 1.0) as f32;
    let un = (ux * ux + uy * uy + uz * uz).sqrt();
    if un < 1e-6 {
        (ux, uy, uz) = (1.0, 0.0, 0.0);
    } else {
        (ux, uy, uz) = (ux / un, uy / un, uz / un);
    }
    // A perpendicular unit vector for the impact parameter.
    let (wx, wy, wz) = {
        let (cx, cy, cz) = (
            uy * 1.0 - uz * 0.0,
            uz * 0.0 - ux * 1.0,
            ux * 0.0 - uy * 0.0,
        );
        // cross(u, (0, 0, 1)) is degenerate when u is near z; fall
        // back to cross(u, (0, 1, 0)).
        let (mut px, mut py, mut pz) = (cx, cy, cz);
        let pn = (px * px + py * py + pz * pz).sqrt();
        if pn < 1e-3 {
            (px, py, pz) = (
                uz * 1.0 - ux * 0.0,
                ux * 0.0 - uy * 1.0,
                uy * 0.0 - uz * 0.0,
            );
        }
        let pn = (px * px + py * py + pz * pz).sqrt();
        (px / pn, py / pn, pz / pn)
    };
    let center = (20.0f32, 20.0f32, 20.0f32);
    let half = 6.0f32;
    let a = world.spawn_atom(
        a_el,
        center.0 - ux * half + wx * imp * 0.5,
        center.1 - uy * half + wy * imp * 0.5,
        center.2 - uz * half + wz * imp * 0.5,
    );
    let b = world.spawn_atom(
        b_el,
        center.0 + ux * half - wx * imp * 0.5,
        center.1 + uy * half - wy * imp * 0.5,
        center.2 + uz * half - wz * imp * 0.5,
    );
    let v = v_c * 0.5;
    {
        let atom = world.atom_mut(a);
        atom.vx = ux * v;
        atom.vy = uy * v;
        atom.vz = uz * v;
    }
    {
        let atom = world.atom_mut(b);
        atom.vx = -ux * v;
        atom.vy = -uy * v;
        atom.vz = -uz * v;
    }
    let r_sum = world.element(a_el).radius + world.element(b_el).radius;
    let cutoff = r_sum * config.non_bonded_margin;
    // Encounter bookkeeping: the latch zone is wide enough that
    // entry and exit are unambiguous, and the tick budget carries
    // the pair from launch (12 A apart) through the zone and out
    // the far side: (12 + 2.5 * cutoff) / v_c + 4. Total
    // relative displacement stays under the 40 A box, so no Wrap
    // image is met twice.
    let zone = 2.5 * cutoff;
    let ticks = ((12.0 + zone) / v_c).ceil() as u64 + 4;
    world.spatial_index.rebuild(&world.atoms);
    let mut sys = Physics::new(config);
    let mut min_r = f32::INFINITY;
    let (ma, mb) = (
        khem_core::ELEMENTS[a_el.0 as usize].mass,
        khem_core::ELEMENTS[b_el.0 as usize].mass,
    );
    let ke_launch = 0.5 * (ma + mb) * v * v;
    let mut in_zone = false;
    let mut latched = false;
    let mut crossed = false;
    let mut ke_exit = 0.0f32;
    for _ in 0..ticks {
        for step in 0..config.integration_substeps {
            if step > 0 {
                world.spatial_index.rebuild(&world.atoms);
            }
            sys.update_velocities(&mut world);
            sys.update_positions(&mut world);
            if latched {
                continue;
            }
            let (ax, ay, az) = {
                let a = world.atom(a);
                (a.x, a.y, a.z)
            };
            let (bx, by, bz) = {
                let b = world.atom(b);
                (b.x, b.y, b.z)
            };
            let (dx, dy, dz) = world.delta(ax, ay, az, bx, by, bz);
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            if r < min_r {
                min_r = r;
            }
            if r < zone {
                in_zone = true;
            } else if in_zone {
                // The encounter is over: latch the verdict and
                // stop sampling (post-encounter flight is steady
                // and Wrap-folded distances must not be read).
                crossed = dx * ux + dy * uy + dz * uz < 0.0;
                let (va, vb) = (world.atom(a), world.atom(b));
                ke_exit = 0.5
                    * (ma * (va.vx * va.vx + va.vy * va.vy + va.vz * va.vz)
                        + mb * (vb.vx * vb.vx + vb.vy * vb.vy + vb.vz * vb.vz));
                latched = true;
            }
        }
        sys.apply_boundary(&mut world);
    }
    assert!(
        latched,
        "K2.1 scatter: the encounter never exited the latch zone \
         (budget {ticks} ticks too short for v_c {v_c:.3})"
    );
    Sample {
        min_ratio: min_r / r_sum,
        mint: ke_exit / ke_launch,
        crossed,
    }
}

/// The four element classes phase A spans: worst tunneling case
/// (H-H: lightest pair, narrowest cutoff), the polar pair, the
/// water-water pair, and the heavy salt pair.
const CLASSES: [(&str, ElementId, ElementId); 4] = [
    ("H-H", ElementId(0), ElementId(0)),
    ("H-O", ElementId(0), ElementId(3)),
    ("O-O", ElementId(3), ElementId(3)),
    ("Na-Cl", ElementId(8), ElementId(9)),
];

/// Closing-speed multiples of the pair's thermal relative scale
/// (module doc): 1 = a common collision, 2 = a fast one, 3 = the
/// p99 tail.
const SPEED_BINS: [f32; 3] = [1.0, 2.0, 3.0];

/// Impact-parameter fractions of the pair's cutoff: 0 = head-on
/// (all relative KE on the line of centers - the wall's binding
/// case), 0.4 = a grazing flyby (the deflection and mint case).
const IMPACT_BINS: [f32; 2] = [0.0, 0.4];

/// The operating temperature of every bin: the pond's setpoint,
/// the K1 operating point.
const T_OPERATING: f32 = 35.0;

/// Runs phase A once for one config: returns (per-bin min ratio,
/// worst mint, crossed count) plus a printed evidence table.
fn phase_a(config: PhysicsConfig, label: &str) -> (f32, f32, u32) {
    eprintln!(
        "K2.1 phase A [{label}]: non_bonded_repulsion {}",
        config.non_bonded_repulsion
    );
    let mut worst_ratio = f32::INFINITY;
    let mut worst_mint = 0.0f32;
    let mut crossed_count = 0u32;
    for (class_idx, &(name, a_el, b_el)) in CLASSES.iter().enumerate() {
        let r_sum = khem_core::ELEMENTS[a_el.0 as usize].radius
            + khem_core::ELEMENTS[b_el.0 as usize].radius;
        let sigma = sigma_rel(&config, a_el, b_el, T_OPERATING);
        for (c_idx, &c) in SPEED_BINS.iter().enumerate() {
            let v_c = c * sigma;
            for (imp_idx, &imp_frac) in IMPACT_BINS.iter().enumerate() {
                let imp = imp_frac * r_sum * config.non_bonded_margin;
                let mut bin_min = f32::INFINITY;
                let mut bin_mint = 0.0f32;
                let mut bin_crossed = 0u32;
                for k in 0..8u64 {
                    let seed = 1_000_000
                        + class_idx as u64 * 10_000
                        + c_idx as u64 * 1_000
                        + imp_idx as u64 * 100
                        + k;
                    let s = scatter(config, a_el, b_el, v_c, imp, seed);
                    if s.min_ratio < bin_min {
                        bin_min = s.min_ratio;
                    }
                    if s.mint > bin_mint {
                        bin_mint = s.mint;
                    }
                    if s.crossed {
                        bin_crossed += 1;
                    }
                }
                eprintln!(
                    "  {name} c={c:.0} (v_c {v_c:.3} A/tick) imp={imp_frac:.1}: \
                     min ratio {bin_min:.3} (bar 0.8), worst mint {bin_mint:.3} \
                     (bar 1.10), crossed {bin_crossed}/8"
                );
                worst_ratio = worst_ratio.min(bin_min);
                worst_mint = worst_mint.max(bin_mint);
                crossed_count += bin_crossed;
            }
        }
    }
    (worst_ratio, worst_mint, crossed_count)
}

/// All atom state finite (no NaN/inf anywhere).
fn all_state_finite(world: &WorldState) -> bool {
    world.atoms.iter().all(|a| {
        a.x.is_finite()
            && a.y.is_finite()
            && a.z.is_finite()
            && a.vx.is_finite()
            && a.vy.is_finite()
            && a.vz.is_finite()
    })
}

// ---- Always-on smoke ---------------------------------------------------

#[test]
fn k2_scatter_smoke() {
    // A head-on H-H pass at twice the thermal scale, at the
    // shipped defaults: finite, no coincidence, no pass-through,
    // no large mint. Cheap regression; the 0.8 bar is the gate's.
    let config = PhysicsConfig::default();
    let sigma = sigma_rel(&config, ElementId(0), ElementId(0), T_OPERATING);
    let s = scatter(config, ElementId(0), ElementId(0), 2.0 * sigma, 0.0, 7);
    assert!(
        s.min_ratio.is_finite() && s.min_ratio > 0.25,
        "K2 smoke: min ratio {} not a sane scattering pass",
        s.min_ratio
    );
    assert!(!s.crossed, "K2 smoke: the pair passed through");
    assert!(s.mint <= 1.5, "K2 smoke: mint {} too large", s.mint);
}

// ---- Gate K2.1: excluded volume ---------------------------------------

/// K2.1 EXCLUDED VOLUME (gate ladder; finding F4's fix, spec
/// 6.6): free atoms no longer pass through each other - the
/// substrate has excluded volume because matter does, documented
/// as smuggled physics. Implemented with the K1.1 substrate work;
/// THIS is the gate's own measurement (the ladder's rule: the
/// PASS run lands as the gate's commit, with harness evidence).
///
/// PASSED 2026-11-04 (see the module doc for the full evidence
/// and the retune story - the gate measured the strength-1 core
/// failing its own letter first, finding F22).
///
/// Bars (the gate's letter, the harness's numbers):
/// - phase A: every launched pair's minimum approach >=
///   0.8 * (r_a + r_b); zero pass-throughs; worst mint <= 1.10.
/// - phase B: over the 10k-tick operating soak, no unbonded
///   pair samples below 0.8 * (r_a + r_b); the margin and the
///   near-miss census are reported as evidence.
///
/// The strength lever and the K1 re-runs its retune forces are
/// recorded in the PASS record (docs/plans/phase-1-kernel.md,
/// K2.1); the sweep probe that picked the value is
/// `k2_1_strength_sweep` below.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k2_1_excluded_volume() {
    let config = PhysicsConfig::default();

    // ---- Phase A: controlled scattering -----------------------------
    let (worst_ratio, worst_mint, crossed) = phase_a(config, "gate");
    assert!(
        worst_ratio >= 0.8,
        "K2.1 FAIL: phase A minimum approach ratio {worst_ratio:.3} < 0.8"
    );
    assert_eq!(crossed, 0, "K2.1 FAIL: {crossed} pairs passed through");
    assert!(
        worst_mint <= 1.10,
        "K2.1 FAIL: phase A worst mint {worst_mint:.3} > 1.10"
    );
    eprintln!(
        "K2.1 phase A PASS: worst min ratio {worst_ratio:.3} (bar 0.8), \
         worst mint {worst_mint:.3} (bar 1.10), zero pass-throughs"
    );

    // ---- Phase B: the operating soak ---------------------------------
    let seed = 42u64;
    let mut world = WorldState::new(30.0, 30.0, 30.0, BoundaryType::Wrap, seed, config);
    world.temp_field.data.fill(T_OPERATING);
    world.setpoint_field.data.fill(T_OPERATING);
    let mix = [("H", 200u32), ("C", 160), ("N", 160), ("O", 240)];
    // The pond's own sprinkle margin (the K1.5 beaker note): no
    // pair is born inside another's exclusion zone - a uniform
    // spawn CAN land two atoms 0.5 A apart (the first phase B run
    // measured 8 such pair-ticks below the bar, all in the first
    // tick window, all spawn artifacts the wall cleaned up in a
    // tick or two), and initial conditions are matter, not a
    // transient the census must forgive. 2 A clears every
    // element pair's 0.8 * (r_a + r_b) bar in the mix (max
    // C-C: 1.23 A) with margin.
    let spawn_margin = 2.0f32;
    for (symbol, count) in mix {
        let el = khem_core::elements::element_id(symbol).expect("element in table");
        for _ in 0..count {
            loop {
                let x = world.rng.f01() as f32 * world.width;
                let y = world.rng.f01() as f32 * world.height;
                let z = world.rng.f01() as f32 * world.depth;
                let clear = world.atoms.iter().all(|a| {
                    let (dx, dy, dz) = world.delta(a.x, a.y, a.z, x, y, z);
                    dx * dx + dy * dy + dz * dz >= spawn_margin * spawn_margin
                });
                if clear {
                    world.spawn_atom(el, x, y, z);
                    break;
                }
            }
        }
    }
    let observer = Observer::new(ObserverConfig {
        khem_version: "0.1.0",
        run_name: "k2_harness".to_string(),
        world_name: "excluded_volume_soak".to_string(),
        seed,
        tick_interval: 10_000,
    });
    let mut sim = Sim::new(config, observer);
    let _ = sim.start(&world);
    eprintln!(
        "K2.1 phase B: 760 free atoms, 30x30x30 Wrap, 35 C, 10k ticks, \
         chemistry live"
    );

    let census_radius = 4.0f32;
    let mut min_ratio = f32::INFINITY;
    let mut violations = 0u64;
    let mut near_80_90 = 0u64;
    let mut near_90_100 = 0u64;
    let mut near_100_125 = 0u64;
    let mut formations = 0u64;
    let mut breaks = 0u64;
    for t in 1..=10_000u64 {
        let events = sim.tick(&mut world);
        for event in &events {
            match event {
                khem_core::observer::Event::BondFormed { .. } => formations += 1,
                khem_core::observer::Event::BondBroken { .. } => breaks += 1,
                _ => {}
            }
        }
        // Census on the post-tick state (the index is current -
        // sim rebuilt it after chemistry). Pairs counted once by
        // id order; bonded pairs are the spring's business.
        for atom in &world.atoms {
            if !atom.alive {
                continue;
            }
            let near = world
                .spatial_index
                .neighbors(atom.x, atom.y, atom.z, census_radius);
            for b_id in near {
                if b_id.0 <= atom.id.0 {
                    continue;
                }
                let b = world.atom(b_id);
                if !b.alive || world.is_bonded(atom.id, b_id) {
                    continue;
                }
                let (dx, dy, dz) = world.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                let r_sum = world.element(atom.element).radius + world.element(b.element).radius;
                let ratio = r / r_sum;
                if ratio < min_ratio {
                    min_ratio = ratio;
                }
                if ratio < 0.8 {
                    violations += 1;
                } else if ratio < 0.9 {
                    near_80_90 += 1;
                } else if ratio < 1.0 {
                    near_90_100 += 1;
                } else if ratio < 1.25 {
                    near_100_125 += 1;
                }
            }
        }
        if t.is_multiple_of(1000) {
            eprintln!(
                "  t={t}: min ratio so far {min_ratio:.3}, violations {violations}, \
                 formations {formations}, breaks {breaks}"
            );
        }
    }
    assert!(
        all_state_finite(&world),
        "K2.1 FAIL: non-finite atom state in the soak"
    );
    eprintln!(
        "K2.1 phase B: min unbonded approach ratio {min_ratio:.3} (bar 0.8), \
         pair-ticks [0.8,0.9) {near_80_90}, [0.9,1.0) {near_90_100}, \
         [1.0,1.25) {near_100_125}, violations {violations}; \
         formations {formations}, breaks {breaks}"
    );
    assert!(
        violations == 0,
        "K2.1 FAIL: {violations} unbonded pair-ticks below 0.8 * (r_a + r_b) \
         in the operating soak (min ratio {min_ratio:.3})"
    );
    assert!(
        min_ratio.is_finite(),
        "K2.1 FAIL: no unbonded close approach ever sampled (census bug)"
    );
    eprintln!("K2.1 PASS: excluded volume holds at the operating point");
}

// ---- The lever probe ---------------------------------------------------

/// The strength sweep that picks non_bonded_repulsion's default:
/// phase A's head-on bins (the binding case) across
/// {1, 4, 8, 16, 32}, with the symplectic bound column
/// dt_sub * sqrt(S / mu) per class (must stay < 2; the pond's
/// spring bound sits at 1.375). Not a gate - the evidence tool
/// whose chosen value, and the K1 re-runs it forces, live in the
/// K2.1 PASS record. Grazing bins are omitted for table width:
/// they deflect early and never bind the decision.
#[test]
#[ignore] // explicit: cargo test --release -- --ignored --nocapture
fn k2_1_strength_sweep() {
    for strength in [1.0f32, 4.0, 8.0, 16.0, 32.0] {
        let config = PhysicsConfig {
            non_bonded_repulsion: strength,
            ..PhysicsConfig::default()
        };
        let bound = |a: ElementId, b: ElementId| -> f32 {
            let (ma, mb) = (
                khem_core::ELEMENTS[a.0 as usize].mass,
                khem_core::ELEMENTS[b.0 as usize].mass,
            );
            let mu = ma * mb / (ma + mb);
            (1.0 / config.integration_substeps as f32) * (strength / mu).sqrt()
        };
        eprintln!(
            "K2.1 sweep S={strength}: symplectic bound per class: {}",
            CLASSES
                .iter()
                .map(|&(name, a, b)| format!("{name} {:.2}", bound(a, b)))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for (class_idx, &(name, a_el, b_el)) in CLASSES.iter().enumerate() {
            let sigma = sigma_rel(&config, a_el, b_el, T_OPERATING);
            for (c_idx, &c) in SPEED_BINS.iter().enumerate() {
                let v_c = c * sigma;
                let mut bin_min = f32::INFINITY;
                let mut bin_mint = 0.0f32;
                let mut bin_crossed = 0u32;
                for k in 0..8u64 {
                    let seed = 1_000_000 + class_idx as u64 * 10_000 + c_idx as u64 * 1_000 + k;
                    let s = scatter(config, a_el, b_el, v_c, 0.0, seed);
                    if s.min_ratio < bin_min {
                        bin_min = s.min_ratio;
                    }
                    if s.mint > bin_mint {
                        bin_mint = s.mint;
                    }
                    if s.crossed {
                        bin_crossed += 1;
                    }
                }
                eprintln!(
                    "  {name} c={c:.0}: min ratio {bin_min:.3}, worst mint {bin_mint:.3}, \
                     crossed {bin_crossed}/8"
                );
            }
        }
    }
}
