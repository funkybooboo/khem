//! khem - the runtime CLI for the khem artificial-chemistry language.
//!
//! Phase 1: the hardcoded primordial pond
//! (docs/plans/phase-1-kernel.md). A file
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
  1  validation error (bad .kem files or bad command line)
  2  runtime error
  3  user interrupt";

/// Phase-1 run length: hardcoded (no run declaration to read yet).
const MAX_TICKS: u64 = 1_000_000;
/// Tick event interval; the language-spec run example uses 1000.
const TICK_INTERVAL: u64 = 1000;

/// What the command line asks for.
#[derive(Debug, PartialEq)]
enum Invocation {
    /// Run the pond: an optional .kem path (accepted but not read
    /// until the phase-3 parser) and an optional seed.
    Run {
        path: Option<String>,
        seed: Option<u64>,
    },
    Help,
    Version,
}

/// Parses arguments. Err carries the usage-error message for
/// stderr; extractable and tested so the CLI surface cannot
/// regress silently.
fn parse_args(args: &[String]) -> Result<Invocation, String> {
    let mut path = None;
    let mut seed = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => return Ok(Invocation::Help),
            "--version" => return Ok(Invocation::Version),
            "--seed" => {
                let Some(value) = args.get(i + 1) else {
                    return Err("--seed requires a value".into());
                };
                match value.parse::<u64>() {
                    Ok(n) => seed = Some(n),
                    Err(_) => {
                        return Err(format!("--seed expects an integer, got {value:?}"));
                    }
                }
                i += 1;
            }
            arg if arg.starts_with('-') => {
                return Err(format!("unknown option {arg:?}"));
            }
            arg => path = Some(arg.to_string()),
        }
        i += 1;
    }
    Ok(Invocation::Run { path, seed })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = match parse_args(&args) {
        Ok(invocation) => invocation,
        Err(message) => {
            eprintln!("khem: {message}");
            eprintln!("{USAGE}");
            return ExitCode::from(1);
        }
    };
    match invocation {
        Invocation::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Invocation::Version => {
            println!("khem {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Invocation::Run { path, seed } => run(path.as_deref(), seed),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_arguments_runs_the_default_pond() {
        assert_eq!(
            parse_args(&args(&[])),
            Ok(Invocation::Run {
                path: None,
                seed: None
            })
        );
    }

    #[test]
    fn parses_seed_and_path() {
        assert_eq!(
            parse_args(&args(&["--seed", "7", "world.kem"])),
            Ok(Invocation::Run {
                path: Some("world.kem".into()),
                seed: Some(7)
            })
        );
        // Order is free; the last positional wins (same as v0.1's
        // original inline loop).
        assert_eq!(
            parse_args(&args(&["a.kem", "--seed", "1", "b.kem"])),
            Ok(Invocation::Run {
                path: Some("b.kem".into()),
                seed: Some(1)
            })
        );
    }

    #[test]
    fn help_and_version_short_circuit() {
        assert_eq!(parse_args(&args(&["--help"])), Ok(Invocation::Help));
        assert_eq!(parse_args(&args(&["--version"])), Ok(Invocation::Version));
    }

    #[test]
    fn rejects_bad_arguments() {
        assert_eq!(
            parse_args(&args(&["--seed"])),
            Err("--seed requires a value".into())
        );
        assert_eq!(
            parse_args(&args(&["--seed", "--help"])),
            Err("--seed expects an integer, got \"--help\"".into())
        );
        assert_eq!(
            parse_args(&args(&["--wat"])),
            Err("unknown option \"--wat\"".into())
        );
    }
}
