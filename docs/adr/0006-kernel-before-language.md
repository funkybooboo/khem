# ADR-0006: Kernel before language

Date: 2026-09-04 (decision reversing the founding conversation's order)
Status: Accepted

## Context

The founding conversation produced two language specs, a runtime spec,
a CLI, an event schema, and a naming system - and zero simulated
ticks. The entire project depends on one unvalidated question:
whether a simplified atom/bond substrate produces dynamics
interesting enough to evolve a seeded cell. A language is worthless
on a dead substrate.

## Decision

Build and validate the hardcoded physics/chemistry kernel first
(phase 1). Write the .kem parser only after the K1-K5 validation
gates pass. Treat the canonical specs as drafts to be revised
against kernel reality. Phase 1 has no parser, no .kem files, no
plugins, and no CLI flags beyond --seed; everything is hardcoded.

## Consequences

- The parser is written once, against a live kernel instead of an
  imagined one.
- If K1-K3 fail after honest parameter sweeps (weeks, not days), the
  substrate is redesigned before anything is built on it. That is
  the plan working, not the plan failing.
- The specs in docs/specs/ are expected to change after phase 1;
  changes land as versioned edits to the canonical documents.