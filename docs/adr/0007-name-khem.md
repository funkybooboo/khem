# ADR-0007: Name khem; extension .kem

Date: 2026-09-04
Status: Accepted

## Context

The founding conversation named the runtime "cerne" (Latin cernere,
to sift), then the language "Weave", then renamed everything to
"Cerne" with .crn files. "cerne" is one letter from CERN - permanent
confusion risk. Naming criteria adopted after that: coined names are
fine (C, Rust, and Verilog do not describe their domains); easy to
say; three-letter file extension; zero overlap with anything notable.

## Decision

The language, the files, and the runtime are all khem: pronounced
"kem", from the root of the word "chemistry" itself (al-kimiya ->
alchemy -> chemistry, per a leading - and contested - etymology from
Egyptian kemet, "the black land", the fertile mud where things grow).
The file wrapper keyword follows the language name: khem "0.1".
Tools take the khem- prefix (khem-view, khem-log, khem-check).

Verification (2026-09-04): the crates.io name is free; a web sweep
found no direct project, company, or term-of-art collisions (only
near-names: the Kemet board game, "Khemit").

Rejected with recorded causes: cerne (CERN); ylem (three small
existing projects; pronunciation ambiguity); zyme (a small dormant
JS repo exists); weft, veld, loam (crates.io names taken - veld was
claimed the day before the sweep); grex (a 176k-download Rust regex
CLI); sinter (Trail of Bits security agent + crate + SinterCast AB);
keld (free, but .kld collides with FreeBSD kernel modules); galatea
(the myth runs backwards - life granted by a goddess, not emergent);
golem (Golem Network crypto; runs-amok connotation); valence (the
Val language owns .val); chemoton and hypercycle (naming after a
theory invites precision attacks); soup and pond (tone; pond
survives as the first world file, primordial_pond); LUCA (Pixar);
petri (Petri nets); agar (agar.io); genesis, prometheus, alloy,
alembic, substrate (all taken or generic).

## Consequences

- The identity system mirrors C/gcc and Rust/rustc: language khem,
  files .kem, runtime khem.
- All conversation-era names are history; the rename map lives in
  docs/specs/README.md.
- If the crates.io name is taken by someone else before first
  publish, this ADR is superseded by a new one; publishing a real
  (non-stub) crate would hold the name.