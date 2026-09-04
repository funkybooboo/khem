# ADR-0001: Record architecture decisions

Date: 2026-09-04
Status: Accepted

## Context

khem was born from a long conversation in which decisions arrived in
layers, with renames between layers (BDL -> Weave -> Cerne -> khem).
Decisions that live only in conversation history are invisible to
future contributors and to the thesis record. The project needs a
durable, citable record of why things are the way they are.

## Decision

Record every decision that shapes the language, the runtime, or the
process as an architecture decision record in docs/adr/: one
numbered file per decision, in the Nygard format (Context /
Decision / Consequences), following
https://github.com/architecture-decision-record/architecture-decision-record

ADRs record WHY. The WHAT lives in docs/specs/ (language and runtime
specifications) and ARCHITECTURE.md (crate layout).

## Consequences

- Decisions become citable, diffable, and reviewable in commits.
- Superseded decisions keep their ADR with a status change instead of
  being silently rewritten; change means a new ADR.
- The founding conversation's decisions were back-filled as
  ADR-0002 through ADR-0005; project-setup decisions are
  ADR-0006 through ADR-0009.