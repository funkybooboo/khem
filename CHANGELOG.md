# Changelog

khem is unreleased (v0.1.0). This changelog records the
substantive arcs as they land; the full timeline with commit
hashes is [plans/history.md](plans/history.md), the gate ladder
with pass records is [plans/gates/](plans/gates/).

## Unreleased

### 2026-11-06: the restructure

The repository reorganized into its proper order
(erledigen-style): plans/ (identity, roadmap, the gate ladder,
history, issues), docs by role (docs/build/), the plans/ and
root files (AGENTS.md, CONTRIBUTING.md, CHANGELOG.md), the CI
gate matrix. The khem-core monolith split into eight
single-responsibility crates - khem-forcefield (the parameters
as data: element table, bond energies, the grouped `ForceField`
config), khem-world (state + the Event contract), khem-physics
(forces, integrator, thermostat, energy), khem-chemistry,
khem-analysis (the observables library), khem-engine (tick loop,
observer, NDJSON), khem-worlds (the pond), and the khem bin.
Behavior identical through the move: the golden hash
byte-identical, the K1.3 water-persistence gate re-run PASS.

### 2026-11-06: the fidelity philosophy (ADR-0014)

The owner session decided the project's standard of truth:
real-world likeness as a striving direction, expressed through
config knobs; gates certify out-of-the-box defaults; defaults as
close to the real world as cheaply as possible; khem is a
general physics/chemistry runner (the pond is one world among
many). The never-add list: explicit Coulomb electrostatics,
polarizability, the quantum core, proton-transfer chemistry. The
cheap directional hydrogen-bond term is approved; the
atom-size gap is un-parked (the real-water substrate sitting
lands before the K2.3 gates).

### 2026-11-06: the pond re-density

The pond re-seeded at real water's own atomic density (0.1022
atoms/A^3, the 3.2 A lattice) with keep-apart construction -
waters retry to placement legality, beaker sprays take the best
of N draws. All pond gates re-run PASS on the final
construction; the golden hash re-cut.

### 2026-11-04: K2 opens

- **K2.1 excluded volume PASS**: the gate measured the
  strength-1 soft core failing its own letter at the thermal
  tail (H-H pairs passing clean through, finding F22); the
  sweep retuned `non_bonded_repulsion` 1.0 -> 8.0 (the
  symplectic bound the ceiling); the whole K1 ladder
  re-validated in the same commit.
- **K2.2 condensed medium PASS**: pure measurement - the pond
  is contact-dense matter (0.978 of atoms hold an intermolecular
  neighbor within 3 A) with the excluded-volume hole held in
  g(r), and the no-first-shell baseline PREREGISTERED (g_max
  1.002): repulsion cannot build cohesion; the attraction
  channel must beat that number.
- **The cohesion channel implemented, DORMANT**
  (`cohesion_scale` 0.0): the like-dissolves-like chi well
  (double-sided quartic, flat 4 A range, intermolecular 1,3
  exemption) in code, spec, and unit tests. The activation
  evidence: every shell-building depth condenses the seeded
  pond - the model's atoms are ~2x smaller than real effective
  sizes, so its liquids live at 2-3x real density and the
  lattice cannot seed below 3.2 A. Activation moved to the
  real-water substrate sitting (ADR-0014).

### 2026-09-08: the 3D port (phase 2, ADR-0013)

The substrate ported to 3D (z on atoms, the 3-torus Wrap,
Grid3D, the 3-cell spatial hash, VSEPR scoring 3D directions,
NDJSON v:2, the pond re-seeded as a slab) and the entire K1
ladder re-climbed, one gate per commit, in 3D the same day.

### 2026-09-05/07: the kernel and K1 (phase 1)

The tick loop, physics, chemistry, energy, the observer with
union-find molecule detection, hand-rolled NDJSON, the hardcoded
pond, the khem bin streaming real output. The K1 ladder closed
in 2D: thermostat coupling (K1.1), force sanity (K1.2), water
persistence (K1.3, sub-stepping), reactive balance (K1.4,
steric-contact formation + rate-capped release), seam symmetry
(K1.5). The re-validation contract recorded and exercised.
