<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 5598-6407
  WHAT: round-2 runtime spec v0.1: CLI (exit codes, stdout/stderr contract),
        NDJSON event schema (v field, START/TICK/BOND_FORMED/BOND_BROKEN/
        NOTABLE/SAVE/END), flat-array data structures, tick order, physics
        and chemistry systems, energy system, observer, scalability V1-V3,
        plugin preparation, physics.cfg, guarantees G01-G14, performance
        targets
  STATUS: CURRENT runtime draft; apply renames from
        06-terminology-and-final-naming.md (--validate -> --check etc.)
-->

PART 2: CERNE RUNTIME SPECIFICATION
============================================================


1. OVERVIEW
------------------------------------------------------------

cerne is a physics engine.
It accepts Weave definitions.
It outputs a structured event stream.
It knows about atoms, bonds, forces, energy.
It knows nothing above that level.

cerne is designed to scale from a laptop
to a large multi-machine system.
v0.1 implements single-machine execution.
The architecture must not prevent future scaling.


2. CLI SPECIFICATION
------------------------------------------------------------

2.1 Basic usage

  cerne [OPTIONS] <file.weave>

  The file passed must declare sim.
  cerne resolves all dependencies from there.

2.2 Options

  --seed <integer>
    Override the seed defined in sim declaration.
    Enables reproducible runs from command line.
    Example: cerne --seed 42 experiment.weave

  --validate
    Parse and validate all files.
    Report errors and warnings.
    Do not run simulation.
    Exit 0 if valid, 1 if errors found.
    Example: cerne --validate world.weave

  --test
    Run a single .weave file in isolation.
    File may declare mol or org.
    Creates a minimal world around it.
    Runs for default test duration.
    Useful for template validation.
    Example: cerne --test minimal_cell.weave

  --info
    Parse a .weave file and print its structure.
    Atom count, bond count, port list, import tree.
    Does not run simulation.
    Example: cerne --info nucleotide_A.weave

  --version
    Print cerne version and exit.

  --help
    Print usage and exit.

2.3 Exit codes

  0   success (simulation completed or validate passed)
  1   validation error (bad Weave files)
  2   runtime error (crash during simulation)
  3   user interrupt (SIGINT/ctrl-c)

2.4 Stdin / stdout / stderr

  stdin:   not used in v0.1
  stdout:  NDJSON event stream (simulation output)
  stderr:  errors, warnings, progress info

  This separation is strict.
  No simulation data goes to stderr.
  No error data goes to stdout.
  Tools that consume stdout can ignore stderr
  or redirect it independently.

  Example:
    cerne experiment.weave > data.ndjson 2> errors.log
    cerne experiment.weave 2>/dev/null | my_tool
    cerne experiment.weave | tee data.ndjson | cerne-view


3. NDJSON OUTPUT SPECIFICATION
------------------------------------------------------------

3.1 Format
  Newline-delimited JSON.
  Each line is a complete, valid JSON object.
  Lines separated by single newline character.
  No trailing comma. No wrapping array.
  Parseable line by line without buffering entire output.

3.2 Common fields
  Every event object contains:
    "v":     integer  schema version, always 1 in v0.1
    "type":  string   event type
    "tick":  integer  simulation tick when event occurred

