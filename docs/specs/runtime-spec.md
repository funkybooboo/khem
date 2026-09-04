# khem runtime specification

Runtime version: 0.1 (emits output schema v:1)
Status: canonical as of 2026-09-04. Drafts until validated (ADR-0006):
revised against phase-1 kernel reality before the parser is built.
Provenance: reconciled from the founding conversation
(initial-idea.md) with the final terminology applied (ADR-0007,
ADR-0009).

## 1. Overview

khem is a physics engine. It accepts .kem definitions, outputs a
structured event stream, and knows about atoms, bonds, forces, and
energy - nothing above that level (G01).

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
    --check            parse and validate all files; report errors and
                       warnings; do not run. Exit 0 if valid, 1 if not.
                       khem --check primordial_pond.kem
    --test             run a single struct or body in isolation in a
                       minimal world for a default test duration.
                       khem --test minimal_cell.kem
    --info             parse a .kem file and print its structure: atom
                       count, bond count, port list, import tree. Does
                       not run.
                       khem --info nucleotide_A.kem
    --version
    --help

### 2.3 Exit codes

    0   success (simulation completed, or --check passed)
    1   validation error (bad .kem files)
    2   runtime error (crash during simulation)
    3   user interrupt (SIGINT / ctrl-c)

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

    v      integer   output schema version; always 1 in khem v0.1
    type   string    event type
    tick   integer   simulation tick when the event occurred

### 3.3 Event types

START - first line, emitted once:

    {"v":1,"type":"start","tick":0,"khem_version":"0.1.0",
     "run_name":"experiment_1","world_name":"primordial_pond",
     "seed":42,"atom_count":4821,"bond_count":341,
     "world_width":200.0,"world_height":200.0}

TICK - every tick_interval ticks:

    {"v":1,"type":"tick","tick":1000,"elapsed_ms":124,
     "ticks_per_sec":8064,"atom_count":4821,"bond_count":2341,
     "temp_min":12.3,"temp_max":847.2,"temp_avg":34.1,
     "pressure_min":0.8,"pressure_max":20.1,"pressure_avg":4.2,
     "free_atoms":{"H":892,"C":234,"O":445},
     "mol_size_dist":{"1":892,"2_5":445,"6_20":89,"21plus":12}}

BOND_FORMED - when output.bond_events is true:

    {"v":1,"type":"bond_formed","tick":1247,"bond_id":4521,
     "atom_a":442,"atom_b":891,"elem_a":"C","elem_b":"O",
     "order":2,"energy":799.0,"x":45.2,"y":123.7}

BOND_BROKEN - when output.bond_events is true:

    {"v":1,"type":"bond_broken","tick":1248,"bond_id":4521,
     "elem_a":"C","elem_b":"O","energy_released":399.5,
     "x":45.3,"y":123.8}

NOTABLE - when a watch condition triggers; always emitted regardless
of output settings:

    {"v":1,"type":"notable","tick":1247900,"event":"largest_molecule",
     "data":{"atom_count":47,"first_seen_tick":891000}}

Event vocabulary:

    largest_molecule     new largest molecule found
    extinction           all molecules below size 2
    population_surge     population change above threshold
    population_crash     population drop above threshold
    bond_type_first      bond type seen for the first time

SAVE - when state is saved:

    {"v":1,"type":"save","tick":1000000,"path":"./saves/tick_1000000.state"}

END - last line, emitted once:

    {"v":1,"type":"end","tick":5000000,"elapsed_ms":620000,
     "reason":"max_ticks_reached"}

Reason vocabulary: max_ticks_reached, user_interrupt, extinction,
runtime_error.

### 3.4 Schema versioning

The v field is the output schema version. khem v0.1 always emits v:1.
Consumers must handle unknown v values gracefully; within a v the
contract only grows additively.

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
        vx:         f32            // angstroms per tick
        vy:         f32
        bonds:      [BondId; 6]    // covers every element's max_bonds
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
        boundary:       BoundaryType
        temp_field:     Grid2D
        pressure_field: Grid2D
        uv_field:       Grid2D
        energy_sources: Vec<EnergySource>
        element_table:  Arc<Vec<ElementProperties>>
        spatial_index:  SpatialIndex
        rng:            Rng               // deterministic, seeded
        event_queue:    Vec<Event>        // observer fills; flush per tick
    }

### 4.8 Grid2D

Field values (temperature, pressure, UV) on a grid coarser than atom
positions. Default cell size 10 angstroms. Index = col + row * cols.

    Grid2D { data: Vec<f32>, cols: u32, rows: u32,
             cell_width: f32, cell_height: f32 }

### 4.9 SpatialIndex

Spatial hash for neighbor queries. Default cell size 5 angstroms.
Rebuilt every tick after position updates; rebuild is O(n), queries
are O(1) average.

    SpatialIndex { cells: HashMap<(i32, i32), Vec<AtomId>>,
                   cell_size: f32 }

### 4.10 BoundaryType

    enum BoundaryType { Wrap, Wall, Open }

## 5. Tick execution

### 5.1 Tick order

Systems execute in strict order. Each reads current state; writes are
committed before the next system reads; no system reads another
system's writes within the same tick.

    1.  EnergySystem::update
    2.  PhysicsSystem::update_velocities
    3.  PhysicsSystem::update_positions
    4.  PhysicsSystem::apply_boundary
    5.  SpatialIndex::rebuild
    6.  ChemistrySystem::break_bonds
    7.  ChemistrySystem::form_bonds
    8.  ObserverSystem::sample
    9.  EventQueue::flush_to_output

### 5.2 Dead atom and bond cleanup

Atoms and bonds are flagged dead, not removed. Compaction runs every
compaction_interval ticks (default 10,000): dead entries removed, IDs
remapped, spatial index rebuilt, compaction noted on stderr (never
stdout).

