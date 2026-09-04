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

A single Rust binary with everything hardcoded:

- element table (the 10 elements with real values, from the idea file)
- the tick loop from the runtime spec draft (05-runtime-spec.md):
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
  conditions, NDJSON schema per 05-runtime-spec.md with the renames
  from 06 applied
- performance: meet the spec targets (10k atoms at >500 t/s on a
  laptop)
- unit tests per system + golden-tick regression tests

## Phase 3 - the khem language (only what the kernel needs)

- .kem parser for the declarations: element / struct / chain / body /
  world / run (grammar from docs/specs/, revised against Kappa lessons
  and phase-1 reality; the drafts are drafts, not scripture)
- validation rules (V01-V15 from the specs) become tests
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

## Open decisions (owner: nate)

- [ ] license (MIT suggested for a thesis-adjacent research artifact)
- [ ] remote hosting (github vs codeberg) - repo is local-only for now
- [ ] first world file name: primordial_pond.kem ("warm little pond"
      is Darwin's phrase for the setting)
- [ ] whether phase 1 prototype lives on a branch or in this repo
      behind a temporary binary