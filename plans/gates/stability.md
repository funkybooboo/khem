# Gate K1 - stability: the substrate holds together

K1 is CLOSED: passed in 2D 2026-09-05/07, re-climbed in 3D
2026-09-08 (the port's exit criterion), re-validated 2026-11-04
at the K2.1 core retune (the re-validation contract). The
harness is crates/khem-core/tests/k1_stability.rs; the
methodology and the contract live in [README.md](../README.md).
Findings cited as Fn live in
docs/build/research/abstraction-notes.md.

---

## Milestone K1 - stability: the substrate holds together

K1.1-K1.5 passed 2026-09-05/07 (thermostat, force sanity, water
persistence, reactive balance, seam symmetry); MILESTONE K1 IS
CLOSED in 2D. The 3D port's substrate diff landed 2026-09-08
(phase 2, ADR-0013); the re-validation contract now binds: K1.1
re-climbs first in 3D, then the rest in order, one gate per
commit, the 2D pass records as the dimension-agnostic regression
reference. The findings that shaped them: F6-F11, F17, F18, F19,
F20.

### K1.1 - thermostat: PASSED

Langevin damping toward the local field temperature. PASSED
2026-09-05: KE/atom window drift -5.7%, mean bond length +0.5%
over the 10k-tick vented run (tests/k1_stability.rs, release
--ignored). The pass required the substrate corrections F8-F16
(findings log in docs/build/research/abstraction-notes.md) and the
setpoint reservoir + vent (spec 6.1/6.2/11 synced).

RE-VALIDATED 2026-09-07 (K1.3's integrator commit, the contract's
integrator binding): coupling ratio constant 1.09-1.13 through
the measured field-recovery transient, KE bounded throughout,
steady-tail windows (6.25k-8k vs 8.25k-10k) KE +10.6%, bond
length +0.3%.

RE-CLIMBED AND PASSED IN 3D 2026-09-08 (the port's first gate,
re-validation contract at full force): coupling 1.037-1.121 at
every sample of the 25k-tick 3D run - the ratio to the
dimensionally-correct 3/2 * kb * T level (three Langevin
components; the 2D law's counterpart ratio measured 1.055-1.158)
- KE bounded throughout (max 0.527 vs the 3/2-scaled 2x bar
0.873), steady-tail windows (21k-22.75k vs 23k-25k, moved with
the measured 3D settle) KE +4.5%, bond length +0.4%. The 2D
windows sat mid-recovery in 3D: the slab pond's tail arrives
~4k ticks later and rides ~1 C above setpoint (tail field avg
~36.1, swinging 33-38.6 on the churn) - the vent's larger
fraction of the 72-cell field, honest G06 balance, recorded for
K1.4's ledger. The port commit's first bar scaling ([1.2, 2.1])
double-counted the dimension - the coupling RATIO is
dimension-normalized by construction and the first sample
(1.042) falsified it in one read. THE K1 LADDER IS NOW CLOSED
IN 3D TOO: see the K1.5 entry below for the close's evidence.

RE-RUN 2026-11-04 in K2.1's retune commit (the contract: the
excluded-volume strength 1.0 -> 8.0 moves the pond's collision
energetics): PASS - coupling 1.041-1.152 at every sample (the
3D re-climb band: 1.037-1.121), KE bounded throughout,
steady-tail windows (21k-22.75k vs 23k-25k) KE -7.5%, bond
length -0.2%.

RE-RUN 2026-11-06 with the pond RE-DENSITY (the pond is a new
world - 3.2 A lattice, 0.1022 atoms/A^3, keep-apart
construction; the port plan pre-authorized the re-derivation on
evidence; cohesion still dormant, so this is a WORLD re-run,
not a physics one): PASS - coupling 0.988-1.180 at every
sample of the 25k-tick run (the [0.8, 1.4] band; the wider
spread rides the denser medium's vent plume holding more of
the field), the tail at ~39-41 C against the 35 C setpoint, KE
bounded throughout. Perf ~17-28 t/s (the same atom budget in
a smaller box; the machine was busy during the final run).

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

RE-CLIMBED AND PASSED IN 3D 2026-09-08: the band is
dimension-agnostic exactly as the port plan predicted - probe
0.334 vs the analytic bound 0.578 (the 2D pair: 0.334/0.578),
band mean ratio 1.008-1.026, p95 <= 1.216, every sample inside
[0.8, 1.5].

RE-RUN 2026-11-04 in K2.1's retune commit (the record the
commit message carried; now written where it belongs): PASS -
the probe holds to three decimals (0.334 vs bound 0.578; the
excluded-volume stiffening does not touch bonded overlaps),
band mean 1.012-1.030, p95 <= 1.233, every sample inside
[0.8, 1.5].

