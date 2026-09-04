//! Deterministic seeded random number generation.
//!
//! Guarantee G02 (same seed in, byte-identical run out) stands on
//! this module. The generator is SplitMix64: integer arithmetic
//! only, identical on every platform. [`Rng::normal`] deliberately
//! avoids transcendentals (sum of twelve uniforms) so reproducibility
//! never depends on libm differences between machines.
//!
//! Spec: docs/specs/runtime-spec.md, section 5.3 (Determinism).

/// SplitMix64. Not cryptographically secure; it does not need to be.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a generator. The zero seed is allowed; the sequence it
    /// produces is fine, merely constant.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next raw 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn f01(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Approximate normal sample: the sum of twelve uniforms.
    ///
    /// Exact in the sense of bit-reproducibility; approximate in
    /// distribution (tails bounded at +/- 6 sigma). Thermal noise
    /// does not care.
    pub fn normal(&mut self, mean: f64, sigma: f64) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += self.f01();
        }
        mean + sigma * (sum - 6.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn uniforms_in_range() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let v = rng.f01();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn normal_is_centered_and_scaled() {
        let n = 10_000;
        let mut rng = Rng::new(99);
        let mut sum = 0.0;
        for _ in 0..n {
            sum += rng.normal(0.0, 1.0);
        }
        let mean = sum / n as f64;
        assert!(mean.abs() < 0.05, "mean {mean}");

        let mut rng = Rng::new(99);
        let mut var = 0.0;
        for _ in 0..n {
            let v = rng.normal(0.0, 1.0);
            var += v * v;
        }
        let var = var / n as f64;
        assert!((0.8..1.2).contains(&var), "var {var}");
    }
}
