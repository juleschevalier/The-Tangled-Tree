//! Perlin noise world generator — implements the WorldGenerator port.

use noise::{NoiseFn, Perlin};
use tangled_core::domain::world::{FruitTree, Terrain, Tile, WorldConfig, WorldMap, WorldPosition};
use tangled_core::ports::outbound::WorldGenerator;

/// Noise threshold above which a Dirt tile receives a fruit tree.
/// Value \in [0.0, 1.0] \u2014 higher = fewer trees. ~0.90 yields ~10% of Dirt tiles.
const TREE_NOISE_THRESHOLD: f64 = 0.90;

/// World generator using Perlin noise for terrain elevation.
///
/// Maps noise values to terrain types based on configurable thresholds:
/// - Below `water_level` → Water
/// - Above `rock_level` → Rock
/// - In between → Dirt (with grass determined by a second noise octave)
pub struct PerlinWorldGenerator {
    /// Scale factor for noise sampling (lower = smoother terrain).
    pub scale: f64,
    /// Scale factor for the grass noise layer.
    pub grass_scale: f64,
}

impl Default for PerlinWorldGenerator {
    fn default() -> Self {
        Self {
            scale: 0.05,
            grass_scale: 0.05,
        }
    }
}

impl WorldGenerator for PerlinWorldGenerator {
    fn generate(&self, config: &WorldConfig) -> WorldMap {
        let terrain_noise = Perlin::new(config.seed as u32);
        // Second noise layer with different seed for grass distribution
        let grass_noise = Perlin::new(config.seed.wrapping_add(12345) as u32);
        // Third noise layer for fruit tree placement
        let tree_noise = Perlin::new(config.seed.wrapping_add(99_999) as u32);

        let mut tiles = Vec::with_capacity((config.width * config.height) as usize);
        let mut trees: Vec<FruitTree> = Vec::new();

        for y in 0..config.height {
            for x in 0..config.width {
                let nx = x as f64 * self.scale;
                let ny = y as f64 * self.scale;

                // Perlin returns values in [-1, 1], normalize to [0, 1]
                let raw = terrain_noise.get([nx, ny]);
                let elevation = (raw + 1.0) / 2.0;

                let terrain = classify_terrain(elevation, config);

                // Grass level: use a second noise octave, normalized to [0, 1]
                let tile = if terrain.can_grow_grass() {
                    let gx = x as f64 * self.grass_scale;
                    let gy = y as f64 * self.grass_scale;
                    let grass_raw = grass_noise.get([gx, gy]);
                    let mut grass = ((grass_raw + 1.0) / 2.0).clamp(0.0, 1.0);
                    // Apply threshold: sparse grass, less abundant
                    if grass < 0.35 {
                        grass = 0.0;
                    }
                    Tile::with_grass(terrain, elevation, grass)
                } else {
                    Tile::new(terrain, elevation)
                };

                // Place a fruit tree on walkable Dirt tiles with enough noise
                if terrain == Terrain::Dirt {
                    let tx = x as f64 * self.scale;
                    let ty = y as f64 * self.scale;
                    let tree_raw = tree_noise.get([tx, ty]);
                    let tree_val = (tree_raw + 1.0) / 2.0;
                    if tree_val > TREE_NOISE_THRESHOLD {
                        // Spread initial lifecycle phase across cycle using a position hash
                        let offset = x.wrapping_mul(317).wrapping_add(y.wrapping_mul(521));
                        let pos = WorldPosition::new(x, y);
                        trees.push(FruitTree::new_with_offset(pos, offset));
                    }
                }

                tiles.push(tile);
            }
        }

        let mut world_map = WorldMap::new(config.width, config.height, tiles);
        world_map.set_trees(trees);
        world_map
    }
}

/// Classify a normalized elevation value into a terrain type.
fn classify_terrain(elevation: f64, config: &WorldConfig) -> Terrain {
    if elevation < config.water_level {
        Terrain::Water
    } else if elevation > config.rock_level {
        Terrain::Rock
    } else {
        Terrain::Dirt
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

        for (pos1, tile1) in map1.iter() {
            let tile2 = map2.get(pos1).unwrap();
            assert_eq!(tile1.terrain, tile2.terrain);
            assert_eq!(tile1.elevation, tile2.elevation);
            assert!((tile1.grass - tile2.grass).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn different_seeds_produce_different_maps() {
        let generator = PerlinWorldGenerator::default();
        let map1 = generator.generate(&WorldConfig::with_seed(1));
        let map2 = generator.generate(&WorldConfig::with_seed(999));

        let different = map1
            .iter()
            .zip(map2.iter())
            .filter(|((_, t1), (_, t2))| t1.terrain != t2.terrain)
            .count();
        assert!(
            different > 0,
            "Different seeds should produce different maps"
        );
    }

    #[test]
    fn has_variety_of_terrain() {
        let generator = PerlinWorldGenerator::default();
        let config = WorldConfig::with_seed(42);
        let map = generator.generate(&config);

        assert!(map.count_terrain(Terrain::Dirt) > 0, "Should have dirt");
        assert!(map.count_terrain(Terrain::Water) > 0, "Should have water");
    }

    #[test]
    fn dirt_tiles_have_grass() {
        let generator = PerlinWorldGenerator::default();
        let config = WorldConfig::with_seed(42);
        let map = generator.generate(&config);

        let dirt_with_grass = map
            .iter()
            .filter(|(_, t)| t.terrain == Terrain::Dirt && t.grass > 0.0)
            .count();
        assert!(dirt_with_grass > 0, "Some dirt tiles should have grass");
    }

    #[test]
    fn classify_terrain_thresholds() {
        let config = WorldConfig {
            water_level: 0.3,
            rock_level: 0.8,
            ..WorldConfig::default()
        };
        assert_eq!(classify_terrain(0.1, &config), Terrain::Water);
        assert_eq!(classify_terrain(0.5, &config), Terrain::Dirt);
        assert_eq!(classify_terrain(0.9, &config), Terrain::Rock);
    }
}
