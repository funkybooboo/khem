//! The chemistry system: bond breaking and formation by
//! Boltzmann/Arrhenius probabilities, the real bond-energy lookup
//! table, VSEPR geometry factors, valence bookkeeping.
//!
//! Phase-1 placement decisions, to fold into the spec at revision
//! (ADR-0006: specs are drafts until validated):
//!
//! - UV bond breaking (spec 8.2) executes inside
//!   [`ChemistrySystem::break_bonds`] alongside the thermal Boltzmann
//!   roll, as one combined probability; the tick order (5.1) gives
//!   chemistry the only bond-mutating steps, and one place that
//!   breaks bonds keeps the RNG discipline legible.
//! - The bond-energy table (spec 7.3) and angle logic (spec 7.4) are
//!   hardcoded here; the spec puts them in physics.cfg, which phase 3
//!   loads. Unknown-pair fallback: geometric mean of the elements'
//!   single-bond reference energies; unknown order for a known pair:
//!   single-bond energy times order (v0 fallbacks).
//! - The 3D VSEPR angles (109.5 tetrahedral and friends) cannot all
//!   exist in 2D; they are used as scoring ideals only - effective
//!   geometry emerges from the gaussian competition. Water's bent
//!   104.5 and the 120/180 orders matter most; they fit 2D.
//! - Formation chooses order 2 when both atoms have 2+ free slots,
//!   else order 1 (spec 7.2). Triples are never formed in v0; they
//!   can only exist if seeded into the initial world.
//! - Events pushed here carry full schema payloads (spec 3.3); the
//!   run declaration's bond_events output flag (language spec) is
//!   an observer/bin concern, applied when the stream is written.
//!
//! RNG discipline (ADR-0005): after the physics system's draws,
//! break_bonds consumes exactly one uniform per LIVE bond per tick,
//! in BondId order; form_bonds consumes exactly one uniform per
//! ELIGIBLE pair, in iterating-AtomId order, candidates in spatial
//! scan order. Ineligible bonds and pairs draw nothing. Tests that
//! call form_bonds must rebuild the spatial index first (G07: the
//! tick loop rebuilds it between position updates and chemistry).
//!
//! Spec: docs/specs/runtime-spec.md, section 7 (Chemistry System).

use crate::config::PhysicsConfig;
use crate::observer::Event;
use crate::world::{AtomId, BondId, ElementId, WorldState};

/// The chemistry system interface, decomposed per the tick order
/// (runtime spec 5.1: break_bonds, form_bonds). v0.1 compiles exactly
/// one implementation; the trait exists so future plugin loading
/// does not require restructuring (runtime spec 10.4).
pub trait ChemistrySystem {
    /// Thermal Boltzmann breaking (spec 7.1) and UV breaking (spec
    /// 8.2, reading the UV field the energy system set this tick).
    fn break_bonds(&mut self, world: &mut WorldState);
    /// Probabilistic bond formation (spec 7.2).
    fn form_bonds(&mut self, world: &mut WorldState);
}

