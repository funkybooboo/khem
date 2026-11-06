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
its excluded-volume cutoff (the same population spec 6.6 rules):

    S(a, b) = cohesion_scale * max(0, 1 - |chi_a - chi_b| / chi_ref)
    well shell: cutoff < r < attraction_range (FLAT, ~4 A)
    U(x)       = -S(a, b) * 16 x^2 (w - x)^2 / w^4,  x = r - cutoff
    F         = -dU/dr

A double-sided quartic well: attractive on the outer slope,
repulsive on the inner, zero at both ends and at the minimum -
conservative everywhere. The first draft was a pure-attractive
ramp; the revision log below carries why it cannot ship. The
flat range (not r_sum-scaled) puts every pair's well minimum at
~3 A: real water's O...O first shell is ~2.8 A and dispersion
contacts cluster at 3.4-3.7 A - cohesion ranges do not scale
with covalent radii. The flat range also keeps the lightest
pair's slopes gentle (stiffness ~ depth/w^2; the H-H well is
2.41 A wide) and pairs whose excluded volume swallows the whole
shell (Na-Cl, cutoff 4.02) simply have no well - big atoms
cohere by chemistry alone, a documented size discrimination off
the K2-K3 path.

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

## The intermolecular scope line

The well models INTERMOLECULAR cohesion. Same-molecule 1,3
pairs (two bonds, one shared atom) are EXEMPT - the same
local-graph locality as the bonded-pair exemption (path 2
instead of path 1; no molecule identity, no transitive
structure anywhere in the force). The measurement that forced
the line: without it, the well's inner slope pushes a water's
own H-H (1.88 A, inside the H-H well shell) open, and no
r_sum-scaled or shaped range can put BOTH water's 1,3
neutrality and the O-O first shell where the physics wants
them - water's H-H distance is 1.774 of its r_sum (fixed by the
seeded 104.5-degree angle) while the liquid's first shell
wants ~2.1-2.3 of r_sum. Depth 0.5 without the exemption
measured the H-O-H angle wandering to ~150 degrees with O-H
bonds at 1.12-1.25 A; with it, a settle probe holds the geometry
frozen to four decimals. Longer paths (1,4+) are NOT exempt: a
chain's distant contacts feeling the well is real
intramolecular collapse, owned by whichever gate owns the
chain (K2.3's lipids).

## Known simplifications, recorded before any tuning

- No hydrogen-bond channel. Like-dissolves-like scores the
  H...O contact LOW (|chi| difference 1.24 - the largest
  common pair), which is backwards for real water, whose
  cohesion is mostly hydrogen bonds. The design bets the
  liquid holds on O-O and H-H cohesion alone; if a K2/K3 gate
  demands donor/acceptor specificity, a separate H-bond term
  is its own memo - never a "lipid" special case.
- No electrostatics. Na-Cl score each other at 0.26
  (|chi| difference 2.23) - and with the flat range their
  excluded volume (cutoff 4.02) swallows the well entirely.
  Salt chemistry is not on the K2-K3 critical path; the law
  models polarity sorting, not ionic bonding. Recorded as a v0
  simplification with a named trigger: the first gate whose
  world needs ions to behave like ions.
- Same-molecule 1,3 pairs feel the REPULSION (not exempt,
  like the repulsion's own sterics rule) but not the well -
  the intermolecular scope line above. The settle probe holds
  water frozen at exactly 104.5 degrees with the exemption.

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

## Revision log (2026-11-04, the implementation sitting)

The channel is IMPLEMENTED and DORMANT (cohesion_scale ships
at 0.0; spec 6.6). Three measured revisions to this memo's
first draft, and one decision the implementation's evidence
forced:

- The pure-attractive ramp cannot ship: a single-signed force
  profile integrates to a potential step at the range boundary
  (S*w^3/6), and pairs crossing the edge would mint exactly
  that - F18's phantom-capture lesson at the force level. The
  implemented shape is the double-sided quartic: conservative
  everywhere, zero net force across the shell.
- The r_sum-scaled range cannot ship either: it put the well
  minimum at 0.5 * (margin + range) * r_sum, which for H-H
  (w 1.06 at range 2.5) gave edge stiffness 32D/w^2 - at
  depth 1 the coupled water molecule EXPLODED (energy ledger:
  conserved to t=1200, then +373 by t=1400) and depth 0.5
  wandered the H-O-H angle to ~150 degrees. The FLAT 4 A range
  fixes both: the O-O minimum at 2.99 A (real water's own
  first shell), gentle slopes for every pair, and the perf hit
  bounded (the query radius grows from 3.3 to 4.0 A, not to
  8.8 A - the first draft multiplied where it should have
  taken a max).
- The intermolecular scope line (the 1,3 exemption) landed as
  a measurement-forced rule, documented above.
- THE POND RE-DENSITY is the named blocker for activation:
  the sweep (tests/k2_self_assembly.rs, k2_3_cohesion_sweep)
  measured every useful depth condensing the seeded pond
  (0.0636 atoms/A^3, 0.63x real water) into a droplet - depth
  0.25: local density 2.4x the box average with a real first
  shell (1.18 relative to local); depth 1.0: a ~10x clump.
  The port plan pre-authorized exactly this re-derivation
  ("Re-derivation, if any, belongs to the re-climb").  The
  substrate-ON commit owns: the pond re-density (spacing 3.2 A
  puts 1024 waters at real water's 0.10 atoms/A^3 in a
  51.2 x 51.2 x 12.8 A slab - the field geometry needs the
  matching look), the activation depth, and the FULL
  re-validation contract this memo already lists. Dormancy is
  provably free: at depth 0 the golden hash returns the
  pre-channel value byte-for-byte.

- THE POND RE-DENSITY (2026-11-06) did NOT dissolve the tension:
  the re-densified pond (3.2 A lattice, 0.1022 atoms/A^3 - real
  water's own atomic density, 51.2 x 51.2 x 12.8 A slab, keep-
  apart construction, seeded at 1024/1024 intact) still phase-
  separated under the well: D=0.5 built a real first shell
  (peak g 2.12 - but NO trough between shells: the compressed
  fluid's shells merge) with the medium collapsing into dense
  blobs over thin vapor by 12k ticks (the occupancy grid shows
  four sub-blobs mid-coalescence, cores pressed against the
  walls); D=1.0 jammed pairs against the walls (peak g 6.15).
  THE STRUCTURAL CAUSE, now measured: the model's atoms are
  ~2x smaller than real atoms' effective sizes (excluded-volume
  cutoffs 1.5-2 A against real vdW contacts ~2.8-3.2), so the
  model's liquids live at ~2-3x real liquid density - and the
  water lattice cannot seed below ~3.2 A spacing (adjacent
  waters' hydrogens clash; a random keep-apart water seed at
  the equilibrium density is a packing problem random draws
  cannot solve). The pond can never sit at its own liquid's
  equilibrium: at any shell-building depth it condenses. That
  is the physics, not a bug. THE ACTIVATION DECISION (its own
  sitting): accept the phase-separated pond - a self-cohering
  liquid blob plus vapor, arguably the honest Wrap-world
  primordial pond, with every pond gate re-derived on it - or
  pick a depth weak enough to hold the seed and let K2.3's
  beakers (free-atom worlds, seedable at any density) carry
  the assembly. The sweep lives in
  tests/k2_self_assembly.rs (k2_3_cohesion_sweep); the
  re-densified pond's own gates re-ran at dormant depth in the
  re-density commit.

## What the substrate commit owes (the re-validation contract)

The attraction moves every operating point, like the K2.1
retune did. Its ACTIVATION commit re-runs, in the same
commit: K1.1 (a potential well changes the KE/field balance),
K1.2 (cohering neighbors bombard bonds), K1.3 (the 1,3
exemption must hold water's geometry; the settle probe is the
fast check), K1.4 (the free-atom beaker now coheres - expect
the size distribution to shift toward larger clusters; the
no-runaway and stationarity bars re-measure it), K1.5 (the
attraction is a minimum-image law over the wrap-aware index -
symmetry must hold), K2.1 (the wall's standoff - the well
sits outside the cutoff, the excluded-volume bar must hold
unchanged), and K2.2 - whose BASELINE BAR (max g <= 1.05
beyond 3.0 A: no shell) is REPLACED by a peak bar (max g
over the first shells > 1.0 by a measured margin). The K2.2
numbers to beat: census 0.978, hole to uniform by 3.5 A,
g_max 1.002.

The golden hash re-cuts consciously; spec 6.6 carries the
well term and section 11 the constants (both landed with the
dormant channel).

## The K2.3 gate bar sketch (the letter, the harness's numbers)

The letter: head-water and tail-tail contact fractions beat
chance by a set margin, with no "membrane" rule anywhere.
Chance is measured by the harness's shuffled reference: the
same atoms, the same element mix, contacts counted against a
uniform-occupancy expectation (the K2.2 g(r) machinery
reused per contact class). The margin and the lipid design are
K2.3's own commit - this memo only fixes what the runtime may
know: atoms, bonds, elements, and nothing else.
