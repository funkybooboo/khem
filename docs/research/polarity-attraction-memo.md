# The non-bonded polarity attraction - design memo

The phase-0 re-open memo the gap register names (PLAN.md,
"Non-bonded polarity attraction"; the K2.3 precondition). The
K2.2 gate just made the target concrete: the measured
steady-tail pond is a uniform-plateau fluid with a 3 A
correlation hole and NO first shell (g_max 1.002 beyond 3.0 A,
census 0.978 - see the K2.2 PASS record,
docs/plans/phase-1-kernel.md). Repulsion cannot build the
missing peak; this memo designs what can, against the
literature, before K2.3 tuning starts.

Written 2026-11-04, after gates K2.1/K2.2.

## What the literature does

Every coarse-grained amphiphile model that self-assembles -
Huang et al.'s four-bead lipids, the Marrink-group solvent-free
POPC, Arnarez et al., Steltenkamp's phantom-solvent monolayer -
uses the same two-channel structure: soft short-range repulsion
plus an effective hydrophobic COHESION (references.md, "the
non-bonded requirement"; abstraction-notes section 4). Nobody
assembles amphiphiles from springs alone. The cohesion is
always non-bonded, always pair-local, and always carries the
free-energy asymmetry that sorts polar from nonpolar: water
prefers water, tails prefer tails, and mixed contacts are
relatively disfavored.

khem has the repulsion (spec 6.6, retuned with K2.1). This memo
designs the cohesion at khem's level: atom-atom, pair-local,
element-derived, no molecule-awareness anywhere.

## The design: like-dissolves-like over electronegativity

One law, one channel, applied to every UNBONDED pair outside
its excluded-volume cutoff (the same population spec 6.6 rules;
bonded pairs stay exempt - springs own them):

    S(a, b) = cohesion_scale * max(0, 1 - |chi_a - chi_b| / chi_ref)
    F_attr  = S(a, b) * ramp(r),  cutoff < r < r_attract
    ramp(r) = (r - cutoff) * (r_attract - r), zero at both ends

A linear-ramp well outside the repulsive core - the house
style (the repulsion is linear too), smooth at both
boundaries, conservative, and soft enough that the
symplectic bound is never in question (the well's curvature
is cohesion_scale-shaped, tuned far below the spring scale).

Why like-dissolves-like and not a mean-polarity law: sorting
REQUIRES the mixed contact to be weaker than BOTH
self-cohesions - the free-energy signature of the hydrophobic
effect at the pair level. An arithmetic interpolation over the
mean polarity puts the mixed pair between the two selves and
sorts nothing. The absolute-difference form does it with one
constant: with chi_ref = 3.0 and the table's Pauling values,
water coheres to water (O-O 1.00, H-H 1.00, O-H 0.59),
hydrocarbon tails cohere to themselves (C-C 1.00, C-H 0.88),
and the mixed contacts sit at 0.59-0.70 - the hydrophobic gap,
from element properties alone.

Honesty flag (the F4 pattern): this is smuggled PHYSICS,
documented as such. The substrate has cohesion because matter
does; the electronegativity shape is there because polarity
sorts matter, not because lipids need it. No element pair is
special-cased, no molecule is named, the law reads the same
element table every other rule reads.

## Known simplifications, recorded before any tuning

- No hydrogen-bond channel. Like-dissolves-like scores the
  H...O contact LOW (|chi| difference 1.24 - the largest
  common pair), which is backwards for real water, whose
  cohesion is mostly hydrogen bonds. The design bets the
  liquid holds on O-O and H-H cohesion alone; if a K2/K3 gate
  demands donor/acceptor specificity, a separate H-bond term
  is its own memo - never a "lipid" special case.
- No electrostatics. Na-Cl score each other at 0.26
  (|chi| difference 2.23) - real ions attract. Salt
  chemistry is not on the K2-K3 critical path; the law models
  polarity sorting, not ionic bonding. Recorded as a v0
  simplification with a named trigger: the first gate whose
  world needs ions to behave like ions.
- Same-molecule 1,3 pairs feel the attraction (not exempt,
  like the repulsion). Water's own H-H sits at 1.88 A, inside
  the H-H attractive shell - the well will pull water's angle
  slightly closed where the repulsive core never touched it.
  The K1.3 re-run and the settle probe measure this; if the
  geometry shift is material, the honest fix is range shaping
  (keep the well outside 1,3 distances), never an exemption
  rule that smells like molecular awareness.

## Preregistered tuning lines (the ADR-0003 honesty rule)

May be tuned inside K2.3, with harness evidence in each
commit: cohesion_scale (the well depth), chi_ref (the
discrimination scale), r_attract's range shape, and the lipid
world's own geometry (head/tail composition, chain length -
harness-seeded structure, freely designable; it is a world,
not the runtime).

May NOT be tuned or added: the electronegativity values
(Pauling, from the element table - the law reads them, it does
not move them), per-pair special cases, molecule-aware terms,
any rule that names a lipid, membrane, head, or tail in the
runtime.

Stop line (the exit criterion's tripwire): if amphiphile
sorting does not pass after honest sweeps of the named
constants - weeks, not days - STOP and redesign the substrate
per the ladder's exit criterion. No rule above the atom/bond
level (G01) rescues a dead channel.

## What the substrate commit owes (the re-validation contract)

The attraction moves every operating point, like the K2.1
retune did. Its commit re-runs, in the same commit: K1.1 (a
potential well changes the KE/field balance), K1.2 (cohering
neighbors bombard bonds), K1.3 (the 1,3 geometry shift above),
K1.4 (the free-atom beaker now coheres - expect the size
distribution to shift toward larger clusters; the no-runaway
and stationarity bars re-measure it), K1.5 (the attraction is
a minimum-image law over the wrap-aware index - symmetry must
hold), K2.1 (the wall's standoff - the well sits outside the
cutoff, the excluded-volume bar must hold unchanged), and
K2.2 - whose BASELINE BAR (max g <= 1.05 beyond 3.0 A: no
shell) is REPLACED by a peak bar (max g over the first shells
> 1.0 by a measured margin). The K2.2 numbers to beat: census
0.978, hole to uniform by 3.5 A, g_max 1.002.

The golden hash re-cuts consciously; spec 6.6 grows the
attraction term and section 11 the constants, in the same
commit.

## The K2.3 gate bar sketch (the letter, the harness's numbers)

The letter: head-water and tail-tail contact fractions beat
chance by a set margin, with no "membrane" rule anywhere.
Chance is measured by the harness's shuffled reference: the
same atoms, the same element mix, contacts counted against a
uniform-occupancy expectation (the K2.2 g(r) machinery
reused per contact class). The margin and the lipid design are
K2.3's own commit - this memo only fixes what the runtime may
know: atoms, bonds, elements, and nothing else.
