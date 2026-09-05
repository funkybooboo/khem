//! NDJSON serialization of events: one JSON object per line, no
//! wrapping array (runtime spec 3.1), schema v:1 (3.3).
//!
//! Hand-rolled deliberately (decision 2026-09-05): the event set is
//! small and fixed, the workspace stays zero-registry-dependency,
//! and G02 stays fully under this crate's control. Rust's float
//! formatting is shortest-round-trip and implemented in pure Rust
//! (no libm), so it is byte-stable across platforms; integral floats
//! print without a trailing ".0" (200, not 200.0) - valid JSON,
//! parseable by every consumer, deterministic.
//!
//! Non-finite floats cannot exist in JSON; they serialize as 0 and
//! should never occur (guarded here so a NaN can never corrupt the
//! stream even if a kernel bug produces one).
//!
//! Key order matches the spec 3.3 examples exactly; consumers must
//! not depend on it, but byte-stable output makes diffs and golden
//! tests meaningful.

use crate::elements::element;
use crate::observer::{Event, WorldStats};

/// Serializes one event to one NDJSON line (no trailing newline;
/// the writer adds it).
pub fn emit(event: &Event) -> String {
    let mut out = String::new();
    match event {
        Event::Start {
            khem_version,
            run_name,
            world_name,
            seed,
            atom_count,
            bond_count,
            world_width,
            world_height,
        } => {
            out.push_str("{\"v\":1,\"type\":\"start\",\"tick\":0");
            out.push_str(&format!(",\"khem_version\":\"{khem_version}\""));
            out.push_str(&format!(",\"run_name\":\"{}\"", escape(run_name)));
            out.push_str(&format!(",\"world_name\":\"{}\"", escape(world_name)));
            out.push_str(&format!(",\"seed\":{seed}"));
            out.push_str(&format!(",\"atom_count\":{atom_count}"));
            out.push_str(&format!(",\"bond_count\":{bond_count}"));
            out.push_str(&format!(",\"world_width\":{}", num(*world_width)));
            out.push_str(&format!(",\"world_height\":{}", num(*world_height)));
            out.push('}');
        }
        Event::Tick {
            tick,
            timing,
            stats,
        } => {
            out.push_str("{\"v\":1,\"type\":\"tick\"");
            out.push_str(&format!(",\"tick\":{tick}"));
            out.push_str(&format!(",\"elapsed_ms\":{}", timing.elapsed_ms));
            out.push_str(&format!(
                ",\"ticks_per_sec\":{}",
                num64(timing.ticks_per_sec)
            ));
            out.push_str(&format!(",\"atom_count\":{}", stats.atom_count));
            out.push_str(&format!(",\"bond_count\":{}", stats.bond_count));
            out.push_str(&format!(
                ",\"temp_min\":{},\"temp_max\":{},\"temp_avg\":{}",
                num(stats.temp_min),
                num(stats.temp_max),
                num(stats.temp_avg)
            ));
            out.push_str(&format!(
                ",\"pressure_min\":{},\"pressure_max\":{},\"pressure_avg\":{}",
                num(stats.pressure_min),
                num(stats.pressure_max),
                num(stats.pressure_avg)
            ));
            out.push_str(",\"free_atoms\":");
            push_free_atoms(&mut out, stats);
            out.push_str(",\"mol_size_dist\":");
            push_mol_dist(&mut out, stats);
            out.push('}');
        }
        Event::BondFormed {
            tick,
            bond_id,
            atom_a,
            atom_b,
            elem_a,
            elem_b,
            order,
            energy,
            x,
            y,
        } => {
            out.push_str("{\"v\":1,\"type\":\"bond_formed\"");
            out.push_str(&format!(",\"tick\":{tick},\"bond_id\":{bond_id}"));
            out.push_str(&format!(",\"atom_a\":{},\"atom_b\":{}", atom_a.0, atom_b.0));
            out.push_str(&format!(
                ",\"elem_a\":\"{}\",\"elem_b\":\"{}\"",
                element(*elem_a).symbol,
                element(*elem_b).symbol
            ));
            out.push_str(&format!(",\"order\":{order},\"energy\":{}", num(*energy)));
            out.push_str(&format!(",\"x\":{},\"y\":{}", num(*x), num(*y)));
            out.push('}');
        }
        Event::BondBroken {
            tick,
            bond_id,
            elem_a,
            elem_b,
            energy_released,
            x,
            y,
        } => {
            out.push_str("{\"v\":1,\"type\":\"bond_broken\"");
            out.push_str(&format!(",\"tick\":{tick},\"bond_id\":{bond_id}"));
            out.push_str(&format!(
                ",\"elem_a\":\"{}\",\"elem_b\":\"{}\"",
                element(*elem_a).symbol,
                element(*elem_b).symbol
            ));
            out.push_str(&format!(
                ",\"energy_released\":{},\"x\":{},\"y\":{}",
                num(*energy_released),
                num(*x),
                num(*y)
            ));
            out.push('}');
        }
        Event::End {
            tick,
            timing,
            reason,
        } => {
            out.push_str("{\"v\":1,\"type\":\"end\"");
            out.push_str(&format!(",\"tick\":{tick}"));
            out.push_str(&format!(",\"elapsed_ms\":{}", timing.elapsed_ms));
            out.push_str(&format!(",\"reason\":\"{reason}\""));
            out.push('}');
        }
    }
    out
}

