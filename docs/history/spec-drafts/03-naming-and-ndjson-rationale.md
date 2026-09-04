<!--
  EXTRACTED FROM: initial-idea.md (founding conversation transcript)
  SOURCE LINES: 4439-4887
  WHAT: naming rationale and decision (runtime = cerne, from Latin cernere,
        to sift/discern), reporting philosophy (NDJSON to stdout, errors to
        stderr), tool ecosystem (cerne-view etc.), NDJSON event draft spec
  STATUS: reporting philosophy and tool design still stand; the language
        name "Weave" proposed here was later replaced by "Cerne"
        (see 06-terminology-and-final-naming.md)
  NOTE: the runtime name "cerne" adopted in this round was superseded
        at project level by "khem" on 2026-09-04; see README.md here.
-->

## Naming First

Let me think about this seriously. The name should reflect what this actually is.

---

## What This Actually Is

```
It is a physics engine that accepts
structural definitions and simulates
fundamental rules.

It is not a game.
It is not a toy.
It is a scientific instrument
that happens to run on a laptop.

The DSL is to this runtime
what C is to GCC.
What Verilog is to ModelSim.
What SPICE netlists are to SPICE.

It deserves a name that reflects that.
```

---

## Name Candidates

```
RUNTIME NAME OPTIONS:
-----------------------------------------------------

primordial
  "primordial world.sim"
  Good: evocative, accurate, simple
  Bad: sounds like a game

abios          (Greek: "life" root)
  "abios world.sim"
  Good: short, scientific root
  Bad: obscure

substrate
  "substrate world.sim"
  Good: accurate - the runtime IS the substrate
  Bad: generic, used everywhere in CS

hadron         (already taken by physics)

cellsim        (too specific - implies cells)

primum         (Latin: "first thing")
  "primum world.sim"
  Good: accurate, scientific feel
  Bad: might be confused with other things

stroma         (Greek: foundation/substrate in biology)
  "stroma world.sim"
  Good: real biology term, means the substrate
        that life builds on
  Bad: very specific existing biology meaning

axion          (taken by physics)

forma          (Latin: form, structure)
  "forma world.sim"
  Good: clean, short, accurate
  Bad: generic

SOMA           (Greek: body)
  "soma world.sim"
  Good: short, biological
  Bad: taken (Huxley novel, video game)

AEON           
  "aeon world.sim"
  Good: implies long time scales, emergence
  Bad: no scientific meaning

VERA           (Latin: truth)
  "vera world.sim"
  Good: implies faithfulness to real rules
  Bad: not obviously scientific

NUCLUS         
  Bad: too close to nucleus

STRATUM        (Latin: layer)
  "stratum world.sim"
  Good: accurate, implies layers of complexity
        building on each other
  Bad: a little long

FUNDAMENT
  too on the nose

LOCUS          (Latin: place, position)
  "locus world.sim"
  Good: used in genetics (locus = position on genome)
        short, real scientific term
  Bad: already heavily used in genetics

PRIMUS         (Latin: first, fundamental)
  "primus world.sim"
  Good: clean, implies fundamental
  Bad: propane brand

CERNE          (from discern, from Latin cernere: to sift)
  "cerne world.sim"
  Good: implies finding/discovering things
        from fundamental sifting
        short, unique
  Bad: CERN association (not bad actually)
```

---

## My Recommendation