3.3 Event types

  START
  Emitted once, first line of output.
  {
    "v": 1,
    "type": "start",
    "tick": 0,
    "cerne_version": "0.1.0",
    "sim_name": "experiment_1",
    "world_name": "primordial_pond",
    "seed": 42,
    "atom_count": 4821,
    "bond_count": 341,
    "world_width": 200.0,
    "world_height": 200.0
  }

  TICK
  Emitted every tick_interval ticks.
  Frequency controlled by sim output.tick_interval.
  {
    "v": 1,
    "type": "tick",
    "tick": 1000,
    "elapsed_ms": 124,
    "ticks_per_sec": 8064,
    "atom_count": 4821,
    "bond_count": 2341,
    "temp_min": 12.3,
    "temp_max": 847.2,
    "temp_avg": 34.1,
    "pressure_min": 0.8,
    "pressure_max": 20.1,
    "pressure_avg": 4.2,
    "free_atoms": {"H": 892, "C": 234, "O": 445},
    "mol_size_dist": {"1": 892, "2_5": 445, "6_20": 89, "21plus": 12}
  }

  BOND_FORMED
  Emitted when bond forms, if output.bond_events true.
  {
    "v": 1,
    "type": "bond_formed",
    "tick": 1247,
    "bond_id": 4521,
    "atom_a": 442,
    "atom_b": 891,
    "elem_a": "C",
    "elem_b": "O",
    "order": 2,
    "energy": 799.0,
    "x": 45.2,
    "y": 123.7
  }

  BOND_BROKEN
  Emitted when bond breaks, if output.bond_events true.
  {
    "v": 1,
    "type": "bond_broken",
    "tick": 1248,
    "bond_id": 4521,
    "elem_a": "C",
    "elem_b": "O",
    "energy_released": 399.5,
    "x": 45.3,
    "y": 123.8
  }

  NOTABLE
  Emitted when watch condition triggered.
  Always emitted regardless of output settings.
  {
    "v": 1,
    "type": "notable",
    "tick": 1247900,
    "event": "largest_molecule",
    "data": {
      "atom_count": 47,
      "first_seen_tick": 891000
    }
  }

  event field vocabulary:
    "largest_molecule"        new largest molecule found
    "extinction"              all molecules below size 2
    "population_surge"        population change above threshold
    "population_crash"        population drop above threshold
    "bond_type_first"         bond type seen for first time

  SAVE
  Emitted when state saved.
  {
    "v": 1,
    "type": "save",
    "tick": 1000000,
    "path": "./saves/tick_1000000.state"
  }

  END
  Emitted once, last line of output.
  {
    "v": 1,
    "type": "end",
    "tick": 5000000,
    "elapsed_ms": 620000,
    "reason": "max_ticks_reached"
  }

  reason vocabulary:
    "max_ticks_reached"
    "user_interrupt"
    "extinction"
    "runtime_error"

3.4 Schema versioning
  The "v" field in every event is the output schema version.
  v0.1 of cerne always emits v:1.
  Consumer tools check this field.
  Future cerne versions may emit higher v values.
  Consumers should handle unknown v gracefully.


4. CORE DATA STRUCTURES
------------------------------------------------------------

4.1 Design principles for scalability

  All primary data in flat arrays, not trees.
  Indexed by integer ID, not pointer.
  ID is index into array.
  No heap allocation per atom per tick.
  Cache-friendly layout.
  Partitionable by spatial region.

  These properties enable:
    V1: single thread iterates all arrays
    V2: multiple threads partition arrays by region
    V3: multiple machines partition arrays by region
  Without changing the data structure.

4.2 AtomId, BondId
  type AtomId = u32
  type BondId = u32
  u32 supports 4 billion atoms/bonds.
  Sufficient for any foreseeable simulation scale.

4.3 ElementId
  type ElementId = u8
  Supports 256 element types.
  More than sufficient.
  Index into element table.

4.4 AtomState
  Fixed size struct. No heap allocation.

  AtomState {
    id:         AtomId
    element:    ElementId
    x:          f32
    y:          f32
    vx:         f32
    vy:         f32
    bonds:      [BondId; 6]    // max 6 bonds, covers all elements
    bond_count: u8
    alive:      bool
    _pad:       [u8; 2]        // alignment padding
  }

  Size: deterministic, cache-line friendly
  alive flag: atoms marked dead, cleaned up periodically
  not removed immediately (would invalidate indices)

4.5 BondState
  Fixed size struct.

  BondState {
    id:      BondId
    atom_a:  AtomId
    atom_b:  AtomId
    order:   u8
    alive:   bool
    _pad:    [u8; 2]
    energy:  f32
  }

