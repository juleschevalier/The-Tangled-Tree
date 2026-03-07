//! Genetic model — genes, chromosomes, mutations, recombination.
//!
//! This module defines the extensible genetic system that drives creature
//! traits and evolution through natural selection.

mod allele_pair;
mod diet;
mod gene;
mod genome;
pub mod rng;
mod scalar_genes;

pub use allele_pair::AllelePair;
pub use diet::Diet;
pub use gene::Gene;
pub use genome::{Genome, MutationConfig};
pub use rng::DeterministicRng;
pub use scalar_genes::{DietGene, SizeGene, SpeedGene};
