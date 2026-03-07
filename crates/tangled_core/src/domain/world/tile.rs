//! A single tile in the world grid.

use super::terrain::Terrain;

/// A tile represents a single cell in the world grid.
///
/// Each tile has a terrain type and a current food level.
/// Food regenerates over time based on the terrain's growth rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    /// The terrain type of this tile.
    pub terrain: Terrain,
    /// Elevation value (0.0 = sea level, 1.0 = mountain peak).
    pub elevation: f64,
    /// Current food available on this tile (0.0–1.0).
    pub food: f64,
}

impl Tile {
    /// Create a new tile with the given terrain and elevation.
    /// Food is initialized based on terrain's growth rate.
    #[must_use]
    pub fn new(terrain: Terrain, elevation: f64) -> Self {
        Self {
            terrain,
            elevation,
            food: terrain.food_growth_rate(),
        }
    }

    /// Regenerate food towards the terrain's maximum growth rate.
    ///
    /// `rate` is the fraction of the gap to fill per tick (0.0–1.0).
    pub fn regenerate_food(&mut self, rate: f64) {
        let max = self.terrain.food_growth_rate();
        if self.food < max {
            self.food += (max - self.food) * rate;
            // Clamp to max
            if self.food > max {
                self.food = max;
            }
        }
    }

    /// Consume food from this tile. Returns the amount actually consumed.
    pub fn consume_food(&mut self, amount: f64) -> f64 {
        let consumed = amount.min(self.food);
        self.food -= consumed;
        consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grass_tile_starts_with_food() {
        let tile = Tile::new(Terrain::Grass, 0.3);
        assert_eq!(tile.food, 1.0);
        assert_eq!(tile.terrain, Terrain::Grass);
    }

    #[test]
    fn water_tile_has_no_food() {
        let tile = Tile::new(Terrain::Water, 0.0);
        assert_eq!(tile.food, 0.0);
    }

    #[test]
    fn consume_food_partial() {
        let mut tile = Tile::new(Terrain::Grass, 0.3);
        let consumed = tile.consume_food(0.4);
        assert!((consumed - 0.4).abs() < f64::EPSILON);
        assert!((tile.food - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn consume_food_more_than_available() {
        let mut tile = Tile::new(Terrain::Sand, 0.5);
        let consumed = tile.consume_food(5.0);
        assert!((consumed - 0.2).abs() < f64::EPSILON); // Sand food_growth_rate = 0.2
        assert!(tile.food.abs() < f64::EPSILON);
    }

    #[test]
    fn regenerate_food_fills_gap() {
        let mut tile = Tile::new(Terrain::Grass, 0.3);
        tile.food = 0.0;
        tile.regenerate_food(0.5); // fill 50% of gap
        assert!((tile.food - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn regenerate_food_on_rock_stays_zero() {
        let mut tile = Tile::new(Terrain::Rock, 0.8);
        tile.regenerate_food(1.0);
        assert_eq!(tile.food, 0.0);
    }
}