4.6 ElementProperties
  Immutable after load. Shared reference.

  ElementProperties {
    symbol:            [u8; 2]
    atomic_number:     u8
    max_bonds:         u8
    valence:           u8
    _pad:              [u8; 3]
    mass:              f32
    electronegativity: f32
    radius:            f32
  }

4.7 WorldState
  The complete mutable simulation state.

  WorldState {
    tick:              u64
    atoms:             Vec<AtomState>
    bonds:             Vec<BondState>
    width:             f32
    height:            f32
    boundary:          BoundaryType
    temp_field:        Grid2D
    pressure_field:    Grid2D
    uv_field:          Grid2D
    energy_sources:    Vec<EnergySource>
    element_table:     Arc<Vec<ElementProperties>>
    spatial_index:     SpatialIndex
    rng:               Rng            // seeded, deterministic
    event_queue:       Vec<Event>     // observer reads, cleared each tick
  }

4.8 Grid2D
  Field values on coarser grid than atom positions.

  Grid2D {
    data:        Vec<f32>
    cols:        u32
    rows:        u32
    cell_width:  f32
    cell_height: f32
  }

  Cell size: configurable, default 10 angstroms.
  Indexed by (col, row) -> col + row * cols.

4.9 SpatialIndex
  Spatial hash for fast neighbor queries.

  SpatialIndex {
    cells:     HashMap<(i32,i32), Vec<AtomId>>
    cell_size: f32
  }

  cell_size default: 5 angstroms.
  Rebuilt each tick after position updates.
  Rebuild is O(n). Query is O(1) average.

4.10 BoundaryType
  enum BoundaryType { Wrap, Wall, Open }


5. TICK EXECUTION
------------------------------------------------------------

5.1 Tick ordering
  Systems execute in strict order.
  Each system reads from current state.
  Writes committed before next system reads.
  No system reads another system's writes in same tick.
  This makes the simulation deterministic and
  order-independent within a tick.

  Tick N execution order:
    1. EnergySystem::update
    2. PhysicsSystem::update_velocities
    3. PhysicsSystem::update_positions
    4. PhysicsSystem::apply_boundary
    5. SpatialIndex::rebuild
    6. ChemistrySystem::break_bonds
    7. ChemistrySystem::form_bonds
    8. ObserverSystem::sample
    9. EventQueue::flush_to_output

5.2 Dead atom/bond cleanup
  Atoms and bonds are not removed immediately when dead.
  alive flag set to false.
  Periodic compaction (every 10,000 ticks default):
    Remove dead atoms from Vec, remap AtomIds
    Remove dead bonds from Vec, remap BondIds
    Rebuild spatial index
    Emit compaction event to stderr (not stdout)
  This avoids O(n) shifts per deletion.

5.3 Determinism guarantee
  Given same WorldState at tick 0 and same seed:
  The simulation produces identical output forever.
  This requires:
    - Fixed tick order (above)
    - Deterministic RNG (seeded, per-tick state)
    - No thread-local state in v0.1
    - No undefined behavior in Rust (guaranteed)
    - Iteration order over atoms is always by AtomId


6. PHYSICS SYSTEM
------------------------------------------------------------

6.1 Temperature and velocity

  Temperature at a grid cell =
    mean kinetic energy of atoms in that cell
    scaled by temperature_scale factor

  KE(atom) = 0.5 * element.mass * (vx^2 + vy^2)

  Thermal velocity perturbation per tick:
    sigma = sqrt(kB * T / mass)   // Maxwell-Boltzmann
    vx += rng.normal(0, sigma)
    vy += rng.normal(0, sigma)

  where kB is Boltzmann constant scaled to sim units.

6.2 Temperature diffusion
  Each tick:
    for each cell c:
      T_new[c] = T[c] * (1 - diffusion_rate)
               + mean(T[neighbors(c)]) * diffusion_rate

  diffusion_rate: physics.cfg, default 0.1
  Neighbors: 4-connected (up, down, left, right)

