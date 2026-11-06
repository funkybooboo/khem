# AGENTS.md

Rules for AI coding agents working in this repository. Humans:
[CONTRIBUTING.md](CONTRIBUTING.md) covers the same ground for you.

## What this repo is

khem is a bet: simplified atoms and bonds, honest and cheap
physics, and evolution from a seeded cell. The constitution is
[plans/identity.md](plans/identity.md) - read it before
proposing anything. The fidelity philosophy (ADR-0014):
real-world likeness via config knobs, as cheaply as possible.

## The gate

- `mise run check` is the ONLY gate: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`. Identical in CI. Rust is
  pinned in mise.toml; do not touch the version without an ADR.
- The long-running gate tests are `#[ignore]`d locally and run
  in the CI matrix (one gate per matrix entry). Run one locally
  with `mise exec -- cargo test --release -p khem-engine --test
  <file> -- --ignored --nocapture`.
- The golden hash (khem-engine's sim tests) pins tick-loop
  determinism. Behavior-neutral refactors keep the hash
  byte-identical; any behavior change re-cuts it via the
  documented procedure and says so in the commit message.

## House rules

1. **Spec/code agreement**: any divergence between the code and
   docs/build/specs/runtime-spec.md is fixed in both in the same
   commit, piecemeal.
2. **Re-validation contract** (plans/README.md): when a later
   gate's work moves an earlier gate's operating assumptions, the
   earlier gate re-runs in the same commit, and the result is part
   of the evidence.
3. **Zero registry dependencies** is deliberate (instant CI, no
   supply chain, in-crate determinism). Adding any dependency
   needs an ADR and an owner decision.
4. **K-numbers** (K1.1 ... K5.3) are the ladder's cross-references
   - they live in plans/ and in doc comments as lineage, NEVER in
   file, function, or test names. Tests are named for the
   behavior they certify.
5. **Leaf knob names** (`non_bonded_margin`, `cohesion_scale`,
   ...) are the historical currency of the evidence chain
   (PASS records, findings, spec sections) - they do not get
   renamed, only regrouped.
6. **ASCII only** in everything the repo writes: code, docs,
   commit messages. No emoji, smart quotes, em dashes, box
   drawing. `->`, `--`, `...`, `[OK]`.
7. No concept above the atom/bond level in the engine (G01); no
   pre-programmed biology - where a rule smells like smuggled
   biology, document it in the spec honestly instead.
8. Docs live by role: plans/ (the bet, the ladder, the queue),
   docs/build/ (architecture, specs, ADRs, research). The WHY of
   every decision gets an ADR (Nygard format, immutable once
   accepted - change means a new ADR).

## Where things live

    crates/khem-forcefield    elements + bond tables + the grouped ForceField config
    crates/khem-world         world state, fields, spatial index, RNG, the Event model
    crates/khem-physics       forces, integrator, thermostat, energy sources
    crates/khem-chemistry     bond formation/breaking laws
    crates/khem-analysis      observables: census, rdf, order parameters
    crates/khem-engine        tick loop, observer, NDJSON; the gate tests live here
    crates/khem-worlds        world constructors (the pond)
    crates/khem               the CLI bin

Dependency direction: forcefield <- world <- {physics, chemistry,
analysis}; engine <- all; worlds <- world+forcefield; bin <-
engine+worlds. Keep it acyclic; world data types (Event,
EnergySource) live DOWN here, behaviors live UP.
