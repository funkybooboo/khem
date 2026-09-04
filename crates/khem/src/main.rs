//! khem - the runtime CLI for the khem artificial-chemistry language.
//!
//! Scaffold only: proves the toolchain and workspace wiring. Phase 1
//! (PLAN.md) replaces the body with a call into khem-core: build the
//! hardcoded primordial pond, run the tick loop, stream NDJSON
//! events. Loading .kem definitions arrives in phase 3 via the
//! khem-lang crate.
//!
//! The binary stays thin forever: parse args, construct a world, run,
//! stream (see ARCHITECTURE.md).

fn main() {
    println!("khem 0.0.1 (scaffold)");
    println!("artificial-chemistry runtime - not implemented yet");
    println!("see PLAN.md: kernel first, language later");
}