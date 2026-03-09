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
    /// Died from starvation (energy reached zero).
    Starvation,
    /// Died from old age (reached max age).
    Age,
}

/// Runtime balancing constants for creature lifecycle.
#[derive(Debug, Clone, Copy)]
pub struct CreatureConfig {
    /// Base energy lost per tick (before metabolism overhead).
    pub energy_drain_per_tick: f32,
    /// Multiplier applied to `speed × size` added to the drain each tick.
    pub metabolism_factor: f32,
    pub max_age_ticks: u64,
    pub reproduction_energy_min: f32,
    pub reproduction_min_age_ticks: u64,
}

impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            energy_drain_per_tick: 1.5,
            metabolism_factor: 0.1,
            max_age_ticks: 10_000,
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
    /// Energy level (0.0–100.0). Creature dies when it reaches 0.
    pub energy: f32,
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

        let metabolism = self.genome.expressed_speed() * self.genome.expressed_size();
        self.energy -= config.energy_drain_per_tick + metabolism * config.metabolism_factor;

        // Determine death cause (in order of precedence: age > starvation)
        if self.age_ticks >= config.max_age_ticks {
            self.energy = self.energy.max(0.0);
            self.status = VitalStatus::Dead;
            self.death_cause = Some(DeathCause::Age);
        } else if self.energy <= 0.0 {
            self.energy = 0.0;
            self.status = VitalStatus::Dead;
            self.death_cause = Some(DeathCause::Starvation);
        }
    }

    /// Restore energy by feeding.
    pub fn feed(&mut self, food_energy: f32) {
        if !self.is_alive() || food_energy <= 0.0 {
            return;
        }

        self.energy = (self.energy + food_energy).clamp(0.0, 100.0);
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
        assert_eq!(creature.age_ticks, 0);
    }

    #[test]
    fn tick_increases_age_and_drains_energy() {
        let mut creature = baseline_creature();

        creature.tick(CreatureConfig::default());

        assert_eq!(creature.age_ticks, 1);
        assert!(creature.energy < 100.0);
    }

    #[test]
    fn starvation_kills_when_energy_reaches_zero() {
        let mut creature = baseline_creature();
        creature.energy = 0.5;
        let config = CreatureConfig {
            energy_drain_per_tick: 5.0,
            ..CreatureConfig::default()
        };

        creature.tick(config);

        assert_eq!(creature.status(), VitalStatus::Dead);
        assert_eq!(creature.death_cause, Some(DeathCause::Starvation));
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
        assert_eq!(creature.death_cause, Some(DeathCause::Age));
    }

    #[test]
    fn feed_restores_energy() {
        let mut creature = baseline_creature();
        creature.energy = 45.0;

        creature.feed(20.0);

        assert_eq!(creature.energy, 65.0);
    }

    #[test]
    fn feed_caps_energy_at_100() {
        let mut creature = baseline_creature();
        creature.energy = 90.0;

        creature.feed(20.0);

        assert_eq!(creature.energy, 100.0);
    }

    #[test]
    fn dead_creature_does_not_change_anymore() {
        let mut creature = baseline_creature();
        creature.tick(CreatureConfig {
            energy_drain_per_tick: 500.0,
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
            ..CreatureConfig::default()
        };

        creature.age_ticks = 3;
        creature.energy = 75.0;

        assert!(creature.can_reproduce(config));

        creature.energy = 60.0;
        assert!(!creature.can_reproduce(config));
    }
}
