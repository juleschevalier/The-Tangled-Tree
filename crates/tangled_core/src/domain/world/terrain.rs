//! Terrain types that define the nature of each tile.

/// The terrain type of a world tile.
///
/// Each variant represents a distinct biome/surface type that affects
/// creature movement, survival, and resource availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    /// Fertile land — creatures can walk, food grows here.
    Grass,
    /// Impassable water — blocks movement, no food.
    Water,
    /// Rocky terrain — passable but slow, no food.
    Rock,
    /// Sandy terrain — passable, scarce food.
    Sand,
}

impl Terrain {
    /// Whether creatures can walk on this terrain.
    #[must_use]
    pub const fn is_walkable(self) -> bool {
        match self {
            Self::Grass | Self::Rock | Self::Sand => true,
            Self::Water => false,
        }
    }

    /// Base food growth rate on this terrain (0.0 = none, 1.0 = maximum).
    #[must_use]
    pub const fn food_growth_rate(self) -> f64 {
        match self {
            Self::Grass => 1.0,
            Self::Sand => 0.2,
            Self::Rock => 0.0,
            Self::Water => 0.0,
        }
    }

    /// Movement speed multiplier on this terrain (1.0 = normal).
    #[must_use]
    pub const fn movement_multiplier(self) -> f64 {
        match self {
            Self::Grass => 1.0,
            Self::Sand => 0.8,
            Self::Rock => 0.5,
            Self::Water => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_is_not_walkable() {
        assert!(!Terrain::Water.is_walkable());
    }

    #[test]
    fn grass_is_walkable_with_full_food() {
        assert!(Terrain::Grass.is_walkable());
        assert_eq!(Terrain::Grass.food_growth_rate(), 1.0);
    }

    #[test]
    fn rock_is_walkable_but_slow() {
        assert!(Terrain::Rock.is_walkable());
        assert_eq!(Terrain::Rock.movement_multiplier(), 0.5);
        assert_eq!(Terrain::Rock.food_growth_rate(), 0.0);
    }

    #[test]
    fn sand_has_partial_food_and_speed() {
        assert!(Terrain::Sand.is_walkable());
        assert!(Terrain::Sand.food_growth_rate() > 0.0);
        assert!(Terrain::Sand.movement_multiplier() < 1.0);
    }
}
