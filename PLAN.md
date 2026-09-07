# khem project plan

## The one question this project hinges on

The founding conversation produced two language specs, a runtime spec,
a CLI, an event schema, and a naming system - and zero simulated ticks.
Everything hinges on an unvalidated assumption: that a simplified
atom/bond substrate (lookup-table bond energies, VSEPR angles,
Arrhenius/Boltzmann probabilities, thermal noise) produces dynamics
interesting enough to evolve a seeded cell.

So the build order is deliberately backwards from the founding
conversation: kernel before language, physics before parsers, evidence
before specs.

## Where this repo stands (2026-09-05, end of the phase-1 kernel session)

Handoff snapshot; the phases below are the plan, this is the state:

- canonical specs (docs/specs/) and twelve ADRs (docs/adr/); the
  founding conversation is preserved in git history only (ADR-0010)
- the phase-1 kernel is BUILT and runs end to end: physics,
  chemistry (spec 7.3/7.4 tables + Boltzmann/UV breaking,
  geometry/temperature/EN-gated formation), energy, observer with
  union-find molecule detection, hand-rolled NDJSON v:1 emitter,
  the Sim nine-step tick loop, the hardcoded pond, and the khem bin
  streaming real output (release: ~680 t/s at 3.4k atoms)
- the K1 harness measured the substrate twice (literal constants,
  then tuning round 1); all findings live in
  docs/research/abstraction-notes.md (F1-F9)
- K1 IS NOT PASSED. Measured structural blocker: the spec has no
  dissipation channel, so additive thermal kicks random-walk energy
  up forever (KE 3.6e14, mean bond length 63 A after 2000 ticks);
  strong_repulsion/r^2 compounds it (v ~ 1e4 per overlap). Constants
  cannot fix this; the Langevin thermostat proposal
  (abstraction-notes section 10) awaits the owner decision below
  - the gate ladder names it K1.1
- toolchain pinned in mise.toml; `mise run check` green locally
  (fmt, clippy, 70 tests) and identical in CI
- hosted at github.com/funkybooboo/khem, public (ADR-0011)

## Phase 0 - literature grounding (in progress)

Goal: steal every abstraction lesson prior work already paid for.
Reading list and mapping in docs/research/references.md. Priorities:

- Dittrich, Ziegler, Banzhaf (2001) - the artificial-chemistry
  taxonomy; the vocabulary for the whole field
- SimSoup papers - structure-driven molecule properties, the same
  philosophy at molecule-type level
- Kappa manual - the grammar of rules over agents with sites; what to
  keep and what to drop from the .kem grammar
- Ganti (2003) - the chemoton: container + metabolism + information;
  the theoretical minimal cell the seed approximates
- Szostak/Bartel/Luisi (2001) + Chen et al. (2004) - what a minimal
  cell must do physically (osmotic growth, division, competition)
- The Genesis Engine correction notice - how simulation studies fool
  themselves; required before designing any watch-condition detector

Output: notes in docs/research/ recording, for each abstraction khem
uses, what prior work supports it and what it simplifies away.

## Phase 1 - physics/chemistry kernel (NO DSL)

The kernel lives in the khem-core lib; the khem bin stays a thin entry
point (see ARCHITECTURE.md). Crate setup has already landed the data
model, element table, deterministic RNG, spatial index, physics
constants, and the CLI skeleton; phase 1 is the systems themselves.
Everything is hardcoded:

- the tick loop from the runtime spec (docs/specs/runtime-spec.md):
  energy -> velocities -> positions -> boundary -> spatial index ->
  bond breaking -> bond formation -> observe
- a hardcoded primordial pond + one hardcoded minimal cell
- NDJSON events to stdout (tick + bond events + watch conditions)

Validation gates - the ladder below is the project's spine, and
all of it must pass before any parser work. A milestone (K_n)
passes when its sub-gates do; a sub-gate is a harness test with
a measured pass criterion - pass it, commit it, move to the
next. Each milestone gets a harness file in tests/ (k1_stability.rs
exists; k2_self_assembly.rs, k3_replication.rs, k4_variation.rs,
k5_selection.rs follow). Constants retune inside a sub-gate as
the harness demands (F1's lesson), but no sub-gate may be
passed by adding a rule above the atom/bond level (G01).
Thresholds are starting points; they move with evidence, never
by wish. Gates are measured by the harness, never by eyeballing
a viewer.

RE-VALIDATION CONTRACT (owner decision 2026-09-07): a passed
gate is passed against its operating assumptions; when a later
gate's work moves those assumptions, the earlier gate RE-RUNS in
that same commit - the re-run result is part of the later gate's
evidence, not an optional follow-up. Current bindings:
- K1.1 re-runs in K1.4's commit (K1.4 moves the pond's settled
  operating temperature; K1.1 passed at the cold ~0.3 C point)