```
THE RUNTIME:

  cerne

  Pronounced: "sern"
  From Latin cernere: to sift, to separate, to discern
  
  Why it fits:
    The runtime sifts through fundamental rules
    and from that sifting, structure emerges.
    
    Also: to discern = to find meaning in complexity.
    That is what this tool does.
    
    Also: CERN association is not bad.
    CERN looks for fundamental things too.
    
    Also: short, memorable, typeable, unique,
    no conflicts in package registries.

  Usage:
    cerne world.sim
    cerne --validate world.sim
    cerne --test minimal_cell.org
    cerne --seed 42 experiment_1.sim

THE DSL:

  BDL is fine as the spec name but
  the file format family needs a name too.

  Options:
    .bdl extension for all files?  No - we have good specific extensions
    The language name:

  FORM          (structures define form)
  WEAVE         (weaving atoms into structure)
  STRATA        (layered definitions)
  LATTICE       (atomic lattice)
  
  LATTICE is good but generic.

  THREAD        
    You write thread files.
    You thread atoms together.
    Threads compose into larger threads.
    "cerne world.thread"
    Interesting but confusing with CPU threads.

  LACE
    You lace atoms together.
    Lace has structure and pattern.
    Lace composes.
    "cerne world.lace"
    Actually interesting.

  WEAVE
    You weave atoms into molecules.
    Molecules weave into structures.
    Structures weave into life.
    "cerne world.weave"
    Good metaphor.

  SCRIPT? SCHEMA? Too generic.

  CONSTRUCT
    You write constructs.
    "cerne world.con"
    Too close to config.

  My recommendation: WEAVE
    The language is called Weave.
    Files are .weave files.
    Or keep specific extensions but
    call the language Weave.

    "cerne reads Weave definitions"
    
    But actually the specific extensions
    are better than one .weave extension
    because they communicate intent.
    So: language = Weave, extensions stay specific.
```

---

## Final Naming Decision

```
RUNTIME EXECUTABLE:
  cerne
  
  cerne simulation.sim           # run a simulation
  cerne --validate world.world   # validate without running
  cerne --test cell.org          # test a template in isolation
  cerne --info molecule.mol      # describe a template
  cerne --seed 42 sim.sim        # reproducible run
  cerne --version
  cerne --help

THE LANGUAGE:
  Weave  (or WDL: Weave Definition Language)
  
FILE EXTENSIONS:
  .elem    element definitions
  .mol     molecule/structure template
  .org     organism template
  .world   world definition
  .sim     simulation run config
  .state   saved simulation snapshot (binary)

PROJECT STRUCTURE NAME:
  A collection of files for one experiment = a Weave project
  Shared templates = the standard library = stdlib/

EXAMPLE USAGE:
  cerne experiment_1.sim
  
  Output to stdout:  simulation tick data, stats, events
  Output to stderr:  errors, warnings, validation failures
  
  Pipe it:
  cerne experiment_1.sim | grep "NOTABLE"
  cerne experiment_1.sim > run_001.log
  cerne experiment_1.sim | my_analysis_tool
```

---

## The Reporting Problem - Solved Properly

You are right that reporting should not be in the runtime. Here is the clean solution.

```
THE RUNTIME OUTPUTS ONE THING: structured data to stdout

Not human readable output.
Not a pretty terminal UI.
Just a stream of structured events.

Like how a C compiler outputs object files.
The runtime outputs a stream.
What you do with that stream is up to you.

FORMAT: newline-delimited JSON (NDJSON)
  One JSON object per line.
  stdout = simulation event stream.
  stderr = errors and warnings only.

Every tick (or every N ticks) cerne outputs:

{"type":"tick","tick":1247891,"t_min":12.3,"t_max":847.2,"t_avg":34.1,...}
{"type":"bond_formed","tick":1247891,"atom_a":442,"atom_b":891,"order":1,...}
{"type":"bond_broken","tick":1247892,"bond_id":234,"energy_released":436.0}
{"type":"molecule","tick":1247900,"size":47,"atom_ids":[1,4,7,22,...],...}
{"type":"notable","tick":1247900,"event":"largest_molecule","size":47}

Then you pipe it to whatever you want:

  cerne sim.sim | cerne-view          # the standard terminal UI (separate tool)
  cerne sim.sim | cerne-log           # write structured log files
  cerne sim.sim | python my_viz.py    # your own visualization
  cerne sim.sim | grep notable        # just the interesting events
  cerne sim.sim > raw.ndjson          # save everything, analyze later
  cerne sim.sim | jq '.type=="tick"'  # filter with standard tools
```

