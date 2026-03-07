//! The world map — a 2D grid of tiles.

use super::terrain::Terrain;
use super::tile::Tile;
use super::world_position::WorldPosition;

/// The world map is a rectangular grid of tiles.
///
/// Tiles are stored in row-major order: `tiles[y * width + x]`.
#[derive(Debug, Clone)]
pub struct WorldMap {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl WorldMap {
    /// Create a world map from a pre-built vector of tiles.
    ///
    /// # Panics
    ///
    /// Panics if `tiles.len() != width * height`.
    #[must_use]
    pub fn new(width: u32, height: u32, tiles: Vec<Tile>) -> Self {
        assert_eq!(
            tiles.len(),
            (width * height) as usize,
            "tiles.len() must equal width * height"
        );
        Self {
            width,
            height,
            tiles,
        }
    }

    /// Create a flat world filled with a single terrain type.
    /// Fertile tiles start with full grass.
    #[must_use]
    pub fn flat(width: u32, height: u32, terrain: Terrain) -> Self {
        let grass = terrain.max_grass();
        let tiles = vec![Tile::with_grass(terrain, 0.5, grass); (width * height) as usize];
        Self {
            width,
            height,
            tiles,
        }
    }

    /// Width of the world in tiles.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height of the world in tiles.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Total number of tiles.
    #[must_use]
    pub const fn tile_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Check if a position is within bounds.
    #[must_use]
    pub const fn in_bounds(&self, pos: WorldPosition) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    /// Get the tile at a position, if in bounds.
    #[must_use]
    pub fn get(&self, pos: WorldPosition) -> Option<&Tile> {
        if self.in_bounds(pos) {
            Some(&self.tiles[self.index(pos)])
        } else {
            None
        }
    }

    /// Get a mutable reference to the tile at a position, if in bounds.
    pub fn get_mut(&mut self, pos: WorldPosition) -> Option<&mut Tile> {
        if self.in_bounds(pos) {
            let idx = self.index(pos);
            Some(&mut self.tiles[idx])
        } else {
            None
        }
    }

    /// Iterate over all positions and their tiles.
    pub fn iter(&self) -> impl Iterator<Item = (WorldPosition, &Tile)> {
        self.tiles.iter().enumerate().map(|(i, tile)| {
            let x = (i as u32) % self.width;
            let y = (i as u32) / self.width;
            (WorldPosition::new(x, y), tile)
        })
    }

    /// Iterate over all positions and their tiles mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (WorldPosition, &mut Tile)> {
        let width = self.width;
        self.tiles.iter_mut().enumerate().map(move |(i, tile)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            (WorldPosition::new(x, y), tile)
        })
    }

    /// Get walkable neighbor positions from a given position.
    #[must_use]
    pub fn walkable_neighbors(&self, pos: WorldPosition) -> Vec<WorldPosition> {
        pos.neighbors(self.width, self.height)
            .into_iter()
            .filter(|p| {
                self.get(*p)
                    .map(|t| t.terrain.is_walkable())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Count tiles matching a terrain type.
    #[must_use]
    pub fn count_terrain(&self, terrain: Terrain) -> usize {
        self.tiles.iter().filter(|t| t.terrain == terrain).count()
    }

    /// Regenerate grass on all fertile tiles.
    pub fn regenerate_all_grass(&mut self, rate: f64) {
        for tile in &mut self.tiles {
            tile.regenerate_grass(rate);
        }
    }

    /// Convert a position to a flat index.
    const fn index(&self, pos: WorldPosition) -> usize {
        (pos.y * self.width + pos.x) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_world_has_correct_dimensions() {
        let map = WorldMap::flat(10, 8, Terrain::Dirt);
        assert_eq!(map.width(), 10);
        assert_eq!(map.height(), 8);
        assert_eq!(map.tile_count(), 80);
    }

    #[test]
    fn get_returns_correct_tile() {
        let map = WorldMap::flat(5, 5, Terrain::Rock);
        let tile = map.get(WorldPosition::new(2, 3)).unwrap();
        assert_eq!(tile.terrain, Terrain::Rock);
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let map = WorldMap::flat(5, 5, Terrain::Dirt);
        assert!(map.get(WorldPosition::new(5, 0)).is_none());
        assert!(map.get(WorldPosition::new(0, 5)).is_none());
    }

    #[test]
    fn in_bounds_check() {
        let map = WorldMap::flat(10, 10, Terrain::Dirt);
        assert!(map.in_bounds(WorldPosition::new(0, 0)));
        assert!(map.in_bounds(WorldPosition::new(9, 9)));
        assert!(!map.in_bounds(WorldPosition::new(10, 0)));
    }

    #[test]
    fn count_terrain_on_flat_map() {
        let map = WorldMap::flat(4, 4, Terrain::Water);
        assert_eq!(map.count_terrain(Terrain::Water), 16);
        assert_eq!(map.count_terrain(Terrain::Dirt), 0);
    }

    #[test]
    fn walkable_neighbors_avoids_water() {
        // Create a 3x3 map: center is dirt, surround with water except east
        let mut tiles = vec![Tile::new(Terrain::Water, 0.0); 9];
        tiles[4] = Tile::new(Terrain::Dirt, 0.5); // center (1,1)
        tiles[5] = Tile::new(Terrain::Dirt, 0.5); // east (2,1)
        let map = WorldMap::new(3, 3, tiles);

        let walkable = map.walkable_neighbors(WorldPosition::new(1, 1));
        assert_eq!(walkable.len(), 1);
        assert_eq!(walkable[0], WorldPosition::new(2, 1));
    }

    #[test]
    fn iter_covers_all_tiles() {
        let map = WorldMap::flat(3, 4, Terrain::Dirt);
        assert_eq!(map.iter().count(), 12);
    }

    #[test]
    fn regenerate_all_grass_works() {
        let mut map = WorldMap::flat(2, 2, Terrain::Dirt);
        // Drain all grass
        for (_, tile) in map.iter_mut() {
            tile.grass = 0.0;
        }
        map.regenerate_all_grass(1.0);
        // All tiles should be back to max
        for (_, tile) in map.iter() {
            assert_eq!(tile.grass, 1.0);
        }
    }

    #[test]
    #[should_panic(expected = "tiles.len() must equal width * height")]
    fn new_panics_on_wrong_tile_count() {
        let _ = WorldMap::new(5, 5, vec![]);
    }
}
