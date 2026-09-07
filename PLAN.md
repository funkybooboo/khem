# khem project plan

khem is a bet: build matter honest enough to be interesting and
cheap enough to run a billion ticks, seed it with a minimal cell,
and let the chemistry do everything above the atom/bond level - if
anything alive appears, it built itself from the rules.

This file is the hub: the hinge question, where the repo stands,
the phase map, and the project-wide rules. The detail lives in
docs/plans/, one file per phase.

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

## Where this repo stands (2026-09-07)

- Phase 1, the kernel, is built and runs end to end: tick loop,
  physics, chemistry, energy, observer with union-find molecule
  detection, hand-rolled NDJSON v:1, the hardcoded pond, the khem
  bin streaming real output. Canonical specs (docs/specs/) and
  nine ADRs (docs/adr/) stay synced with the code; the founding
  conversation lives in git history only.
- The gate ladder is climbing: K1.1-K1.3 PASSED (thermostat,
  force sanity, water persistence - the pond's 1024 waters hold
  intact through 10k-tick vented runs). Next: K1.4 reactive
  balance, then K1.5 seam symmetry. Ladder, rules, and pass
  history: docs/plans/phase-1-kernel.md.
- Findings F1-F18 live in docs/research/abstraction-notes.md;
  the open one is F18 (wide-capture churn, K1.4's lever).
- Release speed measured ~60 t/s at 3.4k atoms (2026-09-07;
  K1.3's sub-stepping costs 4x force passes); the phase-2 perf
  pass owns the target (10k atoms at >500 t/s).
- The .kem language is spec-only; the parser starts only after
  the K gates pass (ADR-0006).
- Toolchain pinned in mise.toml; `mise run check` is the gate,
  identical in CI. Hosted at github.com/funkybooboo/khem,
  public.

## The phase map

| Phase | Plan | Status |
|-------|------|--------|
| 0 - literature grounding | [phase-0-research.md](docs/plans/phase-0-research.md) | notes + bibliography landed; re-opens before detector design |
| 1 - physics/chemistry kernel | [phase-1-kernel.md](docs/plans/phase-1-kernel.md) | in progress: K1.1-K1.3 passed, K1.4 next |
| 2 - runtime hardening | [phase-2-hardening.md](docs/plans/phase-2-hardening.md) | not started; cheap items landed early with phase 1 |
| 3 - the khem language | [phase-3-language.md](docs/plans/phase-3-language.md) | spec-only; gated on K1-K5 |
| 4 - experiments and the thesis | [phase-4-experiments.md](docs/plans/phase-4-experiments.md) | gated on phases 2-3 |

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

- PLAN.md is the hub (status, phase map, project-wide rules);
  docs/plans/ holds one file per phase, written to be read top
  to bottom.
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
- The founding conversation is recoverable from git history only:
  the transcript at commit d8205f1, the spec-draft extractions at
  83a2688 and fefc4b9.

## Open decisions (owner: nate)

- [x] thermostat (gate K1.1): RESOLVED 2026-09-05 - PASSED.
      Langevin damping toward the local field temperature with
      exact signed-delta bookkeeping and the setpoint reservoir;
      see the K1.1 entry in docs/plans/phase-1-kernel.md and the
      findings log. Spec 6.1/6.2/11 synced. RE-VALIDATED
      2026-09-07 in K1.3's integrator commit: coupling law +
      steady-tail windows (the re-validation contract's integrator
      binding, first exercise); the velocity clamp that was part
      of the K1.1 pass is gone - sub-stepping replaced it (K1.3).
- [ ] non-bonded soft repulsion (finding F4; gate K2.1):
      IMPLEMENTED with the K1.1 commit (spec 6.6, chemistry/physics
      tests); the K2.1 scattering-test PASS run is the gate's own
      work, lands only with harness evidence, as its own commit.
- [ ] first world file name: primordial_pond.kem ("warm little
      pond" is Darwin's phrase for the setting).
- [x] license: RESOLVED 2026-09-05 - MIT (LICENSE at root, SPDX MIT
      in crate metadata).
- [x] remote hosting: RESOLVED 2026-09-05 - github.com/funkybooboo/khem,
      public (CI green on the very first push).
- [x] phase 1 placement: RESOLVED 2026-09-04 - kernel code lands in
      the khem-core lib, driven by the khem bin on main
      (ARCHITECTURE.md).
