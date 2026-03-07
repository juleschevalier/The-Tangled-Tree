//! Creature domain entity.

use crate::domain::genetics::Genome;
use crate::domain::world::WorldPosition;

/// Strongly typed creature identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreatureId(pub u64);

/// Current life status of a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VitalStatus {
    Alive,
    Dead,
}

/// Reason why a creature died.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Died from starvation (hunger increased too much).
    Starvation,
    /// Died from energy exhaustion (spent all energy).
    Exhaustion,
    /// Died from old age (reached max age).
    Age,
}

/// Runtime balancing constants for creature lifecycle.
#[derive(Debug, Clone, Copy)]
pub struct CreatureConfig {
    pub base_energy_drain_per_tick: f32,
    pub hunger_gain_per_tick: f32,
    pub starvation_hunger_threshold: f32,
    pub max_age_ticks: u64,
    pub reproduction_hunger_max: f32,
    pub reproduction_energy_min: f32,
    pub reproduction_min_age_ticks: u64,
}

impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            base_energy_drain_per_tick: 0.08,
            hunger_gain_per_tick: 1.5,
            starvation_hunger_threshold: 80.0,
            max_age_ticks: 10_000,
            reproduction_hunger_max: 50.0,
            reproduction_energy_min: 50.0,
            reproduction_min_age_ticks: 100,
        }
    }
}

/// Creature aggregate root (pure domain).
#[derive(Debug, Clone, PartialEq)]
pub struct Creature {
    pub id: CreatureId,
    pub position: WorldPosition,
    pub genome: Genome,
    pub age_ticks: u64,
    pub energy: f32,
    pub hunger: f32,
    status: VitalStatus,
    /// Reason for death (if dead).
    pub death_cause: Option<DeathCause>,
}

impl Creature {
    /// Spawn a new creature with baseline runtime stats.
    #[must_use]
    pub fn spawn(id: CreatureId, position: WorldPosition, genome: Genome) -> Self {
        Self {
            id,
            position,
            genome,
            age_ticks: 0,
            energy: 100.0,
            hunger: 0.0,
            status: VitalStatus::Alive,
            death_cause: None,
        }
    }

    /// Whether the creature is alive.
    #[must_use]
    pub const fn is_alive(&self) -> bool {
        matches!(self.status, VitalStatus::Alive)
    }

    /// Current life status.
    #[must_use]
    pub const fn status(&self) -> VitalStatus {
        self.status
    }

    /// Age the creature by one simulation tick.
    pub fn tick(&mut self, config: CreatureConfig) {
        if !self.is_alive() {
            return;
        }

        self.age_ticks = self.age_ticks.saturating_add(1);

        self.hunger += config.hunger_gain_per_tick;

        let metabolism = self.genome.expressed_speed() * self.genome.expressed_size();
        self.energy -= config.base_energy_drain_per_tick + metabolism;

        // Determine death cause (in order of precedence: age, starvation, exhaustion)
        if self.age_ticks >= config.max_age_ticks {
            self.energy = self.energy.max(0.0);
            self.status = VitalStatus::Dead;
            self.death_cause = Some(DeathCause::Age);
        } else if self.hunger >= config.starvation_hunger_threshold {
            // Starvation is immediately lethal — no lingering "damage over time".
            self.hunger = config.starvation_hunger_threshold;
            self.status = VitalStatus::Dead;
            self.death_cause = Some(DeathCause::Starvation);
        } else if self.energy <= 0.0 {
            self.energy = 0.0;
            self.status = VitalStatus::Dead;
            self.death_cause = Some(DeathCause::Exhaustion);
        }
    }

    /// Increase energy by feeding and reduce hunger proportionally.
    pub fn feed(&mut self, food_energy: f32) {
        if !self.is_alive() || food_energy <= 0.0 {
            return;
        }

        self.energy = (self.energy + food_energy).clamp(0.0, 100.0);
        self.hunger = (self.hunger - food_energy).max(0.0);
    }

