# khem architecture

The target crate layout and the rules that keep it honest. The Cargo
workspace holds any number of library and binary crates; a new crate
joins by adding a directory under crates/ and a line in the root
Cargo.toml members list. The WHY behind each decision lives in
docs/adr/; this file describes the shape they produced.

## Crate map (target)

    crates/
    |-- khem-core   lib   the simulation engine
    |-- khem-lang   lib   .kem parsing, validation, flattening (phase 3)
    |-- khem        bin   the runtime CLI
    |-- khem-view   bin   terminal UI; reads the NDJSON stream (future)
    `-- khem-log    bin   structured logging and replay (future)

## Dependency rules

    khem-lang ----\
                  +--> khem (bin) --NDJSON on stdout--> khem-view, khem-log
    khem-core ---/

1. khem-core is the engine: WorldState, atoms, bonds, physics,
   chemistry, energy, spatial index, observer, tick loop, save/load.
   It knows nothing about the .kem language and never will - runtime
   guarantee G01 (nothing above atom/bond level exists in the
   runtime) is enforced at the crate boundary.
2. khem-lang turns .kem definitions into a khem-core WorldState. It
   depends on khem-core. khem-core never depends on khem-lang.
3. khem (bin) stays thin forever: parse arguments, construct a world,
   run the tick loop, stream events. In phase 1 it hardcodes world
   construction directly against khem-core, no language involved.
4. khem-view and khem-log are pipe consumers with zero workspace
   dependencies by default. The NDJSON stream is a public, versioned
   contract (the "v" field on every event, spec section 3.4); tools
   target the contract, not shared types. Anything that reads the
   stream is a valid viewer - that is the point of stdout-only
   output.
5. New tools (khem-check for deep offline validation, khem-build for
   template composition helpers) follow the same pattern. One bin per
   crate so each tool has its own dependency graph, tests, and
   release cadence; a single crate with multiple [[bin]] targets
   would work, but would let a viewer accidentally link the engine.

## Multiple libraries, multiple binaries

- Every engine capability is a library so it is testable in
  isolation (cargo test -p khem-core) and reusable by future front
  ends (CLI, GUI, distributed runner, fuzzer).
- Binaries are named by the khem- prefix. A crate may declare several
  [[bin]] targets, but one-bin-per-crate is the convention here so
  dependency graphs stay honest.

## Scaling plan (from the runtime spec draft)

- V1 (now): single thread, flat arrays, deterministic tick order.
- V2 (optional, later): thread-per-region with ghost cells. Enabled
  by design choices already fixed: flat AtomState arrays, IDs not
  pointers, no hidden state (spec section 10.2).
- V3 (optional, later): region-per-machine. WorldState serialization
  is required for save/load anyway and is reused for distribution
  (spec section 10.3).
- Plugins: physics and chemistry systems sit behind traits in
  khem-core (spec section 10.4). Dynamic loading is a future option,
  never a V1 feature.

## Exists today vs later

    today    khem-core (data model, element table, config, RNG,
             spatial index; the systems are phase 1)
             khem (bin: CLI skeleton, runtime spec section 2)
    phase 3  khem-lang (after the K1-K5 gates pass)
    later    khem-view, khem-log (once there is an event stream worth
             watching)

Placeholder crates are deliberately NOT created ahead of need. The
workspace accepts a new crate with one directory and one members
line; empty scaffolds invite premature building, which is the exact
failure mode this project's plan guards against (PLAN.md, "scope
discipline").