/// Bond energies, kJ/mol (spec 7.3). Rows are (a, b, order, energy)
/// with a <= b. Element indices: H=0 C=1 N=2 O=3 P=4 S=5 Si=6 Fe=7
/// Na=8 Cl=9 (the canonical ELEMENTS order). Fe-O is ~390, variable
/// in reality (spec notes this).
const BOND_ENERGY_TABLE: [(u8, u8, u8, f32); 29] = [
    (0, 0, 1, 436.0), // H-H
    (0, 1, 1, 413.0), // H-C
    (0, 2, 1, 391.0), // H-N
    (0, 3, 1, 463.0), // H-O
    (0, 5, 1, 363.0), // H-S
    (1, 1, 1, 346.0), // C-C
    (1, 1, 2, 614.0), // C=C
    (1, 1, 3, 839.0), // C#C
    (1, 2, 1, 305.0), // C-N
    (1, 2, 2, 615.0), // C=N
    (1, 2, 3, 891.0), // C#N
    (1, 3, 1, 358.0), // C-O
    (1, 3, 2, 799.0), // C=O
    (1, 4, 1, 264.0), // C-P
    (1, 5, 1, 272.0), // C-S
    (2, 2, 1, 163.0), // N-N
    (2, 2, 2, 418.0), // N=N
    (2, 2, 3, 945.0), // N#N
    (2, 3, 1, 201.0), // N-O
    (2, 3, 2, 607.0), // N=O
    (3, 3, 1, 146.0), // O-O
    (3, 3, 2, 498.0), // O=O
    (3, 4, 1, 335.0), // O-P
    (3, 5, 1, 265.0), // O-S
    (3, 6, 1, 452.0), // Si-O
    (3, 7, 1, 390.0), // Fe-O (approximate)
    (4, 4, 1, 201.0), // P-P
    (5, 5, 1, 266.0), // S-S
    (6, 6, 1, 222.0), // Si-Si
];

/// Per-element single-bond reference energy, kJ/mol, for the
/// unknown-pair geometric-mean fallback: the element's own single
/// bond where the table has one (N-N 163 is genuinely weak; note the
/// fallback makes unknown N pairs weak too - a documented v0
/// behavior), standard values for Na and Cl, which have no table
/// entries.
const SINGLE_REFERENCE: [f32; 10] = [
    436.0, // H
    346.0, // C
    163.0, // N
    146.0, // O
    201.0, // P
    266.0, // S
    222.0, // Si
    390.0, // Fe
    77.0,  // Na
    243.0, // Cl
];

/// Bond energy, kJ/mol (spec 7.3 with the fallback rules above).
pub fn bond_energy(a: ElementId, b: ElementId, order: u8) -> f32 {
    let (lo, hi) = if a.0 <= b.0 { (a.0, b.0) } else { (b.0, a.0) };
    for (x, y, o, e) in BOND_ENERGY_TABLE {
        if x == lo && y == hi && o == order {
            return e;
        }
    }
    // Unknown (pair, order): known single scaled by order, else the
    // geometric mean of the elements' references, scaled by order.
    let single = if lo == hi {
        SINGLE_REFERENCE[lo as usize]
    } else if let Some(e) = table_single(lo, hi) {
        e
    } else {
        (SINGLE_REFERENCE[lo as usize] * SINGLE_REFERENCE[hi as usize]).sqrt()
    };
    single * (order.max(1) as f32)
}

fn table_single(lo: u8, hi: u8) -> Option<f32> {
    for (x, y, o, e) in BOND_ENERGY_TABLE {
        if x == lo && y == hi && o == 1 {
            return Some(e);
        }
    }
    None
}

/// Angular distance between two directions, radians, in [0, pi].
fn angdist(x: f32, y: f32) -> f32 {
    let d = (x - y).rem_euclid(std::f32::consts::TAU);
    d.min(std::f32::consts::TAU - d)
}

/// The ideal angle between an existing bond and a new candidate bond
/// for this atom, degrees (spec 7.4). None means unconstrained (first
/// bond, or H/Na/Cl which cannot coordinate).
///
/// The 3D-to-2D tension is real (four bonds cannot be 109.5 apart in
/// 2D) and accepted: these are scoring ideals, not enforced angles.
fn ideal_angle(world: &WorldState, a: AtomId, candidate_order: u8) -> Option<f32> {
    let atom = world.atom(a);
    if atom.bond_count == 0 {
        return None;
    }
    // Existing double bonds shift carbon and nitrogen geometry.
    let doubles = atom.bonds[..atom.bond_count as usize]
        .iter()
        .flatten()
        .filter(|id| world.bond(**id).order >= 2)
        .count() as u8;
    let has_double = doubles > 0 || candidate_order >= 2;
    match world.element(atom.element).atomic_number {
        1 | 11 | 17 => None, // H, Na, Cl
        6 => Some(if doubles + u8::from(candidate_order >= 2) >= 2 {
            180.0 // two double bonds: linear
        } else if has_double {
            120.0 // trigonal planar
        } else {
            109.5 // tetrahedral (2D: scored, not enforced)
        }),
        7 => Some(if has_double { 120.0 } else { 107.0 }),
        8 => Some(104.5), // bent water geometry, the one that matters most
        15 => Some(if atom.bond_count >= 4 { 90.0 } else { 109.5 }),
        16 => Some(103.0),
        14 => Some(109.5),
        26 => Some(90.0),
        _ => None,
    }
}

