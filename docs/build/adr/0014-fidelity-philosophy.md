# ADR-0014: The fidelity philosophy - real-world likeness, as cheaply as possible

Date: 2026-11-06 (owner decisions, recorded from the design
session of the same date)
Status: Accepted

## Context

The gate ladder had arrived at a fork the project's own
evidence created. The cohesion channel (implemented 2026-11-04,
dormant by measurement) revealed a structural fact: khem's atoms
are effectively ~2x smaller than real atoms (excluded-volume
cutoffs 1.5-2 A against real van der Waals contacts at
~2.8-3.2), so the model's liquids live at 2-3x real density -
and a water lattice cannot seed below ~3.2 A spacing, so the
pond can never sit at its own liquid's equilibrium. At any
cohesion depth strong enough to build a first shell, the pond
condenses.

Fixing that fact opened a deeper question with several
coherent answers: should khem chase real micro-geometry
(per-element vdW radii, measured bond lengths - real data that
drags toward real micro-physics like hydrogen-bond channels and
electrostatics), or accept the model's own self-consistent
scale (the pond becomes whatever its physics wants - a
self-cohering droplet)? The owner answered, in a recorded
Q&A session (2026-11-06), with a philosophy that resolves both
the fork and the project's general direction.

## Decision

Five owner decisions, quoted in substance:

1. **Real-world likeness is the standard of truth** - as a
   striving direction, expressed through configuration knobs,
   never as fixed rules.

2. **Gates certify out-of-the-box defaults** - the README's
   success ladder is a claim about what khem does when run with
   its shipped defaults, not about what its engine can do when
   configured by an expert.

3. **Defaults are as close to the real world as cheaply as
   possible.** The mission, in the owner's words: "I am not
   looking to make a world class simulator that mimics reality
   as close as it can. I am looking to build something that
   mimics real world as closely and as cheaply as we can,
   finding systems that in essence work like the real world.
   From an outsider's perspective the simulation should act
   much like a high end physics simulator but under the hood we
   are doing cheap calculations and shortcuts to get us
   there." Cheap computation and memory are first-class
   constraints (laptop-runnable, billion-tick horizons).

4. **Structural realism is wanted - but never at the cost of
   cheapness**: "if there is a way to cheaply and in essence do
   what real water molecules do that isn't going to kill CPU or
   RAM usage then I am okay to do it."

5. **khem is a general physics/chemistry runner** - any world
   can be set up and run; the primordial pond is one world
   among many, not the project.

Consequences drawn from these, agreed in the same session:

- **Knob-tuning beats real micro-data tables.** Real vdW
  radii and measured bond lengths are free lookups, but they
  drag the design toward real micro-physics (hydrogen-bond
  channels, electrostatics, polarizability) - the
  world-class-simulator path the owner explicitly rejects.
  Worse, a real-data wall would forbid real water's hydrogen
  bonds outright (the H...O contact at 1.9 A sits inside the
  nominal 2.7 A van der Waals contact) - honest hydrogen
  bonding needs the real mechanism, which is not cheap. The
  knobs are therefore tuned to EMERGENT targets: liquid
  density, first-shell position, coordination, phases -
  the outsider's observables.

- **The cheap hydrogen bond is directional, not
  electrostatic.** The mW / Stillinger-Weber / Mercedes-Benz
  water models prove tetrahedral liquids come from a cheap
  directional term, no quantum mechanics, no charges -
  sometimes without explicit hydrogens at all. khem's version:
  a donor-H...acceptor attraction weighted by how well the
  O-H bond vector points at the acceptor, one dot product in
  the already-visited pair loop. This replaces the cohesion
  memo's "no hydrogen-bond channel" simplification with the
  honest cheap essence of the real thing.

- **The never-add list** (named shortcuts, not aspirations):
  explicit Coulomb electrostatics (long-ranged - the honest
  versions need mesh solvers), electronic polarizability, the
  quantum core (bonds stay phenomenological - the founding
  design), and proton-transfer chemistry (someday as reaction
  rules, never as physics).

- **The atom-size gap is un-parked.** The wall-scale retune
  (non_bonded_margin toward real first-shell geometry) plus
  the directional hydrogen-bond term land together as one
  "real-water substrate" work item, BEFORE the K2.3 gates -
  because K2.3 certifies defaults and no honest cohesion
  depth exists at the wrong wall scale.

## Consequences

- The real-water substrate sitting (wall rescale + directional
  H-bond + cohesion re-derivation) is sequenced before K2.3;
  its checkpoint bars are the owner-agreed water-likeness
  contract: real density held without condensation, first
  shell at ~2.8 A, ~4-fold coordination, measurable tetrahedral
  order, the K1/K2 gates still passing, and the tick cost
  within ~10-20%.

- The hydrophobic effect may EMERGE from the H-bond network
  (tails break the network; water prefers water) rather than
  being imposed by the chi-scores - K2.3's amphiphile sorting
  becomes a more honest result, and the chi-well may shrink to
  a correction (re-derived by sweep).

- Every fidelity decision from here forward asks the same two
  questions in order: does it move the outservable behavior
  toward reality, and does it stay cheap? Real micro-data
  (per-element tables) is used only where it costs nothing and
  implies nothing expensive.

- The spec's smuggled-physics register (6.6's honesty flags)
  grows an entry: the directional term is smuggled PHYSICS
  documented as such, exactly like the excluded volume before
  it.
