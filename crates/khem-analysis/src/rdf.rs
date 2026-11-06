//! The radial distribution function g(r): the medium's structural
//! fingerprint - the ratio of the observed pair density at distance
//! r to the uniform-expectation density.
//!
//! Convention (the harness's, kept): each UNORDERED pair (i < j)
//! is counted once, so the expected count per bin is
//! `0.5 * n * rho * v_bin` per sample, where n is the center
//! population, rho the number density, v_bin the bin volume.
//! g = 1.0 is a uniform ideal gas; the excluded-volume hole sits
//! at g = 0 below the wall; a first shell is a peak near the
//! contact distance.
//!
//! The K2.2 gate's preregistered baseline: no first shell at the
//! dormant cohesion depth (g_max 1.002 beyond 3.0 A) - the number
//! the cohesion activation must beat.

/// A g(r) accumulator: bins counted per sample, normalized on
/// demand into the g(r) table.
///
/// # Example
///
/// Two atoms 2 A apart in a 10 A cube, one sample: the pair lands
/// in the 0.25 A bin containing 2.0, and normalization spreads
/// it against the uniform expectation of that bin.
///
/// ```
/// use khem_analysis::Rdf;
///
/// let mut rdf = Rdf::new(6.0, 0.25);
/// rdf.add(2.0);
/// let g = rdf.normalize(2, 10.0 * 10.0 * 10.0, 1);
/// // The 2.0 A pair landed in exactly one bin.
/// assert_eq!(rdf.count_at(2.0), 1);
/// // g of the empty far bins is 0 (no pairs observed).
/// assert_eq!(g[rdf.bin_at(5.0)], 0.0);
/// ```
pub struct Rdf {
    max_r: f32,
    bin_width: f32,
    bins: Vec<u64>,
}

impl Rdf {
    /// New accumulator: pairs up to `max_r`, binned at `bin_width`
    /// (angstroems).
    pub fn new(max_r: f32, bin_width: f32) -> Self {
        assert!(max_r > 0.0 && bin_width > 0.0, "rdf geometry");
        let n_bins = (max_r / bin_width).ceil() as usize;
        Self {
            max_r,
            bin_width,
            bins: vec![0; n_bins],
        }
    }

    /// Record one unordered pair at distance `r`; pairs past
    /// `max_r` are ignored (the caller's cutoff).
    pub fn add(&mut self, r: f32) {
        if r >= 0.0 && r < self.max_r {
            self.bins[(r / self.bin_width) as usize] += 1;
        }
    }

    /// The bin index holding distance `r` (for test inspection).
    pub fn bin_at(&self, r: f32) -> usize {
        (r / self.bin_width) as usize
    }

    /// The raw count in the bin holding `r` (for test inspection).
    pub fn count_at(&self, r: f32) -> u64 {
        self.bins[self.bin_at(r)]
    }

    /// Normalize into g(r): `centers` is the per-sample center
    /// population (atoms eligible as pair-members), `volume` the
    /// world volume, `samples` the number of sampling passes
    /// accumulated.
    pub fn normalize(&self, centers: u64, volume: f64, samples: u64) -> Vec<f64> {
        let rho = centers as f64 / volume;
        self.bins
            .iter()
            .enumerate()
            .map(|(b, count)| {
                let r0 = b as f64 * self.bin_width as f64;
                let r1 = r0 + self.bin_width as f64;
                let v_bin = 4.0 * std::f64::consts::PI / 3.0 * (r1.powi(3) - r0.powi(3));
                let expected = 0.5 * centers as f64 * rho * v_bin * samples as f64;
                *count as f64 / expected.max(1.0)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pair_lands_in_its_bin() {
        let mut rdf = Rdf::new(6.0, 0.25);
        rdf.add(2.1);
        assert_eq!(rdf.count_at(2.0), 1);
        assert_eq!(rdf.count_at(1.0), 0);
    }

    #[test]
    fn pairs_past_max_r_are_ignored() {
        let mut rdf = Rdf::new(2.0, 0.25);
        rdf.add(2.5);
        assert_eq!(rdf.bins.iter().sum::<u64>(), 0);
    }

    #[test]
    fn uniform_box_normalizes_near_one() {
        // 4 atoms uniform in a big box, all 6 pairs at ~the same
        // distance: g of that bin is observed/expected - exactly
        // 1.0 when the expectation is computed from the same
        // density by construction.
        let centers = 4;
        let volume = 64.0; // 4 A cube
        let mut rdf = Rdf::new(6.0, 0.25);
        for _ in 0..6 {
            rdf.add(3.9);
        }
        let g = rdf.normalize(centers, volume, 1);
        let bin = rdf.bin_at(3.9);
        // observed 6; expected 0.5*4*(4/64)*v_bin(3.75..4.0)
        let r0 = bin as f64 * 0.25;
        let r1 = r0 + 0.25;
        let v_bin = 4.0 * std::f64::consts::PI / 3.0 * (r1.powi(3) - r0.powi(3));
        let expected = 0.5 * 4.0 * (4.0 / 64.0) * v_bin;
        assert!((g[bin] - 6.0 / expected).abs() < 1e-12);
    }
}
