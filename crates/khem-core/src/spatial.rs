//! Spatial hash over live atom positions; rebuilt every tick
//! after position updates (runtime spec sections 4.9 and 5.1, tick
//! step 5).
//!
//! Wrap worlds: the index is wrap-aware (finding F11's fix) on
//! every axis - the 3-torus has three seam pairs (the 3D port,
//! ADR-0013). Cell coordinates fold into the grid at both rebuild
//! and query time, so a neighbor across any seam is found -
//! consistent with the minimum-image chemistry that evaluates
//! those pairs. Non-wrap boundaries use raw coordinates (positions
//! are normalized by the boundary step before any query, and
//! opposite walls are genuinely far apart).
//!
//! A radius-r query scans up to 27 cells (3x3x3) at the 5 A cell
//! size against the 2D index's 9 - the port's ~3x pair-scan cost,
//! re-measured against spec 13 by the phase-2 exit.

use std::collections::HashMap;

use crate::world::{AtomId, AtomState, BoundaryType};

/// Spatial hash over live atom positions.
///
/// Cell size defaults to 5 angstroms
/// ([`crate::config::PhysicsConfig::spatial_cell_size`]), roughly
/// the bond search radius. Rebuild is O(n); queries are O(1)
/// average.
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    cells: HashMap<(i32, i32, i32), Vec<AtomId>>,
    cell_size: f32,
    cols: i32,
    rows: i32,
    layers: i32,
    wrap: bool,
}

impl SpatialIndex {
    /// An empty index sized for a world. Rebuild before the first
    /// query.
    ///
    /// # Panics
    /// Panics if `cell_size` is not positive or the world lacks
    /// extent on any axis.
    pub fn new(
        cell_size: f32,
        width: f32,
        height: f32,
        depth: f32,
        boundary: BoundaryType,
    ) -> Self {
        assert!(cell_size > 0.0, "cell size must be positive");
        assert!(
            width > 0.0 && height > 0.0 && depth > 0.0,
            "world must have extent"
        );
        let cols = ((width / cell_size).ceil() as i32).max(1);
        let rows = ((height / cell_size).ceil() as i32).max(1);
        let layers = ((depth / cell_size).ceil() as i32).max(1);
        Self {
            cells: HashMap::new(),
            cell_size,
            cols,
            rows,
            layers,
            wrap: boundary == BoundaryType::Wrap,
        }
    }

    /// Cell size, angstroms.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Rebuilds the index from live atoms. Dead atoms are skipped.
    pub fn rebuild(&mut self, atoms: &[AtomState]) {
        self.cells.clear();
        for atom in atoms {
            if !atom.alive {
                continue;
            }
            let (gx, gy, gz) = self.cell_coords(atom.x, atom.y, atom.z);
            self.cells.entry((gx, gy, gz)).or_default().push(atom.id);
        }
    }

    /// Candidate atoms whose cells overlap the sphere at (x, y, z)
    /// with the given radius. The caller filters by exact distance
    /// (runtime spec 8.3; use [`crate::world::WorldState::delta`]
    /// for the distance in Wrap worlds). Order is by cell, not by
    /// distance; in Wrap worlds the scan folds at the edges, so
    /// seam-crossing candidates appear (once - degenerate grids
    /// with fewer cells than the scan span still yield each cell
    /// at most once).
    pub fn neighbors(&self, x: f32, y: f32, z: f32, radius: f32) -> Vec<AtomId> {
        let mut out = Vec::new();
        let (cx, cy, cz) = self.cell_coords(x, y, z);
        let span = (radius / self.cell_size).ceil().max(1.0) as i32;
        // Tiny grids can wrap the scan onto itself; collect the
        // visited cells first so each contributes once.
        let mut visited: Vec<(i32, i32, i32)> = Vec::new();
        for gx in cx - span..=cx + span {
            for gy in cy - span..=cy + span {
                for gz in cz - span..=cz + span {
                    let cell = if self.wrap {
                        (
                            gx.rem_euclid(self.cols),
                            gy.rem_euclid(self.rows),
                            gz.rem_euclid(self.layers),
                        )
                    } else {
                        (gx, gy, gz)
                    };
                    if visited.contains(&cell) {
                        continue;
                    }
                    visited.push(cell);
                    if let Some(ids) = self.cells.get(&cell) {
                        out.extend_from_slice(ids);
                    }
                }
            }
        }
        out
    }