    /// Move creature to a new position.
    pub fn move_to(&mut self, destination: WorldPosition) {
        if self.is_alive() {
            self.position = destination;
        }
    }

    /// Basic reproduction gate for the MVP.
    #[must_use]
    pub fn can_reproduce(&self, config: CreatureConfig) -> bool {
        self.is_alive()
            && self.age_ticks >= config.reproduction_min_age_ticks
            && self.energy >= config.reproduction_energy_min
            && self.hunger <= config.reproduction_hunger_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_creature() -> Creature {
        Creature::spawn(CreatureId(1), WorldPosition::new(2, 3), Genome::baseline())
    }

    #[test]
    fn spawn_starts_alive_with_baseline_stats() {
        let creature = baseline_creature();

        assert_eq!(creature.status(), VitalStatus::Alive);
        assert_eq!(creature.energy, 100.0);
        assert_eq!(creature.hunger, 0.0);
        assert_eq!(creature.age_ticks, 0);
    }

    #[test]
    fn tick_increases_age_and_hunger() {
        let mut creature = baseline_creature();

        creature.tick(CreatureConfig::default());

        assert_eq!(creature.age_ticks, 1);
        assert!(creature.hunger > 0.0);
    }

    #[test]
    fn starvation_kills_when_hunger_reaches_threshold() {
        let mut creature = baseline_creature();
        let config = CreatureConfig {
            starvation_hunger_threshold: 10.0,
            ..CreatureConfig::default()
        };
        // Set hunger just below threshold — next tick's hunger_gain will push it over
        creature.hunger = 10.0 - config.hunger_gain_per_tick + 0.01;

        creature.tick(config);

        assert_eq!(creature.status(), VitalStatus::Dead);
        assert_eq!(creature.death_cause, Some(DeathCause::Starvation));
    }

    #[test]
    fn creature_dies_when_energy_reaches_zero() {
        let mut creature = baseline_creature();
        creature.energy = 0.5;
        let config = CreatureConfig {
            base_energy_drain_per_tick: 5.0,
            ..CreatureConfig::default()
        };

        creature.tick(config);

        assert_eq!(creature.status(), VitalStatus::Dead);
        assert_eq!(creature.energy, 0.0);
    }

    #[test]
    fn creature_dies_when_max_age_reached() {
        let mut creature = baseline_creature();
        creature.age_ticks = 9;
        let config = CreatureConfig {
            max_age_ticks: 10,
            ..CreatureConfig::default()
        };

        creature.tick(config);

        assert_eq!(creature.status(), VitalStatus::Dead);
    }

    #[test]
    fn feed_restores_energy_and_reduces_hunger() {
        let mut creature = baseline_creature();
        creature.energy = 45.0;
        creature.hunger = 30.0;

        creature.feed(20.0);

        assert_eq!(creature.energy, 65.0);
        assert_eq!(creature.hunger, 10.0);
    }

    #[test]
    fn dead_creature_does_not_change_anymore() {
        let mut creature = baseline_creature();
        creature.tick(CreatureConfig {
            base_energy_drain_per_tick: 500.0,
            ..CreatureConfig::default()
        });

        let snapshot = creature.clone();
        creature.tick(CreatureConfig::default());
        creature.feed(20.0);
        creature.move_to(WorldPosition::new(9, 9));

        assert_eq!(creature, snapshot);
    }

    #[test]
    fn reproduction_gate_works() {
        let mut creature = baseline_creature();
        let config = CreatureConfig {
            reproduction_min_age_ticks: 3,
            reproduction_energy_min: 70.0,
            reproduction_hunger_max: 20.0,
            ..CreatureConfig::default()
        };

        creature.age_ticks = 3;
        creature.energy = 75.0;
        creature.hunger = 15.0;

        assert!(creature.can_reproduce(config));

        creature.hunger = 25.0;
        assert!(!creature.can_reproduce(config));
    }
}
