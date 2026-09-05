# Phase-0 notes: what each abstraction stands on

Status: in progress (PLAN phase 0). Companion to references.md (the
bibliography); this file records, per abstraction khem uses, what
prior work supports it, what it simplifies away, and what the phase-1
kernel has already measured against it. Written 2026-09-05, after the
physics and energy systems landed and the first analytical stability
findings were made (F1-F5 below).

## Findings so far (kernel reality, pre-measurement)

- F1  Literal kB (0.008314) with pond temperatures (15-80 C) makes
      thermal bond breaking impossible: p_break for the weakest bond
      (O-O 146 kJ/mol) is exp(-146/0.66) ~ 0 at 35 C, ~1e-9 even at
      the spec's hottest example temperature. Literal constants give
      a frozen world by construction.
- F2  Explicit Euler with dt=1 diverges when sqrt(spring_k) > 2;
      every bond above 400 kJ/mol qualifies (O-H 463, H-H 436, all
      double/triple bonds). Literal constants give a numerically
      exploding world.
- F3  The language-spec pond (~70k atoms in 200x200 A) is ~20x
      liquid-water density and 7x the v0.1 performance target.
- F4  The spec has no non-bonded interaction: unbonded atoms pass
      through each other. No excluded volume, no liquid structure.
- F5  TICK events carry wall-clock fields (elapsed_ms,
      ticks_per_sec), so G02 (byte-identical output) needs a
      documented carve-out for those two fields.

These are expected: ADR-0006 treats the specs as drafts until
validated against the kernel. The constants retune against harness
measurements; the structural questions (F4, F5) get proposals after
first numbers.

## 1. Substrate shape: a local-topology artificial chemistry

Dittrich, Ziegler, Banzhaf (2001) define an AC as (S, R, A): possible
molecules, collision rules, and an algorithm applying them. khem
maps: S = every molecule expressible as a bond graph over the 10
element table; R = Boltzmann/Arrhenius breaking, geometry- and
electronegativity-weighted formation; A = the fixed nine-step tick.
The taxonomy's "well-stirred vs imposed topology" split lands khem
firmly local-topology (spatial index, position matters) - the choice
the review argues suits prebiotic modeling.

Hutton (2002) argues movement-based ACs allow richer interactions
than grid CA replicators, and that rich interactions are necessary
for evolution beyond the trivial. khem takes the same bet with
continuous space.

Simplified away: no reaction-rule specificity by molecular context
(Squirm3's typed/stateful rules), no catalysts in v0.1, no electrons.

## 2. Atom/bond substrate with real-element tables

SimSoup is the closest philosophy: molecule properties derived from
structure rather than enumerated. khem pushes that philosophy one
level down (properties from element tables + bond energies + VSEPR
angles rather than per-species parameters), which is what makes the
substrate open-ended: any molecule the bond graph allows is a legal
molecule with computable properties, no species list to maintain.

Simplified away: 3D conformations (2D per ADR-0002), electrons,
quantum effects, real kinetics (rate lookup replaces solving).

## 3. Integration scheme: springs in continuous space

Hutton rejected mass-spring physics for Squirm3 as too costly and
used random walks on a grid; all his emergent behavior came from
chemistry rules, not physics. khem keeps springs because bond
geometry and thermal escape (K3's "strands separate thermally")
need forces, but the kernel already hit the classic wall: explicit
Euler is energy-drifting and unstable for oscillators (textbook
result; see the harmonic-oscillator finite-difference experiments
and the symplectic-integrator literature - Verlet/leapfrog/velocity
Verlet are the field's standard for "exceptional stability" at
large steps).

Lesson for the tuning commit: the fix is not only smaller
spring_energy_scale; it is a symplectic or semi-implicit update
(velocity update uses the NEW position forces). Decide with harness
numbers, cite this section in the commit.

## 4. Non-bonded interactions (the F4 gap)

Every coarse-grained lipid self-assembly model - Huang et al.'s
four-bead lipids, the solvent-free POPC models, the phantom-solvent
generic membrane model - self-assembles through explicit NON-bonded
potentials: soft repulsion plus an effective hydrophobic cohesion.
None assembles amphiphiles from springs alone. The chemistry
literature agrees: excluded volume is the mechanism of structure.

Implication: K2 (lipid self-assembly from polarity rules) is not
reachable without at least a soft short-range pair repulsion, and
probably an attraction channel (khem's polarity hook:
electronegativity). Proposal queued until after first measurements:
soft-core repulsion for all pairs via the spatial index, documented
honestly as a substrate addition (guardrail: no smuggled biology -
this is smuggled PHYSICS, and the spec revision will say so).

## 5. Boltzmann bond dynamics with lookup tables

Bond-energy tables and Boltzmann break probabilities are the
phenomenological design (PLAN guardrails: "lookup tables and
phenomenological rates are the design, not a compromise to hide").
F1 confirms the constants - not the functional form - need phase-1
retuning: exp(-E/kT) is the right shape (stronger bonds break less),
and the literature's Arrhenius rates support the form. The retune
is a scaling of kb_scaled, not a redesign.

## 6. Replication lessons (Squirm3 experiments)

Squirm3's three experiments are the dress rehearsal for K3-K5:

- Replication emerged from rules + a random soup, but the world was
  "intrinsically dirty": crosstalk reactions produced mutants and
  crossovers. Dirtiness is a feature to preserve, not a bug to
  stamp out.
- Replication stalled when raw material (specific atom states)
  depleted. The pond needs material turnover for K5 selection to
  have anything to select on.
- A periodic "flood" (half the world replaced) created the selection
  pressure that shortened replicators - and then evolution STOPPED:
  the chemistry had no capacity for features beyond "shorter is
  faster". khem's bet against that plateau is exactly its richer
  substrate (element diversity, bond orders, geometry), and the
  honest expectation is that khem's first plateau will also exist.
  Finding it is phase-1 success, not failure.

## 7. Membranes must grow, not just close

Squirm3's fixed-length membrane loops protected replicators but
suffocated them; Ono & Ikegami and Mayer & Rasmussen's membrane
models grow and divide. K2's acceptance metric should include
membrane GROWTH dynamics, not only "a ring formed".

## 8. Detector discipline

The Genesis Engine correction notice (references.md, required
reading) governs watch-condition design: khem's NOTABLE events must
be measurable state comparisons, preregistered before runs (PLAN
thesis track), never a detector that cannot report "no".

## 9. Performance envelope

Squirm3 (2001 hardware, C++): 100x100 grid ~1000 iterations/s.
JohnnyVon (continuous 2D replicators): "much slower to run" -
cited by Hutton as restricting evolutionary usefulness. khem's
v0.1 target (10k atoms >500 t/s, continuous space, springs, spatial
hash) is aggressive but the architecture's scaling hooks exist for
exactly this reason. Measure against the target after the pond
lands; do not optimize before.

## What this means for the build order

The kernel work continues per PLAN (chemistry -> sim loop ->
observer -> pond -> K1 harness -> measured tuning commit). The
research above changes three things in that plan:

1. The tuning commit evaluates a semi-implicit/leapfrog velocity
   update alongside constant retuning (section 3), not constants
   alone.
2. A non-bonded soft-repulsion proposal is drafted BEFORE K2 work,
   since every precedent says K2 is unreachable without it
   (section 4); it lands only with harness evidence, as its own
   commit + spec revision.
3. The K1 harness records dirtiness metrics (crosstalk bonds,
   unintended species) from the start, per section 6 - the dirt IS
   the variation K4 needs.