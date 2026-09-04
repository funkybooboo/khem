<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 3745-4399
  WHAT: formal spec draft v0.1 of the runtime (Rust), BIOSIM-era branding:
        data structures, tick order, physics/chemistry equations, observer,
        parser, guarantees G01-G10, performance targets
  STATUS: historical draft; superseded by 05-runtime-spec.md
-->

## SPEC 2: The Runtime

```
========================================================
BIOSIM RUNTIME
Formal Specification v0.1
Implementation Language: Rust
========================================================


1. OVERVIEW
--------------------------------------------------------
The runtime is a physics engine.
It knows about atoms, bonds, forces, and energy.
It knows nothing about biology, chemistry names,
molecules, organisms, or life.

It receives a flat list of atoms and bonds.
It applies physical rules every tick.
It reports state.
That is all.

The runtime has two phases:
  LOAD PHASE:   Parse BDL files, build world state
  RUN PHASE:    Execute ticks, apply rules, report


2. ARCHITECTURE
--------------------------------------------------------

  +-----------------------------------------------------+
  |                    BDL PARSER                       |
  |  Reads .sim -> .world -> .org -> .mol -> .elem files   |
  |  Validates all files (V01-V15)                      |
  |  Resolves imports and composition hierarchy         |
  |  Flattens everything to AtomState + BondState       |
  |  Hands WorldState to runtime engine                 |
  +--------------------------+--------------------------+
                             | WorldState
                             v
  +-----------------------------------------------------+
  |                  RUNTIME ENGINE                     |
  |                                                     |
  |  +-------------+  +--------------+  +-----------+  |
  |  |   Physics   |  |  Chemistry   |  |  Energy   |  |
  |  |   System    |  |   System     |  |  System   |  |
  |  +-------------+  +--------------+  +-----------+  |
  |                                                     |
  |  +-------------+  +--------------+                  |
  |  |   Spatial   |  |  Observer    |                  |
  |  |   Index     |  |   System     |                  |
  |  +-------------+  +--------------+                  |
  +-----------------------------------------------------+


3. CORE DATA STRUCTURES
--------------------------------------------------------

3.1 ElementProperties
  Loaded from .elem file at startup.
  Immutable for entire run.

  ElementProperties {
      symbol:            String          // "C", "H", etc
      atomic_number:     u8
      mass:              f32             // daltons
      valence:           u8              // bonding electrons
      electronegativity: f32             // Pauling scale
      radius:            f32             // angstroms
      max_bonds:         u8
  }

3.2 AtomState
  One per atom in the simulation.
  Lives in a flat array indexed by AtomId.
  This is the primary data structure.

  AtomState {
      id:            AtomId              // index into atom array
      element:       u8                  // index into element table
      x:             f32                 // position, angstroms
      y:             f32
      vx:            f32                 // velocity, A per tick
      vy:            f32
      bonds:         [BondId; MAX_BONDS] // current bonds
      bond_count:    u8                  // active bonds
      alive:         bool                // false = removed from sim
  }

  MAX_BONDS = 6 (covers all elements we use)

3.3 BondState
  One per bond in the simulation.
  Lives in a flat array indexed by BondId.

  BondState {
      id:        BondId
      atom_a:    AtomId
      atom_b:    AtomId
      order:     u8              // 1, 2, or 3
      energy:    f32             // kJ/mol, energy stored in bond
      alive:     bool            // false = broken bond
  }

3.4 WorldState
  The complete simulation state at any tick.
  Passed from parser to runtime.
  Mutated each tick.

  WorldState {
      tick:          u64
      atoms:         Vec<AtomState>
      bonds:         Vec<BondState>
      width:         f32
      height:        f32
      boundary:      BoundaryType
      temperature_field: Grid2D<f32>
      pressure_field:    Grid2D<f32>
      uv_field:          Grid2D<f32>
      energy_sources:    Vec<EnergySource>
      spatial_index:     SpatialIndex
      element_table:     Vec<ElementProperties>
  }

3.5 Grid2D
  Divides world into cells for field values.
  Separate from atom positions.
  Used for temperature, pressure, UV.

  Grid2D<T> {
      cells:      Vec<T>          // flat array
      cell_width: f32
      cell_height: f32
      cols:       u32
      rows:       u32
  }

  Cell size: 10 angstroms x 10 angstroms
  (coarser than atom positions, fine for fields)

3.6 SpatialIndex
  Spatial hash grid for fast neighbor lookup.
  Rebuilt or updated every tick.

  SpatialIndex {
      cells: HashMap<(i32, i32), Vec<AtomId>>
      cell_size: f32              // 5 angstroms
  }

  Lookup: get all atoms within radius R of point P
  is O(1) average case with spatial hashing.


4. THE TICK
--------------------------------------------------------
One tick is one complete update of the simulation.
Systems execute in fixed order every tick.

Order of execution per tick:

  1. ENERGY SYSTEM
     Apply energy sources to fields
     Diffuse temperature field
     Diffuse UV field

  2. PHYSICS SYSTEM - VELOCITY UPDATE
     Update atom velocities from temperature field
     Apply field forces (pressure gradients)
     Apply bond forces (keep bonded atoms at correct distance)

  3. PHYSICS SYSTEM - POSITION UPDATE
     Move all atoms: x += vx, y += vy
     Apply boundary conditions (wrap/wall/open)
     Rebuild spatial index

  4. CHEMISTRY SYSTEM - BOND BREAKING
     For each bond:
       calculate break probability from temperature
       if random() < break_probability: break bond
       release energy to local temperature field

  5. CHEMISTRY SYSTEM - BOND FORMATION
     For each atom with available valence:
       query spatial index for nearby atoms
       for each nearby atom with available valence:
         calculate formation probability
         if random() < formation_probability: form bond
         absorb energy from local temperature field

  6. OBSERVER SYSTEM
     Sample world state
     Detect notable events
     Output to terminal or log

Systems 1-5 must complete before system 6.
No system may read from another system's
output during the same tick.
(All reads from previous tick state,
all writes to current tick state.)
This prevents order-dependent artifacts.


5. PHYSICS SYSTEM SPECIFICATION
--------------------------------------------------------

5.1 Temperature and Kinetic Energy

  Temperature of a region is the average
  kinetic energy of atoms in that region.

  KE of atom = 0.5 * mass * (vx^2 + vy^2)

  Temperature field is updated by:
    T_cell = mean(KE of all atoms in cell) * conversion_factor

  conversion_factor maps simulation units to celsius.
  Defined in config. Tunable.

  Atoms receive velocity from temperature:
    On each tick, atom velocity is perturbed by:
    thermal_noise = gaussian(0, sqrt(T / mass))
    vx += thermal_noise_x
    vy += thermal_noise_y

  This is the Maxwell-Boltzmann distribution.
  Hot atoms move fast. Cold atoms move slow.
  Correct physics.

5.2 Temperature Diffusion

  Each tick, temperature field diffuses:
    T_cell_new = T_cell * (1 - diffusion_rate)
               + mean(T_neighbors) * diffusion_rate

  diffusion_rate: configurable, default 0.1

5.3 Bond Forces

  Bonded atoms are attracted to equilibrium distance.
  Equilibrium distance = sum of covalent radii.

  Force magnitude = spring_constant * (distance - equilibrium)
  Direction: along axis between atoms
  Applied to both atoms (Newton's third law)

  spring_constant: derived from bond energy
  Higher bond energy = stiffer spring

  If distance < 0.5 * equilibrium: strong repulsion
  (prevents atoms from passing through each other)

5.4 Pressure

  Pressure in a cell = atom density in that cell
    (number of atoms / cell area)

  Atoms experience force from pressure gradient:
    Force direction: from high to low pressure
    Force magnitude: pressure_gradient * pressure_sensitivity

  pressure_sensitivity: configurable

5.5 Boundary Conditions

  WRAP: atom exits right -> enters left (torus)
        atom exits top -> enters bottom
        Bond that crosses boundary remains valid
        Positions handled with modular arithmetic

  WALL: atom velocity reversed on boundary contact
        Bond cannot cross wall
        Bond breaks if atom reaches wall while bonded

  OPEN: atoms that exit world are removed
        Their bonds are broken first


6. CHEMISTRY SYSTEM SPECIFICATION
--------------------------------------------------------

6.1 Bond Breaking

  For each alive bond each tick:

  bond_break_probability =
      boltzmann_factor(temperature, bond_energy)

  boltzmann_factor(T, E) =
      exp(-E / (k_B * T))

  where:
    k_B = Boltzmann constant (scaled to sim units)
    T   = temperature at bond midpoint location
    E   = bond.energy

  This is the Arrhenius equation.
  Real chemistry.

  If random() < bond_break_probability:
    bond.alive = false
    atom_a.bond_count -= 1
    atom_b.bond_count -= 1
    release (bond.energy * release_fraction) to local T field

  release_fraction: configurable, default 0.5

6.2 Bond Formation

  For each atom A with available valence:
    query spatial_index for atoms within bond_search_radius

  For each candidate atom B:
    if B has available valence:
    if A and B are not already bonded:
    if A and B are not the same atom:

    bond_formation_probability =
        formation_base_rate
        * geometry_factor(A, B, existing_bonds)
        * temperature_factor(temperature_at_midpoint)
        * electronegativity_factor(A.element, B.element)

    geometry_factor:
      Calculates how well B fits given A's existing bonds
      and real bond angle constraints from lookup table
      Range: 0.0 - 1.0
      0.0 = geometrically impossible
      1.0 = perfect geometry

    temperature_factor:
      At very high T: bonds form but break immediately
      At optimal T: bonds form and persist
      At very low T: atoms don't move enough to meet
      Peaks at element-pair-specific optimal temperature

    electronegativity_factor:
      Based on electronegativity difference between A and B
      Higher difference = stronger ionic character = higher probability
      Derived from real Pauling scale values

    bond_order:
      Determined by available valence of both atoms
      Prefer double bonds for C-C when geometry allows
      Prefer single bonds otherwise
      Real rules from valence shell electron pair repulsion

    If random() < bond_formation_probability:
      Create new BondState
      bond.energy = calculate_bond_energy(A.element, B.element, order)
      absorb bond.energy * formation_fraction from local T field

6.3 Bond Energy Calculation

  bond_energy(element_a, element_b, order) =
      base_energy(element_a, element_b) * order * strain_factor

  base_energy from lookup table of real values:
    H-H:   436 kJ/mol
    C-C:   346 kJ/mol
    C=C:   614 kJ/mol
    C#C:   839 kJ/mol
    C-H:   413 kJ/mol
    C-O:   358 kJ/mol
    C=O:   799 kJ/mol
    O-H:   463 kJ/mol
    N-H:   391 kJ/mol
    C-N:   305 kJ/mol
    P-O:   335 kJ/mol
    S-H:   363 kJ/mol
    // ... complete table for all 10 elements

  strain_factor: 0.8 - 1.2
    Depends on local geometry and existing bonds
    Accounts for ring strain, angle strain

6.4 Bond Angle Lookup Table

  Real VSEPR geometry for each element:

  H:  linear              (1 bond only)
  C:  tetrahedral         109.5 deg  (4 bonds)
      trigonal planar     120 deg    (3 bonds + 1 double)
      linear              180 deg    (2 double bonds)
  N:  trigonal pyramidal  107 deg    (3 bonds)
      trigonal planar     120 deg    (2 bonds + 1 double)
  O:  bent                104.5 deg  (2 bonds)
  P:  trigonal bipyramidal 90/120 deg (5 bonds)
  S:  bent                103 deg    (2 bonds)
  Si: tetrahedral         109.5 deg  (4 bonds)
  Fe: octahedral          90 deg     (6 bonds)

  geometry_factor uses these to score candidate bonds.


7. ENERGY SYSTEM SPECIFICATION
--------------------------------------------------------

7.1 Energy Sources

  Each energy source in the world file
  is processed every tick.

  HYDROTHERMAL VENT:
    Adds heat to temperature field in radius
    T_field[cells in radius] += intensity * falloff(distance)
    falloff = 1 / (1 + distance^2)
    Also adds random velocity perturbation to atoms in radius
    (upwelling convection)

  SOLAR UV:
    Applied only to top layer of world (surface cells)
    UV field updated: uv_field[surface_cells] = intensity
    UV effect on atoms:
      For each bond in UV-exposed cells:
        uv_break_probability = uv_intensity * uv_sensitivity(bond)
        if random() < uv_break_probability: break bond
      uv_sensitivity higher for weaker bonds

7.2 Energy Conservation

  The simulation tracks total energy:
    kinetic_energy  = sum(0.5 * m * v^2) for all atoms
    potential_energy = sum(bond.energy) for all bonds
    field_energy    = sum(temperature_field * cell_area)

  Energy is not strictly conserved (open system).
  Energy sources add energy.
  Open boundaries lose energy.
  This is correct for a world with a sun and space.

  Energy balance is logged for debugging.


8. SPATIAL INDEX SPECIFICATION
--------------------------------------------------------

8.1 Structure

  Spatial hash grid.
  Cell size: 5 angstroms (~ bond search radius)
  HashMap from (col, row) integer pair to Vec<AtomId>

8.2 Updates

  Full rebuild every tick:
    Clear all cells
    For each atom: insert into cell at (x/cell_size, y/cell_size)
    O(n) rebuild

  Incremental update considered but full rebuild
  simpler and fast enough at target atom counts.

8.3 Queries

  neighbors(x, y, radius) -> Vec<AtomId>
    Calculate which cells overlap the search circle
    Return all AtomIds in those cells
    Caller filters by exact distance if needed

  Typical query: radius = 5 angstroms
  Typical cell size: 5 angstroms
  Typical cells checked per query: 9 (3x3 grid)


9. OBSERVER SYSTEM SPECIFICATION
--------------------------------------------------------

9.1 Purpose

  The observer does not affect simulation state.
  It only reads state and reports.
  Runs after all physics and chemistry.

9.2 Sampling

  Every N ticks (configured in .sim file):
    Collect statistics
    Detect notable events
    Write to output

9.3 Statistics Collected

  WORLD STATS:
    current tick
    real elapsed time
    ticks per second (actual)
    atom count (alive)
    bond count (alive)
    temperature field: min, max, mean
    pressure field: min, max, mean
    free atom counts by element
    bonded atom counts by element

  MOLECULAR STATS:
    molecule detection:
      walk bond graph from each atom
      connected component = one molecule
      count molecules by size (atom count)
      report size distribution

    largest molecule found this tick
    new largest molecule ever found (flag)

  NOTABLE EVENTS (checked every sample tick):
    largest molecule exceeds watch threshold
    molecule count change exceeds watch threshold
    all molecules below size 2 (extinction)
    novel connected component topology detected

9.4 Terminal Output Format

  Fixed refresh rate from .sim config.

  +===================================================+
  | BIOSIM  tick: 1,247,891  speed: 12,400 t/s        |
  +===================================================+
  | WORLD                                             |
  |   temp:     min 12 degC  avg 34 degC  max 847 degC         |
  |   pressure: min 0.8   avg 4.2   max 20.1 atm      |
  |   atoms:    4,821 alive  |  bonds: 2,341 alive    |
  +===================================================+
  | MOLECULES                                         |
  |   size 1:   892  (free atoms)                     |
  |   size 2-5: 445                                   |
  |   size 6-20: 89                                   |
  |   size 21+:  12  <- NOTABLE                        |
  |   largest:   47 atoms  (first seen tick 891,000)  |
  +===================================================+
  | EVENTS (last 5)                                   |
  |   [1,247,100] new largest molecule: 47 atoms      |
  |   [1,100,000] molecule count +34%                 |
  |   [  891,000] largest molecule: 31 atoms          |
  |   [  500,000] largest molecule: 18 atoms          |
  |   [   10,000] largest molecule: 6 atoms           |
  +===================================================+
  | [+/-] speed  [p] pause  [s] save  [q] quit        |
  +===================================================+


10. PARSER SPECIFICATION
--------------------------------------------------------

10.1 Entry Point

  Parser receives path to .sim file.
  Loads files in dependency order:
    1. .elem file (referenced in .sim or default)
    2. .world file (referenced in .sim)
    3. All .org files (referenced in .world)
    4. All .mol files (referenced transitively)
  Circular dependency check before loading.

10.2 Flattening

  After loading and validating all files:
  Flatten composition hierarchy to WorldState.

  Flattening process:
    Start with world-level place statements
    For each placed .org:
      Expand to its constituent .mol placements
      Apply position offsets
    For each placed .mol:
      If composite: expand to its imports recursively
      If primitive: instantiate its atoms and bonds
      Apply position offsets at each level
    Result: flat list of AtomState and BondState

  All AtomIds assigned during flattening.
  All BondIds assigned during flattening.
  Template hierarchy no longer exists after flattening.

10.3 Error Reporting

  All validation errors reported before runtime starts.
  Format:
    ERROR [V01] file.mol line 14: import 'missing.mol' not found
    ERROR [V04] water.mol line 8: bond O-H exceeds max_bonds for H
    WARNING: no organisms placed in world

  Multiple errors collected and reported together.
  Runtime does not start if any errors present.


11. CONFIGURATION
--------------------------------------------------------
Physical constants and tunable parameters.
Separate from BDL files.
Single config file: physics.cfg

  // Simulation scale factors
  // These map simulation units to real units
  angstrom_scale:        1.0        // 1 sim unit = 1 angstrom
  time_scale:            1.0e-15    // 1 tick = 1 femtosecond
  temperature_scale:     1.0        // 1 sim temp unit = 1 kelvin

  // Physics tuning
  diffusion_rate:        0.1
  pressure_sensitivity:  0.01
  bond_search_radius:    4.0        // angstroms
  formation_base_rate:   0.001      // per eligible pair per tick
  release_fraction:      0.5
  spring_constant:       100.0

  // Performance
  spatial_cell_size:     5.0        // angstroms
  temperature_cell_size: 10.0       // angstroms
  observer_interval:     1000       // ticks between samples

  // These can be tuned without changing any rules.
  // They affect speed and stability of the simulation.
  // They do not affect what chemistry is possible.


12. RUNTIME GUARANTEES
--------------------------------------------------------

The runtime guarantees:

  G01: No BDL concept above atom/bond level exists in runtime
  G02: Tick order is deterministic given same WorldState
  G03: Given same WorldState and same random seed:
       simulation is fully reproducible
  G04: Observer never modifies WorldState
  G05: Bond formation obeys max_bonds for all elements
  G06: Energy sources are the only energy inputs
  G07: Boundary conditions are enforced every tick
  G08: Spatial index is consistent with atom positions
  G09: Parser validates all V01-V15 before runtime starts
  G10: Save state captures complete WorldState
       Load state restores complete WorldState
       Resumed simulation is identical to uninterrupted run


13. PERFORMANCE TARGETS
--------------------------------------------------------

Target hardware: modern laptop, single core
                 (Rust allows easy parallelism later)

  Atom count    Target tick rate
  ------------------------------
  1,000         > 10,000 t/s
  5,000         > 2,000  t/s
  10,000        > 500    t/s
  50,000        > 50     t/s

  Memory:
  10,000 atoms:  < 10 MB
  50,000 atoms:  < 50 MB
  Save state:    < 100 MB per snapshot
```

---