---

## The Tool Ecosystem

```
CORE:
  cerne           the runtime - outputs NDJSON stream

STANDARD TOOLS (separate programs, separate repos):
  cerne-view      terminal UI (reads cerne stdout)
  cerne-log       structured logging and replay
  cerne-check     deep validation of Weave files
  cerne-build     template composition helper
  cerne-test      test a template in isolation

USER TOOLS (anyone can write these):
  anything that reads NDJSON from stdin

This is the Unix philosophy.
cerne does one thing.
cerne does it well.
Everything composes.

EXAMPLES:

  # Watch it in terminal
  cerne experiment.sim | cerne-view

  # Save and watch at same time
  cerne experiment.sim | tee run.ndjson | cerne-view

  # Run headless, analyze after
  cerne experiment.sim > run.ndjson
  cat run.ndjson | cerne-view --replay

  # Just watch for notable events
  cerne experiment.sim | grep '"type":"notable"'

  # Your own Python analysis
  cerne experiment.sim | python analyze.py

  # Reproducible experiment with seed
  cerne --seed 42 experiment.sim | cerne-view
```

---

## The NDJSON Output Spec

```
Every line is valid JSON.
Every object has at minimum:
  "type": string
  "tick": integer

EVENT TYPES:

  tick
    Emitted every sample_interval ticks
    {"type":"tick",
     "tick":1000,
     "elapsed_ms":124,
     "ticks_per_sec":8064,
     "atom_count":4821,
     "bond_count":2341,
     "temp_min":12.3,
     "temp_max":847.2,
     "temp_avg":34.1,
     "pressure_min":0.8,
     "pressure_max":20.1,
     "pressure_avg":4.2,
     "free_atoms":{"H":892,"C":234,"O":445},
     "molecule_size_dist":{"1":892,"2_5":445,"6_20":89,"21+":12}}

  bond_formed
    {"type":"bond_formed",
     "tick":1000,
     "bond_id":4521,
     "atom_a":442,
     "atom_b":891,
     "element_a":"C",
     "element_b":"O",
     "order":2,
     "energy":799.0,
     "x":45.2,
     "y":123.7}

  bond_broken
    {"type":"bond_broken",
     "tick":1001,
     "bond_id":4521,
     "energy_released":399.5,
     "x":45.3,
     "y":123.8}

  notable
    {"type":"notable",
     "tick":1247900,
     "event":"largest_molecule",
     "detail":"47 atoms",
     "molecule_id":8821}

  error
    Goes to stderr, not stdout
    {"type":"error","code":"V01","message":"...","file":"...","line":14}

  warning
    Goes to stderr
    {"type":"warning","code":"W01","message":"..."}

  start
    First line always emitted
    {"type":"start",
     "tick":0,
     "cerne_version":"0.1.0",
     "sim_file":"experiment.sim",
     "world_file":"world.world",
     "atom_count":4821,
     "seed":42}

  end
    Last line if simulation ends naturally
    {"type":"end",
     "tick":5000000,
     "elapsed_ms":62000,
     "reason":"max_ticks_reached"}
```

---

## Summary

```
NAME:         cerne
LANGUAGE:     Weave
EXTENSIONS:   .elem .mol .org .world .sim .state

RUNTIME:
  Input:   .sim file (entry point to Weave definitions)
  Output:  NDJSON stream to stdout
  Errors:  to stderr
  Nothing else.

REPORTING:
  Not in runtime.
  User pipes stdout to whatever they want.
  Standard tool: cerne-view for terminal UI.
  Everything else is user defined.

PHILOSOPHY:
  Do one thing well.
  Compose with other tools.
  Reproducible by default (--seed).
  Auditable (NDJSON is human readable).
  Scriptable (standard Unix pipes).
```

---

Ready to write the formal spec updates with these names and then start the Rust code?
