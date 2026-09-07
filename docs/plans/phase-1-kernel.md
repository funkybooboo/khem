# Phase 1 - the physics/chemistry kernel (no DSL)

The kernel lives in the khem-core lib; the khem bin stays a thin
entry point (see ARCHITECTURE.md). Everything is hardcoded:

- the tick loop from docs/specs/runtime-spec.md (spec 5.1):
  energy -> bath -> [index -> forces -> motion] x substeps ->
  boundary -> index -> bond breaking -> bond formation ->
  observe -> flush
- a hardcoded primordial pond (pond.rs; a hardcoded minimal cell
  follows the same pattern when the K3 gates need it)
- NDJSON events to stdout (tick + bond events; watch conditions
  are phase 3)

Explicit non-goals: no parser, no .kem files, no plugins, no
behavioral CLI options beyond --seed. Hardcode everything. The kernel is
disposable; the answer to the K-gates is not.

## The gate ladder

The ladder is the project's spine, and all of it must pass before
any parser work. A milestone (K_n) passes when its sub-gates do; a
sub-gate is a harness test with a measured pass criterion - pass
it, commit it, move to the next. Each milestone gets a harness
file in tests/ (k1_stability.rs exists; k2_self_assembly.rs,
k3_replication.rs, k4_variation.rs, k5_selection.rs follow).
Constants retune inside a sub-gate as the harness demands (F1's
lesson), but no sub-gate may be passed by adding a rule above the
atom/bond level (G01). Thresholds are starting points; they move
with evidence, never by wish. Gates are measured by the harness,
never by eyeballing a viewer.

Sub-gates are sized to be attackable - a sitting to a week each;
if one sprawls, split it.

## The re-validation contract

Owner decision 2026-09-07: a passed gate is passed against its
operating assumptions; when a later gate's work moves those
assumptions, the earlier gate RE-RUNS in that same commit - the
re-run result is part of the later gate's evidence, not an
optional follow-up. Current bindings:

- K1.1 re-runs in K1.4's commit (K1.4 moves the pond's settled
  operating temperature; K1.1 passed at the cold ~0.3 C point).
  EXERCISED 2026-09-07, K1.4's commit: the re-run FAILED the
  6.25k-10k window letter at +14.5% against the 15% bar - a pass
  by 0.5% is a flaky gate - and the measurement showed why: the
  K1.4 substrate's construction drain (each persistent bond
  sequesters 0.3*E until it breaks; the free population is
  consumed over ~14k ticks) extends the field-recovery transient
  to ~16k, so the old windows sat mid-recovery (field avg still
  31.5 C at 10k). Re-validated PASS with the window riding the
  measured steady tail: run horizon 10k -> 20k, windows
  16k-17.75k vs 18k-20k (KE/atom +2.3%, mean bond length +0.2%),
  coupling law 1.08-1.16 at every sample, KE bounded throughout.
