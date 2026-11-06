//! The conservation tier: scenario properties that must hold on
//! ANY world the engine runs - atom bookkeeping (G04), determinism
//! by construction (G02), and the energy ledger's bounded drift in
//! a closed system. Always on (seconds, not minutes): the tier
//! between the law tests (in-crate, analytic) and the pond gates
//! (#[ignore]d, CI).
//!
//! These are PROPERTIES, not gates: no constants are tuned here,
//! and a failure means an engine bug, not a substrate question.

mod common;

use common::observer;
use khem_engine::Sim;
use khem_forcefield::ForceField;
use khem_forcefield::element_id;
use khem_world::WorldState;

/// A small mixed-atom beaker: free H/C/O/Na/Cl in a 30 A cube,
/// dense enough for chemistry, small enough for a fast run.
fn beaker(seed: u64, config: ForceField) -> WorldState {
    let mut w = WorldState::new(
        30.0,
        30.0,
        30.0,
        khem_world::BoundaryType::Wrap,
        seed,
        config,
    );
    let counts = [("H", 26), ("C", 8), ("O", 10), ("Na", 3), ("Cl", 3)];
    let mut rng = khem_world::Rng::new(seed);
    for (sym, n) in counts {
        let el = element_id(sym).expect("element in table");
        for _ in 0..n {
            loop {
                let x = rng.f01() as f32 * 28.0 + 1.0;
                let y = rng.f01() as f32 * 28.0 + 1.0;
                let z = rng.f01() as f32 * 28.0 + 1.0;
                // keep-apart at the true distance: neighbors() is a
                // cell-level gather, so measure each candidate (the
                // pond's own keep-apart pattern).
                let near = w.spatial_index.neighbors(x, y, z, 1.6);
                let clear = near.iter().all(|&id| {
                    let b = w.atom(id);
                    let (dx, dy, dz) = w.delta(x, y, z, b.x, b.y, b.z);
                    dx * dx + dy * dy + dz * dz > 1.6 * 1.6
                });
                if clear {
                    w.spawn_atom(el, x, y, z);
                    w.spatial_index.rebuild(&w.atoms);
                    break;
                }
            }
        }
    }
    w
}

/// Every bond record agrees with both atoms' slot lists (G04),
/// every slot's bond exists and is alive, and the live count is
/// invariant - after 500 ticks of active chemistry.
#[test]
fn atom_bookkeeping_stays_consistent() {
    let config = ForceField::default();
    let mut w = beaker(11, config);
    let n0 = w.live_atom_count();
    let mut sim = Sim::new(config, observer(11, 10_000));
    let _ = sim.start(&w);
    for _ in 0..500 {
        sim.tick(&mut w);
    }
    assert_eq!(w.live_atom_count(), n0, "nothing spawns or dies");
    // IDs are dense and complete.
    for (i, atom) in w.atoms.iter().enumerate() {
        assert_eq!(atom.id.0 as usize, i, "atom id is its index");
    }
    // Bond slots <-> bond records agree both ways.
    for bond in &w.bonds {
        if !bond.alive {
            continue;
        }
        let a = w.atom(bond.atom_a);
        let b = w.atom(bond.atom_b);
        assert!(
            a.bond_ids().any(|x| x == bond.id),
            "bond {} missing from atom {}'s slots",
            bond.id.0,
            bond.atom_a.0
        );
        assert!(
            b.bond_ids().any(|x| x == bond.id),
            "bond {} missing from atom {}'s slots",
            bond.id.0,
            bond.atom_b.0
        );
    }
    for atom in w.atoms.iter().filter(|a| a.alive) {
        for slot in atom.bond_ids() {
            let bond = w.bond(slot);
            assert!(bond.alive, "dangling slot to a dead bond");
            assert!(
                bond.atom_a == atom.id || bond.atom_b == atom.id,
                "slot points at a bond this atom is not in"
            );
        }
        assert_eq!(
            atom.bond_ids().count(),
            atom.bond_count as usize,
            "bond_count disagrees with the slot list"
        );
    }
}

