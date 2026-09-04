# khem spec drafts - provenance, status, and naming decisions

Everything in this directory is extracted verbatim (then normalized to
ASCII) from initial-idea.md, the founding conversation. These are
historical drafts, not the canonical spec. The canonical spec gets
rewritten in this directory only after phase 1 validates the substrate
(see PLAN.md).

## Extraction map

Line numbers refer to initial-idea.md.

| File | Source lines | What it is | Status |
|---|---|---|---|
| 01-bdl-language-spec.md | 3207-3744 | language spec v0.1, named BDL; extensions .elem/.mol/.org/.world/.sim | superseded by 04 + 06 |
| 02-biosim-runtime-spec.md | 3745-4399 | runtime spec v0.1 (Rust); tick order, physics/chemistry, guarantees G01-G10 | superseded by 05 |
| 03-naming-and-ndjson-rationale.md | 4439-4887 | naming rationale (runtime "cerne" from Latin cernere), reporting philosophy, NDJSON draft, tool ecosystem | philosophy stands; names superseded |
| 04-weave-language-spec.md | 5008-5597 | round-2 language spec: all files .weave, declarations inside files, V-codes | superseded by 06 |
| 05-runtime-spec.md | 5598-6407 | round-2 runtime spec: CLI, NDJSON event schema, data structures, tick order, physics/chemistry equations, V1-V3 scalability, guarantees G01-G14, performance targets | CURRENT runtime draft; apply renames from 06 and the table below |
| 06-terminology-and-final-naming.md | 6413-7243 | final founding-conversation terminology: mol->struct, org->body, layers->region, connect->wire, sim->run; .crn extension; language named Cerne | AUTHORITATIVE deltas over 04 + 05 |

## Naming decisions

In-conversation rounds:

1. Runtime "cerne" (Latin cernere, to sift/discern), language "BDL",
   then "Weave", extensions .elem/.mol/.org/.world/.sim
2. All files .weave, language Weave, runtime cerne
3. Final terminology keywords; files .crn; language renamed Cerne
   (language Cerne / files .crn / runtime cerne)

Project-level round 4 (2026-09-04): after a collision sweep, the
language, files, and runtime were renamed khem. Reasons: "cerne" is one
letter from CERN and invites permanent confusion; "khem" (pronounced
"kem") is the root of the word "chemistry" per a leading etymology
(Egyptian kemet, the fertile black land; etymology contested); it had
the cleanest uniqueness sweep of roughly 40 candidates - crates.io
name verified free, no direct project/company/term collisions.
Rejected with recorded causes: cerne (CERN), ylem (three small
projects; pronunciation ambiguity), zyme (small dormant JS repo),
weft/veld/loam (crates.io taken; veld claimed 2026-09-03), grex
(major Rust regex CLI, 176k downloads), sinter (Trail of Bits tool +
crate + SinterCast AB), keld (.kld collides with FreeBSD kernel
modules), galatea (meaning runs backwards: life by divine gift, not
emergence), golem (crypto network; runs-amok connotation), valence
(the Val language owns .val), chemoton/hypercycle (naming after a
theory invites precision attacks), soup/pond (tone), LUCA (Pixar),
petri (Petri nets), agar (agar.io), genesis/prometheus (taken),
alloy (MIT Alloy), alembic (SQLAlchemy), substrate (generic).

Current canonical identity:

    language   khem          (the way C is C)
    files      .kem          (the way .c is .c)
    runtime    khem          (the way rustc reads .rs)
    tools      khem-view, khem-check, khem-log
    usage      khem experiment_1.kem > run_001.ndjson

Rename mapping when reading the drafts below (conversation-era ->
current):

    Cerne, Weave, BDL   ->  khem     (language and runtime name)
    .crn, .weave        ->  .kem     (file extension)
    cerne-view, cerne-check, cerne-log  ->  khem-view, khem-check, khem-log

Keyword decisions from 06 are final as written (element, struct,
chain, body, world, run; use, place, wire, port, link, inside,
scatter, count, at, as; region, source; wrap/wall/open).

## Normalization notes

Extracted content is verbatim except: box-drawing characters converted
to ASCII (+ - | =), arrows to -> and <-, degree signs to "deg",
angstroms to "A", triple-bond glyphs to "#", superscripts to ^2/^3,
and similar. Internal inconsistencies and later-abandoned ideas are
preserved on purpose; these are history, not truth. initial-idea.md at
the repo root is the unnormalized original.