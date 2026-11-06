# Contributing to khem

## The gate

`mise run check` is the single source of truth - identical in CI:

    mise run check

It runs `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, and `cargo test --workspace
--locked`. Rust is pinned via mise.toml; use `mise exec --` to
run any cargo command under the pinned toolchain.

The long-running gate tests are `#[ignore]`d (10-25 minutes
each). They run in the CI matrix, one per job. Locally:

    mise exec -- cargo test --release -p khem-engine --test water_molecules_persist -- --ignored --nocapture

## How work happens

- The project runs on a gate ladder (K1-K5): harness tests with
  measured pass criteria, one rung at a time. The ladder, its
  pass records, and the methodology live in
  [plans/README.md](plans/README.md) and
  [plans/gates/](plans/gates/).
- The re-validation contract is enforced: when your work moves a
  passed gate's operating assumptions, the gate re-runs in your
  commit, and its result is part of your evidence.
- The implementation and the specs must AGREE: any divergence
  between code and docs/build/specs/runtime-spec.md is fixed in
  both in the same commit.
- The WHY of every decision gets an ADR in
  [docs/build/adr/](docs/build/adr/) (Nygard format:
  Context / Decision / Consequences; immutable once accepted -
  a changed decision gets a new ADR).
- Findings (measured surprises) land in
  [docs/build/research/abstraction-notes.md](docs/build/research/abstraction-notes.md)
  with an ID and a named owner gate.

## Hard constraints

- Zero registry dependencies, deliberately. Adding one requires
  an ADR and an owner decision.
- No GPU requirement, no graphics in the runtime, no concept
  above the atom/bond level in the engine, no pre-programmed
  biology (G01; document honest shortcuts in the spec instead).
- ASCII only in code, docs, and commit messages.
- Determinism by construction (ADR-0005): same seed, same
  stream. The golden hash pins it.

## The crates

    khem-forcefield   the physics parameters as data: element table, bond energies, the grouped ForceField config
    khem-world        the data model: atoms, bonds, fields, spatial index, RNG, the Event contract
    khem-physics      the force laws, the integrator, the thermostat, the energy sources
    khem-chemistry    bonds form and break by law
    khem-analysis     the observables library (census, rdf, order parameters)
    khem-engine       the tick loop, the observer, the NDJSON stream; the gate harnesses
    khem-worlds       world constructors (the pond, the beakers)
    khem              the CLI

Tests follow the simulator pyramid:

- law tests: in-crate unit tests (and doctests) against analytic
  bounds - every build;
- scenario/invariant tests: small worlds, conservation and
  determinism properties - every build;
- regression: the golden hash - every build;
- world/soak gates: `#[ignore]`d, CI matrix (see
  [.github/workflows/gates.yml](.github/workflows/gates.yml)).

## Commit messages

House style: a summary line (`area: what and why`), then a body
with the measured evidence and the reasoning. Behavior changes
say so; refactors prove behavior unchanged (the golden hash).
Gate commits cite their gate ID in a doc comment, never in the
code names.
