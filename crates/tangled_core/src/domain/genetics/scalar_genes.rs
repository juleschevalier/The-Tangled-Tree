//! Concrete gene implementations used by the MVP.

use super::diet::Diet;
use super::gene::Gene;
use super::rng::DeterministicRng;

/// Movement speed gene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedGene(pub f32);

impl SpeedGene {
    pub const MIN: f32 = 0.2;
    pub const MAX: f32 = 3.0;
    const STEP: f32 = 0.15;

    #[must_use]
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for SpeedGene {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Gene for SpeedGene {
    fn mutate(self, rng: &mut DeterministicRng) -> Self {
        let delta = if rng.chance(0.5) {
            Self::STEP
        } else {
            -Self::STEP
        };
        Self::clamped(self.0 + delta)
    }
}

/// Body size gene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeGene(pub f32);

impl SizeGene {
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 2.5;
    const STEP: f32 = 0.1;

    #[must_use]
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for SizeGene {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Gene for SizeGene {
    fn mutate(self, rng: &mut DeterministicRng) -> Self {
        let delta = if rng.chance(0.5) {
            Self::STEP
        } else {
            -Self::STEP
        };
        Self::clamped(self.0 + delta)
    }
}

/// Diet preference gene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DietGene(pub Diet);

impl DietGene {
    #[must_use]
    pub const fn value(self) -> Diet {
        self.0
    }
}

impl Default for DietGene {
    fn default() -> Self {
        Self(Diet::Herbivore)
    }
}

impl Gene for DietGene {
    fn mutate(self, rng: &mut DeterministicRng) -> Self {
        let next = match self.0 {
            Diet::Herbivore => {
                if rng.chance(0.5) {
                    Diet::Omnivore
                } else {
                    Diet::Carnivore
                }
            }
            Diet::Omnivore => {
                if rng.chance(0.5) {
                    Diet::Herbivore
                } else {
                    Diet::Carnivore
                }
            }
            Diet::Carnivore => {
                if rng.chance(0.5) {
                    Diet::Omnivore
                } else {
                    Diet::Herbivore
                }
            }
        };

        Self(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_mutation_stays_in_range() {
        let mut rng = DeterministicRng::new(1);
        let mut gene = SpeedGene::default();
        for _ in 0..200 {
            gene = gene.mutate(&mut rng);
            assert!((SpeedGene::MIN..=SpeedGene::MAX).contains(&gene.value()));
        }
    }

    #[test]
    fn size_mutation_stays_in_range() {
        let mut rng = DeterministicRng::new(2);
        let mut gene = SizeGene::default();
        for _ in 0..200 {
            gene = gene.mutate(&mut rng);
            assert!((SizeGene::MIN..=SizeGene::MAX).contains(&gene.value()));
        }
    }

    #[test]
    fn diet_mutation_changes_to_valid_category() {
        let mut rng = DeterministicRng::new(3);
        let original = DietGene(Diet::Herbivore);
        let mutated = original.mutate(&mut rng);
        assert_ne!(mutated, original);
        assert!(matches!(
            mutated.value(),
            Diet::Herbivore | Diet::Omnivore | Diet::Carnivore
        ));
    }
}
