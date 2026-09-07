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
- F10 No minimum-image convention: a bonded pair 1 A apart across
      the Wrap seam reads as width-1 apart, and the spring shreds
      it (measured: +-103 A/tick on a 1 A pair). Found 2026-09-05
      by the seam test written BEFORE the fix - the exact testing
      gap the owner asked about. Fixed same day (WorldState::delta,
      spec 6.3/7.1/7.2 synced). Post-fix round 3: mean bond length
      58.5 A (from 63) - real but minor; F8/F9 dominate K1's
      failure. Known related gap (F11): the spatial index does not
      wrap, so formation candidates across the seam are suppressed
      (no dynamics corruption; phase 2 with K2/K3).

## Resolution log (2026-09-05, K1.1 session)

The substrate corrections that made K1.1 pass, each found by
measurement (probe-first), each with a law test:

- F8 RESOLVED: the Langevin thermostat (damping + matching noise
  toward the local field temperature) with exact signed-delta
  bookkeeping (the "injected" term double-counted and bled the
  field; the exact invariant field + KE * ke_field_scale is now
  test-pinned). K1.1 PASS: KE/atom -5.7%, mean bond length +0.5%
  window drift over a 10k-tick vented run.
- F9 REVISED AND RESOLVED: the hard core was not a cannon to cap
  but a discontinuity to remove - symplectic Euler pumped it every
  time a thermal kick carried an atom through. One smooth Hooke
  law both directions; strong_repulsion deleted.
- F12: the founding spec applied forces without /mass (F = ma
  missing). Hidden in H-H tests (unit mass); oxygen interactions
  minted ~(0.5*m - 1) * F^2 each tick - the primary furnace.
  Found by the one-water probe (a pair decayed; a trio pumped).
- F13: fast atoms tunnel through the ~2 A non-bonded zone; the
  asymmetric discrete force sampling mints energy proportional to
  speed (measured: runaway at vmax 3, calm at vmax 2). Velocity
  clamp below the mint threshold, removed KE honestly deposited;
  collision sub-stepping is the phase-2 proper fix.
- F14: bonds formed between flyby pairs became 80 A comets.
  Capture gate: no formation above max_form_speed relative speed.
- F15: the substrate lacked mechanical dissociation - overstretched
  bonds random-walked to 30-80 A alive. Break past
  bond_break_factor * r_eq, silently (heat-releasing length breaks
  cascaded in measurement; sinks cannot cascade).
- F16: frozen breaking by composition - the H-dominated free mix
  could only form strong bonds (zero breaks in 2000 ticks).
  Pond rebalanced toward O/N for weak flickering pairs.
- The pond gained its vent and the 35 C setpoint reservoir (spec
  6.2): a vented Wrap world with no sink only heats; flatness
  requires the environment to be a declared bath.

## Spec sync policy

Owner decision 2026-09-05: the implementation and the canonical
spec must agree; every divergence is fixed in both in the same
commit (piecemeal, not a later consolidated revision). The
2026-09-05 sync folded: section 11 constants (the tuned set above),
6.1 bath authority + thermal_kick_scale split, 6.3 minimum-image,
7.1/7.2 mechanics and v0 semantics, 7.3/7.4 fallbacks and
semantics, 8.2 UV placement, 8.3 diagnostics-not-schema, the G02
timing carve-out, and [phase 2]/[phase 3] markers on designed-but-
unbuilt items (CLI modes, signals, compaction, save, watch
conditions). Findings pending decisions (thermostat, F4) are NOT
in the spec yet - they land when decided.

## Resolution log (2026-09-07, K1.3 session)

The integrator commit that made K1.3 pass, each piece measured
(the K1.3 probe classified every bond event of a 10k-tick vented
run by channel):

- F17 RESOLVED: at the dt=1 stability cap the O-H mechanical
  well (stretch energy at the 7.1 break point) was only ~10 kT -
  a thermal-speed hydrogen, or one non-bonded shove into a
  water's H, carried enough to stretch the bond past the break
  point. Measured failing profile: 1482 O-H breaks in 10k ticks,
  ALL mechanical (zero thermal - the shatter was pure
  bombardment, not the Boltzmann channel), 206/1024 waters
  intact. The fix was the plan's named lever pulled forward from
  phase 2: integration sub-stepping (4 sub-steps/tick, dt_sub =
  0.25). The stability bound evaluated at dt_sub admits
  spring_energy_scale 0.032 (worst formable pair H-H at
  dt_sub*sqrt(k/mu) = 1.32 < 2), which puts the O-H well at
  ~80 kT - real water's own ratio - so thermal kicks essentially
  never reach the break point. Re-measured: 0 seeded-water O-H
  breaks, intact 1024/1024 flat (one new water even assembled
  from free atoms). The sub-stepping also resolves the crossings
  the velocity clamp guarded (F13's mint), so the clamp is
  DELETED - the re-validation contract's precondition for the
  E-gates, done early.
- K1.1 re-validated per the contract (the integrator binding):
  the original window letter FAILED (+32% KE/atom, 2k-6k vs
  6k-10k) and the measurement showed why - the quiesced
  chemistry starved the old refrigeration (F18 below), so the
  vent + setpoint reservoir warm the field over ~6k ticks and
  KE/atom rides the recovery. The thermostat itself held
  perfectly: the coupling ratio KE/atom / (kB * field warm-cell
  average) measured a constant 1.10-1.13 at every sample through
  the transient. Re-validated criterion: bounded throughout +
  the coupling law (ratio in [0.8, 1.4]) + flatness over the
  steady tail (6.25k-8k vs 8.25k-10k: KE +10.6%, bond length
  +0.3%). Golden hash consciously updated
  (0xDD4E_87CD_A7FD_94CE -> 0x0896_8E9C_98C9_54F6).
- F18 (OPEN, owned by K1.4): wide-capture churn. Pairs formed
  inside bond_search_radius (4 A) but beyond the 7.1 break
  length (2.5 * r_eq) are phantom captures - they break
  silently on the next chemistry pass, and each such cycle
  absorbs formation_fraction * E from the field with no return
  (measured: 26 of 41 runtime-formed O-H pairs broke
  mechanically; the spring PE minted at wide-capture formation
  flings them at up to ~4.2 A/tick relative). Corollary measured
  in the same run: the old substrate's refrigeration machine
  (shatter feeding formations, each absorbing 0.3*E, field
  avg -162 C) starved itself under the K1.3 substrate - the
  field now RECOVERS to ~26-28 C vs the 35 C setpoint over ~6k
  ticks instead of sitting at -162. K1.4 owns whether the
  residual deficit closes and the molecule-size distribution
  settles; its levers include the search radius, the formation
  fractions, and the vent/setpoint balance.
- Substrate cost measured: 10k-tick pond runs went from ~77 s to
  ~189 s in release (~53 t/s) - 4x force passes plus per-sub-step
  index rebuilds. The phase-2 perf pass owns this (PLAN: no
  optimization before the substrate is behaviorally sane; the
  gates are correctness gates).

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

## 10. Thermostats: the missing bath (finding F8 - RESOLVED, K1.1)

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

IMPLEMENTED 2026-09-05 as gate K1.1 (PASS): the exact form above
plus signed-delta bookkeeping, the setpoint reservoir, and the
velocity clamp (findings log).
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