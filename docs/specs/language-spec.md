# khem language specification

Language version: 0.1 (the version string "0.1" inside .kem files)
Status: canonical as of 2026-09-04. Drafts until validated: phase 1
(PLAN.md) tests the substrate before the parser is built, and this
document is revised against kernel reality at that point (ADR-0006).
Provenance: reconciled from the founding conversation
(initial-idea.md) with the final terminology applied (ADR-0007,
ADR-0009). Conversation-era drafts: docs/history/spec-drafts/.

## 1. What khem is

khem is a domain-specific language for defining physical and
biological structures in terms of fundamental atomic primitives.

khem describes WHAT EXISTS. The runtime decides WHAT RULES APPLY.
The two never mix. A khem definition is purely structural: no
behavior, no logic, no lifecycle - only atoms, bonds, positions,
and composition.

All definitions live in .kem files. The declaration type is declared
inside the file; filenames are arbitrary; the extension is always
.kem. The runtime entry point is a file declaring run.

    khem experiment_1.kem

## 2. File structure

Every .kem file has the same top-level shape:

    khem "0.1" {

        <declaration> <name> {
            ...
        }

    }

- "0.1" is the language version, checked before anything else is
  parsed. khem v0.1 accepts only "0.1"; unknown versions are a hard
  error.
- Exactly one declaration per file; two is a parse error.
- The wrapper keyword is the language name, matching the extension
  convention (.kem files, khem wrapper).
- Unknown fields are a parse error in v0.1: fail loudly rather than
  silently ignore.

Declarations: elements | struct | chain | body | world | run

## 3. Syntax

- UTF-8, case sensitive. Whitespace is not significant. No statement
  terminators. Blocks delimited by { }.
- Identifiers: [a-zA-Z][a-zA-Z0-9_-]*, must start with a letter.
- Comments: // to end of line, /* */ blocks, anywhere whitespace is
  allowed.

Literals:

    integer    [0-9]+              42
    float      [0-9]+ . [0-9]+     104.5      (no scientific notation in v0.1)
    string     "..."               "Hydrogen" (no escapes in v0.1)
    bool       true | false
    coord      ( float , float )   (1.2, -3.4)
    range      float - float       0 - 50
    dimension  integer x integer   200 x 200

## 4. Units

Implicit, fixed by context, never written in the file:

    distance / position    angstroms
    angle                  degrees
    temperature            celsius
    pressure               atmospheres
    energy                 kJ/mol
    mass                   daltons
    intensity              normalized 0.0 - 1.0
    time (internal)        femtoseconds

## 5. Declaration: elements

Defines the physical properties of atomic elements. Exactly one
elements declaration exists per project. All other declarations depend
on it; the runtime loads it before anything else.

    khem "0.1" {

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

      }

    }

Field constraints:

    SYMBOL             1-2 characters, uppercase letters only
    name               non-empty string
    atomic_number      1 - 118
    mass               > 0.0
    valence            1 - 8
    electronegativity  0.0 - 4.0
    radius             > 0.0
    max_bonds          >= valence

All fields required. No defaults. Unknown fields are a parse error.

The standard library ships the 10 elements of the founding design
(H, C, N, O, P, S, Si, Fe, Na, Cl) as elements.kem; a project
overrides the stdlib by providing its own (section 11).

## 6. Declaration: struct

A struct defines a molecule or molecular structure: the fundamental
composable unit. A struct is primitive (atoms and bonds only) or
composite (built from other structs). It cannot be both.

### 6.1 Primitive struct

    khem "0.1" {

      struct water {

        description: "water"
        tier:        1        // optional, 0-9, organizational only

        atoms {
          O1: O  at ( 0.00, 0.00)
          H1: H  at (-0.96, 0.58)
          H2: H  at ( 0.96, 0.58)
        }

        bonds {
          O1 - H1 : single
          O1 - H2 : single
        }

        port donor:    O1
        port acceptor: H1, H2      // multi-atom port

      }

    }

- Atom labels are unique within the struct; positions are relative to
  the struct's (0,0) anchor.
- Bond types: single | double | triple.
- Ports name atoms (or atom groups) as the struct's connection points
  for composition. Port names are unique within the struct.

