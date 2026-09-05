# Phase-0 notes: what each abstraction stands on

Status: in progress (PLAN phase 0). Companion to references.md (the
bibliography); this file records, per abstraction khem uses, what
prior work supports it, what it simplifies away, and what the phase-1
kernel has already measured against it. Written 2026-09-05, after the
physics and energy systems landed and the first analytical stability
findings were made (F1-F5 below).

## Findings so far (kernel reality, measured)

F1-F5 as first analyzed (below), plus the measured set from the K1
harness runs (2026-09-05, 2000 ticks, seed 42, literal-then-retuned
constants; numbers in the tuning commit):

- F6  Formation refrigerates: forming a bond absorbs
      formation_fraction * E from the local field (spec 7.2), and
      with breaking frozen (F1) nothing ever returns it. Measured
      average temperature: -16.7 C (literal) / -9.4 C (retuned)
      from a 35 C start.
- F7  The spec's release 0.5 / absorb 0.3 asymmetry created
      energy: a form+break cycle deposited 0.2 * E into the field
      from nowhere. Fixed in the tuning commit: both 0.3.
- F8  No dissipation channel exists anywhere in the spec (checked:
      springs conserve, wall boundaries reflect, kicks only add).
      Additive thermal noise therefore random-walks velocities up
      without bound; measured KE at 2000 ticks: 4.4e13 (literal,
      spring-pumped) and 3.6e14 (retuned, kicks + repulsion
      slingshots). K1 cannot pass at any constant setting: this is
      structural, not tuning.
- F9  strong_repulsion / r^2 applied directly to velocity (dt = 1)
      is a cannon: one bonded overlap at r = 0.3 imparts v ~ 10^4
      A/tick in a single tick, and with no dissipation that energy
      circulates forever. Measured mean bond length after 2000
      ticks: 63 A (equilibrium ~1.2).

## Findings first analyzed before measurement

- F1  Literal kB (0.008314) with pond temperatures (15-80 C) makes
      thermal bond breaking impossible: p_break for the weakest bond
      (O-O 146 kJ/mol) is exp(-146/0.66) ~ 0 at 35 C, ~1e-9 even at
      the spec's hottest example temperature. Literal constants give
      a frozen world by construction. MEASURED: 0 breaks in 2000
      ticks. Retuned to kb_scaled 0.45 (breaking only): weak bonds
      (O-O) now break at 35 C every ~10k ticks, water's O-H
      essentially never - real chemistry's answer too.
- F2  The tick order (velocities, then positions) already IS
      semi-implicit (symplectic) Euler - the first analysis wrongly
      called it explicit. Symplectic Euler is stable for
      dt * sqrt(k) < 2; at spring_energy_scale 0.01 every bond over
      400 kJ/mol violated the bound (O-H: 2.15) and pumped energy
      geometrically (measured KE 4.4e13). Retuned to 0.002: the
      strongest tabulated bond (N#N 945) gives 1.375, inside the
      bound. One physical kB set two incompatible sim scales
      (breaking rate, kick magnitude) - split into kb_scaled
      (breaking) and thermal_kick_scale (kicks).
- F3  The language-spec pond (~70k atoms in 200x200 A) is ~20x
      liquid-water density and 7x the v0.1 performance target.
      Pond built scaled-down per owner decision.
- F4  The spec has no non-bonded interaction: unbonded atoms pass
      through each other. No excluded volume, no liquid structure.
      Queued as a proposal before K2 (section 4 below).
- F5  TICK events carry wall-clock fields (elapsed_ms,
      ticks_per_sec), so G02 (byte-identical output) needs a
      documented carve-out for those two fields. DONE in the
      observer module doc, pending spec revision.

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
need forces. The kernel hit the classic wall, corrected by
measurement (findings F2/F8/F9): the tick order is already
symplectic Euler (velocities before positions), which is stable for
dt * sqrt(k) < 2 - but that bound is a necessary condition, not a
thermostat. Without a dissipation channel, additive noise pumps
energy forever regardless of integration scheme. The lesson stands:
canonical-ensemble dynamics need a bath, not just a stable
integrator (see section 10 below).

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
exactly this reason. Measured 2026-09-05: the 3422-atom pond runs
~90 t/s in a debug build, ~2000 ticks in 22 s; the release-build
number comes with the phase-2 perf pass. No optimization before
the pond is behaviorally sane.

## 10. Thermostats: the missing bath (finding F8, the K1 blocker)

Every MD system that samples a temperature couples to a bath, not
just a noise source: Langevin dynamics adds friction alongside
the random force; the friction term is what makes the noise
converge to a canonical distribution instead of random-walking
energy upward. khem's spec 6.1 has noise only.

The coherent fix uses parts the spec already gestures at: region
declarations set environmental temperatures (the language spec's
surface/ocean/seafloor regions are bath SETPOINTS), spec 6.1's
first line says cell temperature IS mean kinetic energy (the
atom-to-field return channel), and vents/bond events perturb the
field locally. Proposed model (owner sign-off pending):

    v <- v * (1 - damping) + normal(0, sigma(T_cell))

a Langevin kick with damping toward the local field temperature:
fast atoms relax toward the cell's temperature, the field diffuses
and relaxes toward declared region values, energy bookkeeping
stays closed except at the world boundary (which the spec pins as
G06: sources only). One new knob (thermostat_damping), spec 6.1
revision, harness-gated: K1's KE and bond-length metrics must go
flat with it.

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