//! Tilemap renderer — displays the world map as an isometric tilemap.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tangled_core::domain::world::{Terrain, WorldMap};

/// Tile size for the isometric tilemap (in pixels).
pub const TILE_SIZE: f32 = 64.0;
pub const HALF_TILE: f32 = TILE_SIZE / 2.0;

/// Plugin that handles rendering the world map as a tilemap.
pub struct TilemapRendererPlugin;

impl Plugin for TilemapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terrain_tilemap.run_if(resource_exists::<WorldMapResource>));
    }
}

/// Resource wrapping the domain WorldMap so Bevy can access it.
#[derive(Resource)]
pub struct WorldMapResource(pub WorldMap);

/// System that creates the tilemap from the WorldMap resource.
fn setup_terrain_tilemap(
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
        y: HALF_TILE,
    };
    let grid_size = tile_size.into();

    // Load the terrain texture atlas
    let texture_handle: Handle<Image> = asset_server.load("sprites/terrain.png");

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Populate tiles
    for (pos, tile) in map.iter() {
        let tile_pos = TilePos {
            x: pos.x,
            y: pos.y,
        };
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

/// Map domain terrain types to tile atlas indices.
///
/// The texture atlas layout is:
/// ```text
/// [0: Grass] [1: Water] [2: Rock] [3: Sand]
/// ```
const fn terrain_to_tile_index(terrain: Terrain) -> u32 {
    match terrain {
        Terrain::Grass => 0,
        Terrain::Water => 1,
        Terrain::Rock => 2,
        Terrain::Sand => 3,
    }
}
