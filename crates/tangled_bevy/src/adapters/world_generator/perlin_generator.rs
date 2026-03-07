//! Perlin noise world generator — implements the WorldGenerator port.

use noise::{NoiseFn, Perlin};
use tangled_core::domain::world::{Terrain, Tile, WorldConfig, WorldMap};
use tangled_core::ports::outbound::WorldGenerator;

/// World generator using Perlin noise for terrain elevation.
///
/// Maps noise values to terrain types based on configurable thresholds:
/// - Below `water_level` → Water
/// - Above `rock_level` → Rock
/// - In between → Grass (with Sand near water edges)
pub struct PerlinWorldGenerator {
    /// Scale factor for noise sampling (lower = smoother terrain).
    pub scale: f64,
    /// Sand margin: how far above water_level sand extends.
    pub sand_margin: f64,
}

impl Default for PerlinWorldGenerator {
    fn default() -> Self {
        Self {
            scale: 0.05,
            sand_margin: 0.08,
        }
    }
}

impl WorldGenerator for PerlinWorldGenerator {
    fn generate(&self, config: &WorldConfig) -> WorldMap {
        let perlin = Perlin::new(config.seed as u32);
        let mut tiles = Vec::with_capacity((config.width * config.height) as usize);

        for y in 0..config.height {
            for x in 0..config.width {
                let nx = x as f64 * self.scale;
                let ny = y as f64 * self.scale;

                // Perlin returns values in [-1, 1], normalize to [0, 1]
                let raw = perlin.get([nx, ny]);
                let elevation = (raw + 1.0) / 2.0;

                let terrain = classify_terrain(elevation, config, self.sand_margin);

                tiles.push(Tile::new(terrain, elevation));
            }
        }

        WorldMap::new(config.width, config.height, tiles)
    }
}

/// Classify a normalized elevation value into a terrain type.
fn classify_terrain(elevation: f64, config: &WorldConfig, sand_margin: f64) -> Terrain {
    if elevation < config.water_level {
        Terrain::Water
    } else if elevation < config.water_level + sand_margin {
        Terrain::Sand
    } else if elevation > config.rock_level {
        Terrain::Rock
    } else {
        Terrain::Grass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_dimensions() {
        let generator = PerlinWorldGenerator::default();
        let config = WorldConfig::with_seed(12345);
        let map = generator.generate(&config);
        assert_eq!(map.width(), config.width);
        assert_eq!(map.height(), config.height);
    }

    #[test]
    fn deterministic_with_same_seed() {
        let generator = PerlinWorldGenerator::default();
        let config = WorldConfig::with_seed(42);
        let map1 = generator.generate(&config);
        let map2 = generator.generate(&config);

        // Compare all tiles
        for (pos1, tile1) in map1.iter() {
            let tile2 = map2.get(pos1).unwrap();
            assert_eq!(tile1.terrain, tile2.terrain);
            assert_eq!(tile1.elevation, tile2.elevation);
        }
    }

    #[test]
    fn different_seeds_produce_different_maps() {
        let generator = PerlinWorldGenerator::default();
        let map1 = generator.generate(&WorldConfig::with_seed(1));
        let map2 = generator.generate(&WorldConfig::with_seed(999));

        // At least some tiles should differ
        let different = map1
            .iter()
            .zip(map2.iter())
            .filter(|((_, t1), (_, t2))| t1.terrain != t2.terrain)
            .count();
        assert!(different > 0, "Different seeds should produce different maps");
    }

    #[test]
    fn has_variety_of_terrain() {
        let generator = PerlinWorldGenerator::default();
        let config = WorldConfig::with_seed(42);
        let map = generator.generate(&config);

        assert!(map.count_terrain(Terrain::Grass) > 0, "Should have grass");
        assert!(map.count_terrain(Terrain::Water) > 0, "Should have water");
    }

    #[test]
    fn classify_terrain_thresholds() {
        let config = WorldConfig {
            water_level: 0.3,
            rock_level: 0.8,
            ..WorldConfig::default()
        };
        assert_eq!(classify_terrain(0.1, &config, 0.05), Terrain::Water);
        assert_eq!(classify_terrain(0.32, &config, 0.05), Terrain::Sand);
        assert_eq!(classify_terrain(0.5, &config, 0.05), Terrain::Grass);
        assert_eq!(classify_terrain(0.9, &config, 0.05), Terrain::Rock);
    }
}
