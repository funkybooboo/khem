# Engineering idioms - the restructure's research basis

Date: 2026-11-06 (owner-directed research for the repository
restructure; findings reviewed and accepted the same day -
"do your lean on all three")

The owner directive: an idiomatic repo following best practices.
Four research questions, answered against the literature and
the mature codebases of each domain, then applied to khem.

## 1. Rust idioms (what the ecosystem calls a proper repo)

Surveyed: the Rust API Guidelines (rust-lang), the workspace
structures of bevy, rapier, and tokio.

- **Workspace of small, prefixed, single-responsibility
  crates** (`bevy_ecs`, `bevy_render`...) with a thin facade
  binary. khem's prefix is `khem-`; the monolith `khem-core` is
  the outlier to fix.
- **Doc comments with runnable examples** - doctests are free
  tests, and a physics crate is the ideal case: every force law
  has a closed-form case worth asserting in the documentation
  itself.
- **Grouped, documented config with `Default`** - nested
  structs that mirror the physics (a force-field
  specification, not a flat bag of 25 constants), every field
  carrying its unit and its measured "why" in the doc comment.
- **Newtypes for identifiers** (khem already has AtomId,
  BondId, ElementId) - extended with unit newtypes (Angstrom,
  KiloJoulePerMole) only if unit bugs ever bite; deferred as
  high-churn, low-current-value.
- **Zero registry dependencies** is unusual for idiomatic
  Rust (serde and thiserror are standard), but deliberate
  here: instant CI, no supply chain, determinism fully
  in-crate, and the hand-rolled ndjson already proves the
  pattern works. Becomes a written guardrail, not an accident;
  revisited when the phase-4 parser needs a real choice.

## 2. Molecular-dynamics idioms (GROMACS, LAMMPS, OpenMM)

The mature codes converge on one decomposition, and khem has
independently reinvented most of it - the restructure names
what already exists:

- **The force field is data, separate from the engine.** In
  MD, "force field" = the parameter tables that drive generic
  force laws. khem's element table, bond energies, VSEPR
  angles, and interaction constants ARE a force field -
  scattered today across elements.rs, chemistry.rs, and
  config.rs. Naming it `khem-forcefield` is the most idiomatic
  MD move available, and it pre-shapes phase 4 exactly: .kem
  worlds declaring their physics IS worlds carrying their own
  force fields.
- **System / integrator / thermostat as separate strategies.**
  khem's PhysicsSystem/ChemistrySystem traits are already the
  bevy-ECS idiom; the integrator and thermostat get their own
  names instead of hiding inside physics.rs.
- **Analysis is a separate layer.** Every MD code separates
  runtime observables (cheap, streamed) from offline analysis
  (RDF, coordination, order parameters). khem's g(r), census,
  and tetrahedral-order code belong in a named analysis crate
  shared by the runtime observer (molecule detection) and the
  harness and the future khem-view.
- **Units discipline**: khem documents units (angstroms,
  kJ/mol, ticks) but passes raw f32. Deferred per above.

## 3. Simulator-testing idioms (the pyramid)

Surveyed: MD validation practice (analytic-vs-numeric
regression, conservation-law checking, golden trajectories).

The owner's phrase - "from a few atoms all the way to full
world simulation" - is the standard simulation pyramid:

| Tier | What | Speed | Where |
|------|------|-------|-------|
| Law tests | 1-3 atoms vs analytic bounds (the Hooke probe, the F4 wall tests - the pattern exists) | ms | in-crate unit tests, every build |
| Scenario tests | 10-100 atom ensembles; invariant tests: energy conservation within tolerance, atom count, index completeness, determinism per seed | seconds | tests/ per crate, every build |
| Property/regression | deterministic golden trajectories (the golden hash exists) + conservation properties over random small worlds (hand-rolled, zero-dep) | seconds-minutes | every build |
| World/soak tests | the full pond/beaker gates, 10k-25k ticks | 10-25 min each | CI-only (below), #[ignore] locally |

The additions khem is missing: conservation property tests at
the scenario tier, and doctests on the force laws. The
restructure builds both.

## 4. GitHub Actions idioms (public repo = free runners)

- **Every push:** fmt + clippy + law + scenario tiers - minutes.
- **Main pushes + PRs, as a parallel matrix:** each soak test
  one matrix entry - six 15-minute gates run in parallel wall
  clock instead of ~90 serial minutes on a laptop. The gate
  suite moves OFF the owner's machine; the machine stays free
  for interactive work.
- **Nightly:** full ladder re-validation + a perf job recording
  ticks/sec as an artifact (a cheap perf-regression watchdog).
- **Release workflow:** cargo publish to hold the crates.io
  name (ADR-0007) when there is something real to publish.

## Applied decisions (owner: "do your lean on all three")

1. The gate-ladder IDs (K1.1...K5.3) stay as cross-references
   in plans/ (the methodology, like story IDs) - code and test
   names become plain behavior language.
2. Eight crates: forcefield, world, physics, chemistry,
   analysis, engine, worlds, and the CLI bin.
3. The analysis crate is built now (it arms the real-water
   sitting's measurements).