### 6.2 Composite struct

    khem "0.1" {

      struct nucleotide_A {

        use "adenine_base.kem"  as base
        use "ribose.kem"        as sugar
        use "phosphate.kem"    as phos

        place base  at (0.0, 0.0)
        place sugar at (4.0, 0.0)
        place phos  at (7.5, 0.0)

        wire base.sugar_attach   -> sugar.base_attach
        wire sugar.phosphate_out -> phos.chain_in

        port chain_in:  phos.upstream
        port chain_out: sugar.three_prime
        port pair:      base.pair_bond

      }

    }

- use imports another .kem file that declares struct, under an alias.
- All aliases must be placed. Positions are relative to the parent;
  optional rotate <degrees> rotates the alias around its anchor.
- wire forms a bond between the named ports' atoms. Bond type is
  inferred from available valence unless overridden:
  wire <alias>.<port> -> <alias>.<port> as single
- The ports block exposes imported structs' ports to the next level
  up.

## 7. Declaration: chain

A chain defines a polymer: a sequence of structs connected by a
repeating bond rule (RNA, polypeptides, any repeating chain).

    khem "0.1" {

      chain rna_strand {

        use "nucleotide_A.kem"  as A
        use "nucleotide_U.kem"  as U
        use "nucleotide_G.kem"  as G
        use "nucleotide_C.kem"  as C

        sequence: A U G C A U G C

        link {
          from: chain_out        // port on unit N
          to:   chain_in         // port on unit N+1
          bond: single
        }

        chain_spacing: 4.2        // optional, angstroms between anchors

        port start: first.chain_in
        port end:   last.chain_out
        port pairs: each.pair     // exposed on every unit

      }

    }

- Every symbol used in sequence must be defined by a use statement.
- link names the ports that connect unit N to unit N+1.
- The sequence is data (the genome), never behavior.
- Minimum sequence length is 2; chain_spacing must be > 0.

## 8. Declaration: body

A body defines an organism: a spatial arrangement of structs. A body
is just atoms; no behavior is defined here.

    khem "0.1" {

      body minimal_cell {

        use "vesicle.kem"      as membrane
        use "rna_strand.kem"   as genome
        use "nucleotide_A.kem" as free_A
        use "nucleotide_U.kem" as free_U
        use "nucleotide_G.kem" as free_G
        use "nucleotide_C.kem" as free_C
        use "water.kem"        as h2o

        place membrane at (0.0, 0.0)
        place genome   at (0.0, 0.0) inside membrane

        place free_A count 20  inside membrane scatter
        place free_U count 20  inside membrane scatter
        place free_G count 20  inside membrane scatter
        place free_C count 20  inside membrane scatter
        place h2o    count 200 inside membrane scatter

      }

    }

Place forms:

    place <alias> at (<x>, <y>)                    single placement
    place <alias> at (<x>, <y>) rotate <deg>       rotated placement
    place <alias> count <n> inside <alias> scatter N copies inside another
                                                   struct
    place <alias> count <n> scatter                N copies in body space

- inside requires the target struct to declare an interior port (a
  closed structure such as a vesicle).
- scatter requires count.
- wire connects ports exactly as in composite structs.

## 9. Declaration: world

Defines the complete initial world state: environment, regions,
energy sources, and placement.

    khem "0.1" {

      world primordial_pond {

        size:     200 x 200
        boundary: wrap                 // wrap | wall | open

        use "minimal_cell.kem"  as cell
        use "water.kem"         as h2o
        use "nucleotide_A.kem"   as free_A
        use "lipid.kem"          as lipid

        region surface (y: 150 - 200) {
          temperature: 15
          pressure:    0.8
          uv:          0.7
          place h2o    count 10000 scatter
          place free_A count 200   scatter
        }

        region ocean (y: 50 - 150) {
          temperature: 35
          pressure:    4.0
          uv:          0.1
          place h2o    count 50000 scatter
          place lipid  count 2000  scatter
          place free_A count 500   scatter
          place cell   count 10    scatter
        }

        region seafloor (y: 0 - 50) {
          temperature: 80
          pressure:    20.0
          uv:          0.0
          place Si count 5000 scatter      // bare element symbol
          place Fe count 2000 scatter      // places free atoms
        }

        source hydrothermal_vent {
          type:      hydrothermal   // hydrothermal | solar_uv | radiation
          position:  (100.0, 0.0)
          intensity: 0.8
          radius:    15.0
        }

        source sunlight {
          type:         solar_uv
          intensity:    0.7
          surface_only: true            // defaults to true if omitted
        }

      }

    }