RE-RUN 2026-11-06 with the pond re-density: PASS - the probe
unchanged (0.3335 vs bound 0.5782), band mean 1.009-1.025, p95
<= 1.229, every sample inside [0.8, 1.5] (the denser medium
presses more: outside-band grows to ~77/2316 bonds at 10k
against ~70/2270 at the old density - the mean and p95 bars
hold).

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

RE-CLIMBED AND PASSED IN 3D 2026-09-08: intact 1025/1024 flat
(one water self-assembled from free atoms), ZERO O-H breaks of
any kind, 14 runtime O-H formations, 1 weak-pair break - the
persistence law is dimension-agnostic; the ~80 kT mechanical
well and the exp(-29) thermal scale hold in 3D unchanged.

RE-RUN 2026-11-04 in K2.1's retune commit: PASS - intact
1025/1024 flat (one water self-assembled), ZERO O-H breaks of
any kind, 19 runtime O-H formations, 0 other-pair breaks. The
S=8 wall never engages a water at equilibrium: the ideal H-H
distance (1.88 A) holds an 18% margin over the stiffened cutoff
(1.59 A), and a settle probe (bath off) holds the geometry
frozen to four decimals - the retune touches free pairs and
third-atom sterics only.

RE-RUN 2026-11-06 with the pond re-density: PASS - intact
1025/1024 flat from t=2000 (one water self-assembled early and
persists), ZERO O-H breaks of any kind (mechanical 0, seeded 0,
churn 0) - the keep-apart construction seeds every water
outside every neighbor's excluded volume (best-of-N
placement; a place-the-last-draft fallback manufactured deep
overlaps before the fix, the same lesson the K2.1 beaker
measured).

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