/// JSON number from f32: non-finite guards to 0, -0.0 normalizes to
/// 0. Rust's Display is shortest-round-trip and cross-platform
/// stable (module doc).
fn num(v: f32) -> String {
    if v.is_finite() {
        format!("{}", if v == 0.0 { 0.0 } else { v })
    } else {
        "0".to_string()
    }
}

fn num64(v: f64) -> String {
    if v.is_finite() {
        format!("{}", if v == 0.0 { 0.0 } else { v })
    } else {
        "0".to_string()
    }
}

/// Minimal JSON string escaping for the run/world names: quote,
/// backslash, and the C0 control range. Names are phase-1 ASCII,
/// but the contract must hold for whatever phase 3 parses in.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// free_atoms: only nonzero elements, in canonical element order
/// (deterministic key order), keyed by symbol.
fn push_free_atoms(out: &mut String, stats: &WorldStats) {
    out.push('{');
    let mut first = true;
    for (i, count) in stats.free_atoms.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!(
            "\"{}\":{count}",
            crate::elements::ELEMENTS[i].symbol
        ));
    }
    out.push('}');
}

/// mol_size_dist: all four buckets, always, fixed key order.
fn push_mol_dist(out: &mut String, stats: &WorldStats) {
    out.push_str(&format!(
        "{{\"1\":{},\"2_5\":{},\"6_20\":{},\"21plus\":{}}}",
        stats.mol_size_dist[0],
        stats.mol_size_dist[1],
        stats.mol_size_dist[2],
        stats.mol_size_dist[3]
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::{Timing, WorldStats};
    use crate::world::{AtomId, ElementId};

    fn stats_fixture() -> WorldStats {
        WorldStats {
            atom_count: 3000,
            bond_count: 2000,
            temp_min: 30.1,
            temp_max: 39.9,
            temp_avg: 35.0,
            pressure_min: 0.8,
            pressure_max: 20.1,
            pressure_avg: 4.2,
            free_atoms: [892, 234, 0, 445, 0, 0, 0, 0, 0, 0],
            mol_size_dist: [892, 445, 89, 12],
        }
    }

    #[test]
    fn start_line_matches_schema() {
        let event = Event::Start {
            khem_version: "0.1.0",
            run_name: "experiment_1".into(),
            world_name: "primordial_pond".into(),
            seed: 42,
            atom_count: 3000,
            bond_count: 2000,
            world_width: 150.0,
            world_height: 150.0,
        };
        assert_eq!(
            emit(&event),
            "{\"v\":1,\"type\":\"start\",\"tick\":0,\"khem_version\":\"0.1.0\",\
             \"run_name\":\"experiment_1\",\"world_name\":\"primordial_pond\",\
             \"seed\":42,\"atom_count\":3000,\"bond_count\":2000,\
             \"world_width\":150,\"world_height\":150}"
        );
    }

    #[test]
    fn tick_line_matches_schema() {
        let event = Event::Tick {
            tick: 1000,
            timing: Timing {
                elapsed_ms: 124,
                ticks_per_sec: 8064.5,
            },
            stats: stats_fixture(),
        };
        let line = emit(&event);
        assert!(line.starts_with(
            "{\"v\":1,\"type\":\"tick\",\"tick\":1000,\"elapsed_ms\":124,\
             \"ticks_per_sec\":8064.5,\"atom_count\":3000,\"bond_count\":2000"
        ));
        assert!(line.contains("\"temp_min\":30.1,\"temp_max\":39.9,\"temp_avg\":35"));
        assert!(line.contains("\"free_atoms\":{\"H\":892,\"C\":234,\"O\":445}"));
        assert!(
            line.ends_with("\"mol_size_dist\":{\"1\":892,\"2_5\":445,\"6_20\":89,\"21plus\":12}}")
        );
    }

    #[test]
    fn bond_lines_match_schema() {
        let formed = Event::BondFormed {
            tick: 1247,
            bond_id: 4521,
            atom_a: AtomId(442),
            atom_b: AtomId(891),
            elem_a: ElementId(1),
            elem_b: ElementId(3),
            order: 2,
            energy: 799.0,
            x: 45.2,
            y: 123.7,
        };
        assert_eq!(
            emit(&formed),
            "{\"v\":1,\"type\":\"bond_formed\",\"tick\":1247,\"bond_id\":4521,\
             \"atom_a\":442,\"atom_b\":891,\"elem_a\":\"C\",\"elem_b\":\"O\",\
             \"order\":2,\"energy\":799,\"x\":45.2,\"y\":123.7}"
        );
        let broken = Event::BondBroken {
            tick: 1248,
            bond_id: 4521,
            elem_a: ElementId(1),
            elem_b: ElementId(3),
            energy_released: 399.5,
            x: 45.3,
            y: 123.8,
        };
        assert_eq!(
            emit(&broken),
            "{\"v\":1,\"type\":\"bond_broken\",\"tick\":1248,\"bond_id\":4521,\
             \"elem_a\":\"C\",\"elem_b\":\"O\",\"energy_released\":399.5,\
             \"x\":45.3,\"y\":123.8}"
        );
    }

    #[test]
    fn end_line_matches_schema() {
        let event = Event::End {
            tick: 1000,
            timing: Timing {
                elapsed_ms: 2000,
                ticks_per_sec: 500.0,
            },
            reason: "max_ticks_reached",
        };
        assert_eq!(
            emit(&event),
            "{\"v\":1,\"type\":\"end\",\"tick\":1000,\"elapsed_ms\":2000,\
             \"reason\":\"max_ticks_reached\"}"
        );
    }

    #[test]
    fn escaping_and_non_finite_guards() {
        let event = Event::Start {
            khem_version: "0.1.0",
            run_name: "weird \"name\"\\\n".into(),
            world_name: "w".into(),
            seed: 1,
            atom_count: 0,
            bond_count: 0,
            world_width: f32::NAN,
            world_height: f32::INFINITY,
        };
        let line = emit(&event);
        assert!(line.contains("\"run_name\":\"weird \\\"name\\\"\\\\\\n\""));
        assert!(line.contains("\"world_width\":0,\"world_height\":0"));
        // -0.0 normalizes.
        assert_eq!(num(-0.0), "0");
    }

    #[test]
    fn every_line_is_single_line_json() {
        // No raw newlines anywhere in an emitted event.
        let events = [
            Event::Start {
                khem_version: "0.1.0",
                run_name: "r".into(),
                world_name: "w".into(),
                seed: 1,
                atom_count: 0,
                bond_count: 0,
                world_width: 1.0,
                world_height: 1.0,
            },
            Event::Tick {
                tick: 10,
                timing: Timing::default(),
                stats: WorldStats {
                    atom_count: 1,
                    bond_count: 0,
                    temp_min: 0.0,
                    temp_max: 0.0,
                    temp_avg: 0.0,
                    pressure_min: 0.0,
                    pressure_max: 0.0,
                    pressure_avg: 0.0,
                    free_atoms: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    mol_size_dist: [1, 0, 0, 0],
                },
            },
        ];
        for event in &events {
            let line = emit(event);
            assert!(!line.contains('\n'), "embedded newline in {line}");
            assert!(line.starts_with("{\"v\":1,"));
            assert!(line.ends_with('}'));
        }
    }
}
