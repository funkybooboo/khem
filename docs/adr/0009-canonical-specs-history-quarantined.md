# ADR-0009: Canonical specs; history quarantined

Date: 2026-09-04
Status: Accepted; amended by ADR-0010 (quarantine became removal)

## Context

initial-idea.md is a chat transcript whose end state can only be
understood by reading it top to bottom - names and terminology
changed mid-conversation (BDL -> BIOSIM -> Weave -> Cerne -> khem).
The first spec extraction preserved those layers verbatim in
docs/specs/, so a newcomer could read a superseded draft and
reasonably conclude that the language is called BDL or the runtime
is named BIOSIM.

## Decision

docs/specs/ contains only canonical, current-state specifications
(language-spec.md, runtime-spec.md) with the final terminology
applied throughout - khem, .kem, struct/chain/body/world/run,
--check, V-STRUCT/V-CHAIN/V-BODY/V-WORLD/V-RUN validation codes.
Conversation-era drafts live in docs/history/spec-drafts/ with their
provenance headers. initial-idea.md remains the verbatim founding
transcript. The conversation-era to canonical rename map lives in
docs/specs/README.md. From here on, spec changes are versioned
edits to the canonical documents, never new conversation layers.

## Consequences

- Reading docs/specs/ shows only what khem IS; history is one click
  away but clearly labeled as history.
- The canonical specs must now be maintained as living documents:
  edited in commits, diffed in review, revised against phase-1
  reality (ADR-0006).
- Canonicalization closed three gaps the conversation left dangling:
  the file wrapper keyword (now khem "0.1"), the placement keyword
  (inside, per the final conversation round's example), and typos in
  examples (corrected). These are recorded in docs/specs/README.md.