/// Same seed, same stream: two runs of the same beaker are
/// byte-identical after warm ticks; a different seed is not.
/// (G02, the guarantee behind the golden hash - checked here on
/// a DIFFERENT world so the property, not the pond's constants,
/// is what's pinned.)
#[test]
fn same_seed_runs_are_byte_identical() {
    fn fnv1a(world: &WorldState) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for atom in &world.atoms {
            for v in [
                atom.x.to_bits() as u64,
                atom.y.to_bits() as u64,
                atom.z.to_bits() as u64,
                atom.vx.to_bits() as u64,
                atom.vy.to_bits() as u64,
                atom.vz.to_bits() as u64,
                atom.bond_count as u64,
                u64::from(atom.alive),
                atom.element.0 as u64,
            ] {
                h ^= v;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for bond in &world.bonds {
            for v in [
                bond.atom_a.0 as u64,
                bond.atom_b.0 as u64,
                bond.order as u64,
                u64::from(bond.alive),
            ] {
                h ^= v;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
    let run = |seed: u64| -> u64 {
        let config = ForceField::default();
        let mut w = beaker(seed, config);
        let mut sim = Sim::new(config, observer(seed, 10_000));
        let _ = sim.start(&w);
        for _ in 0..100 {
            sim.tick(&mut w);
        }
        fnv1a(&w)
    };
    assert_eq!(run(11), run(11), "same seed must be byte-identical");
    assert_ne!(run(11), run(12), "different seeds differ");
}

/// No energy mint in a warm beaker: with the default thermostat
/// running against a declared 35 C setpoint (the K1.4 gate's own
/// operating regime, shrunk), the energy ledger must be FLAT over
/// the steady tail - no random-walk heating (the F13/F19 class of
/// mint), no furnace (F8), no drain (the reservoir leak). The
/// ledger counts what the engine's own conservation laws cover:
/// kinetic energy, spring potential, and the temperature field
/// plus the break-heat reservoir. Flatness is asserted against a
/// generous band (the K1.1 gate owns the pond's tight band; this
/// is the always-on smoke of the same property on a small world).
#[test]
fn warm_beaker_energy_is_flat() {
    let config = ForceField::default();
    let mut w = beaker(21, config);
    // Declare the 35 C environment (the phase-1 sentinel: no
    // setpoint = no relaxation; the beaker must declare its bath).
    w.temp_field.data.fill(35.0);
    w.setpoint_field.data.fill(35.0);
    let mut sim = Sim::new(config, observer(21, 10_000));
    let _ = sim.start(&w);

    let ledger = |w: &WorldState| -> f64 {
        let mut ke = 0.0f64;
        for atom in w.atoms.iter().filter(|a| a.alive) {
            ke += 0.5
                * (atom.vx * atom.vx + atom.vy * atom.vy + atom.vz * atom.vz) as f64
                * khem_forcefield::element(atom.element).mass as f64;
        }
        let mut pe = 0.0f64;
        for bond in w.bonds.iter().filter(|b| b.alive) {
            let (a, b) = (w.atom(bond.atom_a), w.atom(bond.atom_b));
            let (dx, dy, dz) = w.delta(a.x, a.y, a.z, b.x, b.y, b.z);
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            let r_eq = khem_forcefield::element(a.element).radius
                + khem_forcefield::element(b.element).radius;
            let k = bond.energy * config.bonds.spring_energy_scale;
            pe += 0.5 * (k as f64) * (r as f64 - r_eq as f64).powi(2);
        }
        // The excluded-volume wall's own potential (the soft core
        // stores 0.5*S*(cutoff-r)^2 per unbonded overlapping pair;
        // pairs enter and leave cutoffs constantly, so omitting it
        // shows as fake drift).
        let mut wall = 0.0f64;
        let margin = config.excluded_volume.non_bonded_margin;
        let strength = config.excluded_volume.non_bonded_repulsion;
        let gather = margin
            * 2.0
            * khem_forcefield::ELEMENTS
                .iter()
                .map(|e| e.radius)
                .fold(f32::MIN, f32::max);
        for (i, atom) in w.atoms.iter().enumerate() {
            if !atom.alive {
                continue;
            }
            let near = w.spatial_index.neighbors(atom.x, atom.y, atom.z, gather);
            for id in near {
                if id.0 as usize <= i || !w.atom(id).alive {
                    continue;
                }
                let b = w.atom(id);
                if w.is_bonded(atom.id, id) {
                    continue; // bonded: the spring owns this pair
                }
                let cutoff = margin
                    * (khem_forcefield::element(atom.element).radius
                        + khem_forcefield::element(b.element).radius);
                let (dx, dy, dz) = w.delta(atom.x, atom.y, atom.z, b.x, b.y, b.z);
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                if r < cutoff {
                    wall += 0.5 * (strength as f64) * (cutoff as f64 - r as f64).powi(2);
                }
            }
        }
        let mut field = 0.0f64;
        for cell in w.temp_field.data.iter().chain(w.release_field.data.iter()) {
            field += *cell as f64;
        }
        ke + pe + wall + field
    };

    // Settle past the construction transient (free-atom
    // construction, initial overlaps relaxing), then compare two
    // windows of the steady tail.
    for _ in 0..1000 {
        sim.tick(&mut w);
    }
    let tail_a = ledger(&w);
    for _ in 0..1000 {
        sim.tick(&mut w);
    }
    let tail_b = ledger(&w);

    // The setpoint reservoir is honest by design (K1.1's letter);
    // this smoke's band catches MINTS, not thermal noise. The
    // known non-conservative channel by design is the mean-field
    // pressure coupling (spec 6.4: the pressure field is rebuilt
    // from density each tick - an environment force, not a pair
    // potential); everything else in the ledger conserves. The
    // band: 5% of the field's own scale (35 C x its cells).
    let scale = 35.0 * w.temp_field.data.len() as f64;
    let drift = (tail_b - tail_a).abs();
    assert!(
        drift < scale * 0.05,
        "warm-beaker ledger drifted {drift:.3} between the tail windows \
         (scale {scale:.0}) - an energy mint or drain, not noise"
    );
}
