//! World representation — map, tiles, resources.
//!
//! The world is a grid of tiles with terrain types, elevation,
//! and resources that creatures interact with.

mod terrain;
mod tile;
mod world_config;
mod world_map;
mod world_position;

pub use terrain::Terrain;
pub use tile::Tile;
pub use world_config::WorldConfig;
pub use world_map::WorldMap;
pub use world_position::WorldPosition;
