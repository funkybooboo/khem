# khem research references

Papers and projects that ground khem's design choices, organized by
the design element they support. DOIs marked [verify] were not
confirmed against publisher records and must be resolved before
thesis-grade citation; all others were verified 2026-09-04. Software
links were checked 2026-09-04.

## Artificial chemistries - the substrate philosophy

- Dittrich, P., Ziegler, J., Banzhaf, W. (2001). Artificial
  Chemistries - A Review. Artificial Life 7(3):225-275.
  doi:10.1162/106454601753238636
  The survey; the taxonomy this project should speak in.
  Free PDF: https://www.cs.mun.ca/~banzhaf/papers/alchemistry_review_MIT.pdf
- SimSoup - artificial chemistry simulator for origin-of-life and
  metabolic-network evolution (Chris Gordon-Smith).
  https://www.simsoup.info/
  Papers: "An Artificial Chemistry Model for Investigation of the
  Evolution of Metabolic Networks"; "Artificial Chemistry Meets
  Pauling" (structure-driven molecule properties - khem's philosophy
  at molecule-type level).
- Stringmol - automata chemistry for molecular evolution (York).
  https://stringmol.york.ac.uk/
  Code: https://github.com/franticspider/stringmol
- Fontana, W. (1992). Algorithmic Chemistry. In Langton et al.
  (eds.), Artificial Life II, Addison-Wesley. [verify]
- Bedau, M. et al. (2000). Open Problems in Artificial Life.
  Artificial Life 6(4):363-376. [verify]

## RNA world and the minimal cell - what we seed

- Gilbert, W. (1986). The RNA World. Nature 319:618.
  doi:10.1038/319618a0
- Joyce, G.F. (1989). RNA evolution and the origins of life.
  Nature 338:217-224. doi:10.1038/338217a0
- Joyce, G.F. (2002). The Antiquity of RNA-based Evolution.
  Nature 418:214-221. doi:10.1038/418214a
- Szostak, J.W., Bartel, D.P., Luisi, P.L. (2001). Synthesizing Life.
  Nature 409:387-390. doi:10.1038/35053176
  The protocell program: replicator + compartment + growth + division.
- Chen, I.A., Roberts, R.W., Szostak, J.W. (2004). The Emergence of
  Competition Between Model Protocells. Science.
  doi:10.1126/science.1100757
  Osmotic effects couple genome and membrane - the physics khem's
  division-by-tension rule wants to echo.
- Powner, M.W., Gerland, B., Sutherland, J.D. (2009). Synthesis of
  activated pyrimidine ribonucleotides in prebiotically plausible
  conditions. Nature 459:239-242. doi:10.1038/nature08013
- Segre, D., Ben-Eli, D., Deamer, D., Lancet, D. (2001). The Lipid
  World. Origins of Life and Evolution of the Biosphere 31:119-145.
  doi:10.1023/A:1006746807104
- Ganti, T. (2003). The Principles of Life. Oxford University Press.
  [book] The chemoton: container + metabolism + information - the
  theoretical minimal cell the khem seed approximates.
- Woese, C.R. (1998). The Universal Ancestor. PNAS 95(12):6854-6859.
  doi:10.1073/pnas.95.12.6854 [verify]

## Origin-of-life context - the warm little pond

- Oparin, A. (1938). The Origin of Life. Macmillan. [book]
- Haldane, J.B.S. (1929). The Origin of Life. The Rationalist Annual.
- Miller, S.L. (1953). A Production of Amino Acids Under Possible
  Primitive Earth Conditions. Science 117:528-529.
  doi:10.1126/science.117.3046.528
- Darwin's "warm little pond": letter to Joseph Hooker, 1871
  (Darwin Correspondence Project). The primordial_pond world file is
  the homage.

## Self-organization and evolution theory - what we expect to emerge

- Turing, A.M. (1952). The Chemical Basis of Morphogenesis.
  Phil. Trans. R. Soc. B 237:37-72. doi:10.1098/rstb.1952.0012
  Structure from reaction + diffusion; the ancestor of field-driven
  simulation.
- Eigen, M., Schuster, P. (1977). The Hypercycle, Part A.
  Naturwissenschaften 64:541-565. doi:10.1007/BF00450633
- Kauffman, S.A. (1986). Autocatalytic Sets of Proteins.
  J. Theor. Biol. 119:1-24. [verify]
- Kauffman, S.A. (1993). The Origins of Order. Oxford University
  Press. [book]
- Maynard Smith, J., Szathmary, E. (1995). The Major Transitions in
  Evolution. Oxford University Press. [book] [verify]

## Rule-based modeling languages - the DSL prior art

- Kappa - rule-based language for interacting agents with sites.
  https://kappalanguage.org/
  Tools: https://github.com/Kappa-Dev/KaSim
  The closest existing "DSL for chemistry"; compare the .kem grammar
  against it line by line before freezing it.
- BioNetGen - rule-based biochemical network modeling.
  https://github.com/RuleWorld/bionetgen [verify]
- SBML - Hucka, M. et al. (2003). The Systems Biology Markup
  Language. Bioinformatics 19(4):524-531. [verify] https://sbml.org
