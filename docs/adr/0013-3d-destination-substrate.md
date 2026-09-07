# ADR-0013: 3D is the destination substrate; the port follows K1.5

Date: 2026-09-07 (owner decision)
Status: Accepted (renumbered 2026-09-07: the port phase is
phase 2, plan file docs/plans/phase-2-3d-port.md; the body's
phase numbers predate the renumbering - hardening is now
phase 3, the language phase 4; the decision is unchanged.
K1.5 passed 2026-09-07 - K1 closed in 2D, the port is unblocked.
The port's substrate diff landed 2026-09-08; the K1 re-climb
in 3D follows - decisions and landing evidence in the plan
file)

## Context

ADR-0002 chose a 2D world deliberately, named its artifacts
(membranes are rings; base pairing is planar), and deferred 3D as
"a port, not a redesign, if ever needed." Research (2026-09-07;
citations in docs/research/references.md, section
"Dimensionality") sharpened both directions, and the deferral
turned out to have a price curve.

2D does not block the mission. The strongest artificial-chemistry
precedents are 2D - Squirm3's and JohnnyVon's replicators, and
Ono & Ikegami's protocell whose membrane-bounded cell
self-organizes, grows, and divides on a 2D lattice - every K gate
is answerable in 2D, and 2D carries advantages this project
already leans on: total observability (the F-findings methodology
assumes the world can be inspected), and atom budget (a 2D
vesicle's wall costs O(R) lipids against the 3D shell's O(R^2) -
at radius 30 A with 6 A spacing, ~30 lipids versus ~300, so one
empty 3D vesicle plus its solvent plausibly eats half of the
phase-2 perf target's entire 10k-atom budget; in 2D the same
enclosure is a few hundred atoms).

But 2D compounds artifacts exactly where the mission's long run
goes - large molecules. Polymer chains in two dimensions cannot
pass through one another (Meyer et al.: "In a 2D melt, chains
cannot overlap"), a topological ceiling on coexisting long-chain
chemistry that only grows over billion-tick horizons. Tetrahedral
carbon, chirality, cis/trans, helices, and bilayers are 3D
objects; their 2D shadows stay recognizable but degenerate
(spec 7.4 concedes 109.5 cannot exist four ways in 2D).

And the timing question is real: the re-validation contract makes
any dimensionality change a full re-climb of the ladder, wherever
it lands. The only lever is WHEN. Every phase after K1.5 encodes
more 2D - K2-K5 rate tuning, the phase-2 save/load state layout,
the phase-3 grammar coordinates, the NDJSON position fields.
Port after those and the project pays twice. Port before K2 and it
pays once.

## Decision

3D is the committed destination substrate. Sequencing:

- 2D stands for V1 through the close of the K1 gates. K1.4 and
  K1.5 land in 2D first: the open pathology (F18 wide-capture
  churn) gets fixed where the baseline is understood - the fix is
  dimension-generic and more needed in 3D (a 3D search ball of
  the same radius holds a larger fraction of its volume past the
  break length) - and the 2D pass records become the port's
  regression vector: energy-ledger closure, the thermostat
  coupling law, and the bond bands are dimension-agnostic
  invariants the port must reproduce.
- The port is its own phase (docs/plans/phase-1.5-3d-port.md),
  gated on K1.5 and landed before K2 tuning. Its exit criterion
  is the K1 ladder re-passed in 3D. K2-K5, phase 2, and phase 3
  then run in 3D; the .kem grammar and stdlib geometry are
  authored for a 3D world once, not migrated.
- The port is a migration, not a mode: one dimensionality at a
  time. No 2D/3D dual-mode runtime, no per-world dimension flag -
  a mode doubles every test matrix and leaks into the hot loops.
  2D survives in git history.

## Consequences

- Every measured number is invalidated at the port, by design:
  the golden hash is re-cut consciously (the K1.3 precedent), the
  tuned constants re-tune during the 3D re-climb, and the perf
  baseline re-opens (a 4 A query scans up to 27 cells instead of
  9 at the current 5 A cell size; spec 13's targets stand).
- The NDJSON public contract (ADR-0004) and the .kem position
  grammar take their 3D encoding at the port - before any
  external consumer or parser exists to break.
- Spec 7.4's VSEPR semantics simplify: the 3D scoring ideals
  become literally expressible (109.5 four ways exists);
  effective geometry keeps emerging from gaussian competition.
- 2D's observability advantage is lost at the port - 3D membranes
  occlude and inspection gets harder. Accepted: the harness and
  the NDJSON stream are the eyes.
- The thesis framing is unchanged (an artificial-chemistry
  platform for emergent evolution, never real chemistry): the
  port buys mechanism geometry, not fidelity claims.