- Regions partition the world by y-range. Ranges must not overlap and
  together must cover 0 to world height exactly.
- uv and intensity must be 0.0 - 1.0.
- A bare element symbol in a place block places free atoms of that
  element.

## 10. Declaration: run

Defines how a simulation executes. The entry point passed to the
khem binary must declare run.

    khem "0.1" {

      run experiment_1 {

        world: "primordial_pond.kem"

        execution {
          tick_rate: max              // max | <integer> target ticks/sec
          max_ticks: unlimited        // unlimited | <integer>
          seed:      42               // random | <integer>, for reproducibility
        }

        output {
          stream:         stdout       // always stdout in v0.1
          format:         ndjson       // always ndjson in v0.1
          tick_interval:  1000         // emit a tick event every N ticks
          bond_events:    false
          notable_only:   false
        }

        watch {
          molecule_size_above:      50
          population_change_above: 20.0   // percent, 0.0 - 100.0
          extinction:              true
        }

      }

    }

## 11. Import resolution and the standard library

- The project root is the directory containing the .kem file passed to
  khem. All import paths resolve relative to it.
- Search order, first match wins:
    1. project root
    2. ./khem/
    3. ~/.khem/stdlib/
    4. system stdlib (installation-defined path)
- khem ships a standard library of primitive .kem files
  (elements.kem, water.kem, and friends). A project overrides a
  stdlib file by providing its own at the same import path.
- Before loading any file, khem builds the full dependency graph from
  all imports and reports every circular import together, not one at
  a time.

## 12. Validation

All validation errors are reported before tick 0; the runtime does not
start if any are present. Warnings are logged but do not halt.

    V-STRUCT-01  primitive struct: at least one atom required
    V-STRUCT-02  bond labels must reference defined atoms
    V-STRUCT-03  bond must not exceed max_bonds of either atom
    V-STRUCT-04  no duplicate bonds between the same atom pair
    V-STRUCT-05  port labels must reference defined atoms
    V-STRUCT-06  atom labels unique within the struct
    V-STRUCT-07  port names unique within the struct
    V-STRUCT-08  composite struct: all imports must resolve to .kem files
    V-STRUCT-09  composite struct: imported files must declare struct
    V-STRUCT-10  composite struct: all aliases must be placed
    V-STRUCT-11  composite struct: wire references must resolve to valid ports
    V-STRUCT-12  composite struct: wired ports must have compatible valence
    V-STRUCT-13  no circular imports
    V-STRUCT-14  alias names unique within the struct
    V-CHAIN-01   every sequence symbol must be defined by a use
    V-CHAIN-02   link ports must exist on the unit structs
    V-CHAIN-03   sequence length >= 2
    V-CHAIN-04   chain_spacing > 0.0
    V-BODY-01    all imports must resolve to .kem files declaring struct
    V-BODY-02    all aliases must appear in a place block
    V-BODY-03    inside targets must have an interior port
    V-BODY-04    scatter requires count
    V-BODY-05    wire references must resolve
    V-WORLD-01   region y ranges must not overlap
    V-WORLD-02   region y ranges must cover 0 to world height exactly
    V-WORLD-03   uv must be 0.0 - 1.0
    V-WORLD-04   source intensity must be 0.0 - 1.0
    V-WORLD-05   all imports must resolve
    V-WORLD-06   source type must be a valid vocabulary word
    V-WORLD-07   size dimensions must be > 0
    V-WORLD-08   solar_uv surface_only defaults to true if omitted
    V-RUN-01     world path must resolve to a .kem file declaring world
    V-RUN-02     tick_rate must be > 0 if not max
    V-RUN-03     max_ticks must be > 0 if not unlimited
    V-RUN-04     tick_interval must be > 0
    V-RUN-05     population_change_above must be 0.0 - 100.0

## 13. Reserved words

    khem elements element struct chain body world run
    use place wire port link inside scatter count at as rotate
    atoms bonds sequence description tier
    single double triple
    region source size boundary wrap wall open
    temperature pressure uv type position intensity radius
    surface_only
    execution output watch tick_rate max_ticks seed
    tick_interval bond_events notable_only
    max unlimited random first last each
    true false

Reserved for future extension mechanisms (parse error if used in
v0.1): plugin, extend, override, macro.