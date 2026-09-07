# khem

khem simulates 3D worlds of atoms - real elements, real
chemistry, thermodynamics deciding what forms and what breaks
(2D through the K1 gates; the 3D port landed 2026-09-08,
ADR-0013).
The runtime has no concept of a cell, a genome, or reproduction:
if anything alive appears, it built itself from the rules.

A world is space filled with atoms: ten elements (H, C, N, O,
P, S, Si, Fe, Na, Cl) carrying real valences, masses, and
electronegativities. Bonds form and break by Boltzmann
probabilities against tables of real bond energies, steered by
VSEPR geometry. Temperature, pressure, and UV fields evolve;
vents heat the seafloor, sunlight the surface.

You seed the world with anything buildable from atoms and bonds -
a beaker of molecules with no life in it, an RNA strand inside a
lipid vesicle, a whole cell. Then you let it run. Nobody knows
whether evolution will take hold. That is the experiment.

## It runs today

The engine is built - pure Rust, two crates, zero dependencies -
and one command streams a live world to stdout:

    cargo run --release -p khem    # the hardcoded primordial pond

    {"v":1,"type":"start","tick":0,"run_name":"primordial_pond",
     "seed":42,"atom_count":3432,"bond_count":2048}
    {"v":1,"type":"bond_formed","tick":16,"elem_a":"O","elem_b":"O",
     "order":2,"energy":498}
    {"v":1,"type":"tick","tick":1000,"atom_count":3432,
     "bond_count":2091,"mol_size_dist":{"1":290,"2_5":1052,
     "6_20":1,"21plus":0}}

Every number above came off this run; the events are abridged
for width - the full field lists are the runtime spec, section 3.3.

One JSON event per line, flushed every tick: stream it, grep it,
chart it, build a viewer on it. Same seed, byte-identical run -
determinism is by construction, not luck.

## What you can do

- Run prebiotic chemistry with no life at all: a warm pond, a
  hydrothermal vent, UV from above.
- Seed life, simple or complex - anything built from atoms and
  bonds.
- Compose worlds the way hardware is composed - parts with
  ports, instantiated and wired into hierarchies; the standard
  library ships water, nucleotides, lipids, a vesicle, a cell.
- Pipe the stream anywhere: one JSON event per line is the whole
  output contract, and anything that reads it is a viewer.
- Run it long: billion-tick experiments, parameter sweeps,
  lineage tracking.

## The trade

Real chemistry would be ideal, but reactive molecular dynamics
is orders of magnitude too slow to ever watch evolution happen.
khem takes the middle path: real element properties and real
bond-energy tables drive phenomenological dynamics cheap enough
to run a billion ticks on a laptop. Where the sim simplifies,
the docs say so - no rule pretends to be deeper than it is.

## What success looks like

Nothing is built on the substrate until it passes measured gates:

    K1  stability       molecules persist; bonds form and break
                        at plausible rates
    K2  self-assembly   lipids in water clump head-out, with no
                        "form a membrane" rule anywhere
    K3  replication     free nucleotides copy a seeded RNA strand
                        by base-pair geometry alone
    K4  variation       copies carry errors at tunable rates
    K5  selection       lineages with different fidelity fare
                        differently in a scarce pond

## The language

Everything above atoms and bonds is described, never programmed.
khem is a hardware description language: what Verilog is to
circuits, .kem is to matter. A struct is a module - atoms, bonds,
ports - composition is instantiation and wiring, and the runtime
is the simulator. Worlds are `.kem` files that compose bottom-up -
element to molecule to strand to cell to world to run:

    struct water {
      atoms {
        O1: O at ( 0.00, 0.00)
        H1: H at (-0.96, 0.58)
        H2: H at ( 0.96, 0.58)
      }
      bonds {
        O1 - H1 : single
        O1 - H2 : single
      }
    }

    chain rna_strand {
      sequence: A U G C A U G C    // a genome, as data
    }

A strand's sequence is data. Copying it is not a feature anywhere
in the runtime - it is something the chemistry must do alone.

## Status

The engine is built and streams real output - every number above
came off an actual run. The K1 gate ladder is closed: the
thermostat (K1.1), force-sanity (K1.2), water-persistence
(K1.3), reactive-balance (K1.4), and seam-symmetry (K1.5)
sub-gates are measured
passes - the pond's 1024 waters hold intact (which took
sub-stepped integration and real-water-stiff bonds; the failing
substrate measured 1482 bombardment breaks and only 206 waters
intact), and the free-atom beaker settles to a stationary
molecule-size distribution: weak O-O bonds flicker at the
measured 10k-tick Boltzmann scale, strong ones persist, and the
field recovers to its 35 C setpoint. The K1.4 attack fixed two
measured structural bugs: wide-capture churn (bonds used to form
anywhere inside the 4 A search disc, a third of them phantoms
born past their own break length, silently refrigerating the
field) and the thermal-release bomb (the break's heat dumped
instantly into one cell vaporized the whole pond once the field
first reached its true steady state - now thermalized at a
bounded rate). K1 is closed: the last gate, cross-seam
formation symmetry (K1.5), passed 2026-09-07 - a Wrap world's
seam is not special; the audit also fixed a seam-mirrored
VSEPR anchor (F20) the raw statistics could not see. The .kem
language is
spec-only - the parser is built only after the gates pass,
because a language on a dead substrate is worthless.
Dimensionality is decided and DONE: the K1 gates landed in 2D,
then the 3D port ran as its own phase before K2 (ADR-0013,
landed 2026-09-08) - so vesicles, base pairs, and carbon get
true geometry instead of 2D shadows before anything tunes
against it. The K1 ladder now re-climbs in 3D, one gate per
commit.

## Documentation

    PLAN.md            where the project stands and where it goes
    ARCHITECTURE.md    how the crates fit together
    docs/specs/        the contracts: the .kem language, the runtime
    docs/plans/        the build order, phase by phase
    docs/adr/          the why behind the design decisions
    docs/research/     prior work and the evidence log

MIT license.
