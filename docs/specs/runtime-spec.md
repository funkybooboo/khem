# khem runtime specification

Runtime version: 0.1 (emits output schema v:2)
Status: canonical as of 2026-09-04. Drafts until validated (ADR-0006):
revised against phase-1 kernel reality before the parser is built.
The substrate is 3D since the phase-2 port (2026-09-08, ADR-0013);
2D survives in git history.
Sync policy (owner decision 2026-09-05): the implementation and
this spec must agree; any divergence is fixed in both in the same
commit. Sync history: 2026-09-05 post-F10 and the K1.1 substrate
corrections (4.9, 6.1, 6.3, 6.6, 7.1, 7.2, section 11);
2026-09-07 the K1.3 integrator (5.1, 6.1, 6.3, 6.5, 6.6, 6.7,
section 11) and the open-boundary BOND_BROKEN event (3.3, 6.7);
2026-09-08 the 3D port (3.2-3.4, 4.4, 4.7-4.9, 5.3, 6.1, 6.2,
6.4, 6.5, 6.7, 7.2, 7.4, 8.1, 8.2, G13).
Items marked
[phase 3] / [phase 4] are designed but not yet implemented.
Provenance: reconciled from the founding conversation (preserved in
git history) with the final terminology applied (ADR-0007,
ADR-0009).

## 1. Overview

khem is the simulator: a physics engine. It accepts .kem
descriptions, outputs a structured event stream, and knows about
atoms, bonds, forces, and energy - nothing above that level (G01).

Terms: the engine is khem-core (the simulation library); the
runtime is the engine plus the khem binary; "the simulator" is the
runtime seen from the language side - the tool that runs what .kem
describes.

khem is designed to scale from a laptop to a large multi-machine
system. v0.1 implements single-machine execution. The architecture
must not prevent future scaling (section 10).

## 2. CLI

### 2.1 Usage

    khem [OPTIONS] <file.kem>

The file passed must declare run. khem resolves all dependencies from
there.

### 2.2 Options

    --seed <integer>   override the seed from the run declaration
                       khem --seed 42 experiment_1.kem
    --check            [phase 4] parse and validate all files; report
                       errors and warnings; do not run. Exit 0 if
                       valid, 1 if not.
                       khem --check primordial_pond.kem
    --test             [phase 4] run a single struct or body in
                       isolation in a minimal world for a default
                       test duration.
                       khem --test minimal_cell.kem
    --info             [phase 4] parse a .kem file and print its
                       structure: atom count, bond count, port list,
                       import tree. Does not run.
                       khem --info nucleotide_A.kem
    --version
    --help

Phase 1: the binary runs the hardcoded primordial pond; a file
argument is accepted but not read (the parser is phase 4), and
--seed is the only behavioral option (default 42).

### 2.3 Exit codes

    0   success (simulation completed, or --check passed)
    1   validation error (bad .kem files or bad command line)
    2   runtime error (crash during simulation)
    3   user interrupt (SIGINT / ctrl-c)  [phase 3: v0.1 phase-1
        builds do not trap signals; ctrl-c kills the process]

### 2.4 Streams

    stdin    not used in v0.1
    stdout   NDJSON event stream (simulation output), nothing else
    stderr   errors, warnings, progress info, nothing else

The separation is strict. No simulation data reaches stderr; no
diagnostics reach stdout. Consumers may redirect each independently.

    khem experiment_1.kem > data.ndjson 2> errors.log
    khem experiment_1.kem 2>/dev/null | my_tool
    khem experiment_1.kem | tee data.ndjson | khem-view

## 3. NDJSON output

### 3.1 Format

Newline-delimited JSON: one complete, valid JSON object per line, no
wrapping array, parseable line by line without buffering the entire
output.

### 3.2 Common fields

Every event contains:

    v      integer   output schema version; 2 since the 3D port
                     (v:1 was the 2D stream; see 3.4)
    type   string    event type
    tick   integer   simulation tick when the event occurred

### 3.3 Event types

start - first line, emitted once:

    {"v":2,"type":"start","tick":0,"khem_version":"0.1.0",
     "run_name":"experiment_1","world_name":"primordial_pond",
     "seed":42,"atom_count":4821,"bond_count":341,
     "world_width":200.0,"world_height":200.0,"world_depth":40.0}

tick - every tick_interval ticks (timing fields are wall-clock
and excluded from reproducibility, see G02):

    {"v":2,"type":"tick","tick":1000,"elapsed_ms":124,
     "ticks_per_sec":8064,"atom_count":4821,"bond_count":2341,
     "temp_min":12.3,"temp_max":847.2,"temp_avg":34.1,
     "pressure_min":0.8,"pressure_max":20.1,"pressure_avg":4.2,
     "free_atoms":{"H":892,"C":234,"O":445},
     "mol_size_dist":{"1":892,"2_5":445,"6_20":89,"21plus":12}}

