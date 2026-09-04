<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 6413-7243 (includes the two closing user decisions)
  WHAT: final terminology overhaul and naming decisions, with rationale:
        mol -> struct, org -> body, layers -> region, energy_sources ->
        source, import -> use, connect -> wire, sim -> run, chain_bond ->
        link; new keywords chain, body, link, source, region; complete
        keyword set; .crn extension; the language itself named Cerne
        (language Cerne / files .crn / runtime cerne)
  STATUS: AUTHORITATIVE deltas over 04 and 05
  NOTE: the final founding-conversation identity (Cerne / .crn / cerne)
        was superseded at project level by khem / .kem / khem on
        2026-09-04; see README.md here. Content below is verbatim.
-->

## Let's Think About This Cleanly

The problem with what we have is we mixed metaphors. Let me strip it back.

---

## The Right Mental Model

```
WHAT WE ARE ACTUALLY BUILDING:

A language called Weave.
A compiler/runtime called cerne.
cerne reads Weave source files.
cerne simulates and streams output.

That is it.
Nothing else.

Just like:
  gcc reads .c files
  cerne reads .weave files

Just like:
  Verilog describes hardware structure
  Weave describes physical/biological structure

Just like:
  A C program has primitives (int, char, float)
  A Weave program has primitives (atoms, bonds)

Just like:
  C lets you compose primitives into structs
  Weave lets you compose atoms into molecules

Just like:
  C lets you compose structs into programs
  Weave lets you compose molecules into worlds
```

---

## The Problem With Our Current Naming

```
CURRENT          PROBLEM
----------------------------------------------------
mol              abbreviation, ambiguous
org              abbreviation, ambiguous
.elem            not a real word in this context
ports            correct but mixed with biology
connect          okay but vague
place            okay but vague
layers           okay but not precise
energy_sources   too wordy, too specific
world            okay but feels like a game
sim              abbreviation
watch            feels like debugging not science

The issue is we mixed:
  chemistry terminology  (mol, elem)
  game terminology       (world)
  vague english          (place, connect, watch)
  abbreviations          (sim, org)
```

---

## Clean Terminology From First Principles

```
What does each thing actually represent?

THING                WHAT IT ACTUALLY IS
---------------------------------------------------------------
A single atom type   A type definition. Like a typedef in C.
                     -> type

A molecule           A structure. Atoms composed together.
                     Like a struct in C.
                     -> struct

An organism          A module. Structures composed together.
                     Like a module in Verilog.
                     -> module  (but this is a reserved word everywhere)
                     -> body    (a physical body, composed of structs)
                     -> assembly (assembled from parts)
                     -> compose
                     -> unit

The world            The top level environment.
                     Like a top-level module in Verilog.
                     Like main() in C.
                     -> world is actually fine, or:
                     -> environment
                     -> scene
                     -> field

The simulation run   The execution parameters.
                     Like compiler flags + runtime config.
                     -> run
                     -> exec
                     -> simulation  (spelled out)

Ports                Connection points between structures.
                     Same word as Verilog. Keep it.
                     -> port  [OK]

Connect              Wiring ports together.
                     Same concept as Verilog.
                     -> wire  (exactly Verilog)
                     -> bind
                     -> connect  [OK]  (actually fine)

Place                Instantiating something in space.
                     Like instantiating a module in Verilog.
                     -> instance
                     -> spawn
                     -> place  [OK]  (actually clear)

Layers               Regions of the environment.
                     -> region  (more precise)
                     -> zone
                     -> layer  [OK]  (actually fine)
```

---

## The Clean Proposal

