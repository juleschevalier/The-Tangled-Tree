//! Core simulation tick logic.

use crate::domain::creatures::{Creature, CreatureConfig, CreatureId, DeathCause};
use crate::domain::genetics::{DeterministicRng, Genome, MutationConfig};
use crate::domain::world::WorldMap;

use super::movement;

/// A single event that occurred during a tick.
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    /// A creature was born.
    Birth { id: CreatureId, tick: u64 },
    /// A creature died, with the age at death and cause.
    Death {
        id: CreatureId,
        tick: u64,
        age_ticks: u64,
        cause: DeathCause,
    },
}

/// Snapshot of population metrics after a tick.
#[derive(Debug, Clone, Default)]
pub struct SimulationState {
    pub tick: u64,
    pub alive_count: usize,
    pub dead_count: usize,
    pub births_this_tick: usize,
    pub deaths_this_tick: usize,
    pub deaths_by_starvation: usize,
    pub deaths_by_exhaustion: usize,
    pub deaths_by_age: usize,
    /// Detailed events for this tick (births + deaths).
    pub events: Vec<SimulationEvent>,
}

/// Stateless simulation tick operator.
///
/// Each call to [`step`](Self::step) advances the world by one tick:
/// 1. **Feed** — each living creature eats from its tile
/// 2. **Tick** — age, hunger, energy drain, possible death
/// 3. **Reproduce** — eligible pairs produce offspring
/// 4. **Regenerate** — world food grows back
pub struct SimulationTick;

