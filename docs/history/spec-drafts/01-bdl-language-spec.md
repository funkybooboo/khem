<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 3207-3744
  WHAT: formal spec draft v0.1 of the definition language, then named BDL
        (BioSim Definition Language), file types .elem/.mol/.org/.world/.sim
  STATUS: historical draft; superseded by 04-weave-language-spec.md and
          06-terminology-and-final-naming.md
  NOTE: content verbatim except ASCII normalization (box drawing to ASCII,
        -> for arrows, # for triple bonds, deg for degrees)
-->

## SPEC 1: The DSL - BioSim Definition Language (BDL)

```
========================================================
BIOSIM DEFINITION LANGUAGE (BDL)
Formal Specification v0.1
========================================================


1. OVERVIEW
--------------------------------------------------------
BDL is a domain-specific language for defining physical
and biological structures in terms of fundamental
atomic primitives.

BDL describes WHAT EXISTS.
The runtime describes WHAT RULES APPLY.
These two things never mix.

A BDL definition is purely structural.
No behavior is defined in BDL.
No logic is defined in BDL.
No lifecycle is defined in BDL.
Only atoms, bonds, positions, and composition.


2. FILE TYPES
--------------------------------------------------------
Extension  Purpose                        Imports
-----------------------------------------------------
.elem      Element definitions            nothing
.mol       Molecule / structure           .mol
.org       Organism                       .mol
.world     World definition               .mol .org
.sim       Simulation run configuration   .world
.state     Saved simulation snapshot      (binary)

Dependency rules:
  .elem   imports nothing
  .mol    imports .mol only
  .org    imports .mol only
  .world  imports .mol and .org
  .sim    imports .world only
  .state  is not a BDL file, it is runtime output

Circular imports are illegal and must be caught
by the parser before runtime begins.


3. SYNTAX RULES
--------------------------------------------------------
3.1 General

  - UTF-8 encoded text files
  - Case sensitive
  - Whitespace is not significant
  - Semicolons are NOT used
  - Blocks delimited by { }
  - Comments: // single line
              /* multi line */
  - Numbers: integer or float
  - Strings: "quoted"
  - Identifiers: [a-z A-Z 0-9 _ -]
                 must start with a letter

3.2 Units

  All physical values use SI-derived units
  defined explicitly in the value:

  distance:     angstroms (A) - written as plain number
  angle:        degrees
  temperature:  celsius
  pressure:     atmospheres
  energy:       kJ/mol
  mass:         daltons

  Example:
    position: (1.2, 0.0)        // angstroms, implicit
    angle: 104.5                 // degrees, implicit
    temperature: 35              // celsius, implicit
    bond_energy: 436             // kJ/mol, implicit

3.3 Coordinates

  2D Cartesian coordinates (x, y)
  Relative within a template definition
  Absolute in world placement
  Origin (0,0) is bottom-left of world
  Within a template, (0,0) is the template anchor point


4. .elem FILE SPECIFICATION
--------------------------------------------------------
Purpose:
  Defines the physical properties of elements.
  This is the foundation everything else builds on.
  Exactly one .elem file exists in a project.
  The runtime reads this before anything else.

Syntax:

  elements {

      element <SYMBOL> {
          name:              <string>
          atomic_number:     <integer>
          mass:              <float>      // daltons
          valence:           <integer>    // bonding electrons
          electronegativity: <float>      // Pauling scale
          radius:            <float>      // angstroms, covalent
          max_bonds:         <integer>    // max simultaneous bonds
      }

  }

Constraints:
  - SYMBOL must be 1-2 uppercase letters
  - All fields are required
  - valence must be 1-8
  - electronegativity must be 0.0 - 4.0
  - radius must be > 0
  - max_bonds must be >= valence

Example:

  elements {

      element H {
          name:              "Hydrogen"
          atomic_number:     1
          mass:              1.008
          valence:           1
          electronegativity: 2.20
          radius:            0.53
          max_bonds:         1
      }

      element C {
          name:              "Carbon"
          atomic_number:     6
          mass:              12.011
          valence:           4
          electronegativity: 2.55
          radius:            0.77
          max_bonds:         4
      }

      // ... all elements

  }


5. .mol FILE SPECIFICATION
--------------------------------------------------------
Purpose:
  Defines a molecule or molecular structure.
  May be a primitive (atoms only) or
  composite (imports other .mol files).
  A .mol file is EITHER primitive OR composite.
  It cannot be both.

5.1 PRIMITIVE mol

  A primitive mol contains only atoms and bonds.
  No imports.

  Syntax:

    mol <name> {

        // optional metadata
        description: <string>
        tier: <integer>         // 0-5, organizational only

        atoms {
            <element>  at (<x>, <y>)  as <label>
            <element>  at (<x>, <y>)  as <label>
            // label is required, unique within this mol
            // used to reference atom in bonds and ports
        }

        bonds {
            <label> - <label> : <bond_type>
            // bond_type: single | double | triple
            // angle is calculated by runtime
            // from real geometry lookup table
        }

        // INTERFACE: what other mols connect to
        // labels must reference atoms defined above
        ports {
            <port_name>: <atom_label>
            <port_name>: <atom_label>, <atom_label>  // multi-atom port
        }

    }

  Constraints:
    - At least 1 atom required
    - All bond labels must reference defined atoms
    - Bond must not exceed max_bonds of either atom
    - Port labels must reference defined atoms
    - Atom labels unique within mol
    - Positions are relative to (0,0) anchor

5.2 COMPOSITE mol

  A composite mol imports other mols
  and connects them at their ports.
  No direct atom definitions.

  Syntax:

    mol <name> {

        description: <string>
        tier: <integer>

        import <filename>.mol  as <alias>
        import <filename>.mol  as <alias>
        // filename relative to project root
        // alias used to reference this instance

        place {
            <alias>  at (<x>, <y>)
            <alias>  at (<x>, <y>)
            // positions are relative
            // orientation defaults to 0 degrees
            // optional: rotate <degrees>
        }

        connect {
            <alias>.<port_name>  ->  <alias>.<port_name>
            // forms a bond between the port atoms
            // bond type inferred from valence
            // can override: type: single|double|triple
        }

        // expose ports to next level up
        ports {
            <port_name>:  <alias>.<port_name>
            <port_name>:  <alias>.<atom_label>
        }

    }

5.3 SEQUENCE mol (special composite for chains)

  A sequence mol defines a chain of repeating units.
  Used for RNA, DNA, polypeptides.

  Syntax:

    mol <name> {

        description: <string>

        // define the unit types
        units {
            <symbol>: <filename>.mol
            <symbol>: <filename>.mol
        }

        // the actual sequence
        sequence: [ <symbol> <symbol> <symbol> ... ]

        // how adjacent units connect
        chain_bond {
            from_port: <port_name>    // port on unit N
            to_port:   <port_name>    // port on unit N+1
            type:      <bond_type>
        }

        // spacing between units
        chain_spacing: <float>        // angstroms

        ports {
            start: first.<port_name>
            end:   last.<port_name>
            each:  each.<port_name>   // exposes port on every unit
        }

    }

  Constraints:
    - units block must define all symbols used in sequence
    - chain_bond ports must exist on unit mols
    - sequence must contain at least 2 units


6. .org FILE SPECIFICATION
--------------------------------------------------------
Purpose:
  Defines an organism as a collection of
  molecular structures with spatial relationships.
  An organism is just atoms.
  No behavior is defined here.

Syntax:

  org <name> {

      description: <string>

      import <filename>.mol  as <alias>
      import <filename>.mol  as <alias>

      place {

          <alias>  at (<x>, <y>)
          // optional modifiers:
          //   inside: <other_alias>  - constrain to interior
          //   scatter: <count>       - place N copies randomly
          //   rotate: <degrees>
          //   count: <integer>

          <alias>  count: <integer>  inside: <alias>  scatter: true

      }

      connect {
          <alias>.<port>  ->  <alias>.<port>
      }

      // ports exposed to world placement
      ports {
          <port_name>: <alias>.<port_name>
      }

  }

Constraints:
  - inside: reference must be a mol with an interior port
  - scatter requires count
  - All imports must resolve to existing files


7. .world FILE SPECIFICATION
--------------------------------------------------------
Purpose:
  Defines the complete world state at tick 0.
  Specifies physical environment.
  Places organisms and molecules.

Syntax:

  world <name> {

      description: <string>

      // world dimensions
      size: <width> x <height>      // in angstroms

      // what happens at edges
      boundary: wrap | wall | open

      // imports
      import <filename>.mol  as <alias>
      import <filename>.org  as <alias>

      // physical layers
      // y ranges define regions with different
      // starting physical conditions
      layers {

          <layer_name> (y: <min> - <max>) {

              temperature: <float>
              pressure:    <float>
              uv:          <float>   // 0.0 - 1.0

              place {
                  <alias>  count: <integer>  scatter: true
                  <element_symbol>  count: <integer>  scatter: true
                  // element_symbol places free atoms
              }

          }

      }

      // persistent energy sources
      energy_sources {

          <source_name> {
              type:        hydrothermal | solar_uv | radiation
              position:    (<x>, <y>)    // for point sources
              intensity:   <float>
              radius:      <float>       // for point sources
              surface_only: true | false // for solar
          }

      }

  }

Constraints:
  - Layer y ranges must not overlap
  - Layer y ranges must cover 0 to world height
  - scatter requires count
  - energy source types are fixed vocabulary
  - uv must be 0.0 - 1.0
  - intensity must be >= 0


8. .sim FILE SPECIFICATION
--------------------------------------------------------
Purpose:
  Defines how a simulation run executes.
  References a world file.
  Configures output, speed, and observation.

Syntax:

  sim <name> {

      description: <string>

      world: <filename>.world

      execution {
          tick_rate:  max | <integer>     // ticks per second
          max_ticks:  unlimited | <integer>
          max_time:   unlimited | <duration>
          // duration format: 10s | 5m | 2h | 1d
      }

      output {
          mode:         terminal | log | both
          refresh:      <integer>  ticks
          log_path:     <string>

          save_state {
              interval: <integer>  ticks
              path:     <string>
              keep:     <integer>  // how many saves to keep
          }
      }

      // what the runtime should flag as notable
      watch {
          molecule_size_above:      <integer>   // atom count
          new_bond_type:            true | false
          population_change_above:  <float>     // percent
          extinction:               true | false
          novel_structure:          true | false
      }

      controls {
          allow_pause:         true | false
          allow_speed_change:  true | false
          allow_save:          true | false
          allow_load:          true | false
      }

  }


9. SCOPING AND RESOLUTION RULES
--------------------------------------------------------
9.1 Import resolution

  All imports are relative to the project root.
  Project root is the directory containing the .sim file.

  Import search order:
    1. Exact path from project root
    2. /templates/primitives/
    3. /templates/molecules/
    4. /templates/structures/
    5. /templates/organisms/

9.2 Name scoping

  Names are scoped to their file.
  alias.port_name is the full reference syntax.
  No global namespace except element symbols.
  Element symbols (H, C, N...) are always global.

9.3 Port resolution

  A port is a reference to one or more atoms.
  When a connect statement wires two ports,
  the runtime forms a bond between those atoms.
  Bond type is inferred from available valence
  unless explicitly specified.

9.4 Anchor points

  Every mol has an anchor at (0,0).
  When placed, the anchor is moved to the place position.
  All atoms move with it.
  Rotation is around the anchor point.


10. VALIDATION RULES
--------------------------------------------------------
The parser must validate before runtime starts:

  V01: All imports resolve to existing files
  V02: No circular imports
  V03: All bond atom labels exist in atoms block
  V04: Bond does not exceed max_bonds of either atom
  V05: All port atom labels exist in atoms block
  V06: All connect port references resolve
  V07: Connected ports have compatible valence
  V08: inside references have interior port
  V09: Layer y ranges do not overlap
  V10: Layer y ranges cover full world height
  V11: All element symbols are defined in .elem file
  V12: Sequence units all defined in units block
  V13: chain_bond ports exist on unit mols
  V14: world boundary is valid vocabulary
  V15: energy source type is valid vocabulary

Validation errors halt execution before runtime starts.
Validation warnings are logged but do not halt.


11. RESERVED WORDS
--------------------------------------------------------
mol org world sim elements
element atoms bonds ports
import place connect
sequence units chain_bond
layers energy_sources
temperature pressure uv
type position intensity radius
single double triple
wrap wall open
max unlimited true false
inside scatter count rotate
at as each first last
```

---