    fn cell_coords(&self, x: f32, y: f32, z: f32) -> (i32, i32, i32) {
        let (cx, cy, cz) = (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
            (z / self.cell_size).floor() as i32,
        );
        if self.wrap {
            (
                cx.rem_euclid(self.cols),
                cy.rem_euclid(self.rows),
                cz.rem_euclid(self.layers),
            )
        } else {
            (cx, cy, cz)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::ElementId;

    fn atom(id: u32, x: f32, y: f32, z: f32) -> AtomState {
        AtomState::new(AtomId(id), ElementId(0), x, y, z)
    }

    fn index(w: f32, h: f32, d: f32, boundary: BoundaryType) -> SpatialIndex {
        SpatialIndex::new(5.0, w, h, d, boundary)
    }

    #[test]
    fn rebuild_and_query() {
        let atoms = vec![
            atom(0, 1.0, 1.0, 1.0),
            atom(1, 50.0, 50.0, 50.0),
            atom(2, 2.0, 2.0, 2.0),
        ];
        let mut idx = index(100.0, 100.0, 100.0, BoundaryType::Wrap);
        idx.rebuild(&atoms);
        let found = idx.neighbors(1.0, 1.0, 1.0, 4.0);
        assert!(found.contains(&AtomId(0)));
        assert!(found.contains(&AtomId(2)));
        assert!(!found.contains(&AtomId(1)));
    }

    #[test]
    fn dead_atoms_skipped() {
        let mut a = atom(0, 1.0, 1.0, 1.0);
        a.alive = false;
        let mut idx = index(100.0, 100.0, 100.0, BoundaryType::Wrap);
        idx.rebuild(&[a]);
        assert!(idx.neighbors(1.0, 1.0, 1.0, 4.0).is_empty());
    }

    #[test]
    fn rebuild_replaces_previous() {
        let mut idx = index(100.0, 100.0, 100.0, BoundaryType::Wrap);
        idx.rebuild(&[atom(0, 1.0, 1.0, 1.0)]);
        idx.rebuild(&[atom(5, 100.0, 100.0, 100.0)]);
        assert!(!idx.neighbors(1.0, 1.0, 1.0, 4.0).contains(&AtomId(0)));
    }

    #[test]
    fn wrap_worlds_find_seam_crossing_candidates_on_every_axis() {
        // F11: an atom 1 A across a seam is a neighbor of a query
        // at the opposite edge - on each of the three axes.
        for axis in 0..3 {
            let mut near = [0.5f32, 50.0, 50.0];
            let mut far = [99.5f32, 50.0, 50.0];
            near[axis] = 0.5;
            far[axis] = 99.5;
            let atoms = vec![
                atom(0, near[0], near[1], near[2]),
                atom(1, far[0], far[1], far[2]),
            ];
            let mut idx = index(100.0, 100.0, 100.0, BoundaryType::Wrap);
            idx.rebuild(&atoms);
            let found = idx.neighbors(near[0], near[1], near[2], 2.0);
            assert!(
                found.contains(&AtomId(1)),
                "seam neighbor on axis {axis} must be found"
            );
            // And symmetrically from the other side.
            let found = idx.neighbors(far[0], far[1], far[2], 2.0);
            assert!(found.contains(&AtomId(0)));
        }
    }

    #[test]
    fn wall_worlds_do_not_fold() {
        // Same geometry, Wall boundary: the far-side atom is NOT a
        // neighbor.
        let atoms = vec![atom(0, 0.5, 50.0, 50.0), atom(1, 99.5, 50.0, 50.0)];
        let mut idx = index(100.0, 100.0, 100.0, BoundaryType::Wall);
        idx.rebuild(&atoms);
        let found = idx.neighbors(0.5, 50.0, 50.0, 2.0);
        assert!(!found.contains(&AtomId(1)), "walls must not fold");
    }

    #[test]
    fn wrap_queries_find_every_close_pair_both_ways() {
        // F11's law as an always-on property: in a Wrap world the
        // index is a pure accelerator, never a filter - for ANY
        // configuration, every live pair within the minimum-image
        // query radius is found by both sides' queries. Brute force
        // is the reference. Half the atoms sit in the x-seam band,
        // where the folded cells are. (Gate K1.5 runs this same law
        // exhaustively, every tick of its beaker; this pins it
        // always-on for any future index change.)
        let (w, h, d) = (60.0, 40.0, 30.0);
        let mut rng = crate::rng::Rng::new(1234);
        let mut atoms = Vec::new();
        for i in 0..150u32 {
            let x = if i % 2 == 0 {
                rng.f01() as f32 * 4.0 // the x = 0 seam band
            } else {
                rng.f01() as f32 * w
            };
            let y = rng.f01() as f32 * h;
            let z = rng.f01() as f32 * d;
            atoms.push(atom(i, x, y, z));
        }
        let mut idx = index(w, h, d, BoundaryType::Wrap);
        idx.rebuild(&atoms);
        let min_image = |ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32| {
            let (dx, dy, dz) = (bx - ax, by - ay, bz - az);
            (
                dx - w * (dx / w).round(),
                dy - h * (dy / h).round(),
                dz - d * (dz / d).round(),
            )
        };
        let mut checked = 0;
        for (i, a) in atoms.iter().enumerate() {
            for (j, b) in atoms.iter().enumerate().skip(i + 1) {
                let (dx, dy, dz) = min_image(a.x, a.y, a.z, b.x, b.y, b.z);
                if dx * dx + dy * dy + dz * dz >= 4.0 * 4.0 {
                    continue;
                }
                checked += 1;
                assert!(
                    idx.neighbors(a.x, a.y, a.z, 4.0)
                        .contains(&AtomId(j as u32)),
                    "query from atom {i} lost atom {j}"
                );
                assert!(
                    idx.neighbors(b.x, b.y, b.z, 4.0)
                        .contains(&AtomId(i as u32)),
                    "query from atom {j} lost atom {i}"
                );
            }
        }
        assert!(
            checked >= 10,
            "probe too sparse: only {checked} close pairs"
        );
    }

    #[test]
    fn degenerate_grids_yield_each_cell_once() {
        // 1x1x1 grid with a span larger than the grid: duplicates
        // must be suppressed.
        let atoms = vec![atom(0, 0.5, 0.5, 0.5)];
        // Force the degenerate grid: cell size 100 over a 10 A
        // world folds every scan onto one cell.
        let mut idx = SpatialIndex::new(100.0, 10.0, 10.0, 10.0, BoundaryType::Wrap);
        idx.rebuild(&atoms);
        let found = idx.neighbors(5.0, 5.0, 5.0, 9.0);
        assert_eq!(found.len(), 1, "each cell at most once, got {found:?}");
    }
}