- K1.1 re-runs in any integrator commit (sub-stepping removes
  the velocity clamp and deepens bond wells - K1.3's lever;
  the golden hash forces such commits to be conscious)
- the velocity clamp (tunneling mint guard) must be REMOVED
  before the E-gates run their long horizons - the mint is
  bounded, measured, and absorbed by the reservoir, but it
  accumulates over million-tick experiment runs
- no soft spot exists only in prose: every known limitation
  lives either in this contract or as a finding in
  docs/research/abstraction-notes.md with a named owner gate

    K1  STABILITY - the substrate holds together
        [measured 2026-09-05: FAILING; the sub-gates name the
        measured causes - F6 through F9, F11]

    K1.1 THERMOSTAT: Langevin damping toward the local field
        temperature. PASSED 2026-09-05: KE/atom window drift -5.7%,
        mean bond length +0.5% over the 10k-tick vented run
        (tests/k1_stability.rs, release --ignored). The pass
        required the substrate corrections F8-F16 (findings log in
        docs/research/abstraction-notes.md) and the setpoint
        reservoir + vent (spec 6.1/6.2/11 synced).
    K1.2 FORCE SANITY: a bonded overlap imparts bounded velocity
        (F9 measured v ~ 1e4 - a cannon). PASS: mean bond length
        stays within [0.8, 1.5] * r_eq over the same run.
    K1.3 WATER PERSISTS: a 35 C pond of H2O keeps its molecules
        - intact count flat, O-H essentially never breaks (real
        chemistry's own exp(-29) answer), the form+break cycle
        mints no energy (the F7 regression stays green).
    K1.4 REACTIVE BALANCE: a beaker of free atoms settles to a
        STATIONARY molecule-size distribution - weak bonds break
        (O-O on a ~10k-tick scale), strong ones persist; no
        runaway crosslinking, no frozen inertness; formation
        refrigeration (F6) stays bounded and recovers.
    K1.5 SEAM CORRECTNESS: the spatial index wraps in Wrap worlds
        (F11) - cross-seam formation is symmetric with the bulk.
        Originally queued for phase 2; promoted, because a Wrap
        world with asymmetric formation cannot pass honest gates.

    K2  SELF-ASSEMBLY - membranes are consequences, not rules
        [precondition: K1; the literature is unanimous that
        amphiphiles assemble through non-bonded potentials,
        never through springs alone - hence K2.1 comes first]

    K2.1 EXCLUDED VOLUME: soft non-bonded repulsion (F4 - smuggled
        PHYSICS, documented as such): free atoms no longer pass
        through each other. PASS: minimum approach distance
        >= 0.8 * (r_a + r_b) in a scattering test.
    K2.2 CONDENSED MEDIUM: the pond behaves as matter, not free
        flight - most atoms hold a non-bonded neighbor within
        3 A, and pair distances show structure.
    K2.3 AMPHIPHILE SORTING: a lipid (polar head, nonpolar tails)
        in water - head-water and tail-tail contact fractions
        beat chance by a set margin. No "membrane" rule exists
        anywhere in the runtime.
    K2.4 VESICLE CLOSES: lipids form a persistent cluster with an
        interior (union-find: cluster >= 12 lipids, survives
        >= 10k ticks), heads pointing outward.
    K2.5 CONTENTS HELD: free nucleotides placed inside a vesicle
        stay inside above a leak threshold.
    K2.6 VESICLE GROWS: a fed vesicle incorporates lipids and
        grows (Squirm3 lesson: membranes must grow, not just
        close). Division is NOT gated in v0.1 - it is the first
        thing to chase after K5.

    K3  REPLICATION - copying is chemistry, not code
        [precondition: K2.5; the strand lives in a vesicle with
        free nucleotides]

    K3.1 PAIRING: free nucleotides bond the correct complement
        (A-U, G-C) far more often than the wrong one (starting
        threshold 4:1) in a minimal beaker. Base-pair geometry
        targets stay flagged as smuggled biology (ADR-0003's
        honesty rule).
    K3.2 TEMPLATING: a seeded strand acquires a full complement
        - >= 80% of bases paired within a fixed tick budget.
    K3.3 SEPARATION: a thermal window exists - pairing holds at
        T_low, the duplex releases at T_high - and the window
        moves by retuning constants, not by adding rules.
    K3.4 THE COPY: template + free nucleotides + thermal cycling
        yields a free daughter strand (complement signature in
        the event stream), and the daughter templates a second
        generation.

    K4  VARIATION - copies carry errors

    K4.1 COPY ERRORS: wrong-base incorporation happens, and the
        measured mismatch rate tracks a single tunable constant
        across a sweep.
    K4.2 VIABLE MUTANTS: most single-substitution daughters still
        copy (the K3.4 criteria) - variation is not instantly
        lethal.

    K5  SELECTION - the pond has ecology
        [precondition: K4; long runs - the phase-2 perf target
        and dead-slot compaction are what make K5 practical]

    K5.1 COMPETITION: two lineages (differing fidelity or speed)
        share one pond with capped nucleotide supply - both
        copy, one wins by a preregistered margin. Needs lineage
        identity in the harness.
    K5.2 TURNOVER: with material feed + decay (the Squirm3 lesson
        - selection starves without turnover), populations grow,
        crash, recover, persisting for many generations without
        extinction or monoculture takeover.
    K5.3 NOVELTY PROBE: reconstructed lineage trees keep producing
        new sequences over time - open-endedness or convergence,
        measured against a metric preregistered BEFORE looking
        (the Genesis Engine rule).

Exit criterion: if K1-K3 do not pass after honest parameter sweeps
(weeks, not days), stop and redesign the substrate before building
anything on top. A beautiful language on a dead substrate is
worthless. Sub-gates are sized to be attackable - a sitting to a
week each; if one sprawls, split it.

Explicit non-goals for phase 1: no parser, no .kem files, no plugins,
no CLI flags beyond --seed. Hardcode everything. The kernel is
disposable; the answer to the K-gates is not.

## Phase 2 - runtime hardening

- deterministic seeded RNG everywhere (reproducibility guarantee)
- save/load world state (a resumed run is identical to an uninterrupted
  one)
- observer: molecule detection via union-find on the bond graph, watch
  conditions, NDJSON schema per docs/specs/runtime-spec.md
- performance: meet the spec targets (10k atoms at >500 t/s on a
  laptop)
- unit tests per system + golden-tick regression tests

## Phase 3 - the khem language (only what the kernel needs)

Starts only after phase 1's K1-K5 gates pass (ADR-0006).

- .kem parser for the declarations: element / struct / chain / body /
  world / run (grammar from docs/specs/language-spec.md, revised
  against Kappa lessons and phase-1 reality; specs are drafts until
  validated, not scripture)
- validation rules (V-STRUCT/V-CHAIN/V-BODY/V-WORLD/V-RUN codes from
  language-spec.md) become tests
- stdlib of primitive .kem files: water, phosphate, ribose, bases,
  nucleotides, lipid, vesicle, rna strand, minimal cell, primordial
  pond
- --check / --test / --info CLI modes
- everything phase 1 hardcoded becomes a .kem file

## Phase 4 - experiments and (maybe) the thesis

The K-gates prove the substrate; the E-gates prove the instrument
- that khem can run real evolution experiments. Each E-gate is a
platform capability, built in the order the experiments need them
(E1 is phase 2's G09 wearing its working clothes):

    E1  SAVE/RESUME: a resumed run is byte-identical to an
        uninterrupted one (G09) - long experiments span sessions.
    E2  LINEAGE TRACKING: event logs reconstruct phylogenies -
        every copy event a parent/child edge, every mutation
        labeled on the edge it changed.
    E3  SWEEPS: a batch runner executes a parameter grid (mutation
        rate, UV, temperature, pond size, scarcity) across seeds
        and collates the results.
    E4  CONTROLS: negative controls are first-class runs -
        no-template, no-UV, dead-strand. Anything that cannot
        report "no" is not a detector (the Genesis Engine rule).
    E5  FIRST EXPERIMENT: mutation rate vs copy fidelity, one
        preregistered prediction, an ablation arm (bond-table
        perturbation), and the full writeup - the thesis-track
        dress rehearsal.

Candidate research questions (preregister metrics BEFORE looking):
  - does evolution of the seeded cell produce open-ended genome
    diversity, or converge to a dominant strain? under which
    parameters?
  - how sensitive are outcomes to substrate abstraction choices
    (bond-energy tables, angle tables, formation rates)?
  - do watch conditions correlate with anything independently
    measurable?

## Thesis track

If this becomes a thesis, the honest framing is: "an
artificial-chemistry platform for studying emergent evolution of
seeded minimal cells" - NOT "simulating abiogenesis". A defensible
thesis needs preregistered metrics, ablations, and negative controls.
Read the Genesis Engine correction notice first: their 100% headline
result was a detector artifact, and their audit trail is the best
available example of how that class of mistake happens in exactly
this kind of simulation. Design so it cannot happen here. The
literature-review skeleton is docs/research/references.md.

## Non-goals and guardrails

- no GPU requirement, ever (V1); threading/distribution are optional
  future work (V2/V3), not dependencies
- no graphics in the runtime - viewers are separate pipe consumers
- no pre-programmed biology above atom/bond rules; where a rule smells
  like smuggled biology (permeability, division thresholds), document
  it honestly in the spec instead of pretending it emerged
- no claims of real chemistry: lookup tables and phenomenological
  rates are the design, not a compromise to hide
- scope discipline: the founding conversation ballooned from "900
  lines of Python" to a full language spec before one tick ran. This
  plan exists to prevent a repeat.

## Beyond v0.1 (horizon, not scheduled)

All of this is enabled by choices already fixed (runtime spec section
10; ARCHITECTURE.md) and none of it is on the critical path:

- V2: thread-per-region with ghost cells - flat arrays, integer IDs,
  and the fixed tick order make this a partitioning problem, not a
  redesign
- V3: region-per-machine distribution - reuses the save/load
  serialization; NDJSON output unchanged
- plugins: dynamic loading of PhysicsSystem/ChemistrySystem trait
  implementations
- tool family: khem-view, khem-log, khem-check, khem-build - one bin
  per crate, zero workspace dependencies, consuming the NDJSON
  contract (ADR-0004, ADR-0008)
- publishing the khem crate to hold the crates.io name (ADR-0007),
  when there is something real to publish
- 3D: a port, not a redesign, if 2D results ever justify it

## How this plan is maintained

- The WHY of every decision lives in docs/adr/ (Nygard format;
  immutable once accepted - change means a new ADR).
- The implementation and the specs must AGREE (owner decision
  2026-09-05): any divergence between code and
  docs/specs/runtime-spec.md is fixed in both in the same commit,
  piecemeal. Designed-but-unbuilt items are marked [phase 2] /
  [phase 3] in the spec rather than allowed to drift.
- The WHAT lives in docs/specs/ (canonical, current-state specs,
  edited in commits and revised against phase-1 reality per ADR-0006)
  and ARCHITECTURE.md (crate map).
- The gate is `mise run check` locally and identical in CI; the
  toolchain is pinned in mise.toml.
- The founding conversation is recoverable from git history only
  (ADR-0010): the transcript at commit d8205f1, the spec-draft
  extractions at 83a2688 and fefc4b9.

## Open decisions (owner: nate)

- [x] thermostat (gate K1.1): RESOLVED 2026-09-05 - PASSED.
      Langevin damping toward the local field temperature with
      exact signed-delta bookkeeping, the setpoint reservoir, and
      the velocity clamp; see the K1.1 entry above and the
      findings log. Spec 6.1/6.2/11 synced.
- [ ] non-bonded soft repulsion (finding F4; gate K2.1): IMPLEMENTED with the K1.1 commit (spec 6.5, chemistry/physics tests); the K2.1 scattering-test PASS run is the gate's own
      work, lands only with harness evidence, as its own commit
- [ ] first world file name: primordial_pond.kem ("warm little pond"
      is Darwin's phrase for the setting)
- [x] license: RESOLVED 2026-09-05 - MIT (LICENSE at root, SPDX MIT in
      crate metadata; ADR-0012)
- [x] remote hosting: RESOLVED 2026-09-05 - github.com/funkybooboo/khem,
      public (ADR-0011; CI green on the very first push)
- [x] phase 1 placement: RESOLVED 2026-09-04 - kernel code lands in
      the khem-core lib, driven by the khem bin on main (ARCHITECTURE.md)