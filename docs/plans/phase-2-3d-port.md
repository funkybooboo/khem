# Phase 2 - the 3D port

Gated on K1.5 - PASSED 2026-09-07, K1 closed in 2D. The port's
substrate diff LANDED 2026-09-08 (one commit, `mise run check`
green): positions/velocities/fields/index/energy/pond/observer/
NDJSON all carry z, the golden hash is re-cut
(0x4B84_4F68_14CC_5899), and the specs are synced in the same
commit (the agreement rule). The RE-CLIMB is open: K1.1-K1.5
re-run in 3D, one gate per commit, with the 2D pass records as
the dimension-agnostic regression reference. Lands before K2
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

## What moved (port inventory - LANDED 2026-09-08)

Measured at the landing: the 3D slab pond (3432 atoms, 1024
waters + 360 free) runs 11.7 t/s in release against the 2D
substrate's ~60 t/s - honest re-measurement, no optimization
(phase 3 owns the perf pass; the ~5x is ~4x candidate density
per query in the slab plus the 27-cell scan against 9, with the
visited-cell dedup cost growing quadratically in scan size).
The 2000-tick diagnostic reads healthy: 1024/1024 waters intact,
bond length mean 1.207 / p95 1.376 A, chemistry active (75
formed-and-alive), KE/atom already at the 3/2 equipartition of
the local field (coupling ~1.04 at the construction dip).

One representation fact measured at the port, for the K1.4
re-climb's ledger: the 6-connected f32 diffusion rounds the
field sum at ~8e-4 degrees/tick on a 1000-cell grid (the 2D grid
measured ~2e-4/tick; with diffusion_rate 0 the drift is exactly
0 - it is stencil rounding, not an exchange leak). It is 3+
orders below the ledger's smallest real flux term; the K1.4
windows tolerate it by construction.

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

DECIDED at the port commit (2026-09-08):

- NDJSON schema mechanics (ADR-0004): ADDITIVE z/world_depth
  fields, and v is BUMPED v:1 -> v:2. Additive keeps x/y
  semantics (a projection of the truth, not a lie), but the bump
  is deliberate: the world model changed dimensionality, and a
  consumer that silently ignored the new fields would mis-model
  geometry from x/y alone - the v field exists to version
  exactly this kind of contract change, and no consumer existed
  at the bump (the cheap moment; spec 3.4 records the reasoning).
  Position ARRAYS were rejected: they change field types, a
  bigger contract change than the geometry needed.
- Phase 4's coordinate grammar (language-spec revised in the
  port commit, before any parser exists): positions and
  placements are 3-TUPLES; world sizes are TRIPLES
  (200 x 200 x 60); regions partition by z (the vertical axis);
  rotation is DEGREES PER AXIS, rotate (<rx>, <ry>, <rz>),
  applied about the fixed x, y, z axes in that order. A .kem
  file is human-authored description and molecular thinking is
  in angles; quaternions are an elaboration detail the runtime
  keeps to itself.
- Pond density: the 2D lattice constant is KEPT and extended to
  a cubic lattice (3.75 A spacing in all axes; 16x16x4 = 1024
  waters, the same atom budget as the 2D pond). Re-derivation,
  if any, belongs to the K1.4 re-tune with 3D evidence.
- Port re-climb observability: HARNESS-SIDE STATISTICS - per-layer
  field sums, projected distance histograms, the event census -
  no viewer work (khem-view stays future; the NDJSON stream is
  the eyes). The re-climb gates add the probes they need as
  harness code, one gate per commit.

DECIDED PROVISIONALLY, the re-climb evidence owns the final call:

- World shape: the pond landed as the SLAB (60x60x15 A) - wide in
  x/y, shallow in z - because it keeps the K1 signal's
  lateral-extent readability and the atom budget near 2D while
  restoring 3D geometry (chains pass; tetrahedra exist). Cube
  versus slab is decided from the K1.1/K1.4 re-climb evidence,
  not taste; if the slab's depth confounds a gate, the pond
  resizes in that gate's commit with its bars re-measured.

STILL OPEN (unchanged):

- AtomState layout (registered in PLAN.md): the port kept the
  array-of-structs (the layout question is bundled with the SoA
  conversion, phase 3's likely perf lever, and decided with the
  perf target re-derivation - the phase-0 re-open;
  phase-3-hardening.md). The port's measured 11.7 t/s baseline
  is that decision's new input.

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
