//! The tick loop and strict system ordering: energy -> velocities ->
//! positions -> boundary -> spatial index -> bond breaking -> bond
//! formation -> observer -> event flush.
//!
//! The loop itself is the plan's phase 1 work; this module exists so
//! the ordering is decided once, in one place.
//!
//! Systems read previous-tick state and write current-tick state; no
//! system reads another system's writes within a tick. This fixed
//! order is what makes runs deterministic and later parallelizable
//! (guarantees G02, G14).
//!
//! Spec: docs/specs/runtime-spec.md, section 5 (Tick Execution).

/// The nine systems in strict tick order (runtime spec 5.1).
///
/// Kept as data, not control flow, so the order is inspectable and
/// testable before the systems exist.
pub const TICK_ORDER: [&str; 9] = [
    "EnergySystem::update",
    "PhysicsSystem::update_velocities",
    "PhysicsSystem::update_positions",
    "PhysicsSystem::apply_boundary",
    "SpatialIndex::rebuild",
    "ChemistrySystem::break_bonds",
    "ChemistrySystem::form_bonds",
    "ObserverSystem::sample",
    "EventQueue::flush_to_output",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_order_is_fixed() {
        assert_eq!(TICK_ORDER.len(), 9);
        assert_eq!(TICK_ORDER[0], "EnergySystem::update");
        assert_eq!(TICK_ORDER[4], "SpatialIndex::rebuild");
        assert_eq!(TICK_ORDER[8], "EventQueue::flush_to_output");
    }
}
