# ADR-0002: 2D phenomenological atom/bond substrate

Date: 2026-09-04 (decision made in the founding conversation)
Status: Accepted

## Context

Real chemistry requires quantum mechanics - impossible at any useful
scale. Pure cellular automata (Conway-style) have no matter to
evolve. Reactive molecular dynamics (ReaxFF in LAMMPS) is physically
real but covers femtoseconds for 10^5-10^6 atoms on supercomputers -
many orders of magnitude short of evolutionary timescales. The
project needs a substrate that is true enough to be interesting and
cheap enough to run for a billion ticks on a laptop.

## Decision

Simulate a 2D continuous-space world of atoms carrying real element
properties (valence, electronegativity, mass, covalent radius) for
the 10 elements H, C, N, O, P, S, Si, Fe, Na, Cl. Bonds form and
break by Boltzmann/Arrhenius probabilities against lookup tables of
real bond energies and VSEPR angles. Temperature, pressure, and UV
fields evolve; energy sources (hydrothermal vents, solar UV) drive
the system away from equilibrium. Right answers come from lookup
tables, not simulated mechanisms: phenomenological, honestly labeled
- a physics-flavored artificial chemistry, never presented as real
chemistry.

## Consequences

- Runs on a laptop at the runtime spec's target tick rates.
- 2D is deliberate: evolution dynamics are preserved, but geometry is
  artifact-prone (membranes are rings; base pairing is planar).
  Accepted for V1; 3D is a port, not a redesign, if ever needed.
- Fidelity claims must always be stated as phenomenological. The
  runtime's guarantee G01 (nothing above atom/bond level exists in the
  engine) is the architectural expression of this decision.
- The K1-K5 validation gates (PLAN.md) test whether this abstraction
  produces living dynamics before anything is built on it. If the
  substrate is dead, it is redesigned, not patched.