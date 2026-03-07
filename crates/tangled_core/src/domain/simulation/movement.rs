//! Creature movement decisions — pure domain logic.
//!
//! Each living creature picks a neighboring tile to move to based on
//! its current hunger, its speed gene, and the available food nearby.
//! Movement is deterministic for a given tick and creature state.

use crate::domain::creatures::Creature;
use crate::domain::genetics::DeterministicRng;
use crate::domain::world::WorldMap;

/// Movement behaviour threshold — creatures hungrier than this seek food.
const FOOD_SEEK_HUNGER_THRESHOLD: f32 = 20.0;

/// Decide and apply movement for all living creatures.
///
/// Called once per simulation tick. Each creature either:
/// - **Seeks food** (if hungry): moves toward the neighbor with the most food
/// - **Wanders** (otherwise): moves to a random walkable neighbor
///
/// The creature's `expressed_speed` gene controls the probability of actually
/// moving on any given tick (range ~0.2–3.0, clamped to 0.0–1.0 probability).
/// Faster creatures move more often.
pub fn move_all_creatures(creatures: &mut [Creature], world_map: &WorldMap, tick: u64) {
    for creature in creatures.iter_mut() {
        if !creature.is_alive() {
            continue;
        }

        // Deterministic RNG per creature per tick
        let seed = tick
            .wrapping_mul(0xBEEF)
            .wrapping_add(creature.id.0.wrapping_mul(0x1337));
        let mut rng = DeterministicRng::new(seed);

        // Speed gene → probability of moving this tick (0.0..=1.0)
        // expressed_speed ranges ~0.2–3.0 ; we normalise to a 0–1 probability
        let move_chance = (creature.genome.expressed_speed() / 3.0).clamp(0.0, 1.0);
        if !rng.chance(move_chance as f64) {
            continue;
        }

        let neighbors = world_map.walkable_neighbors(creature.position);
        if neighbors.is_empty() {
            continue;
        }

        let destination = if creature.hunger > FOOD_SEEK_HUNGER_THRESHOLD {
            // Seek food — pick the neighbor with the most food
            best_food_neighbor(&neighbors, world_map, &mut rng)
        } else {
            // Random wander
            neighbors[rng.index(neighbors.len())]
        };

        creature.move_to(destination);
    }
}

/// Pick the neighbor with the highest food level.
/// On ties, break randomly via the RNG.
fn best_food_neighbor(
    neighbors: &[crate::domain::world::WorldPosition],
    world_map: &WorldMap,
    rng: &mut DeterministicRng,
) -> crate::domain::world::WorldPosition {
    let mut best_food = -1.0_f64;
    let mut best = Vec::new();

    for &pos in neighbors {
        let food = world_map.get(pos).map(|t| t.grass).unwrap_or(0.0);

        if food > best_food {
            best_food = food;
            best.clear();
            best.push(pos);
        } else if (food - best_food).abs() < f64::EPSILON {
            best.push(pos);
        }
    }

    if best.is_empty() {
        // Fallback (shouldn't happen)
        neighbors[rng.index(neighbors.len())]
    } else {
        best[rng.index(best.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::creatures::{Creature, CreatureId};
    use crate::domain::genetics::Genome;
    use crate::domain::world::{Terrain, WorldMap, WorldPosition};

    #[test]
    fn creature_moves_to_walkable_neighbor() {
        let map = WorldMap::flat(5, 5, Terrain::Dirt);
        let mut creatures = vec![Creature::spawn(
            CreatureId(0),
            WorldPosition::new(2, 2),
            Genome::baseline(),
        )];

        // Run many ticks — at least once it should move
        let start_pos = creatures[0].position;
        let mut moved = false;
        for tick in 1..=100 {
            move_all_creatures(&mut creatures, &map, tick);
            if creatures[0].position != start_pos {
                moved = true;
                break;
            }
        }
        assert!(
            moved,
            "creature should have moved at least once in 100 ticks"
        );
    }

    #[test]
    fn dead_creature_does_not_move() {
        let map = WorldMap::flat(5, 5, Terrain::Dirt);
        let mut creature =
            Creature::spawn(CreatureId(0), WorldPosition::new(2, 2), Genome::baseline());

        // Kill the creature
        let kill_config = crate::domain::creatures::CreatureConfig {
            max_age_ticks: 0,
            ..Default::default()
        };
        creature.tick(kill_config);
        assert!(!creature.is_alive());

        let start_pos = creature.position;
        let mut creatures = vec![creature];
        for tick in 1..=50 {
            move_all_creatures(&mut creatures, &map, tick);
        }
        assert_eq!(
            creatures[0].position, start_pos,
            "dead creature must not move"
        );
    }

    #[test]
    fn creature_surrounded_by_water_stays_put() {
        // 3x3 map: center is grass, edges are water
        let mut tiles = Vec::new();
        for y in 0..3u32 {
            for x in 0..3u32 {
                let terrain = if x == 1 && y == 1 {
                    Terrain::Dirt
                } else {
                    Terrain::Water
                };
                tiles.push(crate::domain::world::Tile::new(terrain, 0.5));
            }
        }
        let map = WorldMap::new(3, 3, tiles);
        let mut creatures = vec![Creature::spawn(
            CreatureId(0),
            WorldPosition::new(1, 1),
            Genome::baseline(),
        )];

        for tick in 1..=50 {
            move_all_creatures(&mut creatures, &map, tick);
        }
        assert_eq!(
            creatures[0].position,
            WorldPosition::new(1, 1),
            "creature should stay on the only walkable tile"
        );
    }

    #[test]
    fn hungry_creature_seeks_food() {
        // 1x3 strip: left=rock(no grass), center=rock, right=dirt(grass)
        let tiles = vec![
            crate::domain::world::Tile::new(Terrain::Rock, 0.5),
            crate::domain::world::Tile::new(Terrain::Rock, 0.5),
            crate::domain::world::Tile::with_grass(Terrain::Dirt, 0.5, 1.0),
        ];
        let map = WorldMap::new(3, 1, tiles);

        let mut creature =
            Creature::spawn(CreatureId(0), WorldPosition::new(1, 0), Genome::baseline());
        // Make creature very hungry
        creature.hunger = 80.0;

        let mut creatures = vec![creature];

        // Over many ticks, the hungry creature should prefer the grass tile
        let mut grass_visits = 0u32;
        for tick in 1..=200 {
            move_all_creatures(&mut creatures, &map, tick);
            if creatures[0].position == WorldPosition::new(2, 0) {
                grass_visits += 1;
            }
            // Reset position to center to test bias repeatedly
            creatures[0].move_to(WorldPosition::new(1, 0));
        }

        // The creature should prefer dirt-with-grass (grass=1.0) over rock (grass=0.0)
        // most of the time when it actually moves
        assert!(
            grass_visits > 50,
            "hungry creature should strongly prefer food tile, got {grass_visits}/200"
        );
    }

    #[test]
    fn movement_is_deterministic() {
        let map = WorldMap::flat(10, 10, Terrain::Dirt);

        let make_creatures = || {
            (0..5)
                .map(|i| {
                    Creature::spawn(CreatureId(i), WorldPosition::new(5, 5), Genome::baseline())
                })
                .collect::<Vec<_>>()
        };

        let mut a = make_creatures();
        let mut b = make_creatures();

        for tick in 1..=50 {
            move_all_creatures(&mut a, &map, tick);
            move_all_creatures(&mut b, &map, tick);
        }

        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.position, cb.position, "movement must be deterministic");
        }
    }
}