bond_formed - when output.bond_events is true:

    {"v":2,"type":"bond_formed","tick":1247,"bond_id":4521,
     "atom_a":442,"atom_b":891,"elem_a":"C","elem_b":"O",
     "order":2,"energy":799.0,"x":45.2,"y":123.7,"z":12.4}

bond_broken - when output.bond_events is true:

    {"v":2,"type":"bond_broken","tick":1248,"bond_id":4521,
     "elem_a":"C","elem_b":"O","energy_released":399.5,
     "x":45.3,"y":123.8,"z":12.4}

Emitted for every bond break, including Open-boundary removals
(energy_released 0; the boundary takes the bond with no field
exchange, 6.7).

notable - [phase 3] when a watch condition triggers; always
emitted regardless of output settings:

    {"v":2,"type":"notable","tick":1247900,"event":"largest_molecule",
     "data":{"atom_count":47,"first_seen_tick":891000}}

Event vocabulary:

    largest_molecule     new largest molecule found
    extinction           all molecules below size 2
    population_surge     population change above threshold
    population_crash     population drop above threshold
    bond_type_first      bond type seen for the first time

save - [phase 3] when state is saved:

    {"v":2,"type":"save","tick":1000000,"path":"./saves/tick_1000000.state"}

end - last line, emitted once:

    {"v":1,"type":"end","tick":5000000,"elapsed_ms":620000,
     "reason":"max_ticks_reached"}

Reason vocabulary: max_ticks_reached, user_interrupt, extinction,
runtime_error.

### 3.4 Schema versioning

The v field is the output schema version. khem v0.1 emits v:2 since
the 3D port (2026-09-08): the stream grew `z` on the bond events and
`world_depth` on start - additive fields, but the world model changed
dimensionality, and a consumer that silently ignored the new fields
would mis-model geometry from x/y alone. v:1 was the 2D stream (in
git history); no external consumer existed at the bump, which is why
the port plan called the port "the cheap moment". Consumers must
handle unknown v values gracefully; within a v the contract only
grows additively.

## 4. Core data structures

### 4.1 Design principles

All primary data lives in flat arrays indexed by integer ID. No heap
allocation per atom per tick. No pointers into world state.
Cache-friendly layout, partitionable by spatial region. These choices
are what make V2 (threads) and V3 (machines) possible without
restructuring (section 10).

### 4.2 AtomId, BondId

    type AtomId = u32     // 4 billion atoms
    type BondId = u32     // 4 billion bonds

### 4.3 ElementId

    type ElementId = u8   // 256 element types; index into element table

### 4.4 AtomState

Fixed-size struct, no heap allocation:

    AtomState {
        id:         AtomId
        element:    ElementId
        x:          f32            // angstroms
        y:          f32
        z:          f32
        vx:         f32            // angstroms per tick
        vy:         f32
        vz:         f32
        bonds:      [Option<BondId>; 6]  // first bond_count slots
                                         // are Some; empty slots
                                         // are None (an empty slot
                                         // must be representable;
                                         // raw-array deviation
                                         // from the founding draft,
                                         // synced 2026-09-05)
        bond_count: u8
        alive:      bool
    }

Dead atoms are flagged and compacted periodically (5.2), never
removed immediately - removal would invalidate indices.

### 4.5 BondState

    BondState {
        id:      BondId
        atom_a:  AtomId
        atom_b:  AtomId
        order:   u8              // 1 | 2 | 3
        alive:   bool
        energy:  f32             // kJ/mol
    }

### 4.6 ElementProperties

Immutable after load, shared by reference:

    ElementProperties {
        symbol:            [u8; 2]
        atomic_number:     u8
        max_bonds:         u8
        valence:           u8
        mass:              f32     // daltons
        electronegativity: f32     // Pauling scale
        radius:            f32     // covalent radius, angstroms
    }

### 4.7 WorldState

The complete mutable simulation state:

    WorldState {
        tick:           u64
        atoms:          Vec<AtomState>
        bonds:          Vec<BondState>
        width:          f32
        height:         f32
        depth:          f32
        boundary:       BoundaryType
        temp_field:     Grid3D
        release_field:  Grid3D     // committed bond-break heat,
                                   // drained into temp_field at
                                   // release_rate_cap (6.2; F19)
        setpoint_field: Grid3D     // declared environment setpoints;
                                   // 0 = none (6.2)
        pressure_field: Grid3D
        uv_field:       Grid3D
        energy_sources: Vec<EnergySource>
        element_table:  Arc<Vec<ElementProperties>>
        spatial_index:  SpatialIndex
        rng:            Rng               // deterministic, seeded
        event_queue:    Vec<Event>        // observer fills; flush per tick
    }

