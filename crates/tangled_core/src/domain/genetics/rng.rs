//! Deterministic pseudo-random number generator for domain operations.
//!
//! This keeps `tangled_core` free from external RNG crates while preserving
//! deterministic behavior for reproduction and mutation.

/// Lightweight deterministic RNG based on SplitMix64.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Create a new deterministic RNG from a seed.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next random `u64`.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Random float in `[0.0, 1.0]`.
    #[must_use]
    pub fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        (value as f64) / ((1u64 << 53) as f64)
    }

    /// Bernoulli trial with probability `p`.
    #[must_use]
    pub fn chance(&mut self, p: f64) -> bool {
        debug_assert!((0.0..=1.0).contains(&p));
        self.next_f64() < p
    }

    /// Uniform random index in `[0, upper_bound)`.
    #[must_use]
    pub fn index(&mut self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound > 0);
        (self.next_u64() as usize) % upper_bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = DeterministicRng::new(42);
        let mut b = DeterministicRng::new(42);

        for _ in 0..20 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn chance_zero_and_one_are_stable() {
        let mut rng = DeterministicRng::new(123);
        for _ in 0..20 {
            assert!(!rng.chance(0.0));
            assert!(rng.chance(1.0));
        }
    }

    #[test]
    fn index_is_in_bounds() {
        let mut rng = DeterministicRng::new(7);
        for _ in 0..100 {
            let idx = rng.index(3);
            assert!(idx < 3);
        }
    }
}
