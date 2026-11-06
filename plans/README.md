# Plans

How work is planned, prioritized, and tracked in khem. The
project's bet and rules live in [identity.md](./identity.md);
the queue lives in [roadmap.md](./roadmap.md).

## The pieces

| Path | What it is |
|------|------------|
| [identity.md](./identity.md) | The constitution: the bet, the fidelity philosophy, the guardrails |
| [roadmap.md](./roadmap.md) | Where the project stands + the queue of work |
| [gates/](./gates/) | The gate ladder, one file per milestone (the spine of the whole project) |
| [stories/](./stories/) | Future-phase work items (hardening, language, experiments) |
| [history.md](./history.md) | The timeline record; detailed completed-phase records in [history/](./history/) |
| [issues.md](./issues.md) | The gap register + open decisions, each with its trigger |

## Gate IDs (K-numbers)

The gate IDs (K1.1 ... K5.3) are the methodology's spine, like
story IDs in any tracked project - they name the rungs of the
ladder and never retire. They live in plans/ as cross-references
(gate files, history records, findings). Code and tests do NOT
carry them: files and functions are named for the behavior they
certify (`thermostat_holds_setpoint`, `excluded_volume`), with
the gate ID cited in a doc comment where the lineage matters.

## The gate ladder

The ladder is the project's spine, and all of it must pass before
any parser work. A milestone (K_n) passes when its sub-gates do; a
sub-gate is a harness test with a measured pass criterion - pass
it, commit it, move to the next. Constants retune inside a
sub-gate as the harness demands (F1's lesson), but no sub-gate
may be passed by adding a rule above the atom/bond level (G01).
Thresholds are starting points; they move with evidence, never
by wish. Gates are measured by the harness, never by eyeballing
a viewer.

Sub-gates are sized to be attackable - a sitting to a week each;
if one sprawls, split it.

The ladder itself, with pass records and evidence: [gates/](./gates/).

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
  landed in 2D first (PASSED 2026-09-07): K1.4's F18/F19 fixes
  are dimension-generic (and more needed in 3D), and porting a
  substrate with known open pathologies makes 3D failures
  unattributable.
- the real-water substrate sitting (ADR-0014) moves the wall
  geometry and adds the directional hydrogen-bond term: the full
  ladder re-validates on the new defaults (re-validation #3).
- no soft spot exists only in prose: every known limitation
  lives either in this contract or as a finding in
  docs/build/research/abstraction-notes.md with a named owner
  gate

## Exit criterion

If K1-K3 do not pass after honest parameter sweeps (weeks, not
days), stop and redesign the substrate before building anything
on top. A beautiful language on a dead substrate is worthless.

## The workflow

1. Gates are harness tests with measured pass criteria; a sitting
   ends with the gate's PASS record updated in its
   [gates/](./gates/) file, in the same commit as any constants
   it moved.
2. The implementation and the specs must AGREE (owner decision
   2026-09-05): any divergence between code and
   docs/build/specs/runtime-spec.md is fixed in both in the same
   commit, piecemeal. Designed-but-unbuilt items are marked
   [phase 3] / [phase 4] in the spec rather than allowed to
   drift.
3. The WHY of every decision lives in
   [docs/build/adr/](../docs/build/adr/) (Nygard format;
   immutable once accepted - change means a new ADR).
4. The gate is `mise run check` locally and identical in CI; the
   toolchain is pinned in mise.toml.
5. The founding conversation is recoverable from git history
   only: the transcript at commit d8205f1, the spec-draft
   extractions at 83a2688 and fefc4b9.
