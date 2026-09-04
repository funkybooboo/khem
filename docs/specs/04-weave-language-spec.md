<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 5008-5597
  WHAT: round-2 language spec v0.1: all files .weave, declarations inside
        the file (elements / mol / org / world / sim), weave "0.1" version
        header, import resolution, stdlib, validation codes V-MOL/V-ORG/
        V-WORLD/V-SIM, versioning and extension points
  STATUS: superseded by 06-terminology-and-final-naming.md (mol -> struct,
        org -> body, layers -> region, .weave -> .crn, language renamed
        Cerne); grammar shape still the best language draft
-->

## Updated Formal Spec

```
==========================================================
CERNE / WEAVE
Formal Specification v0.1
==========================================================


============================================================
PART 1: WEAVE LANGUAGE SPECIFICATION
============================================================


1. OVERVIEW
------------------------------------------------------------

Weave is a domain-specific language for defining
physical and biological structures in terms of
fundamental atomic primitives.

Weave describes structure only.
No behavior is defined in Weave.
No logic is defined in Weave.
Only atoms, bonds, positions, and composition.

All Weave definitions live in .weave files.
The declaration type is defined inside the file.
The filename is arbitrary.
The extension is always .weave.


2. FILE STRUCTURE
------------------------------------------------------------

Every .weave file has the same top level structure:

  weave <format_version> {

      <declaration_type> <name> {
          ...
      }

  }

format_version:
  Semantic version string: "0.1"
  The parser checks this before reading anything else.
  Allows future versions to change syntax.
  Files with unknown versions are rejected with clear error.

declaration_type:
  One of: elements | mol | org | world | sim
  Exactly one declaration per file.
  A file with two declarations is a parse error.

Example:

  weave "0.1" {

      mol water {
          // ...
      }

  }


3. SYNTAX RULES
------------------------------------------------------------

3.1 General
  UTF-8 encoded text
  Case sensitive
  Whitespace not significant (spaces, tabs, newlines)
  No statement terminators (no semicolons)
  Blocks delimited by { }
  Identifiers: [a-zA-Z][a-zA-Z0-9_-]*

3.2 Comments
  // single line comment
  /* multi line comment */
  Comments may appear anywhere whitespace is allowed.

3.3 Literal types
  integer:   [0-9]+
             examples: 0, 42, 1000000
  float:     [0-9]+ . [0-9]+
             examples: 1.0, 3.14, 104.5
             no scientific notation in v0.1
  string:    "..." double quoted, no escapes in v0.1
  bool:      true | false
  coord:     ( float , float )
             examples: (0.0, 1.2), (-3.4, 0.0)
  range:     float - float
             examples: 0 - 50, 150.0 - 200.0
  dimension: integer x integer
             examples: 200 x 200, 1000 x 500

3.4 Physical units
  All values use implicit units.
  Units are fixed by context, not written in the file.
  
  distance/position:  angstroms
  angle:              degrees
  temperature:        celsius
  pressure:           atmospheres
  energy:             kJ/mol
  mass:               daltons
  intensity:          normalized 0.0 - 1.0
  time:               femtoseconds (internal)

3.5 Reserved words
  weave elements element mol org world sim
  atoms bonds ports import place connect
  sequence units chain_bond
  layers energy_sources
  at as in each first last
  count scatter rotate inside
  single double triple
  wrap wall open
  true false
  version description tier


4. DECLARATION: elements
------------------------------------------------------------

Purpose:
  Defines physical properties of atomic elements.
  Exactly one elements declaration exists per project.
  All other declarations depend on this.
  The runtime loads this before anything else.

Syntax:

  weave "0.1" {

    elements {

      element <SYMBOL> {
        name:               <string>
        atomic_number:      <integer>
        mass:               <float>
        valence:            <integer>
        electronegativity:  <float>
        radius:             <float>
        max_bonds:          <integer>
      }

      // repeat for each element

    }

  }

Field constraints:
  SYMBOL            1-2 characters, uppercase letters only
  name              non-empty string
  atomic_number     1 - 118
  mass              > 0.0
  valence           1 - 8
  electronegativity 0.0 - 4.0
  radius            > 0.0
  max_bonds         >= valence

All fields required. No defaults.
Unknown fields are a parse error.

Example:

  weave "0.1" {

    elements {

      element H {
        name:               "Hydrogen"
        atomic_number:      1
        mass:               1.008
        valence:            1
        electronegativity:  2.20
        radius:             0.53
        max_bonds:          1
      }

      element C {
        name:               "Carbon"
        atomic_number:      6
        mass:               12.011
        valence:            4
        electronegativity:  2.55
        radius:             0.77
        max_bonds:          4
      }

    }

  }


5. DECLARATION: mol
------------------------------------------------------------

Purpose:
  Defines a molecule or molecular structure.
  The fundamental composable unit of Weave.
  Can be primitive (atoms only) or
  composite (imports other mols).
  Cannot be both.

5.1 Primitive mol

  A primitive mol defines atoms and bonds directly.
  No imports allowed.

  weave "0.1" {

    mol <name> {

      description:  <string>    // optional
      tier:         <integer>   // optional, 0-9, organizational

      atoms {
        <label>: <ELEMENT>  at <coord>
        // label: unique within this mol
        // ELEMENT: must exist in elements declaration
        // coord: relative position, anchor at (0.0, 0.0)
      }

      bonds {
        <label> - <label> : <bond_type>
        // bond_type: single | double | triple
        // both labels must exist in atoms block
        // must not exceed max_bonds of either element
      }

      ports {
        <port_name>: <label>
        <port_name>: <label>, <label>  // multi-atom port
        // label must exist in atoms block
        // port_name unique within this mol
      }

    }

  }

  Constraints:
    V-MOL-01  At least one atom required
    V-MOL-02  All bond labels must reference defined atoms
    V-MOL-03  Bond must not exceed max_bonds of either atom
    V-MOL-04  No duplicate bonds between same atom pair
    V-MOL-05  Port labels must reference defined atoms
    V-MOL-06  Atom labels unique within mol
    V-MOL-07  Port names unique within mol

5.2 Composite mol

  A composite mol imports and connects other mols.
  No direct atom definitions allowed.

  weave "0.1" {

    mol <name> {

      description:  <string>
      tier:         <integer>

      import "<path>.weave"  as <alias>
      // path relative to project root
      // imported file must declare mol
      // alias unique within this mol

      place {
        <alias>  at <coord>
        <alias>  at <coord>  rotate <float>
        // all imported mols must be placed
      }

      connect {
        <alias>.<port_name>  ->  <alias>.<port_name>
        // forms bond between port atoms
        // bond type inferred from available valence
        // override: <alias>.<port> -> <alias>.<port> as <bond_type>
      }

      ports {
        <port_name>: <alias>.<port_name>
        // expose internal ports to outside
      }

    }

  }

  Constraints:
    V-MOL-08  All imports must resolve to existing .weave files
    V-MOL-09  Imported files must declare mol
    V-MOL-10  All aliases must be placed
    V-MOL-11  Connect references must resolve to valid ports
    V-MOL-12  Connected ports must have compatible valence
    V-MOL-13  No circular imports
    V-MOL-14  Alias names unique within mol

5.3 Sequence mol

  Special composite for polymer chains.
  Used for RNA, polypeptides, any repeating chain.

  weave "0.1" {

    mol <name> {

      description:  <string>
      tier:         <integer>

      units {
        <symbol>: "<path>.weave"
        // symbol: single uppercase letter
        // imported file must declare mol
        // that mol must have chain_in and chain_out ports
      }

      sequence: [ <symbol>... ]
      // space separated list of symbols
      // all symbols must be defined in units block
      // minimum length: 2

      chain_bond {
        from_port:  <port_name>   // port on unit N
        to_port:    <port_name>   // port on unit N+1
        type:       <bond_type>   // single | double | triple
      }

      chain_spacing: <float>      // angstroms between unit anchors

      ports {
        start: first.<port_name>
        end:   last.<port_name>
        each:  each.<port_name>   // same port exposed on every unit
      }

    }

  }

  Constraints:
    V-MOL-15  All unit symbols used in sequence must be defined
    V-MOL-16  chain_bond ports must exist on unit mols
    V-MOL-17  Sequence length >= 2
    V-MOL-18  chain_spacing > 0.0


6. DECLARATION: org
------------------------------------------------------------

Purpose:
  Defines an organism as a spatial arrangement
  of molecular structures.
  An organism is just atoms.
  No behavior defined here.

  weave "0.1" {

    org <name> {

      description: <string>

      import "<path>.weave"  as <alias>
      // imported file must declare mol

      place {

        <alias>  at <coord>
        // simple placement

        <alias>  at <coord>  rotate <float>
        // with rotation

        <alias>  count <integer>  in <alias>  scatter
        // place N copies randomly inside another structure
        // 'in' target must have an interior port

        <alias>  count <integer>  scatter
        // place N copies randomly in org space

      }

      connect {
        <alias>.<port_name>  ->  <alias>.<port_name>
      }

      ports {
        <port_name>: <alias>.<port_name>
      }

    }

  }

  Constraints:
    V-ORG-01  All imports must resolve to .weave files declaring mol
    V-ORG-02  All aliases must appear in place block
    V-ORG-03  in target must have interior port defined
    V-ORG-04  scatter requires count
    V-ORG-05  connect references must resolve


7. DECLARATION: world
------------------------------------------------------------

Purpose:
  Defines the complete initial world state.
  Environment, layers, organisms, free chemistry.

  weave "0.1" {

    world <name> {

      description: <string>

      size:      <dimension>    // angstroms
      boundary:  wrap | wall | open

      import "<path>.weave"  as <alias>
      // imported file must declare mol or org

      layers {

        <layer_name> (y: <range>) {

          temperature:  <float>
          pressure:     <float>
          uv:           <float>     // 0.0 - 1.0

          place {
            <alias>     count <integer>  scatter
            <ELEMENT>   count <integer>  scatter
            // ELEMENT places free unbound atoms
          }

        }

      }

      energy_sources {

        <source_name> {
          type:         hydrothermal | solar_uv | radiation
          position:     <coord>        // for point sources
          intensity:    <float>        // 0.0 - 1.0
          radius:       <float>        // angstroms
          surface_only: <bool>         // for solar_uv
        }

      }

    }

  }

  Constraints:
    V-WORLD-01  Layer y ranges must not overlap
    V-WORLD-02  Layer y ranges must cover 0 to world height exactly
    V-WORLD-03  uv must be 0.0 - 1.0
    V-WORLD-04  intensity must be 0.0 - 1.0
    V-WORLD-05  All imports must resolve
    V-WORLD-06  energy source type must be valid vocabulary
    V-WORLD-07  size dimensions must be > 0
    V-WORLD-08  solar_uv surface_only defaults to true if omitted


8. DECLARATION: sim
------------------------------------------------------------

Purpose:
  Defines how a simulation run executes.
  Entry point that cerne is passed.

  weave "0.1" {

    sim <name> {

      description: <string>

      world: "<path>.weave"
      // must declare world

      execution {
        tick_rate:  max | <integer>       // target ticks/sec
        max_ticks:  unlimited | <integer>
        seed:       random | <integer>    // for reproducibility
      }

      output {
        stream:          stdout           // always stdout in v0.1
        format:          ndjson           // always ndjson in v0.1
        tick_interval:   <integer>        // emit tick event every N ticks
        bond_events:     true | false     // emit bond_formed/bond_broken
        notable_only:    true | false     // suppress tick events
      }

      watch {
        // what the runtime considers notable
        // notable events always emitted regardless of output settings
        molecule_size_above:      <integer>
        population_change_above:  <float>    // percent
        bond_type_first:          true | false
        extinction:               true | false
      }

    }

  }

  Constraints:
    V-SIM-01  world path must resolve to .weave declaring world
    V-SIM-02  tick_rate must be > 0 if not max
    V-SIM-03  max_ticks must be > 0 if not unlimited
    V-SIM-04  tick_interval must be > 0
    V-SIM-05  population_change_above must be 0.0 - 100.0


9. IMPORT RESOLUTION
------------------------------------------------------------

9.1 Project root
  The directory containing the .weave file
  passed directly to cerne is the project root.
  All import paths resolve relative to project root.

9.2 Search path
  When an import path does not resolve from project root,
  cerne searches in order:
    1. Project root
    2. ./weave/
    3. ~/.cerne/stdlib/
    4. System stdlib (installation-defined path)

  First match wins.
  If not found anywhere: V-MOL-08 error.

9.3 Standard library
  cerne ships with a stdlib of primitive .weave files.
  elements.weave is part of stdlib.
  If no elements import in project, stdlib elements used.
  Project can override stdlib by providing own elements.weave.

9.4 Circular import detection
  Before loading any file, cerne builds the full
  dependency graph from all imports.
  Cycles detected before any loading begins.
  All cycles reported together, not one at a time.


10. VERSIONING AND EXTENSIBILITY
------------------------------------------------------------

10.1 Format version
  The "0.1" string in weave "0.1" { }
  is the Weave language version.
  cerne v0.1 accepts only "0.1".
  Future cerne versions may accept multiple versions.
  Unknown version: hard error, no attempt to parse.

10.2 Unknown fields
  In v0.1: unknown fields are a parse error.
  Rationale: fail loudly rather than silently ignore.
  Future: may introduce ignore_unknown flag.

10.3 Extension points (reserved for future)
  The following declaration types are reserved
  and will cause a parse error if used in v0.1:
    plugin
    extend
    override
    macro
  They are reserved to prevent naming conflicts
  with future extension mechanisms.

10.4 Comments as metadata
  In v0.1, structured metadata can be embedded
  in comments. No formal spec yet.
  Future versions will formalize this.
  Example:
    // @author: name
    // @source: doi:10.1000/xyz123
    // @validated: true


============================================================
