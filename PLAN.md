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

Validation gates - all must pass before any parser work:

    K1  STABILITY: water and small molecules persist at moderate
        temperature. Bonds form and break at plausible rates - no
        runaway crosslinking, no frozen inertness.
    K2  SELF-ASSEMBLY: lipids in water aggregate head-out (amphipathic
        behavior emerges from polarity rules, not a "form membrane"
        rule).
    K3  REPLICATION: free nucleotides bond to a seeded RNA strand by
        base-pair geometry; strands separate thermally; copies happen.
    K4  VARIATION: copy errors occur at tunable rates and produce
        distinguishable daughter strands.
    K5  SELECTION: lineages with different copy fidelity or speed show
        different survival in a resource-limited pond (populations
        grow, crash, recover).

Exit criterion: if K1-K3 do not pass after honest parameter sweeps
(weeks, not days), stop and redesign the substrate before building
anything on top. A beautiful language on a dead substrate is
worthless.

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

- parameter sweeps: mutation rate vs fidelity, resource scarcity, UV,
  temperature, pond size
- lineage tracking: phylogenies reconstructed from NDJSON event logs
- candidate research questions (preregister metrics BEFORE looking):
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
- The WHAT lives in docs/specs/ (canonical, current-state specs,
  edited in commits and revised against phase-1 reality per ADR-0006)
  and ARCHITECTURE.md (crate map).
- The gate is `mise run check` locally and identical in CI; the
  toolchain is pinned in mise.toml.
- The founding conversation is recoverable from git history only
  (ADR-0010): the transcript at commit d8205f1, the spec-draft
  extractions at 83a2688 and fefc4b9.

## Open decisions (owner: nate)

- [ ] thermostat: Langevin-style damping toward the local field
      temperature (abstraction-notes section 10) - the measured K1
      blocker. Proposal: v <- v*(1-damping) + normal(0, sigma(T));
      region declarations become bath setpoints. One knob,
      spec-6.1 revision, harness-gated (KE and bond length must go
      flat)
- [ ] non-bonded soft repulsion (finding F4): drafted before K2
      work, lands only with harness evidence, as its own commit
- [ ] first world file name: primordial_pond.kem ("warm little pond"
      is Darwin's phrase for the setting)
- [x] license: RESOLVED 2026-09-05 - MIT (LICENSE at root, SPDX MIT in
      crate metadata; ADR-0012)
- [x] remote hosting: RESOLVED 2026-09-05 - github.com/funkybooboo/khem,
      public (ADR-0011; CI green on the very first push)
- [x] phase 1 placement: RESOLVED 2026-09-04 - kernel code lands in
      the khem-core lib, driven by the khem bin on main (ARCHITECTURE.md)