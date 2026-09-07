# Phase 2 - runtime hardening

What makes the kernel runnable for the long horizons the K5 and
E gates need: persistence, speed, and the missing runtime
surfaces. Status: not started; the items that were cheap landed
early with phase 1 (recorded below so the history stays honest).

Landed early, with the phase-1 kernel commits:

- deterministic seeded RNG everywhere (G02) - built in from the
  start, pinned by per-seed determinism tests through the whole
  loop
- observer molecule detection via union-find on the bond graph
- unit tests per system + the golden-tick regression hash (the
  conscious-update safety net for substrate changes)
- wrap-aware spatial index (F11's fix; gate K1.5 owns the
  seam-symmetry measurement)
- the Langevin thermostat + setpoint reservoir (F8's fix, gate
  K1.1) - a phase-1 necessity that the founding spec had punted

Remaining (the spec's [phase 2] markers):

- save/load world state: SAVE events (spec 3.3) + G09 - a
  resumed run is byte-identical to an uninterrupted one. E1 is
  this wearing its working clothes. Lands after the 3D port
  (phase 1.5, ADR-0013), so the state layout it serializes is
  natively 3D.
- dead-slot compaction (spec 5.2): atoms and bonds are flagged
  dead, never removed mid-tick; compaction every
  compaction_interval ticks. Long experiment runs accumulate
  dead slots until then.
- watch conditions: NOTABLE events (spec 3.3). Designing any
  detector requires the Genesis Engine correction notice first
  (phase 0's binding).
- signals: clean ctrl-c interrupt (spec 2.3, exit code 3);
  phase-1 builds die with the process.
- performance: the spec 13 targets (10k atoms at >500 t/s on a
  laptop). Measured 2026-09-07: ~60 t/s at 3.4k atoms - the
  sub-stepping (4x force passes + per-sub-step index rebuilds)
  bought water persistence at 8x the force cost, so this pass is
  now load-bearing. No optimization before the substrate is
  behaviorally sane; the gates are correctness gates. The
  obvious levers: per-sub-step neighbor-query allocation reuse,
  the index rebuild schedule, and a profile before anything
  clever. The 3D port (phase 1.5, ADR-0013) re-opens this
  baseline first - a 4 A query scans up to 27 cells versus 9 at
  the 5 A cell size - so the port records the honest new number
  against the same spec 13 targets.
- OPEN QUESTION (registered in PLAN.md; phase-0 re-open before
  this pass): ~60 t/s measured at 3.4k atoms extrapolates to
  ~20 t/s at 10k atoms - ~25x short of spec 13's >500 t/s, with
  the port's ~3x pair cost on top, while the named levers look
  like 5-10x. Whether the E-gates need that target at all is
  unexamined: a billion ticks at the extrapolated 10k-atom
  rate is ~1.6 years, and E1 save/resume makes multi-session
  runs legal. Re-derive the targets from the experiments' real
  requirements (sustained rate + resume cost + the pond sizes
  the E-gates actually use) before optimizing anything; if SoA
  layout is the answer, decide whether the 3D port bundles it -
  that decision sits with the port (phase-1.5-3d-port.md).