/// The v0.1 chemistry implementation (runtime spec 10.4: exactly one).
pub struct Chemistry {
    config: PhysicsConfig,
}

impl Chemistry {
    pub fn new(config: PhysicsConfig) -> Self {
        Self { config }
    }

    /// Spec 7.2 geometry factor, 0.0-1.0: how well the candidate
    /// direction fits the atom's VSEPR ideals. Unconstrained atoms
    /// score 1.0. Otherwise the best-scoring existing bond anchors
    /// the ideal (on either side of it), scored by a gaussian in
    /// angular deviation with sigma = geometry_sigma.
    pub fn geometry_factor(
        &self,
        world: &WorldState,
        a: AtomId,
        candidate_angle: f32,
        candidate_order: u8,
    ) -> f32 {
        let Some(ideal) = ideal_angle(world, a, candidate_order) else {
            return 1.0;
        };
        let ideal = ideal.to_radians();
        let sigma = self.config.geometry_sigma.to_radians();
        let atom = world.atom(a);
        let (ax, ay) = (atom.x, atom.y);
        let mut best = 0.0f32;
        for id in atom.bonds[..atom.bond_count as usize].iter().flatten() {
            let bond = world.bond(*id);
            let other = if bond.atom_a == a {
                bond.atom_b
            } else {
                bond.atom_a
            };
            let o = world.atom(other);
            let theta = (o.y - ay).atan2(o.x - ax);
            let delta = angdist(candidate_angle, theta + ideal)
                .min(angdist(candidate_angle, theta - ideal));
            let score = (-delta * delta / (2.0 * sigma * sigma)).exp();
            best = best.max(score);
        }
        best
    }
}

impl ChemistrySystem for Chemistry {
    fn break_bonds(&mut self, world: &mut WorldState) {
        let n = world.bonds.len();
        for i in 0..n {
            let (alive, a, b, order, energy) = {
                let bond = &world.bonds[i];
                (
                    bond.alive,
                    bond.atom_a,
                    bond.atom_b,
                    bond.order,
                    bond.energy,
                )
            };
            if !alive {
                continue;
            }
            let (mx, my) = (
                (world.atom(a).x + world.atom(b).x) * 0.5,
                (world.atom(a).y + world.atom(b).y) * 0.5,
            );
            // Spec 7.1. Non-positive temperatures give p_break 0
            // (clamped field reads; exp(-inf) guard).
            let t = world.temp_field.get(mx, my).max(0.0);
            let p_break = if t > 0.0 {
                (-(energy / (self.config.kb_scaled * t))).exp()
            } else {
                0.0
            };
            // Spec 8.2, folded in (module doc).
            let p_uv = world.uv_field.get(mx, my) * self.config.uv_sensitivity.of_order(order);
            let p = 1.0 - (1.0 - p_break) * (1.0 - p_uv);
            if world.rng.f01() < p as f64 && world.break_bond(BondId(i as u32)) {
                // Spec 7.1: release stored heat into the field. The
                // field may go negative where formation absorbed more
                // than was there; thermal kicks clamp at zero and
                // diffusion smooths (documented v0 behavior).
                let released = energy * self.config.release_fraction;
                world.temp_field.add(mx, my, released);
                world.event_queue.push(Event::BondBroken {
                    tick: world.tick,
                    bond_id: i as u32,
                    elem_a: world.atom(a).element,
                    elem_b: world.atom(b).element,
                    energy_released: released,
                    x: mx,
                    y: my,
                });
            }
        }
    }

