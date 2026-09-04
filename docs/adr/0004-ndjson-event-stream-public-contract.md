# ADR-0004: NDJSON event stream as the public contract

Date: 2026-09-04 (decision made in the founding conversation)
Status: Accepted

## Context

Reporting was initially designed inside the runtime with a terminal
UI. Different users want different views; a runtime that renders is
a runtime that grows UI dependencies and cannot be scripted.

## Decision

The runtime emits only newline-delimited JSON events to stdout,
versioned with a v field on every event (schema v:1 in khem v0.1).
Human-readable diagnostics go to stderr. Viewers and analysis tools
are separate programs consuming the stream; they target the documented
event schema (docs/specs/runtime-spec.md section 3), not shared
types.

## Consequences

- khem-view, khem-log, and any future tool never need to link the
  engine (ADR-0008); anything that reads the stream is a valid
  viewer, including jq, grep, and one-line Python.
- The event schema is a public API with compatibility obligations:
  within a v the contract only grows additively; breaking changes
  bump v.
- Terminal UX is deliberately deferred until there is a stream worth
  watching.