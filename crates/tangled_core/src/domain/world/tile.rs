//! A single tile in the world grid.

use super::terrain::Terrain;

/// A tile represents a single cell in the world grid.
///
/// Each tile has a terrain type and a grass level.
/// Grass is the sole food source; it regenerates over time on fertile terrain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    /// The terrain type of this tile.
    pub terrain: Terrain,
    /// Elevation value (0.0 = sea level, 1.0 = mountain peak).
    pub elevation: f64,
    /// Current grass level on this tile (0.0–1.0).
    /// Only meaningful on terrain where `can_grow_grass()` is true.
    pub grass: f64,
}

impl Tile {
    /// Create a new tile with the given terrain and elevation.
    /// Grass is initialized to 0 — the generator sets grass levels.
    #[must_use]
    pub fn new(terrain: Terrain, elevation: f64) -> Self {
        Self {
            terrain,
            elevation,
            grass: 0.0,
        }
    }

    /// Create a new tile with an explicit initial grass level.
    #[must_use]
    pub fn with_grass(terrain: Terrain, elevation: f64, grass: f64) -> Self {
        let max = terrain.max_grass();
        Self {
            terrain,
            elevation,
            grass: grass.clamp(0.0, max),
        }
    }

    /// Regenerate grass towards the terrain's maximum.
    ///
    /// `rate` is the fraction of the gap to fill per tick (0.0–1.0).
    pub fn regenerate_grass(&mut self, rate: f64) {
        let max = self.terrain.max_grass();
        if self.grass < max {
            self.grass += (max - self.grass) * rate;
            if self.grass > max {
                self.grass = max;
            }
        }
    }

    /// Consume grass from this tile. Returns the amount actually consumed.
    pub fn consume_grass(&mut self, amount: f64) -> f64 {
        let consumed = amount.min(self.grass);
        self.grass -= consumed;
        consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirt_tile_starts_with_no_grass_by_default() {
        let tile = Tile::new(Terrain::Dirt, 0.3);
        assert_eq!(tile.grass, 0.0);
        assert_eq!(tile.terrain, Terrain::Dirt);
    }

    #[test]
    fn dirt_tile_with_explicit_grass() {
        let tile = Tile::with_grass(Terrain::Dirt, 0.3, 0.7);
        assert!((tile.grass - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn water_tile_clamps_grass_to_zero() {
        let tile = Tile::with_grass(Terrain::Water, 0.0, 1.0);
        assert_eq!(tile.grass, 0.0);
    }

    #[test]
    fn rock_tile_clamps_grass_to_zero() {
        let tile = Tile::with_grass(Terrain::Rock, 0.8, 0.5);
        assert_eq!(tile.grass, 0.0);
    }

    #[test]
    fn consume_grass_partial() {
        let mut tile = Tile::with_grass(Terrain::Dirt, 0.3, 1.0);
        let consumed = tile.consume_grass(0.4);
        assert!((consumed - 0.4).abs() < f64::EPSILON);
        assert!((tile.grass - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn consume_grass_more_than_available() {
        let mut tile = Tile::with_grass(Terrain::Dirt, 0.5, 0.2);
        let consumed = tile.consume_grass(5.0);
        assert!((consumed - 0.2).abs() < f64::EPSILON);
        assert!(tile.grass.abs() < f64::EPSILON);
    }

    #[test]
    fn regenerate_grass_fills_gap() {
        let mut tile = Tile::with_grass(Terrain::Dirt, 0.3, 0.0);
        tile.regenerate_grass(0.5); // fill 50% of gap toward 1.0
        assert!((tile.grass - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn regenerate_grass_on_rock_stays_zero() {
        let mut tile = Tile::new(Terrain::Rock, 0.8);
        tile.regenerate_grass(1.0);
        assert_eq!(tile.grass, 0.0);
    }
}