### 5.3 Determinism

Given the same WorldState at tick 0 and the same seed, the run is
byte-identical forever (G02, G14). Requirements: fixed tick order
(5.1); one deterministic seeded RNG with per-tick state; no
thread-local state in v0.1; iteration over atoms always by AtomId.

## 6. Physics system

### 6.1 Temperature and velocity

Temperature at a grid cell = mean kinetic energy of atoms in that
cell, scaled to celsius by the scale factors in physics.cfg.

    KE(atom) = 0.5 * element.mass * (vx^2 + vy^2)

Thermal perturbation per tick (Maxwell-Boltzmann):

    sigma = sqrt(kB * T / mass)
    vx += rng.normal(0, sigma)
    vy += rng.normal(0, sigma)

### 6.2 Temperature diffusion

Per cell, 4-connected neighbors:

    T_new = T * (1 - diffusion_rate) + mean(T_neighbors) * diffusion_rate

diffusion_rate default 0.1.

### 6.3 Bond forces

Hooke's law toward equilibrium distance (sum of covalent radii):

    r_eq    = elem_a.radius + elem_b.radius
    F       = spring_k * (r - r_eq)
    spring_k = bond.energy * spring_energy_scale

Applied to both atoms along the bond axis. Strong repulsion when
r < 0.5 * r_eq: F = -strong_repulsion / r^2.

### 6.4 Pressure force

    pressure[cell] = atom_count_in_cell / cell_area

Each atom feels force from the central-difference pressure gradient,
scaled by pressure_sensitivity.

### 6.5 Position update

    x += vx * dt
    y += vy * dt
    dt = 1.0    (one tick = one femtosecond at default scale)

### 6.6 Boundaries

    Wrap   x = x mod width; y = y mod height
    Wall   clamp position; reverse the velocity component
    Open   atom flagged dead; bonds broken first

## 7. Chemistry system

### 7.1 Bond breaking

Per alive bond:

    T       = temperature at the bond midpoint
    p_break = exp(-bond.energy / (kB * T))     // Boltzmann/Arrhenius

If rng < p_break: flag the bond dead, decrement both atoms'
bond_count, release bond.energy * release_fraction into the local
temperature field.

### 7.2 Bond formation

For each atom A with available valence, candidates within
bond_search_radius (default 4.0 angstroms) via the spatial index:

    p_form = base_formation_rate
           * geometry_factor(A, B)
           * temperature_factor(T, elem_a, elem_b)
           * (1.0 + |EN_a - EN_b| * en_bonus)

- geometry_factor scores how well B fits A's existing VSEPR angles,
  from 0.0 (geometrically impossible) to 1.0 (perfect), using the
  angle table (7.4).
- temperature_factor is a gaussian peaked at an optimal temperature
  for the element pair: too cold for atoms to meet, too hot for bonds
  to hold.
- bond order: prefer double when both atoms have 2+ available valence
  and geometry allows; otherwise single.

If rng < p_form: create a BondState with energy from the table (7.3),
increment both bond_counts, absorb bond.energy * formation_fraction
from the local temperature field.

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

Unknown pairs use the geometric mean of single-bond energies. The
table lives in physics.cfg, modifiable without recompilation.

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

Also in physics.cfg, not source code.

## 8. Energy system

### 8.1 Hydrothermal vent

Per tick, for cells within radius:

    falloff = 1.0 / (1.0 + distance^2 / radius^2)
    temp_field[cell] += intensity * falloff * vent_heat_rate

Atoms in radius get upward velocity: vy += convection_rate * falloff.

### 8.2 Solar UV

Per tick, for surface cells (y > surface_threshold * height):

    uv_field[cell] = intensity

Per bond in UV-exposed cells:

    p_uv_break = uv_field[cell] * uv_sensitivity[bond_type]
    if rng < p_uv_break: break the bond

uv_sensitivity defaults per bond order: single 0.0001, double
0.0003, triple 0.0002.

### 8.3 Energy tracking

Total kinetic, bond potential, and field energy are tracked and
emitted in TICK events. Not enforced; used for debugging and analysis.

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
changes what chemistry is possible.

    kb_scaled               0.008314   // Boltzmann constant, sim units
    diffusion_rate          0.1
    pressure_sensitivity    0.01
    spring_energy_scale     0.01
    strong_repulsion        1000.0
    convection_rate         0.001

    bond_search_radius      4.0        // angstroms
    base_formation_rate     0.001       // per eligible pair per tick
    release_fraction        0.5
    formation_fraction      0.3
    en_bonus                0.1

    spatial_cell_size       5.0        // angstroms
    field_cell_size         10.0       // angstroms
    compaction_interval     10000       // ticks
    surface_threshold       0.9         // fraction of world height

    bond_energy { ... }    // the section 7.3 table
    bond_angles { ... }    // the section 7.4 table
    uv_sensitivity { single 0.0001; double 0.0003; triple 0.0002 }

## 12. Runtime guarantees

    G01  No concept above the atom/bond level exists in the runtime
    G02  Given the same .kem files and seed, the output stream is
         byte-identical
    G03  The observer never modifies WorldState
    G04  Bond formation never exceeds an element's max_bonds
    G05  Boundary conditions are applied every tick without exception
    G06  Energy sources are the only energy inputs
    G07  The spatial index is consistent with atom positions at the
         start of ChemistrySystem::update each tick
    G08  All validation errors are reported before tick 0
    G09  Save state is complete: a loaded run produces the identical
         future as an uninterrupted run
    G10  stdout contains only NDJSON events
    G11  stderr contains only human-readable diagnostics
    G12  Exit codes follow section 2.3
    G13  The v field in every event is 1 in khem v0.1
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