```
Keep what is already good.
Fix what is muddled.
Use real terminology.

WEAVE KEYWORDS:
--------------------------------------------------------

weave               the file wrapper, like a namespace
                    weave "0.1" { ... }

// TYPE DEFINITIONS (what things are made of)

element             defines an atomic element
                    element H { ... }
                    clean, real word, not abbreviated

struct              defines a molecule or structure
                    composed from elements or other structs
                    struct water { ... }
                    struct nucleotide_A { ... }
                    struct rna_strand { ... }
                    FAMILIAR: every programmer knows struct
                    ACCURATE: it IS a data structure of atoms

body                defines an organism
                    composed from structs
                    body minimal_cell { ... }
                    ACCURATE: a physical body
                    DISTINCT: clearly different from struct
                    CLEAN: one syllable, real word

world               defines the environment
                    composed from structs and bodies
                    world primordial_pond { ... }
                    FINE: clear, not gamey in this context
                    ACCURATE: it is the world

run                 defines simulation execution
                    run experiment_1 { ... }
                    CLEAN: what it does
                    FAMILIAR: "run this"
                    PRECISE: not abbreviated

// COMPOSITION KEYWORDS

use                 import another .weave file
                    use "water.weave" as water
                    FAMILIAR: Rust, Python, many languages
                    CLEAN: one word

place               instantiate something at a position
                    place water at (1.0, 2.0)
                    CLEAR: you are placing something
                    SPATIAL: implies physical placement

wire                connect two ports
                    wire sugar.chain_out -> phosphate.chain_in
                    EXACT VERILOG TERMINOLOGY
                    ACCURATE: you are wiring ports together

port                connection point on a struct or body
                    port chain_in: O3
                    EXACT VERILOG TERMINOLOGY

region              a zone in the world with properties
                    region ocean { ... }
                    MORE PRECISE THAN layer
                    SCIENTIFIC: regions not layers

source              an energy source in the world
                    source hydrothermal_vent { ... }
                    CLEAN: what it is

// SEQUENCE KEYWORD (for chains like RNA)

chain               a sequence of repeating structs
                    chain rna_strand { ... }
                    ACCURATE: a molecular chain
                    DISTINCT: clearly different from struct
```

---

## What The Language Looks Like Now

```
--------------------------------------------
elements.weave
--------------------------------------------

weave "0.1" {

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


--------------------------------------------
water.weave
--------------------------------------------

weave "0.1" {

  struct water {

    atoms {
      O1: O  at (0.00,  0.00)
      H1: H  at (-0.96, 0.58)
      H2: H  at ( 0.96, 0.58)
    }

    bonds {
      O1 - H1 : single
      O1 - H2 : single
    }

    port donor:    O1
    port acceptor: H1, H2

  }

}


--------------------------------------------
nucleotide_A.weave
--------------------------------------------

weave "0.1" {

  struct nucleotide_A {

    use "adenine_base.weave"  as base
    use "ribose.weave"        as sugar
    use "phosphate.weave"     as phos

    place base  at (0.0, 0.0)
    place sugar at (4.0, 0.0)
    place phos  at (7.5, 0.0)

    wire base.sugar_attach  -> sugar.base_attach
    wire sugar.phosophate_out -> phos.chain_in

    port chain_in:  phos.upstream
    port chain_out: sugar.three_prime
    port pair:      base.pair_bond

  }

}


--------------------------------------------
rna_strand.weave
--------------------------------------------

weave "0.1" {

  chain rna_strand {

    use "nucleotide_A.weave"  as A
    use "nucleotide_U.weave"  as U
    use "nucleotide_G.weave"  as G
    use "nucleotide_C.weave"  as C

    sequence: A U G C A U G C

    link {
      from: chain_out
      to:   chain_in
      bond: single
    }

    port start: first.chain_in
    port end:   last.chain_out
    port pairs: each.pair

  }

}


--------------------------------------------
minimal_cell.weave
--------------------------------------------

weave "0.1" {

  body minimal_cell {

    use "vesicle.weave"      as membrane
    use "rna_strand.weave"   as genome
    use "nucleotide_A.weave" as free_A
    use "nucleotide_U.weave" as free_U
    use "nucleotide_G.weave" as free_G
    use "nucleotide_C.weave" as free_C
    use "water.weave"        as h2o

    place membrane at (0.0, 0.0)
    place genome   at (0.0, 0.0) inside membrane

    place free_A count 20 inside membrane scatter
    place free_U count 20 inside membrane scatter
    place free_G count 20 inside membrane scatter
    place free_C count 20 inside membrane scatter
    place h2o    count 200 inside membrane scatter

  }

}


--------------------------------------------
primordial_pond.weave
--------------------------------------------

weave "0.1" {

  world primordial_pond {

    size:     200 x 200
    boundary: wrap

    use "minimal_cell.weave"  as cell
    use "water.weave"         as h2o
    use "nucleotide_A.weave"  as free_A
    use "nucleotide_U.weave"  as free_U
    use "lipid.weave"         as lipid

    region surface (y: 150 - 200) {
      temperature: 15
      pressure:    0.8
      uv:          0.7
      place h2o   count 10000 scatter
      place free_A count 200  scatter
    }

    region ocean (y: 50 - 150) {
      temperature: 35
      pressure:    4.0
      uv:          0.1
      place h2o    count 50000 scatter
      place lipid  count 2000  scatter
      place free_A count 500   scatter
      place free_U count 500   scatter
      place free_G count 500   scatter
      place free_C count 500   scatter
      place cell   count 10    scatter
    }

    region seafloor (y: 0 - 50) {
      temperature: 80
      pressure:    20.0
      uv:          0.0
      place Si count 5000 scatter
      place Fe count 2000 scatter
      place S  count 1000 scatter
    }

    source hydrothermal_vent {
      type:      hydrothermal
      position:  (100.0, 0.0)
      intensity: 0.8
      radius:    15.0
    }

    source sunlight {
      type:         solar_uv
      intensity:    0.7
      surface_only: true
    }

  }

}


--------------------------------------------
experiment_1.weave
--------------------------------------------

weave "0.1" {

  run experiment_1 {

    world: "primordial_pond.weave"

    execution {
      tick_rate: max
      max_ticks: unlimited
      seed:      42
    }

    output {
      tick_interval:  1000
      bond_events:    false
      notable_only:   false
    }

    watch {
      molecule_size_above:     50
      population_change_above: 20.0
      extinction:              true
    }

  }

}
```

