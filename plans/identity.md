# Identity: the bet, the philosophy, the guardrails

khem is a bet: build matter honest enough to be interesting and
cheap enough to run a billion ticks, seed it with a minimal cell,
and let the chemistry do everything above the atom/bond level -
if anything alive appears, it built itself from the rules.

## The one question this project hinges on

The founding conversation produced two language specs, a runtime
spec, a CLI, an event schema, and a naming system - and zero
simulated ticks. Everything hinges on an unvalidated assumption:
that a simplified atom/bond substrate (lookup-table bond
energies, VSEPR angles, Arrhenius/Boltzmann probabilities,
thermal noise) produces dynamics interesting enough to evolve a
seeded cell.

So the build order is deliberately backwards from the founding
conversation: kernel before language, physics before parsers,
evidence before specs.

## What khem is

khem is a GENERAL physics/chemistry runner: any world can be set
up and run. The primordial pond is one world among many, not the
project. The runtime has no concept of a cell, a genome, or
reproduction - if anything alive appears, it built itself from
the rules (guarantee G01).

## The fidelity philosophy

Decided by the owner, 2026-11-06; the full record is
[ADR-0014](../docs/build/adr/0014-fidelity-philosophy.md).

- **Real-world likeness is the standard of truth** - as a
  striving direction, expressed through configuration knobs,
  never as fixed rules.
- **Gates certify out-of-the-box defaults** - what khem does
  when run with its shipped defaults, not what an expert can
  configure it to do.
- **Defaults are as close to the real world as cheaply as
  possible.** Not a world-class fidelity simulator: mimic the
  real world closely AND cheaply; systems that in essence work
  like the real world. From an outsider's perspective the
  simulation should act much like a high-end physics simulator;
  under the hood it is cheap calculations and shortcuts. Cheap
  compute and memory are first-class (laptop-runnable,
  billion-tick horizons).
- **Structural realism is wanted - but never at the cost of
  cheapness.** A cheap directional hydrogen-bond term qualifies;
  real micro-physics does not.
- **Knob-tuning beats real micro-data tables.** The knobs tune
  to EMERGENT targets (liquid density, first shell, phases) -
  the outsider's observables - because real micro-data drags
  toward the rejected world-class path.
- **Named shortcuts, never added:** explicit Coulomb
  electrostatics (long-ranged - honest versions need mesh
  solvers), electronic polarizability, the quantum core (bonds
  stay phenomenological), proton-transfer chemistry (someday as
  reaction rules, never as physics).

## Non-goals and guardrails

- no GPU requirement, ever (V1); threading/distribution are
  optional future work (V2/V3), not dependencies
- no graphics in the runtime - viewers are separate pipe
  consumers
- no pre-programmed biology above atom/bond rules; where a rule
  smells like smuggled biology (permeability, division
  thresholds), document it honestly in the spec instead of
  pretending it emerged
- no claims of real chemistry: lookup tables and phenomenological
  rates are the design, not a compromise to hide
- zero registry dependencies: the workspace is deliberately
  dependency-free - instant CI, no supply chain, determinism
  fully in-crate; revisited only when the phase-4 parser needs a
  real choice
- scope discipline: the founding conversation ballooned from
  "900 lines of Python" to a full language spec before one tick
  ran. The plan exists to prevent a repeat.
- one dimensionality at a time: 3D is the destination
  substrate (ADR-0013); no dual-mode runtime, no per-world
  dimension flag. 2D survives in git history.
- determinism by construction (ADR-0005): same seed, same
  world, same stream.
