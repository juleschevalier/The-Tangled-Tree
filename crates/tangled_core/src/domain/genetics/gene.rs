//! Shared abstraction for gene mutation behavior.

use super::rng::DeterministicRng;

/// A gene that can mutate into a nearby variant.
pub trait Gene: Copy + Clone + PartialEq {
    /// Return a mutated variant of the current allele value.
    fn mutate(self, rng: &mut DeterministicRng) -> Self;
}
