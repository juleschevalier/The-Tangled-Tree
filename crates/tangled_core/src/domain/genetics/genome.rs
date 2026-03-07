//! Genome representation and genetic operators.

use super::allele_pair::AllelePair;
use super::diet::Diet;
use super::rng::DeterministicRng;
use super::scalar_genes::{DietGene, SizeGene, SpeedGene};

/// Parameters controlling mutation dynamics.
#[derive(Debug, Clone, Copy)]
pub struct MutationConfig {
    /// Probability that an inherited allele mutates.
    pub mutation_rate: f64,
}

impl MutationConfig {
    #[must_use]
    pub fn new(mutation_rate: f64) -> Self {
        Self {
            mutation_rate: mutation_rate.clamp(0.0, 1.0),
        }
    }
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.02,
        }
    }
}

/// Diploid genome for MVP creature traits.
#[derive(Debug, Clone, PartialEq)]
pub struct Genome {
    pub speed: AllelePair<SpeedGene>,
    pub size: AllelePair<SizeGene>,
    pub diet: AllelePair<DietGene>,
}

impl Genome {
    /// Baseline starter genome.
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            speed: AllelePair::new(SpeedGene::default(), SpeedGene::default()),
            size: AllelePair::new(SizeGene::default(), SizeGene::default()),
            diet: AllelePair::new(DietGene::default(), DietGene::default()),
        }
    }

    /// Build offspring genome from two parents with deterministic seed.
    #[must_use]
    pub fn offspring(
        parent_a: &Self,
        parent_b: &Self,
        seed: u64,
        mutation: MutationConfig,
    ) -> Self {
        let mut rng = DeterministicRng::new(seed);

        Self {
            speed: AllelePair::recombine(
                &parent_a.speed,
                &parent_b.speed,
                &mut rng,
                mutation.mutation_rate,
            ),
            size: AllelePair::recombine(
                &parent_a.size,
                &parent_b.size,
                &mut rng,
                mutation.mutation_rate,
            ),
            diet: AllelePair::recombine(
                &parent_a.diet,
                &parent_b.diet,
                &mut rng,
                mutation.mutation_rate,
            ),
        }
    }

    /// Expressed speed phenotype (mean of both alleles).
    #[must_use]
    pub fn expressed_speed(&self) -> f32 {
        (self.speed.maternal.value() + self.speed.paternal.value()) * 0.5
    }

    /// Expressed size phenotype (mean of both alleles).
    #[must_use]
    pub fn expressed_size(&self) -> f32 {
        (self.size.maternal.value() + self.size.paternal.value()) * 0.5
    }

    /// Expressed diet phenotype (simple dominance model).
    #[must_use]
    pub fn expressed_diet(&self) -> Diet {
        let a = self.diet.maternal.value();
        let b = self.diet.paternal.value();

        if a == b {
            return a;
        }

        if matches!(
            (a, b),
            (Diet::Herbivore, Diet::Carnivore) | (Diet::Carnivore, Diet::Herbivore)
        ) {
            return Diet::Omnivore;
        }

        if a == Diet::Omnivore || b == Diet::Omnivore {
            Diet::Omnivore
        } else {
            a
        }
    }
}

impl Default for Genome {
    fn default() -> Self {
        Self::baseline()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parent(speed_a: f32, speed_b: f32, size_a: f32, size_b: f32, diet: Diet) -> Genome {
        Genome {
            speed: AllelePair::new(SpeedGene::clamped(speed_a), SpeedGene::clamped(speed_b)),
            size: AllelePair::new(SizeGene::clamped(size_a), SizeGene::clamped(size_b)),
            diet: AllelePair::new(DietGene(diet), DietGene(diet)),
        }
    }

    #[test]
    fn offspring_is_deterministic_for_same_seed() {
        let parent_a = make_parent(0.8, 1.0, 0.9, 1.1, Diet::Herbivore);
        let parent_b = make_parent(1.4, 1.6, 1.3, 1.5, Diet::Carnivore);
        let config = MutationConfig::new(0.05);

        let child_1 = Genome::offspring(&parent_a, &parent_b, 1234, config);
        let child_2 = Genome::offspring(&parent_a, &parent_b, 1234, config);

        assert_eq!(child_1, child_2);
    }

    #[test]
    fn offspring_without_mutation_keeps_parent_allele_sets() {
        let parent_a = make_parent(0.8, 1.0, 0.9, 1.1, Diet::Herbivore);
        let parent_b = make_parent(1.4, 1.6, 1.3, 1.5, Diet::Carnivore);

        let child = Genome::offspring(&parent_a, &parent_b, 42, MutationConfig::new(0.0));

        assert!(matches!(child.speed.maternal.value(), 0.8 | 1.0));
        assert!(matches!(child.speed.paternal.value(), 1.4 | 1.6));
        assert!(matches!(child.size.maternal.value(), 0.9 | 1.1));
        assert!(matches!(child.size.paternal.value(), 1.3 | 1.5));
    }

    #[test]
    fn expressed_speed_and_size_are_means() {
        let genome = Genome {
            speed: AllelePair::new(SpeedGene::clamped(0.8), SpeedGene::clamped(1.6)),
            size: AllelePair::new(SizeGene::clamped(0.6), SizeGene::clamped(1.4)),
            diet: AllelePair::new(DietGene(Diet::Herbivore), DietGene(Diet::Herbivore)),
        };

        assert!((genome.expressed_speed() - 1.2).abs() < 0.0001);
        assert!((genome.expressed_size() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn herbivore_plus_carnivore_resolves_to_omnivore() {
        let genome = Genome {
            speed: AllelePair::new(SpeedGene::default(), SpeedGene::default()),
            size: AllelePair::new(SizeGene::default(), SizeGene::default()),
            diet: AllelePair::new(DietGene(Diet::Herbivore), DietGene(Diet::Carnivore)),
        };

        assert_eq!(genome.expressed_diet(), Diet::Omnivore);
    }

    #[test]
    fn full_mutation_still_clamps_scalar_ranges() {
        let parent_a = make_parent(0.2, 3.0, 0.5, 2.5, Diet::Herbivore);
        let parent_b = make_parent(0.2, 3.0, 0.5, 2.5, Diet::Carnivore);

        let child = Genome::offspring(&parent_a, &parent_b, 99, MutationConfig::new(1.0));

        assert!((SpeedGene::MIN..=SpeedGene::MAX).contains(&child.speed.maternal.value()));
        assert!((SpeedGene::MIN..=SpeedGene::MAX).contains(&child.speed.paternal.value()));
        assert!((SizeGene::MIN..=SizeGene::MAX).contains(&child.size.maternal.value()));
        assert!((SizeGene::MIN..=SizeGene::MAX).contains(&child.size.paternal.value()));
    }
}
