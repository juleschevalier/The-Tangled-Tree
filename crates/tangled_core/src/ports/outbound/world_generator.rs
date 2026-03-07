//! World generator port — defines how the domain requests world generation.

use crate::domain::world::{WorldConfig, WorldMap};

/// Outbound port for world generation.
///
/// The domain defines *what* it needs (a `WorldMap` from a `WorldConfig`),
/// and adapters decide *how* to generate it (Perlin noise, cellular automata, etc.).
pub trait WorldGenerator {
    /// Generate a world map from the given configuration.
    ///
    /// Implementations must be **deterministic**: same `config.seed` → same map.
    fn generate(&self, config: &WorldConfig) -> WorldMap;
}
