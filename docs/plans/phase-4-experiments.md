# Phase 4 - experiments and (maybe) the thesis

The K-gates prove the substrate; the E-gates prove the
instrument - that khem can run real evolution experiments. Each
E-gate is a platform capability, built in the order the
experiments need them. Same ladder rules as the K-gates
(docs/plans/phase-1-kernel.md): harness-measured criteria, pass
it, commit it, move on; the re-validation contract binds here
too (a passed gate re-runs when a later gate moves its
assumptions).

## The E-gate ladder

### E1 - save/resume

A resumed run is byte-identical to an uninterrupted one (G09) -
long experiments span sessions. This is phase 2's save/load
wearing its working clothes.

### E2 - lineage tracking

Event logs reconstruct phylogenies - every copy event a
parent/child edge, every mutation labeled on the edge it
changed.

### E3 - sweeps

A batch runner executes a parameter grid (mutation rate, UV,
temperature, pond size, scarcity) across seeds and collates the
results.

### E4 - controls

Negative controls are first-class runs - no-template, no-UV,
dead-strand. Anything that cannot report "no" is not a detector
(the Genesis Engine rule).

### E5 - first experiment

Mutation rate vs copy fidelity: one preregistered prediction, an
ablation arm (bond-table perturbation), and the full writeup -
the thesis-track dress rehearsal.

## Candidate research questions

Preregister metrics BEFORE looking:

- does evolution of the seeded cell produce open-ended genome
  diversity, or converge to a dominant strain? under which
  parameters?
- how sensitive are outcomes to substrate abstraction choices
  (bond-energy tables, angle tables, formation rates)?
- do watch conditions correlate with anything independently
  measurable?

## Thesis track

If this becomes a thesis, the honest framing is: "an
artificial-chemistry platform for studying emergent evolution of
seeded minimal cells" - NOT "simulating abiogenesis". A
defensible thesis needs preregistered metrics, ablations, and
negative controls. Read the Genesis Engine correction notice
first (the phase-0 binding): their 100% headline result was a
detector artifact, and their audit trail is the best available
example of how that class of mistake happens in exactly this
kind of simulation. Design so it cannot happen here. The
literature-review skeleton is docs/research/references.md.