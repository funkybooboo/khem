# Roadmap

The queue. The constitution is [identity.md](./identity.md); the
gate ladder is [gates/](./gates/); the timeline record is
[history.md](./history.md).

## Where we stand (2026-11-06)

- The kernel is built and runs end to end: tick loop, physics,
  chemistry, energy, observer, NDJSON v:2, the hardcoded pond,
  the khem bin streaming real output. Specs and ADRs stay
  synced with the code.
- **K1 (stability) is CLOSED** - passed in 2D, re-climbed in 3D
  at the port, re-validated at the K2.1 retune:
  [gates/stability.md](./gates/stability.md).
- **K2 (self-assembly) is OPEN**: K2.1 (excluded volume) and
  K2.2 (condensed medium) PASSED 2026-11-04; the cohesion
  channel is implemented and DORMANT; the pond re-density
  landed 2026-11-06 at real water's atomic density:
  [gates/self-assembly.md](./gates/self-assembly.md).
- The fidelity philosophy is decided
  ([ADR-0014](../docs/build/adr/0014-fidelity-philosophy.md)).
- The repository restructure (erledigen-style plans/, eight
  crates, behavior-named tests, the CI soak matrix) is IN
  FLIGHT - the owner's directive: proper order before the
  substrate work continues.

## The queue (numeric order)

1. **The repository restructure** (in flight): plans/ +
   docs/build/ reorganization, the eight-crate split,
   behavior-named test files + the missing conservation and
   doctest tiers, AGENTS.md/CHANGELOG.md/CONTRIBUTING.md, the
   CI soak-test matrix. Behavior must stay identical through
   the move (the golden hash proves it).
2. **The real-water substrate sitting**
   ([ADR-0014](../docs/build/adr/0014-fidelity-philosophy.md)):
   the wall rescale (non_bonded_margin 1.5 -> ~2.2-2.3) plus
   the directional hydrogen-bond term plus the cohesion depth
   re-derived at the new geometry, one sitting, one
   re-validation (#3: the whole ladder). Checkpoint contract
   with the owner before committing the substrate: density
   ~0.033 molecules/A^3 held uniform, O-O first shell ~2.8 A,
   coordination ~4, measurable tetrahedral order, gates still
   passing, tick cost within +10-20%.
3. **K2.3 - K2.6** (amphiphile sorting through vesicle growth)
   on the real-geometry defaults:
   [gates/self-assembly.md](./gates/self-assembly.md).
4. **K3 - K5** (replication, variation, selection) - the K3
   mechanism memo must land first (see
   [issues.md](./issues.md)).
5. **Phase 3 - runtime hardening** (perf pass, save/load,
   watch conditions): [stories/phase-3-hardening.md](./stories/phase-3-hardening.md).
6. **Phase 4 - the khem language** (.kem parser, gated on K1-K5
   per ADR-0006): [stories/phase-4-language.md](./stories/phase-4-language.md).
7. **Phase 5 - experiments and the thesis**:
   [stories/phase-5-experiments.md](./stories/phase-5-experiments.md).

Renumbered 2026-09-07: the 3D port (ADR-0013) was inserted as
phase 2, shifting hardening to 3, the language to 4, and
experiments to 5. Dated records keep the numbering of their
day: pre-renumbering text calls the port 1.5, hardening 2, the
language 3, experiments 4.

## Beyond v0.1 (horizon, not scheduled)

All of this is enabled by choices already fixed (runtime spec
section 10; the architecture doc) and none of it is on the
critical path:

- V2: thread-per-region with ghost cells - flat arrays, integer
  IDs, and the fixed tick order make this a partitioning
  problem, not a redesign
- V3: region-per-machine distribution - reuses the save/load
  serialization; NDJSON output unchanged
- plugins: dynamic loading of PhysicsSystem/ChemistrySystem
  trait implementations
- tool family: khem-view, khem-log, khem-check, khem-build - one
  bin per crate, zero workspace dependencies, consuming the
  NDJSON contract (ADR-0004, ADR-0008)
- publishing the khem crate to hold the crates.io name
  (ADR-0007), when there is something real to publish

## Fixed decisions

- License: MIT. Hosting: github.com/funkybooboo/khem, public.
- Toolchain pinned in mise.toml; `mise run check` is the gate,
  identical in CI.
- First world file name: primordial_pond.kem ("warm little
  pond" is Darwin's phrase for the setting) - open until the
  parser exists.