### 4.8 Grid3D

Field values (temperature, pressure, UV) on a grid coarser than atom
positions. Default cell size 10 angstroms. Index =
col + row * cols + layer * cols * rows (the 2D flat index plus a
layer stride - the port's one structural change to the grid).

    Grid3D { data: Vec<f32>, cols: u32, rows: u32, layers: u32,
             cell_width: f32, cell_height: f32, cell_depth: f32 }

### 4.9 SpatialIndex

Spatial hash for neighbor queries. Default cell size 5 angstroms.
Rebuilt inside the sub-step loop and again after boundary
application (5.1); rebuild is O(n), queries are O(1) average. Wrap-aware on every axis since the 3D port (finding F11; the
2-torus became the 3-torus): cell coordinates fold at the grid
edges in Wrap worlds so seam-crossing candidates are found -
consistent with the minimum-image chemistry that evaluates them;
Wall and Open use raw coordinates. A radius-r query scans up to
27 cells (3x3x3) at the 5 A cell size against the 2D index's 9.
The index is a pure accelerator, never a filter: every pair within
the minimum-image query radius is found by both sides' queries (a
law, pinned always-on in the spatial tests and measured
pair-for-pair against brute force by gate K1.5, 2026-09-07; the
3D re-climb re-runs it on three seams).

    SpatialIndex { cells: HashMap<(i32, i32, i32), Vec<AtomId>>,
                   cell_size: f32 }

### 4.10 BoundaryType

    enum BoundaryType { Wrap, Wall, Open }

## 5. Tick execution

### 5.1 Tick order

Systems execute in strict order. Each reads current state; writes are
committed before the next system reads; no system reads another
system's writes within the same tick. The tick is dt = 1 for
chemistry, the thermostat, and the fields, but the force/motion
pair is SUB-STEPPED: steps 3-5 below repeat integration_substeps
times per tick at dt_sub = 1/integration_substeps (6.5, gate K1.3).

    1.  EnergySystem::update
    2.  PhysicsSystem::apply_bath        (temperature diffusion,
                                           setpoint relaxation,
                                           Langevin kicks; once)
    3.  SpatialIndex::rebuild               \ the sub-step block:
    4.  PhysicsSystem::update_velocities    | these three repeat
    5.  PhysicsSystem::update_positions     | integration_substeps
                                            /  times per tick
    6.  PhysicsSystem::apply_boundary
    7.  SpatialIndex::rebuild
    8.  ChemistrySystem::break_bonds
    9.  ChemistrySystem::form_bonds
    10. ObserverSystem::sample
    11. EventQueue::flush_to_output

The first sub-step enters with the index already current (the
previous tick's post-boundary rebuild; the bath moves velocities,
not positions), so the loop rebuilds before every sub-step after
the first - mid-tick motion would otherwise leave the non-bonded
candidate sets stale behind the motion they must resolve, the
exact asymmetric sampling the sub-stepping exists to remove.

### 5.2 Dead atom and bond cleanup [phase 3]

Atoms and bonds are flagged dead, not removed. Compaction runs every
compaction_interval ticks (default 10,000): dead entries removed, IDs
remapped, spatial index rebuilt, compaction noted on stderr (never
stdout). Not yet implemented: phase 1 flags and skips dead entries;
nothing compacts, so long runs accumulate dead-slot memory.

### 5.3 Determinism

Given the same WorldState at tick 0 and the same seed, the run is
byte-identical forever (G02, G14). Requirements: fixed tick order
(5.1); one deterministic seeded RNG with per-tick state; no
thread-local state in v0.1; iteration over atoms always by AtomId.

RNG draw discipline (pinned by the phase-1 kernel, ADR-0005;
three components per atom since the 3D port - x, y, z): the
physics system draws first - exactly three normal draws per live
atom per tick, in AtomId order, inside the once-per-tick bath step
(dead atoms draw nothing; zero-temperature atoms draw no-op
samples). Bond breaking draws exactly one uniform per live bond
per tick, in BondId order, except bonds the mechanical overstretch
rule breaks first: those break deterministically before the roll
and draw nothing. Formation draws exactly one uniform per
eligible pair, in iterating-AtomId order with candidates in spatial
scan order; ineligible pairs draw nothing. The integration
sub-steps draw nothing.

## 6. Physics system

### 6.1 Temperature and velocity

