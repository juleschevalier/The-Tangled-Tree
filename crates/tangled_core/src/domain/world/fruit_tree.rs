//! Fruit tree — a world entity that provides a periodic high-energy food source.
//!
//! Trees cycle through three states:
//! - **Growing**: building up energy reserves (no fruit available)
//! - **Fruiting**: ripe fruit is available for creatures to harvest
//! - **Resting**: recovering after harvest (or after fruit expired)
//!
//! A creature standing on a tree's tile during the Fruiting phase can
//! harvest the fruit for a large energy gain, immediately transitioning
//! the tree to the Resting state.

use super::world_position::WorldPosition;

/// Duration of each tree state, in simulation ticks.
pub const TREE_GROWING_DURATION: u32 = 400;
pub const TREE_FRUITING_DURATION: u32 = 300;
pub const TREE_RESTING_DURATION: u32 = 200;

/// Total cycle length in ticks.
const TREE_CYCLE_LENGTH: u32 =
    TREE_GROWING_DURATION + TREE_FRUITING_DURATION + TREE_RESTING_DURATION;

/// State of a fruit tree in its growth cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FruitTreeState {
    /// Growing phase — no fruit yet.
    Growing,
    /// Fruiting phase — ripe fruit available for creatures.
    Fruiting,
    /// Resting phase — recovering after harvest or fruit expiry.
    Resting,
}

/// A fruit tree located on a world tile.
///
/// Trees are part of the world state and are ticked alongside terrain.
#[derive(Debug, Clone, PartialEq)]
pub struct FruitTree {
    /// Position on the world grid.
    pub position: WorldPosition,
    /// Current lifecycle state.
    pub state: FruitTreeState,
    /// How many ticks have elapsed in the current state.
    pub ticks_in_state: u32,
}

impl FruitTree {
    /// Create a new tree in the Growing state, starting fresh.
    #[must_use]
    pub fn new(position: WorldPosition) -> Self {
        Self {
            position,
            state: FruitTreeState::Growing,
            ticks_in_state: 0,
        }
    }

    /// Create a new tree with an initial offset into the lifecycle cycle.
    ///
    /// Used during world generation to spread trees across different phases
    /// so the simulation starts with some trees already fruiting.
    #[must_use]
    pub fn new_with_offset(position: WorldPosition, offset_ticks: u32) -> Self {
        let t = offset_ticks % TREE_CYCLE_LENGTH;

        if t < TREE_GROWING_DURATION {
            Self {
                position,
                state: FruitTreeState::Growing,
                ticks_in_state: t,
            }
        } else if t < TREE_GROWING_DURATION + TREE_FRUITING_DURATION {
            Self {
                position,
                state: FruitTreeState::Fruiting,
                ticks_in_state: t - TREE_GROWING_DURATION,
            }
        } else {
            Self {
                position,
                state: FruitTreeState::Resting,
                ticks_in_state: t - TREE_GROWING_DURATION - TREE_FRUITING_DURATION,
            }
        }
    }

    /// Whether this tree currently has harvestable fruit.
    #[must_use]
    pub fn has_fruit(&self) -> bool {
        self.state == FruitTreeState::Fruiting
    }

    /// Attempt to harvest fruit from this tree.
    ///
    /// Returns `true` if fruit was successfully taken (tree was Fruiting),
    /// `false` otherwise. On success, the tree immediately transitions to
    /// the Resting state.
    pub fn harvest(&mut self) -> bool {
        if self.state == FruitTreeState::Fruiting {
            self.state = FruitTreeState::Resting;
            self.ticks_in_state = 0;
            true
        } else {
            false
        }
    }

