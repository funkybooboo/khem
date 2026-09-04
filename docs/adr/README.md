# khem architecture decision records

ADRs record the WHY behind decisions that shape the language, the
runtime, or the process. One numbered file per decision, in the
Nygard format (Context / Decision / Consequences), following
https://github.com/architecture-decision-record/architecture-decision-record

Rules:

- one decision per file, numbered by decision order (NNNN-slug.md)
- the WHAT lives in docs/specs/ and ARCHITECTURE.md; the WHY lives
  here
- ADRs are immutable once accepted; a changed decision gets a new
  ADR and the old one's status becomes "Superseded by ADR-NNNN"

Index:

- 0001 Record architecture decisions
- 0002 2D phenomenological atom/bond substrate
- 0003 Seed a minimal cell; do not wait for abiogenesis
- 0004 NDJSON event stream as the public contract
- 0005 Determinism by construction
- 0006 Kernel before language
- 0007 Name khem; extension .kem
- 0008 Workspace crate layout
- 0009 Canonical specs; history quarantined
- 0010 Git history is the only archive