# History

The timeline record. The living queue is [roadmap.md](./roadmap.md);
the constitution is [identity.md](./identity.md). Detailed
completed-phase records live in [history/](./history/);
findings F1-F22 live in
[docs/build/research/abstraction-notes.md](../docs/build/research/abstraction-notes.md).

---

## 2026-08: the founding conversation and the ground floor

- d8205f1 - the founding transcript imported (the spec-draft
  extractions at 83a2688 / fefc4b9 follow; the conversation now
  lives in git history only, per the scope-discipline rule).
- 875fd7a / a4662af - the khem runtime crate scaffolded, then
  the workspace split into the engine lib + CLI bin.
- fefc4b9 / 7deec25 - the specs made canonical; ADR-0001 through
  0009 recorded (determinism by construction, kernel before
  language, the .kem name, NDJSON as the public contract).
- debaa57 - toolchain pinned with mise, CI workflow added.

## 2026-09-04/05: the kernel comes up

- d761b73 - the khem-core data model and CLI skeleton.
- 6131d98, 6baa239, 2f35aa8 - the physics, energy, and chemistry
  systems (spec 6/7/8), the system traits decomposed.
- 0339d3b - phase-0 literature grounding: Squirm3, Verlet
  integration, the abstraction notes begin (findings F1-F5).
- 8be9443 - the observer with union-find molecule detection, the
  NDJSON emitter, the Sim tick loop.
- 2464333 - the hardcoded pond, the bin wired, the K1 harness.
- 9708553 - the minimum-image convention (F10); the rule-law
  test suite; the golden regression hash.
- ef88783 - **gate K1.1 PASS**: the Langevin thermostat holds
  the vented pond at its setpoint (2D).
- 5f8ce10 - MIT license; the repo goes public.

## 2026-09-06/07: K1 closes in 2D

- da471bb - **gate K1.2 PASS**: bounded overlap impulse, the
  bond band held.
- cdf9708 - **gate K1.3 PASS**: sub-stepped integration; the
  velocity clamp retired; the pond's water holds (1025/1024).
- c380827 - the re-validation contract recorded (owner
  decision), first exercised inside K1.3/K1.4's own commits.
- 5b9d6be - **gate K1.4 PASS**: steric-contact formation,
  rate-capped release; the reactive balance holds at 20k ticks.
- 1ea9d8c - **gate K1.5 PASS**: seam symmetry - the K1 ladder
  closed in 2D.
- fc02182 - ADR-0013: 3D is the destination substrate; the port
  gated on K1.5.

## 2026-09-08: the 3D port (phase 2)

- df27e9e - the substrate ported to 3D: z on atoms, the 3-torus
  Wrap, Grid3D, the 3-cell spatial hash, VSEPR scoring 3D
  directions, NDJSON v:2, the pond re-seeded as a slab.
- 12e6ffc, 73cca38, 2929d86, 2b3faf1 - **the K1 ladder
  re-climbed in 3D, one gate per commit** (K1.1 through K1.5),
  the port's exit criterion met; phase 2 complete. The F21
  scan-order artifact found and resolved by reversal; the K1.4
  floor bar re-derived (F6's drain over half the cells).
- 06e70ed - the README catches up to the 3D port.
- Detailed port decisions and evidence:
  [history/phase-2-3d-port.md](./history/phase-2-3d-port.md).

## 2026-11-04: K2 opens

- 38f4c79 - **gate K2.1 PASS**: excluded volume. The gate
  measured the strength-1 soft core failing its own letter at
  the thermal tail (F22: H-H pairs passing clean through at the
  p99 closing speed); the sweep retuned non_bonded_repulsion
  1.0 -> 8.0; the whole K1 ladder re-validated in the same
  commit, all five PASS.
- ebd2bfa - **gate K2.2 PASS**: condensed medium, pure
  measurement - contact-dense matter (0.978 of atoms hold an
  intermolecular neighbor within 3 A), the excluded-volume hole
  held in g(r), and the no-first-shell baseline PREREGISTERED
  (g_max 1.002 beyond 3.0 A: repulsion cannot build cohesion).
- 55dc299 - the polarity-attraction memo: the like-dissolves-like
  chi well designed (K2.3's precondition).
- aba761d - **the cohesion channel implemented, DORMANT**
  (cohesion_scale 0.0, the golden hash byte-identical): the
  double-sided quartic well, flat 4 A range, the intermolecular
  1,3 exemption. Spec 6.6 synced.

## 2026-11-06: the pond re-density and the philosophy

- 2343be7 - the pond re-densified to real water's atomic
  density: the 3.2 A lattice (0.1022 atoms/A^3) in the
  51.2 x 51.2 x 12.8 A slab, keep-apart construction seeding
  1024/1024 waters intact; all pond gates re-run PASS; the
  golden hash re-cut (0xF761_ADD4_6DA9_BC26). The activation
  sweep's evidence lands in the polarity memo's revision log:
  every shell-building depth condenses the pond (D=0.5: first
  shell peak g 2.12, blobs over vapor by 12k; D=1.0: jammed,
  6.15) - the structural cause: the model's atoms are ~2x
  smaller than real effective sizes, so its liquids live at
  2-3x real density and the lattice cannot seed below 3.2 A.
- The owner session decides the fidelity philosophy
  (ADR-0014, recorded at cf04541): real-world likeness via
  config knobs; gates certify out-of-box defaults; as close to
  the real world as cheaply as possible; khem is a general
  runner; the atom-size gap is un-parked - the real-water
  substrate sitting (wall rescale + directional H-bond term)
  lands before K2.3.

## 2026-11-06: the restructure (this arc)

- cf04541 - the philosophy into the docs (ADR-0014, the
  engineering-idioms research, the general-runner identity in
  the README/PLAN).
- The plans/ reorganization (this commit): PLAN.md's content
  distributed into plans/ (identity, roadmap, gates, history,
  issues); docs organized by role under docs/build/; the
  K-numbers survive as the ladder's cross-references.
- The eight-crate split, the behavior-named test files, the
  conservation/doctest tiers, the CI soak matrix: the restructure
  commits that follow, each with the golden hash carried.