- K1.1 re-runs in any integrator commit (sub-stepping removes
  the velocity clamp and deepens bond wells - K1.3's lever; the
  golden hash forces such commits to be conscious). EXERCISED
  2026-09-07, K1.3's commit: the re-run FAILED the original
  letter (KE/atom +32% window drift) and the measurement showed
  why - the quiesced chemistry starved the old refrigeration, so
  the vent + setpoint reservoir warm the field over ~6k ticks
  and KE rides the recovery. Re-validated PASS: thermostat
  coupling (KE/atom vs the field's warm-cell thermal level)
  constant 1.09-1.13 at every sample through the transient, KE
  bounded throughout, flatness over the steady tail (6.25k-8k vs
  8.25k-10k: KE +10.6%, bond length +0.3%)
- the velocity clamp (tunneling mint guard) was REMOVED in the
  K1.3 integrator commit (2026-09-07): the sub-stepping resolves
  the crossings the clamp guarded, satisfying this contract's
  requirement that the clamp be gone before the E-gates run
  their long horizons
- the 3D port (phase 2, ADR-0013) moves EVERY gate's
  assumptions at once - dimension itself. Its exit criterion is
  the whole K1 ladder re-run in 3D, gate by gate, in order, with
  the 2D pass records as the dimension-agnostic regression
  reference (ledger closure, coupling law, band bounds). K1.5
  still lands in 2D first: K1.4's F18/F19 fixes are
  dimension-generic (and more needed in 3D), and porting a
  substrate with known open pathologies makes 3D failures
  unattributable.
- no soft spot exists only in prose: every known limitation
  lives either in this contract or as a finding in
  docs/research/abstraction-notes.md with a named owner gate

## Milestone K1 - stability: the substrate holds together

K1.1-K1.4 passed 2026-09-05/07 (thermostat, force sanity, water
persistence, reactive balance); K1.5 remains. The findings that
shaped them: F6-F11, F17, F18, F19.

### K1.1 - thermostat: PASSED

Langevin damping toward the local field temperature. PASSED
2026-09-05: KE/atom window drift -5.7%, mean bond length +0.5%
over the 10k-tick vented run (tests/k1_stability.rs, release
--ignored). The pass required the substrate corrections F8-F16
(findings log in docs/research/abstraction-notes.md) and the
setpoint reservoir + vent (spec 6.1/6.2/11 synced).

RE-VALIDATED 2026-09-07 (K1.3's integrator commit, the contract's
integrator binding): coupling ratio constant 1.09-1.13 through
the measured field-recovery transient, KE bounded throughout,
steady-tail windows (6.25k-8k vs 8.25k-10k) KE +10.6%, bond
length +0.3%.

### K1.2 - force sanity: PASSED

A bonded overlap imparts bounded velocity (F9 measured v ~ 1e4 -
a cannon). PASSED 2026-09-07: the overlap probe (two bonded O
atoms at 0.05 * r_eq, zero field, one tick) imparts 0.046 A/tick
per atom against the analytic single-tick Hooke bound
k * r_eq / m = 0.048; the mean per-bond stretch ratio holds
1.000-1.002 (p95 <= 1.013, worst sample 13/1496 bonds outside
the band) at every sample of the same 10k-tick vented run
(tests/k1_stability.rs, release --ignored).

RE-VALIDATED 2026-09-07 in K1.3's integrator commit (stiffer
springs move the operating point): PASS - probe under the
analytic bound, band mean ratio 1.003-1.010, p95 <= 1.21, every
sample inside [0.8, 1.5].

### K1.3 - water persists: PASSED