impl SimulationTick {
    /// Advance the simulation by one tick. Mutates creatures and world in place.
    pub fn step(
        creatures: &mut Vec<Creature>,
        world_map: &mut WorldMap,
        creature_config: CreatureConfig,
        mutation_config: MutationConfig,
        current_tick: u64,
        grass_regen_rate: f64,
    ) -> SimulationState {
        let mut events = Vec::new();

        // Snapshot alive set before tick
        let alive_before: Vec<CreatureId> = creatures
            .iter()
            .filter(|c| c.is_alive())
            .map(|c| c.id)
            .collect();

        // 1. Move — creatures decide where to go
        movement::move_all_creatures(creatures, world_map, current_tick);

        // 2. Feed — each living creature eats grass from its tile
        for creature in creatures.iter_mut() {
            if !creature.is_alive() {
                continue;
            }
            if let Some(tile) = world_map.get_mut(creature.position) {
                let consumed = tile.consume_grass(0.3);
                creature.feed(consumed as f32 * 30.0);
            }
        }

        // 3. Tick — lifecycle step (age, hunger, energy, death)
        for creature in creatures.iter_mut() {
            creature.tick(creature_config);
        }

        // Detect deaths that happened this tick
        for creature in creatures.iter() {
            if !creature.is_alive()
                && alive_before.contains(&creature.id)
                && let Some(cause) = creature.death_cause
            {
                events.push(SimulationEvent::Death {
                    id: creature.id,
                    tick: current_tick,
                    age_ticks: creature.age_ticks,
                    cause,
                });
            }
        }

        // 4. Reproduce — find eligible pairs, create offspring
        let births = Self::reproduce(
            creatures,
            world_map,
            creature_config,
            mutation_config,
            current_tick,
        );

        // 5. Regenerate grass on all tiles
        world_map.regenerate_all_grass(grass_regen_rate);

        // Record birth events
        for child in &births {
            events.push(SimulationEvent::Birth {
                id: child.id,
                tick: current_tick,
            });
        }

        // Compute metrics
        let alive_after = creatures.iter().filter(|c| c.is_alive()).count();
        let total_dead = creatures.iter().filter(|c| !c.is_alive()).count();
        let deaths_this_tick = events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::Death { .. }))
            .count();
        let births_count = births.len();

        // Count deaths by cause
        let mut deaths_by_starvation = 0;
        let mut deaths_by_exhaustion = 0;
        let mut deaths_by_age = 0;
        for event in &events {
            if let SimulationEvent::Death { cause, .. } = event {
                match cause {
                    DeathCause::Starvation => deaths_by_starvation += 1,
                    DeathCause::Exhaustion => deaths_by_exhaustion += 1,
                    DeathCause::Age => deaths_by_age += 1,
                }
            }
        }

        // Add offspring to the population
        creatures.extend(births);

        SimulationState {
            tick: current_tick,
            alive_count: alive_after + births_count,
            dead_count: total_dead,
            births_this_tick: births_count,
            deaths_this_tick,
            deaths_by_starvation,
            deaths_by_exhaustion,
            deaths_by_age,
            events,
        }
    }

    /// Find eligible pairs and produce offspring near parents.
    fn reproduce(
        creatures: &[Creature],
        world_map: &WorldMap,
        creature_config: CreatureConfig,
        mutation_config: MutationConfig,
        current_tick: u64,
    ) -> Vec<Creature> {
        let eligible: Vec<&Creature> = creatures
            .iter()
            .filter(|c| c.can_reproduce(creature_config))
            .collect();

        if eligible.len() < 2 {
            return Vec::new();
        }

        let mut offspring = Vec::new();
        let next_id_base = creatures.len() as u64 + current_tick * 1000;
        let mut rng = DeterministicRng::new(current_tick.wrapping_mul(0xDEAD));

        // Simple pairing: consecutive eligible creatures that share a tile
        // or are neighbors can reproduce.
        let mut i = 0;
        while i + 1 < eligible.len() {
            let parent_a = eligible[i];
            let parent_b = eligible[i + 1];

            let distance = parent_a.position.manhattan_distance(parent_b.position);
            if distance <= 1 {
                let child_seed = current_tick
                    .wrapping_add(parent_a.id.0)
                    .wrapping_add(parent_b.id.0);
                let child_genome = Genome::offspring(
                    &parent_a.genome,
                    &parent_b.genome,
                    child_seed,
                    mutation_config,
                );

                // Place child on a walkable neighbor of parent_a
                let spawn_pos = world_map
                    .walkable_neighbors(parent_a.position)
                    .into_iter()
                    .nth(rng.index(world_map.walkable_neighbors(parent_a.position).len().max(1)))
                    .unwrap_or(parent_a.position);

                let child_id = CreatureId(next_id_base + offspring.len() as u64);
                offspring.push(Creature::spawn(child_id, spawn_pos, child_genome));

                i += 2; // skip both parents
            } else {
                i += 1;
            }
        }

        offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::creatures::Creature;
    use crate::domain::genetics::Genome;
    use crate::domain::world::{Terrain, WorldMap, WorldPosition};

    fn setup_world_and_creatures(count: usize) -> (Vec<Creature>, WorldMap) {
        let map = WorldMap::flat(10, 10, Terrain::Dirt);
        let creatures: Vec<Creature> = (0..count)
            .map(|i| {
                Creature::spawn(
                    CreatureId(i as u64),
                    WorldPosition::new((i as u32) % 10, (i as u32) / 10),
                    Genome::baseline(),
                )
            })
            .collect();
        (creatures, map)
    }

    #[test]
    fn single_tick_advances_age() {
        let (mut creatures, mut map) = setup_world_and_creatures(5);
        let state = SimulationTick::step(
            &mut creatures,
            &mut map,
            CreatureConfig::default(),
            MutationConfig::default(),
            1,
            0.05,
        );

        assert_eq!(state.tick, 1);
        assert!(state.alive_count > 0);
        for c in &creatures[..5] {
            assert_eq!(c.age_ticks, 1);
        }
    }

    #[test]
    fn creatures_die_over_time_without_food() {
        let map = WorldMap::flat(5, 5, Terrain::Rock); // no grass
        let mut creatures = vec![Creature::spawn(
            CreatureId(0),
            WorldPosition::new(0, 0),
            Genome::baseline(),
        )];

        let config = CreatureConfig {
            base_energy_drain_per_tick: 10.0,
            ..CreatureConfig::default()
        };

        let mut state = SimulationState::default();
        for tick in 1..=50 {
            state = SimulationTick::step(
                &mut creatures,
                &mut map.clone(),
                config,
                MutationConfig::default(),
                tick,
                0.0,
            );
        }

        assert_eq!(state.alive_count, 0);
    }

    #[test]
    fn grass_regeneration_happens() {
        let (mut creatures, mut map) = setup_world_and_creatures(1);

        // Drain the tile grass first
        if let Some(tile) = map.get_mut(WorldPosition::new(0, 0)) {
            tile.consume_grass(1.0);
            assert_eq!(tile.grass, 0.0);
        }

        SimulationTick::step(
            &mut creatures,
            &mut map,
            CreatureConfig::default(),
            MutationConfig::default(),
            1,
            0.5,
        );

        let tile = map.get(WorldPosition::new(0, 0)).unwrap();
        assert!(tile.grass > 0.0, "grass should have regenerated");
    }

    #[test]
    fn simulation_is_deterministic() {
        let (mut creatures_a, mut map_a) = setup_world_and_creatures(10);
        let (mut creatures_b, mut map_b) = setup_world_and_creatures(10);

        let config = CreatureConfig::default();
        let mutation = MutationConfig::default();

        for tick in 1..=20 {
            SimulationTick::step(&mut creatures_a, &mut map_a, config, mutation, tick, 0.05);
            SimulationTick::step(&mut creatures_b, &mut map_b, config, mutation, tick, 0.05);
        }

        assert_eq!(creatures_a.len(), creatures_b.len());
        for (a, b) in creatures_a.iter().zip(creatures_b.iter()) {
            assert_eq!(a.age_ticks, b.age_ticks);
            assert_eq!(a.position, b.position);
            assert_eq!(a.status(), b.status());
        }
    }
}