6.3 Bond forces
  For each bond:
    r_eq = elem_a.radius + elem_b.radius  // equilibrium distance
    r    = distance(atom_a, atom_b)
    F    = spring_k * (r - r_eq)          // Hooke's law
    direction = normalize(pos_b - pos_a)
    atom_a.v += F * direction / mass_a
    atom_b.v -= F * direction / mass_b

  spring_k derived from bond energy:
    spring_k = bond.energy * spring_energy_scale

  Repulsion when r < 0.5 * r_eq:
    F = -strong_repulsion / r^2
    Applied to both atoms away from each other.

6.4 Pressure force
  pressure[cell] = atom_count_in_cell / cell_area

  For each atom:
    grad_p = pressure_gradient_at(atom.x, atom.y)
    atom.vx -= grad_p.x * pressure_sensitivity
    atom.vy -= grad_p.y * pressure_sensitivity

  Central difference for gradient.

6.5 Position update
  atom.x += atom.vx * dt
  atom.y += atom.vy * dt
  dt = 1.0 (one tick = one femtosecond at default scale)

6.6 Boundary application
  WRAP:
    atom.x = atom.x mod world.width
    atom.y = atom.y mod world.height
  WALL:
    if atom.x < 0: atom.x = 0, atom.vx = |atom.vx|
    if atom.x > width: atom.x = width, atom.vx = -|atom.vx|
    same for y
  OPEN:
    if outside bounds: atom.alive = false
    break all bonds first


7. CHEMISTRY SYSTEM
------------------------------------------------------------

7.1 Bond breaking

  For each alive bond b:

    T = temperature at midpoint of b.atom_a, b.atom_b

    p_break = exp( -b.energy / (kB * T) )
    // Boltzmann factor, real Arrhenius equation

    if rng.float() < p_break:
      b.alive = false
      atom_a.bond_count -= 1
      atom_b.bond_count -= 1
      release = b.energy * release_fraction
      temp_field.add_energy_at(midpoint, release)

7.2 Bond formation

  For each atom A where bond_count < max_bonds:
    candidates = spatial_index.query(A.x, A.y, bond_search_radius)

    for each candidate B in candidates:
      if B.bond_count >= B.element.max_bonds: skip
      if already_bonded(A, B): skip
      if A.id == B.id: skip

      T   = temperature at midpoint(A, B)
      EN_diff = |elem_a.electronegativity - elem_b.electronegativity|

      p_form = base_rate
             * geometry_factor(A, B)
             * temperature_factor(T, elem_a, elem_b)
             * (1.0 + EN_diff * en_bonus)

      geometry_factor:
        Scores how well B fits given A's existing bond angles
        Uses real VSEPR geometry from lookup table
        Returns 0.0 to 1.0

      temperature_factor:
        Low T:      low (atoms not reactive)
        Optimal T:  high (sweet spot for this bond type)
        High T:     low (bonds form but immediately break)
        Modeled as gaussian peaked at optimal_temp(elem_a, elem_b)

      if rng.float() < p_form:
        order = determine_order(A, B)
          // prefer double if both have 2+ available valence
          // and geometry supports it
          // otherwise single

        energy = bond_energy_table[elem_a][elem_b][order]

        create BondState(A.id, B.id, order, energy)
        A.bond_count += 1
        B.bond_count += 1
        absorb = energy * formation_fraction
        temp_field.remove_energy_at(midpoint, absorb)

7.3 Bond energy table (kJ/mol, real values)
  H-H:    436   single only
  H-C:    413   single only
  H-N:    391   single only
  H-O:    463   single only
  H-S:    363   single only
  C-C:    346   single
  C=C:    614   double
  C#C:    839   triple
  C-N:    305   single
  C=N:    615   double
  C#N:    891   triple
  C-O:    358   single
  C=O:    799   double
  C-S:    272   single
  C-P:    264   single
  N-N:    163   single
  N=N:    418   double
  N#N:    945   triple
  N-O:    201   single
  N=O:    607   double
  O-O:    146   single
  O=O:    498   double
  O-P:    335   single
  O-S:    265   single
  P-P:    201   single
  S-S:    266   single
  Si-O:   452   single
  Si-Si:  222   single
  Fe-O:   approximately 390, variable

  Unknown pair: use geometric mean of single bond energies.
  This table is defined in physics.cfg not in source code.
  Modifiable without recompilation.