A 35 C pond of H2O keeps its molecules - intact count flat, O-H
essentially never breaks (real chemistry's own exp(-29) answer),
the form+break cycle mints no energy (the F7 regression stays
green). PASSED 2026-09-07: intact 1024/1024 at every sample
through the 10k-tick vented run (one new water even assembled
from free atoms), ZERO seeded-water O-H breaks; the failing
substrate measured 1482 mechanical O-H breaks (all bombardment
overstretch, none thermal) and 206/1024 intact.

The fix was the re-validation contract's named lever, pulled
forward from phase 3: integration sub-stepping (4 sub-steps/tick,
dt_sub = 0.25) lets springs sit 8x stiffer inside the symplectic
bound evaluated at dt_sub - the O-H mechanical well went from
~10 kT (a thermal-speed hydrogen carries enough to shatter it)
to ~80 kT, real water's own ratio - and removes the velocity
clamp by resolving the crossings it guarded. Runtime O-H pairs
the free population forms and re-separates (26 breaks vs 41
formations) are reactive churn, counted and reported for K1.4,
not water loss. Spec 5.1/6.1/6.3/6.5/11 synced; golden hash
consciously updated; K1.1 and K1.2 re-ran in the same commit
(the contract's integrator binding, above).

### K1.4 - reactive balance: PASSED

A beaker of free atoms settles to a STATIONARY molecule-size
distribution - weak bonds break (O-O on a ~10k-tick scale), strong
ones persist; no runaway crosslinking, no frozen inertness;
formation refrigeration (F6) stays bounded and recovers. PASSED
2026-09-07 over a 20k-tick vented run (tests/k1_stability.rs,
release --ignored), every bar measured:

- the field dips to avg 12.2 C during the construction burst (each
  persistent bond sequesters 0.3*E; the free population is
  consumed over ~14k ticks) and RECOVERS to tail avg 35.6 C
  against the 35 C setpoint - F6 bounded and recovering;
- the size distribution is stationary on the tail (16k-18k vs
  18k-20k: 2-5 bucket -0.0%, free singles -8.8%, bonds +0.4%,
  cluster count -2 molecules); no runaway (largest molecule 22,
  21+ bucket 1, bonds 2406);
- the weak/strong asymmetry measured exactly at the ladder's
  scale: 18 O-O thermal breaks at mean age 10,007 ticks and 5
  N-N, ZERO thermal breaks of any strong pair, zero seeded-water
  breaks, zero phantom formations, one mechanical break (a
  collision outlier);
- the tail stays active (29 formations, 11 thermal breaks over
  15k-20k) - the flicker is alive, no frozen inertness.

The attack found and fixed two structural findings:

- F18 (wide-capture churn): formation used to accept any pair
  inside the 4 A search disc - 33% of formations were phantoms
  born past the mechanical break length, each silently keeping
  the absorbed formation heat, a standing refrigeration machine
  that held the field 8-12 C below setpoint and killed the
  thermal break channel. Fix: STERIC CONTACT - a bond may only be
  born where it can live (bond_form_factor 1.5, the
  excluded-volume standoff), with base_formation_rate retuned
  0.001 -> 0.01 for the contact geometry (F1's lesson: the rate
  was tuned for the wide-disc artifact).
- F19 (the thermal-release bomb): with the refrigeration gone,
  the field reached its true 35 C steady state for the first
  time - and the pond vaporized within ~500 ticks (every bond,
  field 1771 C): the thermal release was a delta function into
  one cell, and p_break is exponential in T, so each spike broke
  the neighbors and each secondary re-spiked. Fix: the release
  COMMITs to a per-cell reservoir (release_field) that the bath
  drains at release_rate_cap 2.0 - finite thermalization, F7's
  cycle conservation preserved, the cascade cut to ~1e-4
  secondaries per break.

K1.1 re-ran in the same commit per the contract (the binding
above): PASS after the windows rode the measured steady tail.

### K1.5 - seam correctness: OPEN (next)

The spatial index wraps in Wrap worlds (F11) - cross-seam
formation is symmetric with the bulk. Originally queued for
phase 3; promoted, because a Wrap world with asymmetric formation
cannot pass honest gates.

K1.5 is the 2D ladder's close: the 3D port follows (phase 2,
ADR-0013) and re-climbs K1.1-K1.5 in 3D before K2; K2-K5 then
climb in 3D.

## Milestone K2 - self-assembly: membranes are consequences, not rules

Precondition: K1. The literature is unanimous that amphiphiles
assemble through non-bonded potentials, never through springs
alone - hence K2.1 comes first.

### K2.1 - excluded volume

Soft non-bonded repulsion (F4 - smuggled PHYSICS, documented as
such): free atoms no longer pass through each other. PASS:
minimum approach distance >= 0.8 * (r_a + r_b) in a scattering
test. (Implemented with the K1.1 substrate work, spec 6.6; the
scattering-test PASS run is the gate's own commit.)

### K2.2 - condensed medium

The pond behaves as matter, not free flight - most atoms hold a
non-bonded neighbor within 3 A, and pair distances show
structure.

### K2.3 - amphiphile sorting

A lipid (polar head, nonpolar tails) in water - head-water and
tail-tail contact fractions beat chance by a set margin. No
"membrane" rule exists anywhere in the runtime.

DESIGN DEBT (registered in PLAN.md): the substrate's only
attraction is the bond spring; spec 6.6 is repulsion-only and
records the lesson itself ("the lipid literature is unanimous
that self-assembly needs non-bonded potentials, never springs
alone"). Amphiphile sorting needs a non-bonded polarity
ATTRACTION - an element-derived potential (electronegativity
differences), never a "lipids clump" rule - designed and
honesty-flagged per the F4 pattern BEFORE K2.3 tuning starts.
Phase 0 re-opens for this memo; the phantom-solvent and
four-bead amphiphile models (references.md) are the starting
point.

### K2.4 - vesicle closes

Lipids form a persistent cluster with an interior (union-find:
cluster >= 12 lipids, survives >= 10k ticks), heads pointing
outward.

### K2.5 - contents held

Free nucleotides placed inside a vesicle stay inside above a
leak threshold.

### K2.6 - vesicle grows

A fed vesicle incorporates lipids and grows (Squirm3 lesson:
membranes must grow, not just close). Division is NOT gated in
v0.1 - it is the first thing to chase after K5.

## Milestone K3 - replication: copying is chemistry, not code

Precondition: K2.5; the strand lives in a vesicle with free
nucleotides.

DESIGN DEBT (registered in PLAN.md): the K3 mechanism memo is a
precondition for K3.1 - three mechanisms the ladder assumes but
no phase designs yet. (1) The pairing channel: which bonds pair
bases - the table's weak entries (O-O 146, N-N 163, N-O 201
kJ/mol) are the candidates and K1.4's flicker channel would
activate them, but no document says base pairing IS those bonds.
(2) The thermal window (K3.3): weak-bond thermal breaking is
essentially never at pond temperature (~60 kT), so
hold-at-T_low / release-at-T_high needs a design - vent-adjacent
denaturation, a shallower effective well, a retuned break law -
and that choice IS the gate. (3) Ligation geometry: whether two
nucleotides paired side-by-side on a template ever reach
backbone-bond distance under VSEPR competition is calculable and
uncalculated - paper geometry plus a static beaker probe first.
The memo also preregisters the honesty line BEFORE any tuning:
what "base-pair geometry targets" may be tuned to (ADR-0003's
smuggled-biology flag), and what counts as an honest sweep for
the phase-1 exit criterion - so the stop-and-redesign tripwire
cannot be gamed in either direction. Written before K3 starts;
it feeds the port's grammar decisions (rotation representation).

### K3.1 - pairing

Free nucleotides bond the correct complement (A-U, G-C) far more
often than the wrong one (starting threshold 4:1) in a minimal
beaker. Base-pair geometry targets stay flagged as smuggled
biology (ADR-0003's honesty rule).

### K3.2 - templating

A seeded strand acquires a full complement - >= 80% of bases
paired within a fixed tick budget.

### K3.3 - separation

A thermal window exists - pairing holds at T_low, the duplex
releases at T_high - and the window moves by retuning constants,
not by adding rules.

### K3.4 - the copy

Template + free nucleotides + thermal cycling yields a free
daughter strand (complement signature in the event stream), and
the daughter templates a second generation.

## Milestone K4 - variation: copies carry errors

### K4.1 - copy errors

Wrong-base incorporation happens, and the measured mismatch rate
tracks a single tunable constant across a sweep.

### K4.2 - viable mutants

Most single-substitution daughters still copy (the K3.4 criteria)
- variation is not instantly lethal.

## Milestone K5 - selection: the pond has ecology

Precondition: K4; long runs - the phase-3 perf target and
dead-slot compaction are what make K5 practical.

### K5.1 - competition

Two lineages (differing fidelity or speed) share one pond with
capped nucleotide supply - both copy, one wins by a preregistered
margin. Needs lineage identity in the harness.

### K5.2 - turnover

With material feed + decay (the Squirm3 lesson - selection
starves without turnover), populations grow, crash, recover,
persisting for many generations without extinction or monoculture
takeover.

DESIGN DEBT (registered in PLAN.md): neither mechanism exists
yet. Decay has an honest candidate already in the spec - UV
photolysis folded into the break law (spec 8.2) - but nothing
declares it THE decay channel or prices its rate; material feed
has no mechanism at all (energy sources exist; material inflow
does not). A short turnover memo before K5: decay = photolysis
(or why not), feed = an honest inflow design - flagged per the
F4 pattern if either smells like smuggled biology.

### K5.3 - novelty probe

Reconstructed lineage trees keep producing new sequences over
time - open-endedness or convergence, measured against a metric
preregistered BEFORE looking (the Genesis Engine rule).

## Exit criterion

If K1-K3 do not pass after honest parameter sweeps (weeks, not
days), stop and redesign the substrate before building anything
on top. A beautiful language on a dead substrate is worthless.
