//! khem - the runtime CLI for the khem artificial-chemistry language.
//!
//! Phase 1: the hardcoded primordial pond (PLAN.md). A file
//! argument is accepted but not read - the .kem parser arrives in
//! phase 3 via the khem-lang crate; until then the only knobs are
//! --seed and the constants in khem-core's config. The binary stays
//! thin forever: parse arguments, construct a world, run, stream
//! (see ARCHITECTURE.md).
//!
//! Streams (runtime spec 2.4, guarantees G10/G11): stdout carries
//! only NDJSON events, stderr only human diagnostics. stdout is
//! flushed per tick (spec 9.4) so pipe consumers receive data
//! promptly. SIGINT handling (exit 3, END with user_interrupt)
//! arrives with phase 2 hardening; in phase 1 ctrl-c simply kills
//! the process.
//!
//! Argument parsing is hand-rolled std-only for now: phase 1 needs
//! nothing beyond --seed, and keeping the engine dependency-free
//! matters more than CLI ergonomics. Phase 3 CLI growth
//! (--check/--test/--info) may adopt a parser crate; that decision
//! gets an ADR when it is made.

use std::io::Write;
use std::process::ExitCode;

use khem_core::ndjson;
use khem_core::observer::{Observer, ObserverConfig};
use khem_core::{PhysicsConfig, Sim, pond};

const USAGE: &str = "\
usage: khem [OPTIONS] [<file.kem>]

phase 1: runs the hardcoded primordial pond; a file argument is
accepted but ignored (the .kem parser arrives in phase 3)

options:
  --seed <N>   set the run seed (default 42)
  --version    print version and exit
  --help       print this help and exit

exit codes (runtime spec section 2.3):
  0  success
  1  validation error (bad .kem files)
  2  runtime error
  3  user interrupt";

/// Phase-1 run length: hardcoded (no run declaration to read yet).
const MAX_TICKS: u64 = 1_000_000;
/// Tick event interval; the language-spec run example uses 1000.
const TICK_INTERVAL: u64 = 1000;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seed: Option<u64> = None;
    let mut path: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--version" => {
                println!("khem {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--seed" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("khem: --seed requires a value");
                    eprintln!("{USAGE}");
                    return ExitCode::from(1);
                };
                match value.parse::<u64>() {
                    Ok(n) => seed = Some(n),
                    Err(_) => {
                        eprintln!("khem: --seed expects an integer, got {value:?}");
                        return ExitCode::from(1);
                    }
                }
                i += 1;
            }
            arg if arg.starts_with('-') => {
                eprintln!("khem: unknown option {arg:?}");
                eprintln!("{USAGE}");
                return ExitCode::from(1);
            }
            arg => path = Some(arg),
        }
        i += 1;
    }

    run(path, seed)
}

/// Runs the hardcoded primordial pond and streams NDJSON to stdout
/// (runtime spec sections 2, 3, 5).
fn run(path: Option<&str>, seed: Option<u64>) -> ExitCode {
    if let Some(path) = path {
        eprintln!(
            "khem: phase 1: {path:?} not read - parser arrives in phase 3; \
             running the hardcoded primordial pond"
        );
    }
    let seed = seed.unwrap_or(42);
    let config = PhysicsConfig::default();
    let mut world = pond::primordial_pond(seed, config);
    let observer = Observer::new(ObserverConfig {
        khem_version: env!("CARGO_PKG_VERSION"),
        run_name: "primordial_pond".to_string(),
        world_name: "primordial_pond".to_string(),
        seed,
        tick_interval: TICK_INTERVAL,
    });
    let mut sim = Sim::new(config, observer);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let start = sim.start(&world);
    if write_event(&mut out, &start).is_err() {
        return ExitCode::from(2);
    }
    for _ in 0..MAX_TICKS {
        let events = sim.tick(&mut world);
        let wrote = events
            .iter()
            .try_for_each(|event| write_event(&mut out, event));
        if wrote.is_err() || out.flush().is_err() {
            // A closed pipe (e.g. `khem ... | head`) surfaces here;
            // phase-1 behavior: exit 2 with a stderr note (spec 2.3
            // has no dedicated code; revisited with phase 2).
            eprintln!("khem: stdout write failed; stopping");
            return ExitCode::from(2);
        }
    }
    let end = sim.end(&world);
    if write_event(&mut out, &end).is_err() {
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn write_event(out: &mut impl Write, event: &khem_core::Event) -> std::io::Result<()> {
    writeln!(out, "{}", ndjson::emit(event))
}
