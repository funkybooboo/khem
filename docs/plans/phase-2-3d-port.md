# Phase 2 - the 3D port

Gated on K1.5 - PASSED 2026-09-07, K1 closed in 2D; this phase
is unblocked and next. Lands before K2
tuning, phase-3 save/load, and the phase-4 parser. The WHY is
ADR-0013; the timing in one line: the re-validation contract makes
any dimension change a full ladder re-climb wherever it lands, and
this window is the last one where nothing downstream has encoded
2D yet - K2 rates, the save format, the grammar, and the event
fields all come after.

K1.5's own lesson for the re-climb: the seam gate's classes get
THINNER in 3D (three seams, a cross-seam band that is a surface
rather than an edge), and its unconditioned rate ratio measured
0.58-1.01 across seeds even in 2D - pool-composition noise, not
substrate asymmetry. The 3D re-climb conditions on sum-p from the
start (Chemistry::pair_probability is the exposed census helper);
it does not re-learn the lesson by failing it.

## Preconditions

- K1.4 and K1.5 PASSED in 2D, F18 fixed. The fix is
  dimension-generic and MORE needed in 3D (a 3D search ball holds
  a larger past-break-length fraction of its volume than the 2D
  disc), so it lands in 2D and the port starts from a chemically
  sane substrate. Porting a substrate with known open pathologies
  makes every 3D failure unattributable - the exact failure mode
  the probe-first discipline exists to prevent.
- No in-flight substrate work from other sessions: this is the
  largest single diff the repo will take; land it alone.

## What moves (port inventory)

- AtomState/WorldState: positions and velocities gain z/vz;
  worlds gain depth. Wrap becomes the 3-torus (three seam pairs),
  Wall the box; Open keeps its meaning.
- Grid2D -> Grid3D (temp/setpoint/pressure/UV): the flat index
  gains a layer stride; the diffusion kernel gains the z neighbor
  pair (spec 6.2's 4-connected stencil becomes 6-connected).
  Diffusion conserves the field sum exactly as before - the K1.4
  ledger closure must keep closing in 3D.
- SpatialIndex: 3D cell hash. A 4 A query scans up to 27 cells
  (3x3x3) against today's 9 at the 5 A cell size; wrap-folding
  extends to the third axis (F11's mechanism, which K1.5 measured
  in 2D).
- WorldState::delta() folds the third axis; every seam-crossing
  site (springs, chemistry distances, bond midpoints) already
  routes through it (spec 6.3).
- Energy sources: the vent's convection axis becomes the vertical
  axis; UV's surface becomes the top cell layer; falloff takes
  the 3D distance. Same fields, one more axis - no new semantics.
- pond.rs: seeding sprinkles a 3D extent (shape decision below).
- observer: union-find molecule detection is bond-graph work,
  dimension-agnostic - unchanged (the port's one free lunch).
- Golden hash: re-cut consciously in the port commit - every
  number changes by design. The abstraction-notes log entry
  follows the K1.3 precedent; the 2D value is recoverable from
  git history.

## What does not move (the port's test vector)

The tick order (ADR-0005), sub-stepping and the spring laws, the
formation/break laws, the bond-energy and VSEPR tables (spec
7.3/7.4 - the 3D scoring ideals become literally expressible),
the energy ledger (absorbed = vent + setpoint relaxation +
thermostat backflow, exactly as measured in 2D), the Langevin
coupling law, and the event vocabulary. Anything that smells like
a behavior change beyond geometry gets the probe-first K1.4
discipline - the port must not smuggle fixes while every number
moves.

## The re-climb (exit criterion)

The re-validation contract binds with full force: dimension moves
every gate's assumptions, so the port re-runs the K1 ladder in
order, one gate per commit, with the 2D pass records as the
dimension-agnostic regression reference:

1. K1.1 thermostat: coupling-ratio law + bounded KE in 3D.
2. K1.2 force sanity: overlap probe + per-bond band.
3. K1.3 water persistence: the seeded waters stay intact.
4. K1.4 reactive balance: F18 fix active from day one; rates
   re-tuned with the 2D methodology (starve-then-retune).
5. K1.5 seam symmetry: all three seams.

Each pass records the 2D-versus-3D number pair - what moved and
why is the port's evidence.

## Decisions this phase owns

- NDJSON schema mechanics (ADR-0004): additive z/world_depth
  fields versus position arrays; v:1 preserved or bumped. No
  external consumers exist before this phase - this is the cheap
  moment, and the decision is made here, not after khem-view.
- World shape: cube versus slab (wide in x/y, shallow in z). A
  slab keeps atom budgets nearer 2D while restoring 3D geometry;
  a cube is the honest pond. Decide from the K1.1/K1.4 re-climb
  evidence, not taste.
- Phase 4's coordinate grammar: 3-tuples and the rotation
  representation (degrees per axis versus quaternion), decided
  here so the language lands once; language-spec is revised in
  the port commits, not after the parser exists.
- Pond density: keep the 2D pond's ~3.4 A mean interatomic
  spacing or re-derive. Part of the K1.4 re-tune, not a separate
  question.
- AtomState layout (registered in PLAN.md): keep the current
  array-of-structs through the port, or bundle the SoA
  conversion (phase 3's likely perf lever) into it so the layout
  churn is paid once. Decided with the perf target re-derivation
  (the phase-0 re-open; phase-3-hardening.md).
- Port re-climb observability (registered in PLAN.md): no
  viewer exists and 3D occludes what 2D showed by default.
  Decide the debugging surface before the re-climb starts -
  harness-side statistics (per-layer field sums, projected
  distance histograms) rather than viewer work.

## Out of scope

- No dual-mode runtime (ADR-0013): 2D survives in git history.
- No perf optimization beyond honest re-measurement against
  spec 13 - phase 3 owns the perf pass; the port records the new
  baseline.
- No viewer work; khem-view is future either way.

## Exit

All five K1 gates re-passed in 3D with harness evidence; the
ledger closes in 3D; perf re-measured; code and specs change in
the same commits (the agreement rule). Phase 1 resumes - K2
climbs in 3D.