7.4 Bond angle table (degrees, VSEPR, real values)
  H:  no constraint (1 bond max)
  C:  4 bonds: 109.5 (tetrahedral)
      3 bonds + 1 double: 120 (trigonal planar)
      2 bonds both double: 180 (linear)
  N:  3 bonds: 107 (trigonal pyramidal)
      2 bonds + 1 double: 120
  O:  2 bonds: 104.5 (bent)
  P:  5 bonds: 90/120 (trigonal bipyramidal)
      4 bonds: 109.5 (tetrahedral)
  S:  2 bonds: 103 (bent)
      higher: variable
  Si: 4 bonds: 109.5 (tetrahedral, same as C)
  Fe: up to 6 bonds: 90 (octahedral)
  Na: 1 bond: no constraint
  Cl: 1 bond: no constraint

  Angle lookup is used by geometry_factor.
  Also defined in physics.cfg, not source code.


8. ENERGY SYSTEM
------------------------------------------------------------

8.1 Hydrothermal vent
  Each tick, for all cells within radius of vent:
    falloff = 1.0 / (1.0 + distance^2 / radius^2)
    temp_field[cell] += intensity * falloff * vent_heat_rate
  Also adds upward velocity to atoms in radius:
    atom.vy += convection_rate * falloff
  convection_rate: physics.cfg

8.2 Solar UV
  Each tick, for surface cells (y > surface_threshold):
    uv_field[cell] = intensity * (1.0 - cloud_factor)
  For each bond in UV-exposed cells:
    p_uv_break = uv_field[cell] * uv_sensitivity[bond_type]
    if rng.float() < p_uv_break: break bond
  uv_sensitivity per bond type: physics.cfg

8.3 Energy tracking
  cerne tracks:
    total kinetic energy
    total bond potential energy
    total field energy
  These are emitted in TICK events.
  Not used for enforcement.
  Used for debugging and analysis.


9. OBSERVER SYSTEM
------------------------------------------------------------

9.1 Role
  Read-only access to WorldState.
  Never modifies simulation state.
  Runs after all physics and chemistry each tick.
  Populates event_queue which is flushed to stdout.

9.2 Molecule detection
  Each sample tick (every tick_interval ticks):
    Run connected components on bond graph.
    AtomId -> component ID via union-find.
    Each component is a molecule.
    Collect size distribution.
    Track largest molecule ever seen.

9.3 Watch condition evaluation
  After molecule detection:
    Check each watch condition from sim declaration.
    If triggered: add NOTABLE event to event_queue.
    Watch conditions are stateful
    (track previous state to detect changes).

9.4 Output
  event_queue flushed to stdout after observer runs.
  Each event serialized to JSON, written as one line.
  stdout flushed after each tick's events.
  (flush ensures pipe consumers receive data promptly)


10. SCALABILITY DESIGN
------------------------------------------------------------

10.1 V1 (this version)
  Single thread.
  Single machine.
  All data in one Vec.
  Target: 10,000 atoms at > 500 t/s on a laptop.

10.2 V2 preparation (not implemented, must not be prevented)
  Spatial decomposition:
    Divide world into regions.
    Each region owns a subset of atoms.
    Regions are independent except at boundaries.
    V1 design already supports this:
      SpatialIndex is region-aware
      AtomState uses IDs not pointers
      No global mutable state in physics systems

  Threading model for V2:
    One thread per region.
    Boundary atoms handled with ghost cell pattern.
    (Ghost cells are standard in parallel physics sims.)
    Rust ownership model makes this safe.

