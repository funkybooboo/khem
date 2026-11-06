# Gate K2 - self-assembly: membranes are consequences, not rules

K2 is OPEN. K2.1 (excluded volume) and K2.2 (condensed medium)
PASSED 2026-11-04; the cohesion channel is implemented and
DORMANT (cohesion_scale 0.0 - the like-dissolves-like well in
code and spec 6.6); the pond re-density landed 2026-11-06 (real
water's atomic density on a 3.2 A lattice, keep-apart
construction).

NEXT, per [ADR-0014](../../docs/build/adr/0014-fidelity-philosophy.md):
the REAL-WATER SUBSTRATE SITTING - the wall rescale
(non_bonded_margin toward real first-shell geometry) plus the
directional hydrogen-bond term as one sitting, one
re-validation - then K2.3 climbs on the real-geometry defaults.
The sitting's checkpoint contract (owner-agreed): molecular
density ~0.033 molecules/A^3 held uniform (no blobs), O-O
first shell ~2.8 A, coordination ~4, measurable tetrahedral
order, the gates still passing, tick cost within +10-20%.

The harness is crates/khem-core/tests/k2_self_assembly.rs; the
methodology lives in [README.md](../README.md).

---

## Milestone K2 - self-assembly: membranes are consequences, not rules

Precondition: K1. The literature is unanimous that amphiphiles
assemble through non-bonded potentials, never through springs
alone - hence K2.1 comes first.

### K2.1 - excluded volume: PASSED

