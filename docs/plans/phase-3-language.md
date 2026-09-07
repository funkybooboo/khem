# Phase 3 - the khem language (only what the kernel needs)

Starts only after phase 1's K1-K5 gates pass (ADR-0006). A
language on a dead substrate is worthless; the parser is the
reward the gates buy.

Phase 3 lands on a 3D kernel: the port (phase 1.5, ADR-0013)
precedes K2, so the K2-K5 passes, the grammar's positions and
rotations, and the stdlib's part geometry are authored for a 3D
world once - nothing migrates after the parser exists. The
coordinate arity and the rotation representation are decided in
the port phase.

The language is an HDL for matter (language-spec.md section 1):
.kem descriptions in, a WorldState of atoms and bonds out. Phase 3
builds that front end.

- .kem parser for the declarations: elements / struct / chain /
  body / world / run (grammar from
  docs/specs/language-spec.md, revised against Kappa lessons
  and phase-1 reality; specs are drafts until validated, not
  scripture)
- validation rules (V-STRUCT/V-CHAIN/V-BODY/V-WORLD/V-RUN codes
  from language-spec.md) become tests
- stdlib of primitive .kem files: water, phosphate, ribose,
  bases, nucleotides, lipid, vesicle, rna strand, minimal cell,
  primordial pond
- CLI modes: --check / --test / --info (spec 2.2, the spec's
  [phase 3] markers)
- everything phase 1 hardcoded becomes a .kem file: the pond,
  the minimal cell, the element and bond tables; physics.cfg
  loading replaces the compiled defaults (spec 11)
