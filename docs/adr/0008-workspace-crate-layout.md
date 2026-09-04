# ADR-0008: Workspace crate layout

Date: 2026-09-04
Status: Accepted

## Context

The long-term design needs an engine, a language, and a family of
tools. Guarantee G01 (nothing above the atom/bond level exists in the
runtime) must be enforceable, not aspirational. Tools must not
couple to the engine.

## Decision

One Cargo workspace, one binary per crate:

- khem-core (lib) - the simulation engine. Never knows about the
  language; G01 is enforced at the crate boundary.
- khem-lang (lib, phase 3) - parses and validates .kem definitions
  into a khem-core WorldState. Depends on khem-core; never the
  reverse.
- khem (bin) - the runtime CLI, thin forever: parse arguments,
  construct a world, run the tick loop, stream events.
- khem-view, khem-log (future bins) - pipe consumers with zero
  workspace dependencies; they target the NDJSON contract
  (ADR-0004), not shared types.

Shared dependency declarations via [workspace.dependencies].
Placeholder crates are deliberately not created ahead of need; a new
crate costs one directory and one members line.

## Consequences

- khem-core cannot import khem-lang without a dependency cycle; the
  G01 boundary is structural.
- A viewer cannot accidentally link the engine.
- khem-core is testable in isolation (cargo test -p khem-core) and
  reusable by future front ends (GUI, distributed runner, fuzzer).
- Adding tools never touches the engine's dependency graph.