    fn form_bonds(&mut self, world: &mut WorldState) {
        let radius = self.config.bond_search_radius;
        let radius2 = radius * radius;
        let n = world.atoms.len();
        for i in 0..n {
            let (a_id, a_el, ax, ay, a_alive) = {
                let a = &world.atoms[i];
                (a.id, a.element, a.x, a.y, a.alive)
            };
            if !a_alive {
                continue;
            }
            // Candidates come from the spatial index the tick loop
            // rebuilt after position updates (G07).
            let candidates = world.spatial_index.neighbors(ax, ay, radius);
            for b_id in candidates {
                // Each unordered pair is attempted exactly once: from
                // the iteration of the lower AtomId. The iterating
                // atom is the geometry anchor (documented asymmetry).
                if b_id.0 <= a_id.0 {
                    continue;
                }
                // Capacity may have changed by a bond formed below.
                let a_count = world.atom(a_id).bond_count;
                let a_max = world.element(a_el).max_bonds;
                if a_count >= a_max {
                    break;
                }
                let (b_el, bx, by, b_alive) = {
                    let b = world.atom(b_id);
                    (b.element, b.x, b.y, b.alive)
                };
                let b_max = world.element(b_el).max_bonds;
                let b_count = world.atom(b_id).bond_count;
                if !b_alive || b_count >= b_max {
                    continue;
                }
                let (dx, dy) = (bx - ax, by - ay);
                if dx * dx + dy * dy > radius2 {
                    continue;
                }
                if world.is_bonded(a_id, b_id) {
                    continue;
                }
                // Spec 7.2: double when both have 2+ free slots.
                let order: u8 = if a_max - a_count >= 2 && b_max - b_count >= 2 {
                    2
                } else {
                    1
                };
                let energy = bond_energy(a_el, b_el, order);
                // Temperature factor: gaussian around the pair's
                // optimal temperature (module doc: v0 fill).
                let t = world
                    .temp_field
                    .get((ax + bx) * 0.5, (ay + by) * 0.5)
                    .max(0.0);
                let t_opt = self.config.t_opt_scale * energy;
                let dt = t - t_opt;
                let t_factor = (-dt * dt / (2.0 * self.config.t_width * self.config.t_width)).exp();
                let en = (world.element(a_el).electronegativity
                    - world.element(b_el).electronegativity)
                    .abs();
                let p = self.config.base_formation_rate
                    * self.geometry_factor(world, a_id, dy.atan2(dx), order)
                    * t_factor
                    * (1.0 + en * self.config.en_bonus);
                if world.rng.f01() < p as f64
                    && let Some(bond_id) = world.form_bond(a_id, b_id, order, energy)
                {
                    // Spec 7.2: formation absorbs heat. May go
                    // negative; see break_bonds.
                    let (mx, my) = ((ax + bx) * 0.5, (ay + by) * 0.5);
                    world
                        .temp_field
                        .add(mx, my, -energy * self.config.formation_fraction);
                    world.event_queue.push(Event::BondFormed {
                        tick: world.tick,
                        bond_id: bond_id.0,
                        atom_a: a_id,
                        atom_b: b_id,
                        elem_a: a_el,
                        elem_b: b_el,
                        order,
                        energy,
                        x: mx,
                        y: my,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::element_id;
    use crate::energy::EnergySystem;
    use crate::world::{BoundaryType, MAX_BONDS};

    fn world(seed: u64) -> WorldState {
        WorldState::new(
            100.0,
            100.0,
            BoundaryType::Wrap,
            seed,
            PhysicsConfig::default(),
        )
    }

    fn chemistry(config: PhysicsConfig) -> Chemistry {
        Chemistry::new(config)
    }

    fn rebuild_index(world: &mut WorldState) {
        world.spatial_index.rebuild(&world.atoms);
    }

    #[test]
    fn bond_energy_table_lookups() {
        let h = element_id("H").unwrap();
        let c = element_id("C").unwrap();
        let o = element_id("O").unwrap();
        let na = element_id("Na").unwrap();
        let cl = element_id("Cl").unwrap();
        // Exact entries, both argument orders.
        assert_eq!(bond_energy(h, o, 1), 463.0);
        assert_eq!(bond_energy(o, h, 1), 463.0);
        assert_eq!(bond_energy(c, o, 2), 799.0);
        assert_eq!(bond_energy(c, c, 3), 839.0);
        // Unknown pair: geometric mean of references.
        let expected = (77.0f32 * 243.0).sqrt();
        assert_eq!(bond_energy(na, cl, 1), expected);
        // Unknown order for a known pair: single * order.
        assert_eq!(bond_energy(o, o, 2), 498.0); // O=O is in the table
        assert_eq!(bond_energy(h, h, 2), 872.0);
    }

    #[test]
    fn ideal_angle_covers_the_table() {
        let mut w = world(1);
        let h = w.spawn_atom(element_id("H").unwrap(), 50.0, 50.0);
        let o = w.spawn_atom(element_id("O").unwrap(), 50.0, 50.0);
        let c = w.spawn_atom(element_id("C").unwrap(), 50.0, 50.0);
        // No existing bonds: unconstrained.
        assert_eq!(ideal_angle(&w, o, 1), None);
        // Give each a first bond (separate partners: an H can hold
        // only one bond, so the same partner cannot serve twice).
        let h_o = w.spawn_atom(element_id("H").unwrap(), 90.0, 90.0);
        let h_c = w.spawn_atom(element_id("H").unwrap(), 80.0, 90.0);
        w.form_bond(o, h_o, 1, 463.0);
        w.form_bond(c, h_c, 1, 346.0);
        assert_eq!(ideal_angle(&w, o, 1), Some(104.5));
        assert_eq!(ideal_angle(&w, c, 1), Some(109.5));
        assert_eq!(ideal_angle(&w, c, 2), Some(120.0));
        assert_eq!(ideal_angle(&w, h, 1), None); // H has no bond; None either way
    }

    #[test]
    fn geometry_factor_peaks_at_ideal_angle() {
        let mut w = world(1);
        let o = w.spawn_atom(element_id("O").unwrap(), 50.0, 50.0);
        let h1 = w.spawn_atom(element_id("H").unwrap(), 51.0, 50.0); // bond at 0 deg
        w.form_bond(o, h1, 1, 463.0);
        let chem = chemistry(PhysicsConfig::default());
        let at_ideal = chem.geometry_factor(&w, o, 104.5f32.to_radians(), 1);
        let at_opposite = chem.geometry_factor(&w, o, std::f32::consts::PI, 1);
        assert!(
            at_ideal > 0.99,
            "ideal direction should score ~1, got {at_ideal}"
        );
        assert!(
            at_opposite < 0.2,
            "opposite direction should score low, got {at_opposite}"
        );
        // Unconstrained atom (free carbon) scores 1.
        let c = w.spawn_atom(element_id("C").unwrap(), 70.0, 70.0);
        assert_eq!(chem.geometry_factor(&w, c, 0.0, 1), 1.0);
    }

    #[test]
    fn hot_weak_bonds_break_and_release_heat() {
        let mut w = world(7);
        let a = w.spawn_atom(element_id("O").unwrap(), 50.0, 50.0);
        let b = w.spawn_atom(element_id("O").unwrap(), 51.0, 50.0);
        // A pathologically weak bond in a very hot field: p_break =
        // exp(-1/(0.008314*10000)) ~ 0.988 per tick.
        w.form_bond(a, b, 1, 1.0);
        w.temp_field.set(50.0, 50.0, 10_000.0);
        let mut chem = chemistry(PhysicsConfig::default());
        for _ in 0..10 {
            chem.break_bonds(&mut w);
        }
        assert!(!w.bond(BondId(0)).alive, "weak hot bond must break");
        assert_eq!(w.atom(a).bond_count, 0);
        // Heat released at the midpoint: energy * release_fraction.
        let released = w.temp_field.get(50.5, 50.0);
        assert!((released - 10_000.5).abs() < 1.0, "released {released}");
        // One break event queued.
        assert!(matches!(
            w.event_queue.first(),
            Some(Event::BondBroken { .. })
        ));
    }

    #[test]
    fn uv_breaks_bonds_independent_of_temperature() {
        let config = PhysicsConfig {
            uv_sensitivity: crate::config::UvSensitivity {
                single: 0.5,
                ..PhysicsConfig::default().uv_sensitivity
            },
            ..PhysicsConfig::default()
        };
        let mut w = world(11);
        let a = w.spawn_atom(element_id("C").unwrap(), 50.0, 95.0);
        let b = w.spawn_atom(element_id("C").unwrap(), 51.0, 95.0);
        w.form_bond(a, b, 1, 346.0); // strong bond, frozen at T=0
        w.energy_sources
            .push(crate::energy::EnergySource::solar_uv(1.0, true));
        crate::energy::Energy::new(config).update(&mut w); // set uv field
        let mut chem = chemistry(config);
        for _ in 0..20 {
            chem.break_bonds(&mut w);
        }
        assert!(!w.bond(BondId(0)).alive, "UV alone must break the bond");
    }

    #[test]
    fn nearby_free_atoms_form_bonds() {
        // Force formation: rate 1.0, temperature exactly at the
        // pair optimum (t_opt_scale * 436).
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        let mut w = world(3);
        let a = w.spawn_atom(element_id("H").unwrap(), 50.0, 50.0);
        let b = w.spawn_atom(element_id("H").unwrap(), 52.0, 50.0);
        w.temp_field.set(51.0, 50.0, 43.6);
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(
            w.bonds.len(),
            1,
            "exactly one bond (dedup + duplicate reject)"
        );
        assert_eq!(w.bond(BondId(0)).energy, 436.0);
        assert_eq!(w.atom(a).bond_count, 1);
        assert_eq!(w.atom(b).bond_count, 1);
        // Formation absorbed heat at the midpoint.
        let after = w.temp_field.get(51.0, 50.0);
        assert!(after < 43.6, "formation must cool, got {after}");
        assert!(matches!(
            w.event_queue.first(),
            Some(Event::BondFormed { .. })
        ));
    }

    #[test]
    fn formation_respects_capacity_and_distance() {
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        // G04: a full H cannot bond a third atom.
        let mut w = world(4);
        let a = w.spawn_atom(element_id("H").unwrap(), 50.0, 50.0);
        let b = w.spawn_atom(element_id("H").unwrap(), 51.0, 50.0);
        let c = w.spawn_atom(element_id("H").unwrap(), 52.0, 50.0);
        w.form_bond(a, b, 1, 436.0);
        w.temp_field.set(51.0, 50.0, 43.6);
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(w.bonds.len(), 1, "no second bond on a full atom");
        assert_eq!(w.atom(a).bond_count, 1, "a is full");
        assert_eq!(w.atom(c).bond_count, 0, "the bystander gains nothing");
        // Out of radius: no bond even at rate 1.0.
        let mut w = world(4);
        let d = w.spawn_atom(element_id("H").unwrap(), 10.0, 10.0);
        let e = w.spawn_atom(element_id("H").unwrap(), 90.0, 90.0);
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(w.bonds.len(), 0, "distant pair must not bond");
        assert_eq!((d.0, e.0), (0, 1));
    }

    #[test]
    fn double_bond_prefers_two_free_slots() {
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        let mut w = world(5);
        // Two carbons, both with all slots free: order 2 expected.
        let a = w.spawn_atom(element_id("C").unwrap(), 50.0, 50.0);
        let b = w.spawn_atom(element_id("C").unwrap(), 51.0, 50.0);
        w.temp_field.set(50.5, 50.0, 61.4); // t_opt = 0.1 * 614
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(w.bonds.len(), 1);
        assert_eq!(w.bond(BondId(0)).order, 2, "C=C preferred with free slots");
        assert_eq!(w.bond(BondId(0)).energy, 614.0);
        assert_eq!(w.atom(a).bond_count, 1, "one double bond on each carbon");
        assert_eq!(w.atom(b).bond_count, 1);
        // H + C: H has one slot, so single.
        let mut w = world(5);
        let h = w.spawn_atom(element_id("H").unwrap(), 50.0, 50.0);
        let c = w.spawn_atom(element_id("C").unwrap(), 51.0, 50.0);
        w.temp_field.set(50.5, 50.0, 41.3);
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(w.bond(BondId(0)).order, 1);
        assert_eq!((h.0, c.0), (0, 1));
    }

    #[test]
    fn geometry_gate_blocks_bad_angles() {
        // With rate 1.0, formation happens only through the
        // geometry gate: an O with one bond pointing +x will not
        // accept a candidate at ~180 degrees (sigma 30).
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        let mut w = world(6);
        let o = w.spawn_atom(element_id("O").unwrap(), 50.0, 50.0);
        let h1 = w.spawn_atom(element_id("H").unwrap(), 51.0, 50.0);
        let h2 = w.spawn_atom(element_id("H").unwrap(), 49.2, 50.0); // ~180 deg, too close to anti-ideal
        w.form_bond(o, h1, 1, 463.0);
        w.temp_field.set(50.0, 50.0, 46.3); // t_opt for O-H
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        assert_eq!(
            w.bonds.len(),
            1,
            "180-degree candidate must fail the geometry gate"
        );
        let _ = h2;
    }

    #[test]
    fn chemistry_is_deterministic_per_seed() {
        fn run(seed: u64) -> (usize, Vec<bool>) {
            let config = PhysicsConfig::default();
            let mut w = world(seed);
            // A handful of atoms in a warm field, chemistry only.
            for i in 0..6 {
                w.spawn_atom(element_id("H").unwrap(), 50.0 + i as f32, 50.0);
            }
            w.temp_field.set(50.0, 50.0, 400.0);
            rebuild_index(&mut w);
            let mut chem = chemistry(config);
            chem.form_bonds(&mut w);
            chem.break_bonds(&mut w);
            (w.bonds.len(), w.bonds.iter().map(|b| b.alive).collect())
        }
        assert_eq!(run(42), run(42));
    }

    #[test]
    fn max_bonds_never_exceeded_through_chemistry() {
        let config = PhysicsConfig {
            base_formation_rate: 1.0,
            ..PhysicsConfig::default()
        };
        let mut w = world(8);
        // One carbon surrounded by six hydrogens within radius; only
        // four bonds may form (G04 via form_bond).
        let c = w.spawn_atom(element_id("C").unwrap(), 50.0, 50.0);
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::TAU / 6.0;
            w.spawn_atom(
                element_id("H").unwrap(),
                50.0 + 3.0 * angle.cos(),
                50.0 + 3.0 * angle.sin(),
            );
        }
        w.temp_field.set(50.0, 50.0, 46.3);
        rebuild_index(&mut w);
        chemistry(config).form_bonds(&mut w);
        // G04 across the whole world: no atom exceeds its element's
        // max_bonds, no matter which pairs formed (neighboring H
        // atoms are within the search radius and may pair up).
        for atom in &w.atoms {
            let max = w.element(atom.element).max_bonds;
            assert!(
                atom.bond_count <= max,
                "atom {} has {} bonds, max {}",
                atom.id.0,
                atom.bond_count,
                max
            );
        }
        assert!(w.atom(c).bond_count > 0, "carbon should gain bonds");
        assert!(w.atom(c).bond_count <= 4, "carbon max_bonds is 4");
        let _ = MAX_BONDS;
    }
}
