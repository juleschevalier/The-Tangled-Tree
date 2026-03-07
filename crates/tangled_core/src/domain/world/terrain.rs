//! Terrain types that define the nature of each tile.

/// The terrain type of a world tile.
///
/// Each variant represents a distinct surface type that affects
/// creature movement and whether grass (food) can grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    /// Dirt — walkable, grass can grow here (the primary fertile terrain).
    Dirt,
    /// Impassable water — blocks movement, no grass.
    Water,
    /// Rocky terrain — walkable but slow, grass cannot grow.
    Rock,
}

impl Terrain {
    /// Whether creatures can walk on this terrain.
    #[must_use]
    pub const fn is_walkable(self) -> bool {
        match self {
            Self::Dirt | Self::Rock => true,
            Self::Water => false,
        }
    }

    /// Whether grass (food) can grow on this terrain.
    #[must_use]
    pub const fn can_grow_grass(self) -> bool {
        matches!(self, Self::Dirt)
    }

    /// Maximum grass level on this terrain (0.0 = none, 1.0 = lush).
    #[must_use]
    pub const fn max_grass(self) -> f64 {
        match self {
            Self::Dirt => 1.0,
            Self::Rock | Self::Water => 0.0,
        }
    }

    /// Movement speed multiplier on this terrain (1.0 = normal).
    #[must_use]
    pub const fn movement_multiplier(self) -> f64 {
        match self {
            Self::Dirt => 1.0,
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
        assert!(!Terrain::Water.can_grow_grass());
    }

    #[test]
    fn dirt_is_walkable_and_fertile() {
        assert!(Terrain::Dirt.is_walkable());
        assert!(Terrain::Dirt.can_grow_grass());
        assert_eq!(Terrain::Dirt.max_grass(), 1.0);
    }

    #[test]
    fn rock_is_walkable_but_barren() {
        assert!(Terrain::Rock.is_walkable());
        assert!(!Terrain::Rock.can_grow_grass());
        assert_eq!(Terrain::Rock.max_grass(), 0.0);
        assert_eq!(Terrain::Rock.movement_multiplier(), 0.5);
    }
}