10.3 V3 preparation (not implemented, must not be prevented)
  Distribution:
    Regions can run on separate machines.
    WorldState serialization already complete
    (needed for save/load, reused for distribution).
    NDJSON output stream design does not change.
    Aggregation node collects streams from all machines.

10.4 Plugin preparation (not implemented)
  Physics system behind a trait:
    trait PhysicsSystem { fn update(&mut self, world: &mut WorldState); }
  Chemistry system behind a trait:
    trait ChemistrySystem { fn update(&mut self, world: &mut WorldState); }
  V1: one implementation of each trait, compiled in.
  Future: load dynamic library implementing trait.


11. CONFIGURATION
------------------------------------------------------------

physics.cfg controls all tunable constants.
This file is separate from Weave definitions.
Located in project root or system default.
Modifiable without recompilation.
Changes affect simulation behavior not correctness.

  // Boltzmann constant, scaled to sim units
  kb_scaled:               0.008314

  // Physics
  diffusion_rate:          0.1
  pressure_sensitivity:    0.01
  spring_energy_scale:     0.01
  strong_repulsion:        1000.0
  convection_rate:         0.001

  // Chemistry
  bond_search_radius:      4.0
  base_formation_rate:     0.001
  release_fraction:        0.5
  formation_fraction:      0.3
  en_bonus:                0.1

  // Grid
  spatial_cell_size:       5.0
  field_cell_size:         10.0

  // Performance
  compaction_interval:     10000
  surface_threshold:       0.9    // fraction of world height

  // Bond energy table (kJ/mol)
  bond_energy {
    H-H:   436.0
    H-C:   413.0
    // ... full table
  }

  // Bond angle table (degrees)
  bond_angles {
    C-4:   109.5
    C-3d:  120.0
    C-2d:  180.0
    O-2:   104.5
    // ... full table
  }

  // UV sensitivity per bond type
  uv_sensitivity {
    single: 0.0001
    double: 0.0003
    triple: 0.0002
  }


12. RUNTIME GUARANTEES
------------------------------------------------------------

G01  No concept above atom/bond level exists in runtime
G02  Given same seed and same .weave files:
     output stream is identical byte for byte
G03  Observer never modifies WorldState
G04  Bond formation never exceeds element max_bonds
G05  Boundary conditions applied every tick without exception
G06  Energy sources are the only energy inputs to the system
G07  Spatial index is consistent with atom positions
     at the start of ChemistrySystem::update each tick
G08  All validation errors reported before tick 0
G09  Save state is complete: load produces identical future
G10  stdout contains only NDJSON events
G11  stderr contains only human-readable diagnostic text
G12  Exit codes are as specified in CLI spec section 2.3
G13  The "v" field in every output event is always 1 in v0.1
G14  Tick ordering is as specified in section 5.1
     and does not vary


13. PERFORMANCE TARGETS
------------------------------------------------------------

  Hardware baseline: 2020-era laptop, single core, 8GB RAM

  Atoms     Target tick rate    Memory
  ------------------------------------
  1,000     > 10,000 t/s        < 5 MB
  5,000     > 2,000  t/s        < 20 MB
  10,000    > 500    t/s        < 40 MB
  50,000    > 50     t/s        < 200 MB
  100,000   > 10     t/s        < 400 MB

  Larger simulations possible on larger hardware.
  No hardcoded atom limit in the runtime.
  Limit is hardware RAM and acceptable tick rate.

  Save state size:
    approximately 100 bytes per atom
    10,000 atoms ~ 1 MB per save
    50,000 atoms ~ 5 MB per save
```

---

## Summary

```
LANGUAGE:    Weave
RUNTIME:     cerne
FILES:       all .weave, type declared inside
ENTRY POINT: cerne experiment.weave
OUTPUT:      NDJSON to stdout, errors to stderr
SCALING:     laptop to cluster, architecture supports it
PLUGINS:     reserved, not v0.1, architecture supports it
VERSIONING:  weave "0.1" in every file, v:1 in every event
```

---

Ready to write Rust code?
