//! Diploid allele pair representation.

use super::gene::Gene;
use super::rng::DeterministicRng;

/// Two alleles for a single locus (diploid inheritance).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AllelePair<T> {
    pub maternal: T,
    pub paternal: T,
}

impl<T> AllelePair<T> {
    #[must_use]
    pub const fn new(maternal: T, paternal: T) -> Self {
        Self { maternal, paternal }
    }

    #[must_use]
    pub fn choose_one(&self, rng: &mut DeterministicRng) -> T
    where
        T: Copy,
    {
        if rng.chance(0.5) {
            self.maternal
        } else {
            self.paternal
        }
    }
}

impl<T> AllelePair<T>
where
    T: Gene,
{
    /// Recombine one allele from each parent, then apply mutation probability.
    #[must_use]
    pub fn recombine(
        from_parent_a: &Self,
        from_parent_b: &Self,
        rng: &mut DeterministicRng,
        mutation_rate: f64,
    ) -> Self {
        let mut allele_a = from_parent_a.choose_one(rng);
        let mut allele_b = from_parent_b.choose_one(rng);

        if rng.chance(mutation_rate) {
            allele_a = allele_a.mutate(rng);
        }
        if rng.chance(mutation_rate) {
            allele_b = allele_b.mutate(rng);
        }

        Self::new(allele_a, allele_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct IncrementGene(i32);

    impl Gene for IncrementGene {
        fn mutate(self, _rng: &mut DeterministicRng) -> Self {
            Self(self.0 + 1)
        }
    }

    #[test]
    fn recombine_with_zero_mutation_preserves_parent_values() {
        let parent_a = AllelePair::new(IncrementGene(1), IncrementGene(2));
        let parent_b = AllelePair::new(IncrementGene(10), IncrementGene(20));
        let mut rng = DeterministicRng::new(5);

        let child = AllelePair::recombine(&parent_a, &parent_b, &mut rng, 0.0);

        assert!(matches!(child.maternal.0, 1 | 2));
        assert!(matches!(child.paternal.0, 10 | 20));
    }

    #[test]
    fn recombine_with_full_mutation_mutates_both_alleles() {
        let parent_a = AllelePair::new(IncrementGene(1), IncrementGene(2));
        let parent_b = AllelePair::new(IncrementGene(10), IncrementGene(20));
        let mut rng = DeterministicRng::new(7);

        let child = AllelePair::recombine(&parent_a, &parent_b, &mut rng, 1.0);

        assert!(matches!(child.maternal.0, 2 | 3));
        assert!(matches!(child.paternal.0, 11 | 21));
    }
}
