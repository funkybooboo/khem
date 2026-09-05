# khem

khem is an artificial-chemistry runtime and a small language for seeding
worlds with simple molecular life, then watching what the rules do.

Conway's Game of Life gave us a grid, a handful of rules, and seeded
patterns - and complexity emerged. khem asks the same question one level
down: in a 2D world of atoms with real valence, bond energies, VSEPR
geometry, thermal noise, and a few energy sources, what happens when you
seed a minimal RNA-world cell - an RNA strand inside a lipid vesicle -
and let the rules run for a billion ticks?

khem names the language, its files, and its runtime, the way C does.
You write `.kem` definition files (elements, structs, chains, bodies,
worlds, runs). The `khem` runtime flattens them into atoms and bonds,
executes ticks, and streams newline-delimited JSON events to stdout.

    khem experiment_1.kem > run_001.ndjson

## Status

Pre-prototype. Canonical specs, architecture decision records, a
build plan, a research reading list, and a runtime scaffold exist.
The simulation engine is a data-model skeleton; the systems are the
plan's phase 1. Read PLAN.md before writing any - the build order is
kernel first, language later. The founding conversation is preserved
in git history only (ADR-0010).

## The name

Pronounced "kem". khem is the root of the word "chemistry" itself: per
one of the leading etymologies, al-kimiya (Arabic alchemy) descends from
Egyptian kemet, "the black land" - the fertile mud of the Nile, where
water, soil, and sun made things grow. A chemistry language named after
the fertile black mud. The etymology is contested; the name is not the
claim. The naming decision and the rejected-candidates record live in
docs/adr/0007-name-khem.md; the conversation-era rename map lives in
docs/specs/README.md.

## What this project is

- A bottom-up 2D artificial chemistry. Atoms carry real element
  properties (valence, electronegativity, mass, covalent radius). Bonds
  form and break by Boltzmann/Arrhenius-style probabilities against
  lookup tables of real bond energies and VSEPR angles. Temperature,
  pressure, and UV fields evolve. Energy sources (hydrothermal vents,
  solar UV) drive the system away from equilibrium.
- Seeded, not spontaneous. Worlds start with a minimal cell built
  purely from atoms and bonds. Nothing above the atom/bond level is
  coded into the runtime; replication, mutation, membrane division, and
  evolution are supposed to fall out of the rules, not be programmed in.
- A DSL + runtime pair in the Verilog tradition. Definition files
  compose (water -> nucleotide -> RNA strand -> cell -> pond ->
  experiment) and are testable in isolation. The runtime knows atoms,
  bonds, fields, energy - nothing else.
- Terminal-first, laptop-first. NDJSON to stdout, diagnostics to
  stderr, viewers are separate pipe consumers. Deterministic seeds make
  runs reproducible. Save/load lets long experiments span sessions. No
  GPU required; the architecture leaves room for threading and
  distribution (V2/V3) without depending on them.
- A candidate thesis project (see PLAN.md, "Thesis track").

## What this project is not

- Not a game. No objectives, no balancing, no scripted organisms.
- Not a faithful chemistry or physics simulator. No quantum mechanics,
  no femtosecond molecular dynamics. The dynamics are phenomenological:
  a physics-flavored artificial chemistry. The specs in docs/specs/
  use real constants and real equations where cheap, and honest
  simplifications everywhere else.
- Not a claim of abiogenesis. Life is seeded by hand. The open question
  is whether interesting evolution emerges from the substrate - and the
  plan gates all further work on that question (PLAN.md, phase 1).
- Not pre-coded biology wearing a physics costume - with one caveat the
  project admits openly: some rules (permeability, base-pair geometry
  targets) smell like smuggled biology. The design docs flag them
  rather than pretending they emerged.

## Relationship to prior work

khem assembles pieces that all exist separately; nobody has combined
them. Annotated bibliography with links and DOIs:
docs/research/references.md.

