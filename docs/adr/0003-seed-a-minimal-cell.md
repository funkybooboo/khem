# ADR-0003: Seed a minimal cell; do not wait for abiogenesis

Date: 2026-09-04 (decision made in the founding conversation)
Status: Accepted

## Context

Real abiogenesis took roughly 500 million years. No affordable
simulation waits for that. The founding conversation pivoted from
"watch life emerge from nothing" to the Conway pattern: fixed rules,
hand-seeded patterns, emergence as the only content.

## Decision

Seed worlds with a minimal RNA-world cell - an RNA strand inside a
lipid vesicle with free nucleotides - built purely from atoms and
bonds, defined as a body declaration in .kem files. Nothing above
the atom/bond level is coded into the runtime; replication,
mutation, membrane division, and evolution must fall out of the
rules. If replication does not emerge (validation gate K3 fails
after honest parameter sweeps), redesign the substrate rather than
patch in special cases.

## Consequences

- Feasible timescales: the study is emergent evolution of seeded
  minimal cells - NOT abiogenesis, and never framed as such. The
  thesis framing (PLAN.md) depends on this distinction.
- Some rules (permeability behavior, base-pair geometry targets)
  smell like smuggled biology. They get flagged honestly in the specs
  rather than hidden behind emergence claims.
- The seed is data - a body declaration, testable in isolation with
  --test - not code.