The temperature field is the thermal bath: initialized by the
world definition, held near its declared setpoints (6.2), and
perturbed by energy sources (8) and bond events (7.1, 7.2). The
Langevin thermostat couples atoms to it (implemented 2026-09-05,
gate K1.1):

    s   = sqrt(thermostat_damping * (2 - thermostat_damping))
          * sqrt(thermal_kick_scale * T / mass)
    v'  = v * (1 - thermostat_damping) + rng.normal(0, s)

applied per component (x, y, z - three normal draws per atom per
tick), so velocities relax to the local field temperature with the
correct stationary variance (KE/atom settles at 3/2 * kb * T,
three-dimensional equipartition) instead of random-walking upward
forever (finding F8's fix). T <= 0 gives s 0 (pure damping; the
draws still happen, keeping the RNG stream uniform).

Fluctuation-dissipation bookkeeping keeps the field the honest
ledger: the damping DEPOSITS the KE it removed into the atom's
cell, and the noise's energy is paid through the same signed
delta (it arrives inside v'). Net zero at equilibrium; off
equilibrium energy flows both ways. The invariant:
field + KE * ke_field_scale is constant through the bath.

All accumulated forces are divided by the atom's mass at
application (F = ma; the founding draft said "applied to both
atoms" without /mass - with unit-mass hydrogen hiding the error,
each oxygen interaction minted ~(0.5*m - 1) * F^2: finding F12).
No velocity clamp exists: the sub-stepped integrator (6.5) resolves
the fast-crossing "tunneling mint" the founding substrate needed
one for (history and measurements: F13, F17 in
docs/research/abstraction-notes.md).

### 6.2 Temperature diffusion

Executes at the start of PhysicsSystem::apply_bath, before
the kicks sample the field (5.1 names no slot for it; the kernel
pinned this one). FIRST, the thermal-release reservoir drains:
each cell moves at most release_rate_cap degrees of committed
bond-break heat (7.1) from release_field into its temperature
cell - a finite thermalization rate (finding F19: the
instantaneous dump was a bomb. One O-H release is ~139 degrees;
spiked into a water-lattice cell it lifted local p_break through
the exponential, broke the neighbors' O-H bonds, and each
secondary break re-spiked the cell - measured, the pond reached
its steady-state 35 C and every bond broke within ~500 ticks,
field at 1771 C. The bounded drain holds a cell at ~+20 C above
its neighbors even under a full-cap stream, and cuts the cascade
to ~1e-4 expected secondaries per break - the feedback cannot
close - while conserving the full release into the field (F7's
cycle law, spread over E/0.3/cap ticks). Then diffusion, per
cell, 6-connected neighbors (the x, y, and z neighbor pairs since
the 3D port; the 2D stencil was 4-connected), wrapped at the
grid edges on every axis (grids wrap like the Wrap boundary, 4.8). After
diffusion, cells with a declared setpoint (> 0 in
setpoint_field) relax toward it at field_relax_rate - the
environment reservoir, the pond's heat sink. Without it a vented
Wrap world only heats (a vent injects continuously; nothing
leaves), and no steady state exists to be stable against (gate
K1.1's vented-pond flatness criterion requires it). Region
declarations (phase 4) are the setpoint source; the phase-1 pond
declares 35 C everywhere.

    T_new = T * (1 - diffusion_rate) + mean(6 neighbors) * diffusion_rate

diffusion_rate default 0.1. release_rate_cap default 2.0.

### 6.3 Bond forces

Hooke's law toward equilibrium distance (sum of covalent radii):

    r_eq    = elem_a.radius + elem_b.radius
    F       = spring_k * (r - r_eq)
    spring_k = bond.energy * spring_energy_scale

Applied to both atoms along the bond axis, equal and opposite,
divided by mass at application (F = ma; see 6.1). ONE smooth Hooke
law both directions: stretched is attractive, compressed is
repulsive through the same F = k * (r - r_eq), bounded at
k * r_eq near coincidence - no hard core (a force discontinuity
pumps symplectic integrators; history: F9,
docs/research/abstraction-notes.md). Coincident atoms (r ~ 0) have
no defined axis; the force is skipped and the next kick separates
them.

In Wrap worlds every pair displacement uses the minimum-image
convention on every axis (the shortest vector between the atoms,
crossing whichever of the three seams is shorter): raw deltas read
a seam-adjacent pair as width - 1 angstroms apart and the spring
shreds it (finding F10, found by test 2026-09-05, fixed the same
day). All pair rules (springs, chemistry distance checks, bond
midpoints, VSEPR anchor directions) use it.

The stability law (pinned by test for every formable bond at the
configured scale): symplectic Euler is stable for
dt_sub * sqrt(k / reduced_mass) < 2, evaluated at the integration
sub-step (6.5); spring_energy_scale sits inside that bound with
margin - which at dt_sub = 0.25 holds real-water bond well depths
(O-H ~80 kT; worst formable pair H-H at 1.32 < 2). History and
measurements: F2, F17 in docs/research/abstraction-notes.md.

### 6.4 Pressure force

    pressure[cell] = atom_count_in_cell / cell_volume

Each atom feels force from the central-difference pressure
gradient on every axis, scaled by pressure_sensitivity, divided by
mass at application (6.1).

### 6.5 Position update and integration

    x += vx * dt_sub
    y += vy * dt_sub
    z += vz * dt_sub
    dt_sub = 1.0 / integration_substeps

Implemented 2026-09-07 (gate K1.3): the force/integration pair
(update_velocities then update_positions, 5.1 steps 4-5) repeats
integration_substeps times per tick; the sub-steps sum to dt = 1.0
per tick (one tick = one femtosecond at default scale), so
chemistry, the thermostat bath, and the field updates all keep
the tick as their time unit. The sub-stepping resolves the
short-range dynamics at dt_sub: an atom crossing the non-bonded
zone in several sub-steps samples the force symmetrically (no
tunneling mint, 6.1), and the spring stability bound (6.3) is
evaluated at dt_sub - which is what allows real-water bond well
depths at the configured spring scale. Default 4 (dt_sub = 0.25):
sub-step displacement of the fastest thermal atoms (~2-3 A/tick
tail) stays under half the narrowest interaction zone.

### 6.6 Non-bonded interactions

Excluded volume (implemented 2026-09-05; the founding draft had
none - unbonded atoms passed through each other, and the lipid
literature is unanimous that self-assembly needs non-bonded
potentials, never springs alone):

    cutoff = (radius_a + radius_b) * non_bonded_margin
    F      = non_bonded_repulsion * (cutoff - r),  for r < cutoff

Applied to every UNBONDED live pair (bonded pairs are exempt -
springs own them; same-molecule 1,3 pairs are NOT exempt: real
sterics, mild by construction since VSEPR ideals keep most
beyond cutoff), along the minimum-image axis, equal and opposite,
divided by mass at application. Candidates come from the
wrap-aware spatial index (4.9). This is smuggled PHYSICS,
documented as such: the substrate has excluded volume because
matter does, not because any biology needs it.

### 6.7 Boundaries

    Wrap   x = x mod width; y = y mod height; z = z mod depth
    Wall   clamp position; reverse the velocity component (all axes)
    Open   atom flagged dead (left the box on any axis); bonds
           broken first

Open-boundary bond breaks emit BOND_BROKEN (energy_released 0, no
field exchange): the stream stays a complete record of bond
liveness for spec 3.3 consumers - a bond never silently vanishes
at the edge.

## 7. Chemistry system

### 7.1 Bond breaking

Per alive bond, in BondId order. FIRST, mechanical dissociation:
bonds stretched past bond_break_factor * r_eq break
deterministically, no RNG roll, NO heat release - the stretch
already spent the energy, and the vanishing spring potential is a
sink (a heat-releasing length break would cascade: break heat ->
kicks -> shoves -> breaks; sinks cannot). Real bonds do not
stretch to multiples of their length (measured without the rule:
30-80 A "bonds"; F15). Then the thermal roll:

    T       = temperature at the bond midpoint (minimum-image)
    p_break = exp(-bond.energy / (kb_scaled * T))   // Boltzmann

T <= 0 gives p_break 0. UV photolysis (8.2) is folded into the same
per-bond roll as one combined probability; if rng < p: flag the bond
dead, decrement both atoms' bond_count, and COMMIT bond.energy *
release_fraction to release_field at the midpoint - the bath
carries it into the temperature field at release_rate_cap (6.2:
finite thermalization; the instantaneous cell dump was a
detonation feedback, F19).

### 7.2 Bond formation

For each atom A with available bond slots, candidates within
bond_search_radius (default 4.0 angstroms) via the spatial index:
each unordered pair is attempted at most once per tick, from the
iteration of the lower AtomId (A is the geometry anchor; a small
documented asymmetry). Eligibility, checked before the RNG draw:

    - both atoms alive
    - bond capacity on both sides
    - minimum-image distance within the search radius
    - not already bonded to each other
    - steric contact: the pair sits within bond_form_factor *
      r_eq - bonds form where the atoms touch, never at
      multiples of their own length. A bond born past the 7.1
      break length is a phantom the next pass kills silently,
      keeping its absorbed formation heat; a wide-but-legal birth
      mints the stretch PE that flings the pair into a
      mechanical break within ticks (finding F18; measured 123 +
      40 of 373 formations in the 10k-tick K1.4 probe)
    - capture: relative speed below max_form_speed - a pair
      flying past cannot bond (it would have to absorb their
      relative KE as stretch and become a comet; F14)

    p_form = base_formation_rate
           * geometry_factor(A, B)
           * temperature_factor(T, elem_a, elem_b)
           * (1.0 + |EN_a - EN_b| * en_bonus)

- geometry_factor (semantics pinned by the kernel; 3D since the
  port): an atom with no existing bonds is unconstrained (1.0),
  as are H, Na, Cl. Otherwise the ideal adjacent-bond angle comes
  from the 7.4 table for the atom's coordination state (doubles
  shift carbon to 120/180, nitrogen to 120); the candidate
  DIRECTION (a unit vector) is scored against each existing bond's
  MINIMUM-IMAGE direction (6.3: every pair rule; a
  seam-straddling bond's raw direction reads mirrored - pi off in
  the crossing axis - and candidates would score against a
  phantom ideal; finding F20, fixed 2026-09-07 with gate K1.5):
  the deviation is |angle(candidate, bond) - ideal|, the angle
  between the two unit vectors against the 7.4 ideal, scored by a
  gaussian with sigma = geometry_sigma; the best-scoring existing
  bond anchors the factor. The table values are scoring ideals,
  not enforced angles - and since the port they are literally
  expressible (the 109.5 tetrahedral ideal is a cone off the
  anchor, four ways in 3D; the 2D substrate could only approximate
  it in-plane); effective geometry emerges from the competition
  (water's 104.5 fits and matters most).
- temperature_factor: gaussian in T around the pair's optimum
  t_opt = t_opt_scale * bond_energy (stronger bonds tolerate
  hotter formation), width t_width.
- bond order: 2 when both atoms have 2+ free bond slots
  (max_bonds - bond_count), else 1. Triples are never formed in
  v0.1; they exist only if seeded into the initial world.

If rng < p_form: create a BondState with energy from the table (7.3),
increment both bond_counts, absorb bond.energy * formation_fraction
from the local temperature field at the minimum-image midpoint. The
field may go negative; kicks clamp at zero T and diffusion smooths
(formation refrigerates a source-less pond - finding F6, measured).

Diagnostic surface (added 2026-09-07 with gate K1.5):
`Chemistry::pair_probability` exposes the law read-only - order,
energy, p_form, and the minimum-image midpoint for a checked-
eligible pair, with the anchor passed first (the lower AtomId,
7.2's documented asymmetry). No RNG draw, no mutation. Gate K1.5's
census conditions on the realized candidate pool with it (sum of
p per class vs actual formations); the raw formations-per-
eligible-pair ratio is pool-composition noise on a co-evolving
population (measured: per-seed draws 0.58-1.01 in a beaker whose
formation path was class-symmetric throughout) and is reported,
not barred.

### 7.3 Bond energy table (kJ/mol, real values)

    H-H  436      H-C  413      H-N  391      H-O  463      H-S  363
    C-C  346      C=C  614      C#C  839
    C-N  305      C=N  615      C#N  891
    C-O  358      C=O  799      C-S  272      C-P  264
    N-N  163      N=N  418      N#N  945
    N-O  201      N=O  607
    O-O  146      O=O  498      O-P  335      O-S  265
    P-P  201      S-S  266
    Si-O 452      Si-Si 222
    Fe-O ~390 (variable)

Unknown pairs use the geometric mean of the elements' single-bond
reference energies (each element's own single bond where the table
has one; Na 77 and Cl 243 fill the two element gaps; note N's
reference is its genuinely weak 163). An unknown order for a known
pair scales the pair's single-bond energy by order (v0.1 fallbacks,
pinned by the kernel and test-locked).

Phase 1 hardcodes this table in khem-core/src/chemistry.rs, with a
test transcribing every row against this section; physics.cfg
loading arrives in phase 4 with the language.

### 7.4 Bond angle table (VSEPR, degrees)

    H    no constraint (1 bond max)
    C    4 bonds: 109.5 tetrahedral
         3 bonds + 1 double: 120 trigonal planar
         2 doubles: 180 linear
    N    3 bonds: 107 trigonal pyramidal
         2 bonds + 1 double: 120
    O    2 bonds: 104.5 bent
    P    5 bonds: 90/120 trigonal bipyramidal
         4 bonds: 109.5 tetrahedral
    S    2 bonds: 103 bent
    Si   4 bonds: 109.5 tetrahedral
    Fe   up to 6 bonds: 90 octahedral
    Na   1 bond: no constraint
    Cl   1 bond: no constraint

Also in physics.cfg, not source code. [phase 4; phase 1 hardcodes
this in khem-core/src/chemistry.rs with test-locked values]

Semantics (pinned by the kernel; 3D since the port): the angles
are scoring ideals for the 7.2 geometry factor, not enforced
constraints. In 2D the values could not all exist (109.5 four
ways exceeds the plane), so effective geometry emerged from
gaussian competition against in-plane approximations; since the
port the ideals are literally expressible, and the competition
still decides effective geometry.
Elements with no listed constraint score 1.0 always. Carbon's
coordination states: 109.5 with only single bonds, 120 when one
double is involved, 180 with two. Phosphorus switches to 90 for
its fifth bond.

## 8. Energy system

### 8.1 Hydrothermal vent

Per tick, for cells within radius (3D distance; a source position
is a 3-tuple since the port):

    falloff = 1.0 / (1.0 + distance^2 / radius^2)
    temp_field[cell] += intensity * falloff * vent_heat_rate

Atoms in radius get upward velocity along the VERTICAL AXIS
(+z, toward the surface - the top cell layer, 8.2; the 2D
substrate's "up" was +y): vz += convection_rate * falloff.

### 8.2 Solar UV

Per tick, the energy system writes the UV field: surface cells
(the top layer - cells centered above surface_threshold * depth
on the z axis, the vertical since the port) carry the source
intensity, all other cells zero. The field is per-tick state,
rebuilt wholesale.

UV bond breaking executes in ChemistrySystem::break_bonds (5.1
gives chemistry the only bond-mutating steps; one place breaks
bonds, and one RNG roll per bond combines thermal and UV - 7.1):

    p_uv_break = uv_field[cell] * uv_sensitivity[bond_type]

uv_sensitivity defaults per bond order: single 0.0001, double
0.0003, triple 0.0002.

### 8.3 Energy tracking

Total kinetic, bond potential, and field energy are computed as
diagnostics by the K1 harness (tests/k1_stability.rs) for
stability-gate measurement. They are NOT fields in the v:1 NDJSON
schema (3.3 has none); adding them would be an additive schema
change, made when a consumer needs them.

## 9. Observer system

### 9.1 Role

Read-only access to WorldState (G03). Never modifies simulation
state. Runs after all physics and chemistry each tick.

### 9.2 Molecule detection

Every sample tick (tick_interval): connected components on the bond
graph via union-find; each component is a molecule; collect the size
distribution; track the largest molecule ever seen.

### 9.3 Watch conditions

Each watch condition from the run declaration is evaluated after
molecule detection. Conditions are stateful (previous state is needed
to detect change). Triggered conditions emit NOTABLE events.

### 9.4 Output

The event queue is flushed to stdout after the observer runs; each
event is one JSON line; stdout is flushed per tick so pipe consumers
receive data promptly.

## 10. Scalability

### 10.1 V1 (this version)

Single thread, single machine, all data in one Vec. Target: 10,000
atoms at >500 t/s on a laptop (section 13).

### 10.2 V2 preparation (must not be prevented)

Spatial decomposition: regions own disjoint atom sets; regions are
independent except at boundaries; one thread per region; boundary
atoms handled with the ghost-cell pattern (standard in parallel
physics). Already enabled by: SpatialIndex is region-aware; AtomState
uses IDs, not pointers; no global mutable state in the systems.

### 10.3 V3 preparation (must not be prevented)

Distribution: regions on separate machines. WorldState serialization
is already complete (save/load, G09) and is reused for distribution.
NDJSON output is unchanged; an aggregation node collects per-machine
streams.

### 10.4 Plugin preparation

The physics and chemistry systems sit behind traits:

    trait PhysicsSystem    { fn update(&mut self, world: &mut WorldState); }
    trait ChemistrySystem  { fn update(&mut self, world: &mut WorldState); }

v0.1 compiles exactly one implementation of each. Future versions may
load dynamic libraries implementing the traits.

## 11. Configuration (physics.cfg)

All tunable constants live in physics.cfg - separate from .kem
definitions, in the project root or a system default path, modifiable
without recompilation. Tuning affects behavior and stability; it never
changes what chemistry is possible. [Phase 1: these are the defaults
in khem-core/src/config.rs, test-locked; physics.cfg loading arrives
in phase 4.]

Values below are the phase-1 tuned set (tuned 2026-09-05;
integration_substeps and spring_energy_scale retuned 2026-09-07
with the K1.3 integrator); the founding draft's literals are
recorded in git history and in
docs/research/abstraction-notes.md (findings F1, F2, F7) with the
measurements that changed them:

    kb_scaled               0.45     // Boltzmann for BREAKING (7.1);
                                     // literal 0.008314 gave zero
                                     // breaks at pond temps (F1)
    thermal_kick_scale      0.008314 // kick sigma scale (6.1); split
                                     // from kb_scaled - one physical
                                     // constant set two
                                     // incompatible sim scales
    thermostat_damping      0.1      // Langevin gamma (6.1, K1.1)
    ke_field_scale          0.01     // field degrees per KE unit
                                     // exchanged by the bath
    integration_substeps    4        // force/motion sub-steps per
                                     // tick, dt_sub = 1/n (6.5,
                                     // K1.3; replaced the velocity
                                     // clamp - see 6.1)
    diffusion_rate          0.1
    field_relax_rate        0.002    // setpoint relaxation (6.2,
                                     // K1.1: the environment
                                     // reservoir / heat sink)
    pressure_sensitivity    0.01
    spring_energy_scale     0.032    // F2 stability bound at dt_sub
                                     // + K1.3 well depth: O-H ~80 kT
    convection_rate         0.001
    vent_heat_rate          0.1      // used by 8.1's formula; was
                                     // missing from this block

    bond_search_radius      4.0      // angstroms; candidate
                                     // GATHERING radius only
    bond_form_factor       1.5      // steric contact cap (7.2):
                                     // pairs bond within this
                                     // multiple of r_eq - a
                                     // bond may only be born
                                     // where it can live (F18)
    base_formation_rate     0.01     // per eligible pair per tick
                                     // at steric contact; retuned
                                     // from 0.001 with K1.4 when
                                     // capture moved from the
                                     // 4 A disc to contact
    release_fraction        0.3      // == formation_fraction; the
                                     // literal 0.5 vs 0.3 minted
                                     // 0.2*E per form+break cycle
                                     // (F7)
    release_rate_cap       2.0      // degrees per cell per tick:
                                     // thermalization rate of the
                                     // committed break heat (6.2,
                                     // F19 - the instant dump was
                                     // a bomb)
    formation_fraction      0.3
    en_bonus                0.1
    max_form_speed          1.5      // capture gate (7.2): no
                                     // bonding above this relative
                                     // speed
    bond_break_factor       2.5      // mechanical dissociation
                                     // (7.1): break past this
                                     // multiple of r_eq, silently
    geometry_sigma          30.0     // degrees; 7.2 geometry-factor
                                     // tolerance (new)
    t_opt_scale             0.1      // T_opt = t_opt_scale * bond
                                     // energy; 7.2 temperature
                                     // factor (new)
    t_width                 20.0     // degrees (new)

    spatial_cell_size       5.0      // angstroms
    field_cell_size         10.0     // angstroms
    compaction_interval     10000    // ticks [phase 3]
    surface_threshold       0.9      // fraction of world depth (z,
                                     // the vertical since the port)

    non_bonded_repulsion    1.0      // excluded volume strength
                                     // (6.6)
    non_bonded_margin      1.5      // cutoff = (r_a + r_b) *
                                     // margin (6.6)

    bond_energy { ... }    // the section 7.3 table
    bond_angles { ... }    // the section 7.4 table
    uv_sensitivity { single 0.0001; double 0.0003; triple 0.0002 }

Removed 2026-09-05: strong_repulsion and max_repulsion_force (the
hard core is gone; 6.3 has the story).

## 12. Runtime guarantees

    G01  No concept above the atom/bond level exists in the runtime
    G02  Given the same .kem files and seed, the output stream is
         byte-identical, modulo the wall-clock fields elapsed_ms and
         ticks_per_sec (timing is excluded; the rest is bit-for-bit
         reproducible)
    G03  The observer never modifies WorldState
    G04  Bond formation never exceeds an element's max_bonds
    G05  Boundary conditions are applied every tick without exception
    G06  Energy sources are the only energy inputs besides the
         declared environment: setpoint-relaxation cells exchange
         with an explicit declared reservoir (6.2); nothing else
         couples the world to an outside bath
    G07  The spatial index is consistent with atom positions at the
         start of ChemistrySystem::update each tick
    G08  All validation errors are reported before tick 0
    G09  Save state is complete: a loaded run produces the identical
         future as an uninterrupted run [phase 3]
    G10  stdout contains only NDJSON events
    G11  stderr contains only human-readable diagnostics
    G12  Exit codes follow section 2.3
    G13  The v field in every event is 2 in khem v0.1 (v:1 was the
         2D stream; the 3D port bumped the schema - 3.4)
    G14  Tick ordering follows section 5.1 and does not vary

## 13. Performance targets

Baseline: 2020-era laptop, single core.

    Atoms      Target tick rate     Memory
    1,000      > 10,000 t/s         < 5 MB
    5,000      >  2,000 t/s         < 20 MB
    10,000     >    500 t/s         < 40 MB
    50,000     >     50 t/s         < 200 MB
    100,000    >     10 t/s         < 400 MB

No hardcoded atom limit; hardware is the limit. Save state is
roughly 100 bytes per atom (10,000 atoms is about 1 MB per
snapshot).
