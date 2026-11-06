# Issues: the gap register + open decisions

Cross-cutting material: mechanisms the ladder assumes but no
phase designs, and questions that are not any single gate's to
answer. Each row names the trigger - we answer it when we get
there, not before.

Gate-status questions live in the ladder:
[gates/](./gates/).

| Gap / open question | Bites | Answered by | Detail |
|---|---|---|---|
| K3 mechanism: which bonds pair bases, how the duplex releases (the thermal window), and whether adjacent paired nucleotides reach ligation distance - all undesigned or uncalculated | K3.1-K3.4 | the K3 mechanism memo, before K3 starts (it feeds the phase-4 grammar decisions) | [gates/replication.md](./gates/replication.md); phase-0 re-open |
| Turnover mechanisms: decay (UV photolysis is the honest candidate, undeclared) and material feed (no mechanism at all) | K5.2 | the turnover memo, before K5 | [gates/selection.md](./gates/selection.md); phase-0 re-open |
| Perf target re-derivation: ~60 t/s measured at 3.4k atoms in 2D, ~11.7 t/s in 3D, vs spec 13's >500 t/s; whether the E-gates need that target at all is unexamined | phase-3 perf pass | re-derive spec 13 from the E-gates' real requirements before the perf pass; the layout-bundle question decides with it | [stories/phase-3-hardening.md](./stories/phase-3-hardening.md) |
| World shape provisional (slab) pending long-run evidence; layout bundling open pending the perf re-derivation | phases 3-4 | the perf re-derivation | [history/phase-2-3d-port.md](./history/phase-2-3d-port.md), "Decisions this phase owns" |

## Resolved (kept for the record)

- [x] Non-bonded polarity attraction (bit K2.3-K2.6): RESOLVED
      2026-11-04, IMPLEMENTED AND DORMANT (cohesion_scale 0.0):
      the like-dissolves-like chi well in code + spec 6.6.
      Activation folded into the real-water substrate sitting
      (ADR-0014): the depth re-derives at the new wall
      geometry, and the hydrophobic effect may EMERGE from the
      hydrogen-bond network instead of being imposed.
      [gates/self-assembly.md](./gates/self-assembly.md);
      [docs/build/research/polarity-attraction-memo.md](../docs/build/research/polarity-attraction-memo.md)
- [x] Port re-climb observability (bit phase 2): ANSWERED at
      the port - harness-side statistics, no viewer work.
- [x] The port's NDJSON encoding, coordinate grammar, density,
      observability: DECIDED at the port (NDJSON v:2).
- [x] Dimensionality (ADR-0013): 3D destination, port gated on
      K1.5 - landed 2026-09-08.
- [x] Thermostat (gate K1.1), non-bonded repulsion (F4/K2.1),
      license (MIT), hosting (public GitHub), phase-1 placement:
      all RESOLVED - records in
      [history.md](./history.md).

## Open decisions (owner: nate)

- [ ] first world file name: primordial_pond.kem ("warm little
      pond" is Darwin's phrase for the setting) - open until the
      parser exists.
- [ ] the real-water substrate's final defaults: the sitting
      presents its 6-bar water-likeness table to the owner
      BEFORE the substrate commits (ADR-0014's checkpoint
      contract).
