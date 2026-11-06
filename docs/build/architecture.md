# The khem architecture (the WHAT; the WHY lives in
# docs/build/adr/).

## The crates

    khem-forcefield    the physics parameters as DATA: the element table,
                       the bond-energy table, the grouped ForceField config
    khem-world         the data model: WorldState, atoms, bonds, fields,
                       the spatial hash, the deterministic RNG, boundaries -
                       and the Event model of the public NDJSON contract
    khem-physics       the force laws (springs, excluded volume, cohesion),
                       the sub-stepped integrator, the Langevin thermostat,
                       the energy sources
    khem-chemistry     bond formation and breaking by law
    khem-analysis      the observables library: the molecule census
                       (shared with the runtime observer), radial
                       distribution, order parameters
    khem-engine        the tick loop (spec 5.1 order), the observer,
                       the NDJSON emitter; the gate harnesses live here
    khem-worlds        world constructors: the primordial pond, beakers
    khem               the CLI bin

Dependency direction (the MD convention - parameters as data
separate from the engine that reads them):

    khem-forcefield <- khem-world <- { physics, chemistry, analysis }
                                     khem-engine <- { all }
                                     khem-worlds <- { world, forcefield }
                                     khem (bin) <- { engine, worlds }

Inversions to keep: the Event model and the energy-source
declarations live DOWN in khem-world (the systems that mutate
the world produce events into the world's own queue; sources are
declared world content), so the laws never depend on the engine.
The molecule census lives in khem-analysis so the runtime
observer, the harnesses, and future tools share one census.

The bin stays thin forever: parse arguments, construct a world,
run, stream. No world knowledge enters the engine - worlds are
data passed in.

## The guarantees (spec 10)

- G01: no concept above the atom/bond level exists in the
  engine; enforced by the crate boundaries as much as by review.
- G02: determinism by construction (ADR-0005) - same seed,
  same stream; pinned by the golden hash (a fnv1a over
  atom+bond+field state after warm ticks, byte-identical across
  behavior-neutral refactors).
- G03: the observer is read-only over the world (`&WorldState`).
- G04: bond bookkeeping is always consistent (atom bond slots
  <-> bond records), pinned by test.
- G10/G11: stdout carries only NDJSON; stderr only diagnostics.

## The event stream

The NDJSON stream (schema v:2) is the public contract
(ADR-0004): khem-view and the tool family consume the stream,
never the Rust types. Types live in khem-world; the emitter in
khem-engine; tools are separate bins.

## Testing

The simulator pyramid:

- law tests: in-crate unit tests and doctests against analytic
  bounds (the Hooke probe, the well laws) - every build;
- scenario/invariant tests: small worlds; conservation and
  determinism properties - every build;
- regression: the golden hash - every build;
- world/soak gates: `#[ignore]`d release tests in khem-engine,
  one CI job per gate (.github/workflows/gates.yml), the full
  ladder nightly (.github/workflows/nightly.yml).