    /// Advance the tree lifecycle by one simulation tick.
    ///
    /// State transitions:
    /// - `Growing` (400 ticks) → `Fruiting`
    /// - `Fruiting` (300 ticks, if not harvested) → `Resting`
    /// - `Resting` (200 ticks) → `Growing`
    pub fn tick(&mut self) {
        self.ticks_in_state += 1;
        match self.state {
            FruitTreeState::Growing if self.ticks_in_state >= TREE_GROWING_DURATION => {
                self.state = FruitTreeState::Fruiting;
                self.ticks_in_state = 0;
            }
            FruitTreeState::Fruiting if self.ticks_in_state >= TREE_FRUITING_DURATION => {
                // Fruit expired without being harvested
                self.state = FruitTreeState::Resting;
                self.ticks_in_state = 0;
            }
            FruitTreeState::Resting if self.ticks_in_state >= TREE_RESTING_DURATION => {
                self.state = FruitTreeState::Growing;
                self.ticks_in_state = 0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree_starts_growing() {
        let tree = FruitTree::new(WorldPosition::new(1, 1));
        assert_eq!(tree.state, FruitTreeState::Growing);
        assert_eq!(tree.ticks_in_state, 0);
        assert!(!tree.has_fruit());
    }

    #[test]
    fn transitions_to_fruiting_after_growing_duration() {
        let mut tree = FruitTree::new(WorldPosition::new(0, 0));
        for _ in 0..TREE_GROWING_DURATION {
            tree.tick();
        }
        assert_eq!(tree.state, FruitTreeState::Fruiting);
        assert!(tree.has_fruit());
    }

    #[test]
    fn fruit_expires_after_fruiting_duration() {
        let mut tree = FruitTree::new_with_offset(WorldPosition::new(0, 0), TREE_GROWING_DURATION);
        assert_eq!(tree.state, FruitTreeState::Fruiting);

        for _ in 0..TREE_FRUITING_DURATION {
            tree.tick();
        }
        assert_eq!(tree.state, FruitTreeState::Resting);
        assert!(!tree.has_fruit());
    }

    #[test]
    fn harvest_succeeds_when_fruiting() {
        let mut tree = FruitTree::new_with_offset(WorldPosition::new(0, 0), TREE_GROWING_DURATION);
        assert!(tree.has_fruit());

        let harvested = tree.harvest();
        assert!(harvested);
        assert_eq!(tree.state, FruitTreeState::Resting);
    }

    #[test]
    fn harvest_fails_when_not_fruiting() {
        let mut tree = FruitTree::new(WorldPosition::new(0, 0));
        // Still Growing
        let harvested = tree.harvest();
        assert!(!harvested);
        assert_eq!(tree.state, FruitTreeState::Growing);
    }

    #[test]
    fn resting_transitions_to_growing() {
        let mut tree = FruitTree::new_with_offset(
            WorldPosition::new(0, 0),
            TREE_GROWING_DURATION + TREE_FRUITING_DURATION,
        );
        assert_eq!(tree.state, FruitTreeState::Resting);

        for _ in 0..TREE_RESTING_DURATION {
            tree.tick();
        }
        assert_eq!(tree.state, FruitTreeState::Growing);
    }

    #[test]
    fn offset_constructor_spreads_across_states() {
        let pos = WorldPosition::new(0, 0);

        let growing = FruitTree::new_with_offset(pos, 0);
        assert_eq!(growing.state, FruitTreeState::Growing);

        let fruiting = FruitTree::new_with_offset(pos, TREE_GROWING_DURATION);
        assert_eq!(fruiting.state, FruitTreeState::Fruiting);

        let resting =
            FruitTree::new_with_offset(pos, TREE_GROWING_DURATION + TREE_FRUITING_DURATION);
        assert_eq!(resting.state, FruitTreeState::Resting);
    }

    #[test]
    fn full_cycle_completes_deterministically() {
        let mut tree = FruitTree::new(WorldPosition::new(3, 3));
        let total = TREE_GROWING_DURATION + TREE_FRUITING_DURATION + TREE_RESTING_DURATION;
        for _ in 0..total {
            tree.tick();
        }
        // Should be back to Growing at tick 0
        assert_eq!(tree.state, FruitTreeState::Growing);
        assert_eq!(tree.ticks_in_state, 0);
    }
}