| Project | What it is | khem builds on | khem differs |
|---|---|---|---|
| Kappa (kappalanguage.org) | rule-based language for interacting molecular agents + KaSim simulator | DSL/runtime split; rules over agents with binding sites (khem's ports/wire) | models known biochemistry at protein granularity; no space, energy, or evolution |
| BioNetGen (github.com/RuleWorld/bionetgen) | rule-based biochemical modeling | rule composition, network-free simulation | reaction networks, no spatial matter substrate |
| SBML (sbml.org) | standard exchange format for biochemical models | auditable text models consumed by many runtimes | describes known networks; no emergence |
| MCell + MDL (mcell.org) | spatial stochastic particle biochemistry with a model description language | MDL precedent, reaction-diffusion in space | abstract molecule species, not bonded atoms |
| LAMMPS + ReaxFF (docs.lammps.org) | reactive molecular dynamics, real bond-order potentials | atoms forming/breaking bonds under physics | femtosecond fidelity, orders of magnitude too slow for evolution; no DSL, no observer |
| SimSoup (simsoup.info) | artificial chemistry for origin-of-life research | the motivation; structure-driven molecule properties | molecule-type interaction networks; no atom/bond spatial substrate |
| Stringmol (stringmol.york.ac.uk) | automata chemistry for molecular evolution | artificial chemistry + evolution in silico | molecules are strings, not spatial atoms |
| Avida (avida.devosoft.org) | digital evolution platform | seeded self-replicators, open-ended evolution, measurement discipline | organisms are programs competing for CPU cycles; no matter or chemistry |
| The Bibites (thebibites.com) | real-time artificial life with neural-net creatures | the "watch evolution happen" experience goal | organism abstractions pre-coded; GUI-first, not a substrate |
| Ribossome (github.com/Manalokosdev/Ribossome) | GPU evolution sim, RNA-inspired genome-to-body translation | Rust runtime, RNA-world inspiration, emergent ecosystems | abstract codon genetics, GPU-bound, biology pre-programmed |
| Primordial (github.com/itzrnvr/primordial) | browser origin-of-life sim | seeded cells around a hydrothermal vent | JS/3D, biology pre-programmed, no chemistry substrate |
| Genesis Engine (github.com/AVADSA25/genesis-engine) | protocell study with Monte Carlo + published preprint | protocell dynamics in warm-pond-class models | single-hypothesis study, not a platform; its withdrawn headline result is khem's methodological cautionary tale |
| lifeSimulatoR (github.com/NoushinN/lifesimulatoR), protocell sims (github.com/chrisk60331/protocell-simulation) | simplified origin-of-life scenario models | protocell kinetics, OoL motivation | equation/statistical layer, not a runtime or language |
| Conway's Game of Life / Golly (golly.sourceforge.net) | cellular automata engine | the whole framing: minimal rules + seeded patterns -> emergence | no chemistry, genomes, or evolution |

How khem builds on prior work, compressed:

- From Conway: rules + seeded patterns, emergence as the only content.
- From artificial chemistries (Dittrich taxonomy, SimSoup, Stringmol):
  chemistry networks as the ground layer of evolution; molecule
  properties derived from structure.
- From RNA-world theory and the protocell program (Gilbert, Joyce,
  Szostak, Ganti): the exact seed - replicator plus compartment.
- From rule-based modeling (Kappa, BioNetGen, SBML, MCell's MDL): the
  DSL + runtime split and grammar lessons for .kem.
- From reactive MD (ReaxFF): the fidelity ceiling this project steps
  back from - real reactive atom chemistry is six-plus orders of
  magnitude too slow for evolutionary timescales.
- From digital evolution (Avida): measurement discipline for open-ended
  evolution.
- From Unix: do one thing, stream structured output, compose with pipes.

## Repository layout

    khem/
    |-- README.md            this file: identity, prior work, is / is not
    |-- PLAN.md              build order: kernel first, language later
    |-- ARCHITECTURE.md      target crate map, dependency rules, scaling plan
    |-- mise.toml            toolchain pin + tasks (mise run check = the CI gate)
    |-- docs/
    |   |-- adr/             architecture decision records (the WHY)
    |   |-- specs/           canonical language + runtime specs
    |   |   `-- README.md    spec index + conversation-era rename map
    |   `-- research/
    |       `-- references.md  papers + projects, mapped to design choices
    `-- crates/
        |-- khem-core/       lib: the simulation engine (phase 1 lands here)
        `-- khem/            bin: the runtime CLI (scaffold)

Everything in the repo is ASCII. The founding conversation and the
conversation-era spec drafts are preserved in git history only
(ADR-0010).

## Development

    mise install          # pinned toolchain (rust 1.98.0, from mise.toml)
    mise run build        # cargo build --workspace
    mise run test         # cargo test --workspace --locked
    mise run fmt           # format
    mise run lint          # clippy, warnings are errors
    mise run check         # the CI gate, locally

CI (.github/workflows/ci.yml) runs `mise run check` on every push and
pull request at github.com/funkybooboo/khem (private, ADR-0011), using
the same mise-pinned toolchain.