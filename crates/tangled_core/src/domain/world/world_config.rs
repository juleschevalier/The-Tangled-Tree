//! World generation configuration.
//!
//! These types configure how the world is generated, independently
//! of the concrete generation algorithm.

/// Configuration for world generation.
///
/// This is a pure data struct — no behavior, no deps.
/// The adapter decides *how* to generate; this says *what* to generate.
#[derive(Debug, Clone)]
pub struct WorldConfig {
    /// Random seed for deterministic generation.
    pub seed: u64,
    /// Width of the world in tiles.
    pub width: u32,
    /// Height of the world in tiles.
    pub height: u32,
    /// Water level threshold (0.0–1.0). Noise values below this become water.
    pub water_level: f64,
    /// Rock level threshold (0.0–1.0). Noise values above this become rock.
    pub rock_level: f64,
}

impl WorldConfig {
    /// Create a default config with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            width: 32,
            height: 32,
            water_level: 0.35,
            rock_level: 0.75,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self::with_seed(42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_reasonable() {
        let config = WorldConfig::default();
        assert_eq!(config.seed, 42);
        assert!(config.width > 0);
        assert!(config.height > 0);
        assert!(config.water_level < config.rock_level);
    }
}