Soft non-bonded repulsion (F4 - smuggled PHYSICS, documented as
such): free atoms no longer pass through each other. Bar:
minimum approach distance >= 0.8 * (r_a + r_b) in a scattering
test. (Implemented with the K1.1 substrate work, spec 6.6; the
scattering-test PASS run is the gate's own commit.)

PASSED 2026-11-04 (tests/k2_self_assembly.rs: the gate, the
strength-sweep probe, and an always-on smoke) - and the gate
EARNED ITS KEEP BEFORE IT PASSED. The harness, two phases:

- PHASE A, controlled scattering: pairs launched head-on and
  grazing at closing speeds c x sigma_rel(35 C), c in {1, 2, 3}
  (a common collision, a fast one, the p99 tail a 10k-tick
  beaker samples routinely), four element classes (H-H the
  worst tunneling case, H-O, O-O, Na-Cl), physics-only at the
  integrator's exact sub-step structure, minimum approach
  measured at every sub-step boundary. It MEASURED THE
  STRENGTH-1 CORE FAILING THE GATE'S OWN LETTER: an H-H pair at
  the p99 closing speed (2.28 A/tick) passed clean THROUGH
  (min approach 0.015 of the radius sum, 8/8 crossings), a
  2-sigma pair penetrated to 0.48, and every crossing was
  energy-honest (pair KE at exit 0.99x launch) - the sub-
  stepping resolved the wall exactly as spec 5.1 claims; the
  wall was too weak to matter at the thermal tail (F22).
- The strength sweep ({1, 4, 8, 16, 32} head-on bins, the
  symplectic bound as the ceiling) picked the retune:
  non_bonded_repulsion 1.0 -> 8.0, the minimal swept strength
  that holds every bin (worst: H-H c=3 at 0.959, 20% over the
  bar, the analytic turning point to three decimals; mint
  1.00-1.04; dt_sub * sqrt(S/mu) = 1.00 for the worst pair,
  against the stiffest spring's own 1.375). 16.0 holds more
  margin but spends it closer to the bound (1.41); 32.0 was
  measured and REJECTED - the bound hits 1.99 and the
  discretized wall mints 2.0-3.6x pair KE (the mint law found
  the stability edge exactly where the bound said it would).
  Spec 6.6/11 synced; golden hash re-cut consciously.
- The wall does not touch the molecules: water's ideal H-H
  sits at 1.88 A against its 1.59 A cutoff (18% margin) and a
  settle probe (bath off) holds the geometry frozen - the
  stiffened core engages free pairs and third-atom sterics
  only.
- PHASE B, the operating soak: the K1.5 beaker mix (760 free
  atoms, 30x30x30 Wrap) at the pond's 35 C, full Sim with
  chemistry live (bonded pairs excluded from the census - a
  bonded pair is the spring's business, K1.2's band), 10k
  ticks, spawn keep-apart 2 A per the pond's own sprinkle
  margin (a uniform spawn measured 8 pair-ticks below the bar -
  all spawn artifacts the wall cleaned within ticks; initial
  conditions are matter, not a transient the census must
  forgive). PASS with the operating point far from the bar:
  minimum unbonded approach 1.001 over the run - never below
  the radius sum, zero violations, zero pair-ticks below 1.0,
  62 in [1.0, 1.25), 173 formations.

THE RE-VALIDATION CONTRACT, EXERCISED AT FULL FORCE (the retune
moves every gate's operating point): K1.1-K1.5 all re-ran in
this commit and all PASS - see each gate's RE-RUN 2026-11-04
entry above (the K1.4 re-run realigned the harness's weak class
to the table's own entries - N-O is the K3 note's weak
candidate - and the K1.5 re-run watched the F21 scan-order
artifact wash out at the stiffer core).

K2.2 climbs next (condensed medium), on a substrate whose atoms
bounce instead of passing through.

RE-RUN 2026-11-06 with the pond re-density: PASS, with one
honest bar re-derivation. The beaker matches the pond's new
operating point (760 atoms of the K1.5 mix in a 19.5^3 box,
0.1022 atoms/A^3 - the soak must be what the pond feels),
seeded by the pond's own keep-apart law. The first draft kept
phase B's flat 0.8 bar, which the SPARSE beaker held - but the
dense beaker measured transient CLUSTER SQUEEZES to 0.6-0.8
* r_sum: a third atom sandwiched inside a bonded pair's span,
held by the bond and the geometry against the (linear,
deep-soft) wall until the cluster rearranges (18 pair-ticks
in 10k, all in the formation window, none from spawn once
the placement fell back to BEST-OF-N instead of place-the-
last-draft). Free pairs hold 0.8+ (phase A unchanged to the
digit: worst bin 0.959, mint 1.043, zero crossings), and the
excluded-volume claim's strongest testable form at the
operating point is NOTHING PASSES THROUGH: the phase B bar
is min ratio >= 0.5 (measured 0.608, far above any crossing -
a passed-through pair samples ~0), with the squeeze channel
counted and reported as evidence. If K2.3+ shows the squeeze
channel matters (membrane defects), the wall's shape at deep
overlap is the named lever.

### K2.2 - condensed medium: PASSED

The pond behaves as matter, not free flight - most atoms hold a
non-bonded neighbor within 3 A, and pair distances show
structure.

PASSED 2026-11-04 (tests/k2_self_assembly.rs: the 25k-tick pond,
steady tail sampled every 100 ticks over 21k-25k; INTERMOLECULAR
pairs only - different molecule by the bond graph's union-find:
intramolecular 1,3 pairs are molecular structure, and a gas of
free-flying waters would pass a naive census on them alone,
which is exactly the free-flight failure this gate exists to
catch):

- THE CENSUS: 0.978 of atoms hold a non-bonded intermolecular
  neighbor within 3.0 A (bar: the letter's "most" = 0.5; min
  over the 41 samples 0.973) - the medium is contact-dense.
- THE STRUCTURE: the intermolecular g(r) against the uniform
  reference (rho 0.0636 atoms/A^3) shows the excluded-volume
  hole - the structure a repulsion-only substrate CAN hold: g =
  0 below 0.75 A, <= 0.025 below 1.25 A (the hole's core; the
  thermal tail presses small pairs to ~1.0 * r_sum, so the
  core bar sits below the smallest pair's standoff), a
  monotone ramp below uniform out to 3.25 A (max g 0.944
  below 3.0 A, against the free-flight reference's flat 1.0),
  uniform beyond (1.001-1.002 out to 4.5 A).
- THE BASELINE, PREREGISTERED (max g beyond 3.0 A: 1.002, bar
  <= 1.05): there is NO first-shell peak. Repulsion cannot
  build one (spec 6.6 is repulsion-only and says so); the
  peak needs the element-derived attraction channel - the
  registered K2.3 design debt (PLAN gap register). The bar
  exists so the attraction channel's effect is measurable
  against THIS baseline: its substrate commit re-runs this
  gate and replaces the baseline bar with a peak bar. The
  K2.3 memo's design has to beat: census 0.978, hole to
  uniform by 3.5 A, g_max 1.002 - a uniform-plateau fluid with
  a 3 A correlation hole.

The pond is molten by the tail (Langevin diffusion carries an
O ~100+ A over the run; the port's 3.75 A water lattice is
fully mixed), so the measured object is the fluid the substrate
actually holds: contact-dense matter with excluded volume, no
cohesion. No constants moved; no re-validation trigger - this
gate is pure measurement.

The K2.3 attraction memo is the next precondition (phase-0
re-open), then the K2.3 substrate and its gates.

RE-RUN 2026-11-06 with the pond re-density (a WORLD re-run at
dormant cohesion - the medium is the same repulsion-only
physics at the new 0.1022 atoms/A^3): PASS with every bar
unchanged, and the numbers moved the way denser matter
should: census 0.999 (min 0.997 over 41 samples - at real
water's own atomic density essentially every atom holds an
intermolecular neighbor within 3 A), hole core 0.035 below
1.25 A, hole extent max 0.963 below 3.0 A, and the
repulsion-only baseline still flat: max g 0.978 beyond 3.0 A,
no shell - the number the activation must beat.

### K2.3 - amphiphile sorting

A lipid (polar head, nonpolar tails) in water - head-water and
tail-tail contact fractions beat chance by a set margin. No
"membrane" rule exists anywhere in the runtime.

DESIGN DEBT (the gap register, plans/issues.md): the substrate's only
attraction is the bond spring; spec 6.6 is repulsion-only and
records the lesson itself ("the lipid literature is unanimous
that self-assembly needs non-bonded potentials, never springs
alone"). Amphiphile sorting needs a non-bonded polarity
ATTRACTION - an element-derived potential (electronegativity
differences), never a "lipids clump" rule - designed and
honesty-flagged per the F4 pattern BEFORE K2.3 tuning starts.
Phase 0 re-opens for this memo; the phantom-solvent and
four-bead amphiphile models (references.md) are the starting
point.

### K2.4 - vesicle closes

Lipids form a persistent cluster with an interior (union-find:
cluster >= 12 lipids, survives >= 10k ticks), heads pointing
outward.

### K2.5 - contents held

Free nucleotides placed inside a vesicle stay inside above a
leak threshold.

### K2.6 - vesicle grows

A fed vesicle incorporates lipids and grows (Squirm3 lesson:
membranes must grow, not just close). Division is NOT gated in
v0.1 - it is the first thing to chase after K5.

