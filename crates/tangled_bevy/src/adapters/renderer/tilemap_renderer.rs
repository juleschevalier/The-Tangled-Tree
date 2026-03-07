//! Tilemap renderer — displays the world map as an isometric tilemap.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tangled_core::domain::world::{Terrain, WorldMap};

/// Tile pixel dimensions in the spritesheet (32×32 px per tile).
pub const TILE_SIZE: f32 = 32.0;
pub const HALF_TILE: f32 = TILE_SIZE / 2.0;

/// Plugin that handles rendering the world map as a tilemap.
pub struct TilemapRendererPlugin;

impl Plugin for TilemapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_terrain_tilemap.run_if(resource_exists::<WorldMapResource>),
        );
    }
}

/// Resource wrapping the domain WorldMap so Bevy can access it.
#[derive(Resource)]
pub struct WorldMapResource(pub WorldMap);

/// System that creates the tilemap from the WorldMap resource.
pub(crate) fn setup_terrain_tilemap(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    asset_server: Res<AssetServer>,
) {
    let map = &world_map.0;
    let map_size = TilemapSize {
        x: map.width(),
        y: map.height(),
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    // Diamond isometric requires a 2:1 width:height grid spacing for the
    // correct perspective — independent of the sprite's actual pixel size.
    let grid_size = TilemapGridSize {
        x: TILE_SIZE,
        y: HALF_TILE,
    };

    // Load the terrain spritesheet atlas (32×32 px per tile)
    // TODO: rename file to match your actual asset filename
    let texture_handle: Handle<Image> = asset_server.load("sprites/tileset.png");

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Populate tiles
    for (pos, tile) in map.iter() {
        let tile_pos = TilePos { x: pos.x, y: pos.y };
        let texture_index = terrain_to_tile_index(tile.terrain);

        let tile_entity = commands
            .spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(texture_index),
                ..Default::default()
            })
            .id();

        tile_storage.set(&tile_pos, tile_entity);
    }

    // Configure the tilemap
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

/// Map domain terrain types to spritesheet tile indices.
///
/// Spritesheet: 32×32 px per tile, row-major order.
/// ```text
/// Grass      → 23  (Herbe)
/// Water      → 111 (Eau)
/// Rock       → 18  (Terre)
/// Sand       → 37  (Herbe haute — closest available)
/// ```
const fn terrain_to_tile_index(terrain: Terrain) -> u32 {
    match terrain {
        Terrain::Grass => 23,
        Terrain::Water => 111,
        Terrain::Rock => 18,
        Terrain::Sand => 37,
    }
}
