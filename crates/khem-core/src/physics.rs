//! The physics system: thermal velocity perturbation
//! (Maxwell-Boltzmann), temperature diffusion, bond spring forces,
//! pressure-gradient forces, position updates, boundary conditions.
//!
//! Setup scope: the system trait (runtime spec 10.4) so the plugin
//! boundary exists from day one. The implementation is the plan's
//! phase 1 work, gate K1 (stability).
//!
//! Spec: docs/specs/runtime-spec.md, section 6 (Physics System).

use crate::world::WorldState;

/// The physics system interface. v0.1 compiles exactly one
/// implementation; the trait exists so future plugin loading does
/// not require restructuring (runtime spec 10.4).
pub trait PhysicsSystem {
    fn update(&mut self, world: &mut WorldState);
}