---

## The Complete Keyword Set

```
FILE LEVEL:
  weave               file declaration
  element             atomic element definition
  struct              molecule/structure definition
  chain               polymer chain definition
  body                organism definition
  world               world/environment definition
  run                 simulation execution definition

COMPOSITION:
  use                 import a .weave file
  place               instantiate in space
  wire                connect two ports
  port                declare a connection point
  link                chain bond rule (inside chain only)
  inside              spatial containment
  scatter             random placement
  count               quantity
  at                  position
  as                  alias

STRUCTURE INTERNALS:
  atoms               atom definitions block
  bonds               bond definitions block
  sequence            chain sequence string
  single              bond order
  double              bond order
  triple              bond order

WORLD INTERNALS:
  region              environmental zone
  source              energy source
  size                world dimensions
  boundary            edge behavior
  wrap / wall / open  boundary types
  temperature         field value
  pressure            field value
  uv                  field value

RUN INTERNALS:
  execution           tick/seed config
  output              stream config
  watch               notable event config
  tick_rate           speed
  max_ticks           duration
  seed                rng seed
  tick_interval       output frequency
  bond_events         output toggle
  notable_only        output toggle
  max / unlimited / random   special values

UNIVERSAL:
  true / false        booleans
```

---

## What This Gives Us

```
CLEAN:
  Seven top-level concepts:
  element struct chain body world run
  That maps to:
  atom  molecule  polymer  organism  environment  execution

FAMILIAR:
  use    -> Rust/Python imports
  struct -> C/Rust data structures
  wire   -> Verilog connections
  port   -> Verilog interface points
  run    -> intuitive execution

UNAMBIGUOUS:
  element  is always an atom type definition
  struct   is always a structural composition
  chain    is always a polymer sequence
  body     is always an organism
  world    is always the environment
  run      is always the execution config

COMPOSABLE:
  struct  uses  struct
  chain   uses  struct
  body    uses  struct
  world   uses  struct and body
  run     uses  world

EXTENSIBLE:
  New top-level keywords can be added
  without breaking existing files
  because the weave version string
  controls what is valid
```

---

## cerne CLI - Clean And Final

```
USAGE:
  cerne <file.weave>              run a simulation
  cerne --check <file.weave>      validate without running
  cerne --test <file.weave>       test a struct or body
  cerne --info <file.weave>       describe contents
  cerne --seed <n> <file.weave>   override seed
  cerne --version                 print version
  cerne --help                    print help

INPUT:   .weave files
OUTPUT:  NDJSON to stdout
ERRORS:  human readable to stderr

cerne experiment_1.weave
cerne --check primordial_pond.weave
cerne --test minimal_cell.weave
cerne --seed 42 experiment_1.weave
cerne --seed 42 experiment_1.weave | cerne-view
cerne --seed 42 experiment_1.weave > run_001.ndjson
```

---

## Summary Of Changes From Previous Spec

```
CHANGED:
  mol       -> struct
  org       -> body
  .elem     -> element (inside .weave)
  layers    -> region
  energy_sources -> source
  import    -> use
  connect   -> wire
  sim       -> run
  chain_bond -> link

KEPT:
  port      (already correct, Verilog term)
  place     (already clear)
  world     (already clear)
  element   (already correct)
  scatter   (already clear)
  bonds     (already correct)
  atoms     (already correct)
  single/double/triple (already correct)
  wrap/wall/open (already correct)

ADDED:
  chain     (new keyword for polymer sequences)
  body      (replaces org, clearer)
  link      (replaces chain_bond, cleaner)
  source    (replaces energy_sources, cleaner)
  region    (replaces layers, more precise)
```

Ready to write the Rust code?

### USER
just like rust files are called .rs and c are called .c then cernes should be call .crn

