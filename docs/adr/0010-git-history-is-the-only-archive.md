# ADR-0010: Git history is the only archive

Date: 2026-09-04
Status: Accepted

## Context

ADR-0009 quarantined the founding conversation (initial-idea.md) and
the six conversation-era spec drafts (docs/history/spec-drafts/) in
the working tree so that history would stay available. But keeping
old documents in the tree duplicates what git already does: the
transcript is committed verbatim at d8205f1, the drafts at 83a2688
and fefc4b9. A chat log whose end state requires reading top to
bottom, plus superseded drafts, are noise for anyone reading the
current tree - and permanent weight for every future clone.

## Decision

The working tree contains only current-state documents. The founding
conversation and the conversation-era drafts are removed; git
history is the archive:

- initial-idea.md, verbatim: commit d8205f1
- the six extracted spec drafts: added at 83a2688, moved to
  docs/history at fefc4b9, removed at this commit

Recovery, if ever needed:

    git show d8205f1:initial-idea.md > initial-idea.md
    git show fefc4b9:docs/history/spec-drafts/05-runtime-spec.md

Specs and ADRs cite "the founding conversation (git history)" instead
of repo paths.

## Consequences

- The tree reads as current-state only; history is one git show
  away.
- Nothing is rewritten: clones still carry the full history. If a
  clean public history is ever wanted (for a thesis release, say),
  that is a separate, deliberate decision with its own ADR.
- ADR-0009 is amended by this ADR: quarantine became removal.