//! khem-analysis: the observables library - measurements shared by
//! the runtime observer, the gate harnesses, and the future
//! khem-view tool family.
//!
//! Runtime needs the molecule census (union-find over the bond
//! graph, spec 9.2 - also used by the observer's stats buckets);
//! the harnesses need the radial distribution function, the
//! coordination census, the tetrahedral order parameter, and the
//! conservation checks. Everything here is read-only over the
//! world (G03) and allocation-isolated: it never touches the
//! world's own storage.

pub mod coordination;
pub mod molecules;
pub mod rdf;
pub mod tetrahedral;

pub use coordination::{intermolecular_contact_census, mean_coordination};
pub use molecules::{molecule_roots, molecule_sizes};
pub use rdf::Rdf;
pub use tetrahedral::tetrahedral_order;