### ASSISTANT
## Yes. Simple And Right.

```
.c    C source files        compiled by gcc/clang
.rs   Rust source files     compiled by rustc
.v    Verilog source files  compiled by iverilog
.crn  Weave source files    compiled by cerne
```

---

## Updated File Convention

```
BEFORE          AFTER
------------------------------
water.weave     water.crn
elements.weave  elements.crn
minimal_cell.weave  minimal_cell.crn
experiment_1.weave  experiment_1.crn

ALL source files are .crn
The type is declared INSIDE the file
The filename is arbitrary
The extension is always .crn
```

---

## What This Looks Like

```
project/
+-- elements.crn           declares: element
+-- water.crn              declares: struct
+-- phosphate.crn          declares: struct
+-- ribose.crn             declares: struct
+-- adenine_base.crn       declares: struct
+-- nucleotide_A.crn       declares: struct
+-- nucleotide_U.crn       declares: struct
+-- nucleotide_G.crn       declares: struct
+-- nucleotide_C.crn       declares: struct
+-- lipid.crn              declares: struct
+-- rna_strand.crn         declares: chain
+-- vesicle.crn            declares: struct
+-- minimal_cell.crn       declares: body
+-- primordial_pond.crn    declares: world
+-- experiment_1.crn       declares: run
```

---

## CLI stays exactly the same, just .crn files

```
cerne experiment_1.crn
cerne --check primordial_pond.crn
cerne --test minimal_cell.crn
cerne --info nucleotide_A.crn
cerne --seed 42 experiment_1.crn

cerne experiment_1.crn > run_001.ndjson
cerne experiment_1.crn | cerne-view
cerne experiment_1.crn 2>errors.log | cerne-view
```

---

## The Complete Picture Now

```
LANGUAGE:   Weave
EXTENSION:  .crn
RUNTIME:    cerne
INPUT:      .crn files
OUTPUT:     NDJSON -> stdout
ERRORS:     human readable -> stderr

cerne reads .crn
exactly as
gcc reads .c
rustc reads .rs
iverilog reads .v
```

Clean. Consistent. Done.

**Ready to write the Rust code now?**

### USER
it shouldnt be called weave it should but be called cerne files, like its c files, or verilog files, or rust files, etc... is this a correct understanding? shouldnt our system follow this pattern?

### ASSISTANT
## Yes. Exactly Right.

```
LANGUAGE    FILES           COMPILER/RUNTIME
---------------------------------------------
C           C files         gcc
Rust        Rust files      rustc
Verilog     Verilog files   iverilog
Cerne       Cerne files     cerne
```

---

## The Pattern Is Simple

```
C:
  The language is called C
  The files are called C files
  The files end in .c
  The compiler is called gcc (separate tool name)

Rust:
  The language is called Rust
  The files are called Rust files
  The files end in .rs
  The compiler is called rustc

Verilog:
  The language is called Verilog
  The files are called Verilog files
  The files end in .v
  The simulator is called iverilog/ModelSim

Ours:
  The language is called Cerne
  The files are called Cerne files
  The files end in .crn
  The runtime is called cerne
```

---

## So The DSL Has No Separate Name

```
BEFORE (wrong):
  Language:  Weave
  Files:     Weave files / .crn files
  Runtime:   cerne

AFTER (correct):
  Language:  Cerne
  Files:     Cerne files / .crn files
  Runtime:   cerne

Just like nobody says
  "I am writing a GCC file"
  they say
  "I am writing a C file"

Nobody says
  "I am writing a Weave file"
  they say
  "I am writing a Cerne file"
```

---

## Everything In One Clean Statement

```
Cerne is a biological physics simulation language.
Cerne files describe atoms, structures, and worlds.
Cerne files use the .crn extension.
The cerne runtime reads .crn files and simulates them.

cerne experiment.crn
```

---

## The Full Picture, Final

```
LANGUAGE:    Cerne
EXTENSION:   .crn
RUNTIME:     cerne (the executable)
INPUT:       .crn files
OUTPUT:      NDJSON -> stdout
ERRORS:      text -> stderr

project/
+-- elements.crn
+-- water.crn
+-- nucleotide_A.crn
+-- rna_strand.crn
+-- minimal_cell.crn
+-- primordial_pond.crn
+-- experiment_1.crn

cerne experiment_1.crn
cerne --check minimal_cell.crn
cerne --test minimal_cell.crn
cerne --seed 42 experiment_1.crn
```

---

Clean, consistent, correct.

**Now should we write the formal language spec one final time with all of this in place, and then start the Rust code?**