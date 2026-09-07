# khem project plan

khem is a bet: build matter honest enough to be interesting and
cheap enough to run a billion ticks, seed it with a minimal cell,
and let the chemistry do everything above the atom/bond level - if
anything alive appears, it built itself from the rules.

This file is the hub: the hinge question, where the repo
stands, the phase map, the known gaps, and the project-wide
rules. The detail lives in docs/plans/, one file per phase.

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

## Where this repo stands (2026-09-08)

- Phase 1, the kernel, is built and runs end to end: tick loop,
  physics, chemistry, energy, observer with union-find molecule
  detection, hand-rolled NDJSON v:1, the hardcoded pond, the khem
  bin streaming real output. Canonical specs (docs/specs/) and
  ten ADRs (docs/adr/) stay synced with the code; the founding
  conversation lives in git history only.
- The K1 ladder is CLOSED: K1.1-K1.5 PASSED (thermostat,
  force sanity, water persistence, reactive balance, seam
  symmetry - the pond's
  1024 waters hold intact and the free-atom beaker settles to a
  stationary molecule-size distribution over 20k-tick vented
  runs: weak O-O bonds flicker at the measured 10k-tick
  Boltzmann scale, strong ones persist, the field recovers to its
  35 C setpoint; and a Wrap world's seam is not special -
  cross-seam formation matches the law's own expectation under
  the composition-conditioned census, the wrap-aware index
  verified pair-for-pair complete against brute force).
  The 3D port's substrate diff landed 2026-09-08 (phase 2,
  ADR-0013): z/vz on atoms, the 3-torus Wrap, Grid3D with a
  6-connected stencil, a 3-cell spatial hash, the VSEPR geometry
  factor scoring 3D directions (the tetrahedral ideal now
  literally expressible), the vertical axis moved to z (vent
  convection up, UV on the top layer), the pond re-seeded as the
  60x60x15 A slab at the same 3432-atom budget, NDJSON v:2
  (additive z/world_depth), and the golden hash re-cut. Perf
  honestly re-measured: 11.7 t/s at 3.4k atoms (2D: ~60; phase 3
  owns the target's levers). Next: the K1 re-climb in 3D, one
  gate per commit. Ladder, rules, and pass history:
  docs/plans/phase-1-kernel.md; port decisions:
  docs/plans/phase-2-3d-port.md.
- Findings F1-F20 live in docs/research/abstraction-notes.md;
  all resolved through K1.5; the newest is F20 (the mirrored
  VSEPR anchor - a seam-straddling bond's raw direction scored
  candidates against a phantom ideal; fixed with the
  minimum-image direction).
- Dimensionality is decided (ADR-0013): 2D carries the K1 gates -
  the F18 fix is dimension-generic, and the 2D passes become the
  port's regression vector - then the 3D port lands before K2
  tuning, the phase-3 save format, or the phase-4 grammar encodes
  more 2D. Port plan: docs/plans/phase-2-3d-port.md.
- Release speed measured ~60 t/s at 3.4k atoms (2026-09-07;
  K1.3's sub-stepping costs 4x force passes); the phase-3 perf
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
| 1 - physics/chemistry kernel | [phase-1-kernel.md](docs/plans/phase-1-kernel.md) | K1 passed (K1.1-K1.5, 2026-09-07); K2-K5 climb after the port |
| 2 - the 3D port | [phase-2-3d-port.md](docs/plans/phase-2-3d-port.md) | substrate diff landed 2026-09-08; K1 re-climb open |
| 3 - runtime hardening | [phase-3-hardening.md](docs/plans/phase-3-hardening.md) | not started; cheap items landed early with phase 1 |
| 4 - the khem language | [phase-4-language.md](docs/plans/phase-4-language.md) | spec-only; gated on K1-K5 |
| 5 - experiments and the thesis | [phase-5-experiments.md](docs/plans/phase-5-experiments.md) | gated on phases 3-4 |

Renumbered 2026-09-07: the 3D port (ADR-0013) was inserted as
phase 2, shifting hardening to 3, the language to 4, and
experiments to 5. Dated records - ADRs, the findings log, pass
histories - keep the numbering of their day: pre-renumbering
text calls the port 1.5, hardening 2, the language 3,
experiments 4.

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
- one dimensionality at a time: the runtime is 2D through the K1
  gates, then the phase-2 port makes it 3D - no dual-mode
  runtime, no per-world dimension flag (ADR-0013); 2D survives
  in git history.

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

## How this plan is maintained

- PLAN.md is the hub (status, phase map, gap register,
  project-wide rules); docs/plans/ holds one file per phase,
  written to be read top to bottom.
- The WHY of every decision lives in docs/adr/ (Nygard format;
  immutable once accepted - change means a new ADR).
- The implementation and the specs must AGREE (owner decision
  2026-09-05): any divergence between code and
  docs/specs/runtime-spec.md is fixed in both in the same commit,
  piecemeal. Designed-but-unbuilt items are marked [phase 3] /
  [phase 4] in the spec rather than allowed to drift.
- The WHAT lives in docs/specs/ (canonical, current-state specs,
  edited in commits and revised against phase-1 reality per ADR-0006)
  and ARCHITECTURE.md (crate map).
- The gate is `mise run check` locally and identical in CI; the
  toolchain is pinned in mise.toml.
- The founding conversation is recoverable from git history only:
  the transcript at commit d8205f1, the spec-draft extractions at
  83a2688 and fefc4b9.

## Known gaps and open questions

Gate-status questions (which gate is open, what its pass
criteria are) live in the ladder: docs/plans/phase-1-kernel.md.
This register is the cross-cutting material: mechanisms the
ladder assumes but no phase designs, and questions that are not
any single gate's to answer. Each row names the trigger - we
answer it when we get there, not before.

| Gap / open question | Bites | Answered by | Detail |
|---|---|---|---|
| Non-bonded polarity attraction: the substrate's only attraction is the bond spring (spec 6.6 is repulsion-only and says so); K2's amphiphile sorting needs an element-derived attractive potential, designed and honesty-flagged first | K2.3-K2.6 | before K2.3 tuning | phase-1-kernel.md (K2.3 note); phase-0 re-open |
| K3 mechanism: which bonds pair bases, how the duplex releases (the thermal window), and whether adjacent paired nucleotides reach ligation distance - all undesigned or uncalculated | K3.1-K3.4 | the K3 mechanism memo, before K3 starts (it feeds the port's grammar decisions) | phase-1-kernel.md (K3 note); phase-0 re-open |
| Turnover mechanisms: decay (UV photolysis is the honest candidate, undeclared) and material feed (no mechanism at all) | K5.2 | the turnover memo, before K5 | phase-1-kernel.md (K5.2 note); phase-0 re-open |
| Perf target re-derivation: ~60 t/s measured at 3.4k atoms extrapolates to ~20 t/s at 10k vs spec 13's >500 t/s (~25x), the port's ~3x pair cost lands on top, and the named levers look like 5-10x; whether the E-gates need that target at all is unexamined | phase-3 perf pass | re-derive spec 13 from the E-gates' real requirements before the perf pass; the layout-bundle question decides with the port | phase-3-hardening.md; phase-2-3d-port.md |
| Port re-climb observability: no viewer exists, and the 3D K1.4 retune happens without one | phase 2 | ANSWERED at the port (2026-09-08): harness-side statistics - per-layer field sums, projected distance histograms, the event census; no viewer work. The re-climb gates add the probes they need as harness code | phase-2-3d-port.md |
| The port's own open decisions (NDJSON encoding, world shape, rotation grammar, density, layout bundling) | phase 2 | NDJSON v:2 + coordinate grammar + density + observability DECIDED at the port (2026-09-08); world shape provisional (slab) pending K1.1/K1.4 evidence; layout bundling open pending the perf re-derivation | phase-2-3d-port.md, "Decisions this phase owns" |

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
- [x] dimensionality (ADR-0013): RESOLVED 2026-09-07 - 3D is the
      destination substrate. 2D stands through the K1 gates; the
      port is phase 2, gated on K1.5 and landed before K2 tuning
      or any phase 3/4 surface encodes more 2D. Port plan:
      docs/plans/phase-2-3d-port.md.
- [x] license: RESOLVED 2026-09-05 - MIT (LICENSE at root, SPDX MIT
      in crate metadata).
- [x] remote hosting: RESOLVED 2026-09-05 - github.com/funkybooboo/khem,
      public (CI green on the very first push).
- [x] phase 1 placement: RESOLVED 2026-09-04 - kernel code lands in
      the khem-core lib, driven by the khem bin on main
      (ARCHITECTURE.md).