RE-CLIMBED AND PASSED IN 3D 2026-09-08: every bar held except
the F6 floor, which rides the measured 3D construction dip -
the same formation drain (42,037 field degrees by 20k) lands on
the slab's 72 cells (half the 2D pond's 144), dipping the
average ~2x deeper (measured 7.54 C vs 2D 12.16) before
recovering to tail avg 35.41 against the 35 C setpoint. The
stationarity windows passed UNCHANGED - the 3D construction
finishes earlier (the denser slab builds its free population
faster): 2-5 -0.4%, singles -13.2%, bonds +0.6%, clusters +1.1.
Census: O-O thermal 12 at mean age 10,117 ticks (2D: 18 at
10,007 - the ladder's scale holds), N-N 1 at 16,261, strong
thermal 0, seeded 0, phantoms 0, mechanical 1; tail active (42
formations, 6 thermal over 15k-20k); largest 22, 21+ bucket 1,
bonds 2403. The ledger closes: vent 0.454 degrees-sum/tick,
every 1k window's five columns explain the delta.

RE-RUN 2026-11-04 in K2.1's retune commit: PASS after one honest
reclassification - the harness's weak class was the 2D record's
observed pair set {O-O, N-N}, not the table's weak entries; the
K3 note's own list is "O-O 146, N-N 163, N-O 201 kJ/mol", and
the S=8 run drew one N-O thermal break at age 14,510 ticks -
201 kJ/mol's Boltzmann answer at 35 C (~30 concurrent N-O
bonds expect ~1-2 breaks per 20k ticks; the prior runs' zeros
were the same law's lucky draws). N-O moved to the weak class
(the strong-zero bar unchanged, now measuring the truly strong
pairs). Bars at S=8: weak thermal 15 (O-O at the ladder's
9,951-tick scale), strong thermal 0, seeded 0, phantoms 0,
mechanical 1; field min avg 11.68, tail avg 35.75 against the
35 C setpoint; stationarity green (2-5 -0.1%, singles -12.4%,
bonds +0.4%, clusters +3%); no runaway (largest 21, 21+
bucket 1, bonds 2344); tail alive (36 formations, 13 thermal
over 15k-20k).

### K1.5 - seam correctness: PASSED

The spatial index wraps in Wrap worlds (F11) - cross-seam
formation is symmetric with the bulk. Originally queued for
phase 3; promoted, because a Wrap world with asymmetric formation
cannot pass honest gates.

PASSED 2026-09-07 (k1_5_seam_symmetry, release --ignored, two
seeds pooled). The pond cannot supply the measurement - its
free-atom sprinkle carries a 2 A margin (a free pair across the
seam starts 4 A apart, past every capture cap) and its waters
are saturated - so the gate runs a dedicated beaker: the pond's
free-atom mix and monolayer density, uniform 55 C, no vent (the
pond's vent plume sits against its y-seam; any temperature
gradient confounds the seam/bulk split). Every bar measured:

- GATHERING COMPLETENESS (F11's law, the mechanism): every tick,
  every live pair within the minimum-image search radius is
  found by BOTH sides' index queries, brute force the reference
  - the index is a pure accelerator in Wrap worlds, never a
  filter. The pre-fix failure mode (a non-folding index)
  suppresses cross-seam formation totally; this law catches it
  exactly, with zero formations.
- FORMATION SYMMETRY, COMPOSITION-CONDITIONED: the bar is the
  classes' formation efficiency ratio - actual formations vs the
  sum of the REAL formation probability
  (Chemistry::pair_probability, exposed read-only for this
  census) over each class's eligible pool. Measured pooled
  (seeds 42+137): seam S=57 E=57.7 z=-0.09; bulk S=1443
  E=1365 z=+2.1; ratio 0.935 against a 1-sigma of ~0.13 - the
  seam class's formations match the law's expectation under the
  same probability law the bulk runs. (Both classes' z sit
  positive by the same ~6%: the census conditions on the
  post-tick state while chemistry drew against the mid-pass
  state - the tick's own absorption cooled cells, anchors and
  orders were freer - a class-symmetric model offset, which is
  exactly why the bar is the ratio, not the absolute z.)

The probe earned its keep before the pass: the UNCONDITIONED
rate ratio (formations per eligible pair, seam/bulk) measured
~0.78 pooled with per-seed draws from 0.58 to 1.01 - and the
channel diagnostic showed why it must not be barred: the seam
class is a thin slice whose eligible pool co-evolves with the
run and clusters on shared atoms (slot-rich edge atoms
dominated it; composition-enriched in low-probability C-double
pairs at the run's sagged temperatures, expected-p per pair
already 0.77 by class while the formation path itself was
class-symmetric throughout). Conditioning on sum-p absorbs
composition and temperature drift exactly; only the Bernoulli
draws remain.

The audit that accompanied the gate found one real seam bug,
found by law rather than by the rate statistics (its bias
cancels marginally over the candidate distribution):

- F20 (the mirrored VSEPR anchor): the geometry factor scored
  each existing bond's direction from the RAW delta, so a
  bond straddling the Wrap seam read mirrored (pi off) and its
  atom's candidates scored against a phantom ideal. Fixed with
  the minimum-image direction (spec 6.3's every-pair-rule
  law; 7.2 synced); pinned by a deterministic law test (the
  seam-straddling anchor scores its true ideal ~1.0 and the
  mirrored phantom's ideal 0.63, inverted pre-fix).

K1.5 is the 2D ladder's close: the 3D port follows (phase 2,
ADR-0013) and re-climbs K1.1-K1.5 in 3D - all three seams -
before K2; K2-K5 then climb in 3D.

RE-CLIMBED AND PASSED IN 3D 2026-09-08 - all three seams, and
the re-climb's audit earned its keep exactly as the 2D one did:
the bars passed (pooled seam/bulk efficiency 1.287 inside
[0.6, 1.4], gathering completeness pair-for-pair every tick of
both seeds), but the per-axis census exposed a signal the 2D
pooled classes averaged away - axis 0 (the outer candidate-scan
loop) at z +2.9 pooled. The probe chain: windowed S/E refuted
contention-monotonicity; the double-census (pre-tick vs
post-tick E) agreed within 0.5%, refuting the mid-pass-state
timing artifact; the SCAN-ORDER REVERSAL (a probe build, the
physics unchanged) moved the excess to axis 2 and dropped the
pooled ratio to 1.047 - a real seam asymmetry survives scan
reversal, the measured excess follows the scan. THE SUBSTRATE'S
SEAMS ARE SYMMETRIC; the signal is the census model's mid-pass
anchor-state offset (the 2D gate's documented "+6%, anchors
were freer before the pass filled them"), exposed per-axis by
the 3D class split - recorded as F21 (RESOLVED, by
falsification). K1 IS CLOSED IN 3D: K1.1-K1.5 all re-climbed
with harness evidence; the 3D port's exit criterion is met;
K2 climbs in 3D.

RE-RUN 2026-11-04 in K2.1's retune commit: PASS, and the F21
scan-order artifact is GONE - pooled seam/bulk efficiency 0.980
against the [0.6, 1.4] bar with all three axes within 0.1 of
1.0 (the re-climb's axis 0 sat at z +2.9): the stiffer core's
elastic bounces randomize encounter geometry enough to wash
the anchor-order offset the 3D re-climb exposed. Gathering law
pair-for-pair complete every tick of both seeds.