- MCell and its MDL - particle-based spatial stochastic biochemistry.
  https://mcell.org/
  MCell4 with BioNetGen: A Monte Carlo Simulator of Rule-Based
  Reaction-Diffusion Systems with Python Interface. PLOS Comput.
  Biol. (2024). doi:10.1371/journal.pcbi.1011800 [verify authors]

## Reactive and coarse-grained MD - the fidelity ceiling

- van Duin, A.C.T. et al. (2001). ReaxFF: A Reactive Force Field for
  Hydrocarbons. J. Phys. Chem. A 105:9396-9409. [verify]
- Senftle, T.P. et al. (2021). The ReaxFF reactive force-field:
  development, applications and future directions. npj Computational
  Materials. [verify]
- Marrink, S.J. et al. (2007). The MARTINI Force Field.
  J. Phys. Chem. B 111:7812-7824. [verify]
- Groot, R.D., Warren, P.B. (1997). Dissipative particle dynamics.
  J. Chem. Phys. 107:4423-4435. [verify]
- LAMMPS reaxff pair style:
  https://docs.lammps.org/pair_style_reaxff.html

These define the "too true, too slow" ceiling: real reactive MD covers
nanoseconds for 10^5-10^6 atoms; evolution needs many orders of
magnitude more. khem trades fidelity for reachable timescales - that
trade IS the project.

### Integration-scheme literature (feeds the tuning commit)

- Jackson, A. ( undergrad experiment writeup, UCL): Finite-Difference
  Simulation of a One-Dimensional Harmonic Oscillator - explicit
  Euler "far too inaccurate and unstable for serious use" next to
  velocity-Verlet and Gear predictor-corrector.
  https://anjackson.net/page/files/y3-pm1-fd-ho.pdf
- Skeel, R.D. et al. (1997). Common Molecular Dynamics Algorithms
  Revisited: Accuracy and Optimal Time Steps of Stormer-Leapfrog
  Integrators. J. Comput. Phys. 142:109-130. doi:10.1006/jcph.1997.5740
- Skeel, R.D. (1998). Stability of molecular dynamics simulations of
  classical systems. J. Chem. Phys. 109:3435. doi:10.1063/1.4768891
  (shadow Hamiltonian: why symplectic integrators do not drift.)

### Coarse-grained lipid self-assembly (the non-bonded requirement)

- Huang, J. et al. (2012) and follow-ups: four-bead flexible lipid
  model, MD lipids + multiparticle-collision solvent;
  doi:10.1063/1.4736414 surveys the family.
- Marrink group solvent-free CG POPC model: bonded + nonbonded +
  effective hydrophobic cohesion. J. Phys. Chem. B
  114:doi:10.1021/jp102543j
- Arnarez, D. et al. soft solvent-free lipid model (weighted-density
  functional nonbonded terms). arXiv:1002.2072
- Steltenkamp, S. et al. generic monolayer/bilayer model, "phantom
  solvent". arXiv:physics/0608226

Lesson recorded in abstraction-notes.md section 4: every model that
self-assembles amphiphiles uses explicit non-bonded potentials; khem's
spring-only substrate cannot pass K2 without a documented addition.

## Digital evolution platforms - the evolution prior art

- Hutton, T.J. (2002). Evolvable Self-Replicating Molecules in an
  Artificial Chemistry. Artificial Life 8(4):361-376.
  doi:10.1162/106454602321202417 (Squirm3: emergence, dirtiness,
  resource depletion, flood selection - the K3-K5 dress rehearsal.)
- Hutton, T.J. (2007). Evolvable Self-Reproducing Cells in a
  Two-Dimensional Artificial Chemistry. Artificial Life 13(1):11-30.
  doi:10.1162/artl.2007.13.1.11
  https://github.com/timhutton/squirm3
- Smith, A., Turney, P., Ewaschuk, R. (2002). JohnnyVon:
  Self-replicating automata in continuous two-dimensional space.
  NRC Technical Report ERB-1099 (continuous space at evolutionary
  timescales; the perf cautionary tale.)

- Ofria, C., Wilke, C.O. (2004). Avida: A Software Platform for
  Research in Computational Evolutionary Biology. Artificial Life
  10(2):191-229. doi:10.1162/106454604773563612
  https://avida.devosoft.org/
  Code: https://github.com/devosoft/avida
- Ray, T.S. (1991). An Approach to the Synthesis of Life.
  In Langton et al. (eds.), Artificial Life II, Addison-Wesley.
  (Tierra, Avida's ancestor.) [verify]

## Methodological guardrails - how not to fool yourself

- The Genesis Engine correction notice - a protocell simulation study
  that published a 100% headline result, then withdrew it after an
  internal audit found the detector could not report anything else.
  https://github.com/AVADSA25/genesis-engine
  (see paper/v6_supplement/) Required reading before designing any
  watch-condition detector.
- Nosek, B.A. et al. (2018). The preregistration revolution.
  PNAS 115(11):2600-2606. [verify]
- Bedau et al. (2000), Open Problems in Artificial Life (above),
  doubles as a guardrail list: know which open problem you are
  attacking.

## Similar-but-different software

See README.md ("Relationship to prior work") for the comparison table
covering Kappa, BioNetGen, SBML, MCell, LAMMPS/ReaxFF, SimSoup,
Stringmol, Avida, The Bibites, Ribossome, Primordial, Genesis Engine,
the protocell research sims, and the Conway/Golly lineage.