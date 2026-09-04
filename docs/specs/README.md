# khem specifications

This directory holds the canonical, current-state specifications:
khem as it is defined now, not the path the founding conversation
took to get here.

- language-spec.md - the khem language: .kem files, the six
  declarations (elements, struct, chain, body, world, run), syntax,
  units, validation codes, reserved words
- runtime-spec.md - the khem runtime: CLI, NDJSON event schema, core
  data structures, tick execution, physics and chemistry systems,
  energy and observer systems, scalability, configuration,
  guarantees, performance targets

The specs are drafts until validated: phase 1 (PLAN.md) tests whether
the substrate produces living dynamics before the parser is built,
and both documents are revised against kernel reality at that point
(ADR-0006).

## History and provenance

- initial-idea.md (repo root) - the founding conversation, verbatim.
  It is a chat transcript: the end state can only be reconstructed by
  reading it top to bottom, because names and terminology changed
  mid-conversation.
- docs/history/spec-drafts/ - the six verbatim spec extractions from
  that conversation, each written in the terminology of the moment it
  was drafted. Superseded. Do not implement from these.
- docs/adr/ - the decisions behind everything, including the naming
  sweep (ADR-0007).

## Conversation-era to canonical rename map

Readers of the history drafts translate with this table:

| Conversation-era | Canonical |
|---|---|
| BDL, BIOSIM, Weave, Cerne (language and runtime names) | khem |
| .elem, .mol, .org, .world, .sim, .weave, .crn (extensions) | .kem for all files |
| file wrapper: weave "0.1" | khem "0.1" |
| mol declaration | struct |
| org declaration | body |
| sim declaration | run |
| layers | regions |
| energy_sources | sources |
| import | use |
| connect | wire |
| chain_bond | link |
| placement keyword in | inside |
| --validate | --check |
| event field sim_name | run_name |
| validation codes V-MOL-xx, V-ORG-xx, V-SIM-xx | V-STRUCT-xx, V-BODY-xx, V-RUN-xx (plus new V-CHAIN-xx for chains) |
| stdlib search paths ~/.cerne/stdlib/ and ./weave/ | ~/.khem/stdlib/ and ./khem/ |
| sequence: [A U G C] | sequence: A U G C (bare, space-separated) |

Gaps the canonical spec closes (the conversation left them
dangling):

- The file wrapper keyword was never explicitly renamed after the
  language was renamed; the canonical spec applies the rename
  consistently (wrapper = language name = khem).
- The final conversation round's example used inside where the
  round-2 grammar said in; the canonical keyword is inside.
- Obvious typos in conversation-era examples are corrected in
  canonical examples.