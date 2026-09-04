//! khem - the runtime CLI for the khem artificial-chemistry language.
//!
//! Scaffold: the CLI surface from runtime spec section 2 exists
//! (usage, exit codes, stdout/stderr contract, --seed), but `run` is
//! not implemented - phase 1 replaces its body with a call into
//! khem-core: build the hardcoded primordial pond, run the tick
//! loop, stream NDJSON events. Loading .kem definitions arrives in
//! phase 3 via the khem-lang crate.
//!
//! The binary stays thin forever: parse arguments, construct a
//! world, run, stream (see ARCHITECTURE.md).
//!
//! Argument parsing is hand-rolled std-only for now: phase 1 needs
//! nothing beyond --seed, and keeping the engine dependency-free
//! matters more than CLI ergonomics. Phase 3 CLI growth
//! (--check/--test/--info) may adopt a parser crate; that decision
//! gets an ADR when it is made.

use std::process::ExitCode;

const USAGE: &str = "\
usage: khem [OPTIONS] <file.kem>

options:
  --seed <N>   override the seed from the run declaration
  --version    print version and exit
  --help       print this help and exit

exit codes (runtime spec section 2.3):
  0  success
  1  validation error (bad .kem files)
  2  runtime error
  3  user interrupt";

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

    match path {
        Some(path) => run(path, seed),
        None => {
            eprintln!("{USAGE}");
            ExitCode::from(1)
        }
    }
}

/// Runs a simulation. Phase 1 wires this into khem-core (see PLAN.md).
fn run(path: &str, seed: Option<u64>) -> ExitCode {
    eprintln!("khem: runtime not implemented yet (got {path:?}, seed {seed:?})");
    eprintln!("khem: phase 1 wires run() into khem-core - see PLAN.md");
    ExitCode::from(2)
}
