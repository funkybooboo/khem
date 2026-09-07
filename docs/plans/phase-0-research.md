# Phase 0 - literature grounding

Goal: steal every abstraction lesson prior work already paid for,
before the kernel pays for them again.

Status: the reading list and the abstraction map have landed -
docs/research/references.md is the bibliography (with the
prior-work table), and docs/research/abstraction-notes.md records,
for each abstraction khem uses, what prior work supports it, what
it simplifies away, and what the phase-1 kernel has measured
against it (findings F1-F19). Phase 0 re-opens at named trigger points (the gap register in
PLAN.md is the index) - each re-open is a short memo against
the literature before the gate that needs the design pays for
it instead:

- before K2.3 tuning - the non-bonded polarity potential memo
  (phantom-solvent and four-bead amphiphile models;
  abstraction-notes section 4 is the lesson)
- before K3 starts - the K3 mechanism memo (pairing channel,
  thermal window, ligation geometry, preregistered honesty
  line)
- before K5 - the turnover memo (decay = UV photolysis
  candidate; material feed)
- before the phase-3 perf pass - the spec 13 target
  re-derivation from the E-gates' real requirements
- before any watch-condition detector is designed - the Genesis
  Engine correction notice is required reading first (the
  phase-5 file carries that binding)

The reading list, in priority order:

- Dittrich, Ziegler, Banzhaf (2001) - the artificial-chemistry
  taxonomy; the vocabulary for the whole field
- SimSoup papers - structure-driven molecule properties, the same
  philosophy at molecule-type level
- Kappa manual - the grammar of rules over agents with sites; what
  to keep and what to drop from the .kem grammar
- Ganti (2003) - the chemoton: container + metabolism + information;
  the theoretical minimal cell the seed approximates
- Szostak/Bartel/Luisi (2001) + Chen et al. (2004) - what a minimal
  cell must do physically (osmotic growth, division, competition)
- The Genesis Engine correction notice - how simulation studies
  fool themselves; required before designing any watch-condition
  detector
