//! Creature spawner — places initial creatures on walkable tiles.

use super::{Creature, CreatureId};
use crate::domain::genetics::{DeterministicRng, Genome};
use crate::domain::world::{WorldMap, WorldPosition};

/// Spawns an initial population of creatures on walkable tiles.
///
/// Uses a deterministic RNG seeded from the main simulation seed
/// so the same seed always produces the same initial population.
pub struct CreatureSpawner;

impl CreatureSpawner {
    /// Spawn `count` creatures on random walkable tiles.
    ///
    /// Returns fewer creatures than requested if there are fewer
    /// walkable tiles than the count.
    #[must_use]
    pub fn spawn_initial(world_map: &WorldMap, count: usize, seed: u64) -> Vec<Creature> {
        let walkable: Vec<WorldPosition> = world_map
            .iter()
            .filter(|(_, tile)| tile.terrain.is_walkable())
            .map(|(pos, _)| pos)
            .collect();

        if walkable.is_empty() {
            return Vec::new();
        }

        let mut rng = DeterministicRng::new(seed);
        let actual_count = count.min(walkable.len());

        (0..actual_count)
            .map(|i| {
                let idx = rng.index(walkable.len());
                let position = walkable[idx];
                let genome = Genome::baseline();
                Creature::spawn(CreatureId(i as u64), position, genome)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::world::{Terrain, WorldMap};

    #[test]
    fn spawn_on_flat_dirt_world() {
        let map = WorldMap::flat(10, 10, Terrain::Dirt);
        let creatures = CreatureSpawner::spawn_initial(&map, 5, 42);

        assert_eq!(creatures.len(), 5);
        for c in &creatures {
            assert!(c.is_alive());
            assert!(map.in_bounds(c.position));
            let tile = map.get(c.position).unwrap();
            assert!(tile.terrain.is_walkable());
        }
    }

    #[test]
    fn spawn_on_all_water_yields_empty() {
        let map = WorldMap::flat(10, 10, Terrain::Water);
        let creatures = CreatureSpawner::spawn_initial(&map, 5, 42);

        assert!(creatures.is_empty());
    }

    #[test]
    fn spawn_clamped_to_walkable_count() {
        // 2x2 map, only 1 walkable tile
        use crate::domain::world::Tile;
        let tiles = vec![
            Tile::new(Terrain::Water, 0.0),
            Tile::new(Terrain::Dirt, 0.5),
            Tile::new(Terrain::Water, 0.0),
            Tile::new(Terrain::Water, 0.0),
        ];
        let map = WorldMap::new(2, 2, tiles);
        let creatures = CreatureSpawner::spawn_initial(&map, 10, 1);

        assert_eq!(creatures.len(), 1);
        assert_eq!(creatures[0].position, WorldPosition::new(1, 0));
    }

    #[test]
    fn spawn_is_deterministic() {
        let map = WorldMap::flat(20, 20, Terrain::Dirt);
        let a = CreatureSpawner::spawn_initial(&map, 8, 999);
        let b = CreatureSpawner::spawn_initial(&map, 8, 999);

        assert_eq!(a.len(), b.len());
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.position, cb.position);
        }
    }

    #[test]
    fn each_creature_has_unique_id() {
        let map = WorldMap::flat(10, 10, Terrain::Dirt);
        let creatures = CreatureSpawner::spawn_initial(&map, 10, 42);

        let mut ids: Vec<_> = creatures.iter().map(|c| c.id).collect();
        ids.sort_by_key(|id| id.0);
        ids.dedup();
        assert_eq!(ids.len(), 10);
    }
}
