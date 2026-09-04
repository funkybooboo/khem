# ADR-0005: Determinism by construction

Date: 2026-09-04 (decision made in the founding conversation)
Status: Accepted

## Context

Scientific claims require reproducibility. Future threading (V2) and
distribution (V3) must not change results. Non-determinism that
appears only under parallelism is the worst kind of bug, and this
project's results may end up in a thesis.

## Decision

Determinism is a structural property, not a feature:

- fixed tick order: nine systems in strict sequence, no system reads
  another system's writes within a tick (G14)
- one deterministic, seeded RNG with per-tick state (G02)
- flat arrays indexed by integer IDs; iteration order always by ID
- no thread-local state in v0.1
- dead entities are flagged and compacted periodically, never
  removed mid-tick

## Consequences

- Same .kem files plus same seed equals byte-identical output,
  forever (G02, G14).
- Save/load must round-trip exactly (G09); a resumed run is identical
  to an uninterrupted one.
- V2 (thread-per-region) and V3 (region-per-machine) scale the same
  design instead of redesigning it; the aggregation of per-region
  updates must preserve ID-ordered iteration semantics.
- RNG usage discipline (which system consumes the stream, in what
  order) must be pinned down during phase 1 kernel work, before
  there is behavior to break.