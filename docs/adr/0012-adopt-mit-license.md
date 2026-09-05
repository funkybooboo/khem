# ADR-0012: Adopt the MIT license

Date: 2026-09-05
Status: Accepted

## Context

The license was an open decision in PLAN.md (MIT suggested). The
project is a thesis-candidate research artifact; the khem crate may
be published to crates.io to hold the name (ADR-0007); contributions
so far are personal.

## Decision

MIT: LICENSE at the repo root, SPDX identifier "MIT" in the crate
metadata (declared at the workspace level, inherited by every crate).

## Consequences

- Maximally permissive: reuse, fork, and embed without friction,
  including commercial derivatives - the right default for a research
  artifact whose goals are adoption and replication.
- Thesis use is unaffected; downstream users owe nothing beyond
  notice retention.
- crates.io publication requires exactly this declaration; the
  metadata is now in place.
- Future contributions are accepted under the same terms; no CLA.