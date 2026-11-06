# Gate K3 - replication: copying is chemistry, not code

OPEN - not started; climbs after K2 closes. The K3 mechanism
memo (base pairing, duplex release, ligation geometry) is a
[gap-register](../issues.md) item that must land before K3
starts, because it feeds the phase-4 grammar decisions. The
methodology lives in [README.md](../README.md).

---

## Milestone K3 - replication: copying is chemistry, not code

Precondition: K2.5; the strand lives in a vesicle with free
nucleotides.

DESIGN DEBT (the gap register, plans/issues.md): the K3 mechanism memo is a
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

