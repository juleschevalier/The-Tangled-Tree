//! Tilemap renderer — displays the world map as colored diamond sprites.
//!
//! Each tile is rendered as a white diamond image tinted with terrain/grass color.
//! No spritesheet required — just programmatic diamond shapes.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use tangled_core::domain::world::{Terrain, WorldMap, WorldPosition};

/// Diamond grid spacing: 2:1 width:height ratio for isometric perspective.
pub const GRID_WIDTH: f32 = 32.0;
pub const GRID_HEIGHT: f32 = 16.0;

/// Plugin that handles rendering the world map as colored diamond sprites.
pub struct TilemapRendererPlugin;

impl Plugin for TilemapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_terrain_sprites.run_if(resource_exists::<WorldMapResource>),
        )
        .add_systems(
            Update,
            update_tile_colors.run_if(resource_exists::<WorldMapResource>),
        );
    }
}

/// Resource wrapping the domain WorldMap so Bevy can access it.
#[derive(Resource)]
pub struct WorldMapResource(pub WorldMap);

/// Resource holding tilemap world-space info (grid size + offset) for other systems.
#[derive(Resource, Clone)]
pub struct TilemapInfo {
    pub grid_size: Vec2,
    pub offset: Vec2,
}

/// Axis-aligned bounding box of the entire map in world space.
/// Used by the camera to prevent panning outside the map.
#[derive(Resource, Clone, Copy)]
pub struct MapBounds {
    pub min: Vec2,
    pub max: Vec2,
}

/// Marker component linking a sprite entity to a world tile position.
#[derive(Component)]
pub struct TileSprite {
    pub x: u32,
    pub y: u32,
}

/// Project a diamond isometric tile position to world space.
///
/// Replicates the Diamond basis used by isometric tilemaps:
/// ```text
/// local.x = grid.x * 0.5 * (tile_x + tile_y)
/// local.y = grid.y * 0.5 * (tile_y - tile_x)
/// ```
pub fn diamond_tile_to_world(x: u32, y: u32, grid_size: Vec2, offset: Vec2) -> Vec2 {
    let xf = x as f32;
    let yf = y as f32;
    let local = Vec2::new(grid_size.x * 0.5 * (xf + yf), grid_size.y * 0.5 * (yf - xf));
    offset + local
}

/// System that creates colored diamond sprites for each tile at startup.
pub(crate) fn setup_terrain_sprites(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    mut images: ResMut<Assets<Image>>,
) {
    let map = &world_map.0;
    let grid_size = Vec2::new(GRID_WIDTH, GRID_HEIGHT);

    // Compute offset to center the map around world origin (0, 0).
    let w = map.width() as f32;
    let h = map.height() as f32;
    let offset = Vec2::new(
        -grid_size.x * 0.25 * (w - 1.0 + h - 1.0),
        -grid_size.y * 0.25 * (h - 1.0 - w + 1.0),
    );

    commands.insert_resource(TilemapInfo { grid_size, offset });

    // Compute world-space AABB of the map from the 4 diamond extremes.
    // Extra padding so edge tiles aren't clipped at the viewport border.
    let padding = Vec2::new(grid_size.x * 2.0, grid_size.y * 2.0);
    let bounds = MapBounds {
        min: Vec2::new(offset.x, offset.y - grid_size.y * 0.5 * (w - 1.0)) - padding,
        max: Vec2::new(
            offset.x + grid_size.x * 0.5 * (w - 1.0 + h - 1.0),
            offset.y + grid_size.y * 0.5 * (h - 1.0),
        ) + padding,
    };
    commands.insert_resource(bounds);

    // Create the shared white diamond texture (tinted per-tile)
    let diamond_image = create_diamond_image();
    let diamond_handle = images.add(diamond_image);

    for (pos, tile) in map.iter() {
        let world_pos = diamond_tile_to_world(pos.x, pos.y, grid_size, offset);
        let color = tile_color(tile.terrain, tile.grass);

        commands.spawn((
            Sprite {
                image: diamond_handle.clone(),
                color,
                custom_size: Some(Vec2::new(GRID_WIDTH + 1.0, GRID_HEIGHT + 1.0)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
            TileSprite { x: pos.x, y: pos.y },
        ));
    }

    info!(
        "Tilemap rendered: {}x{} diamond sprites",
        map.width(),
        map.height()
    );
}

/// System that updates tile sprite colors when the world map changes (grass regen, consumption).
fn update_tile_colors(
    world_map: Res<WorldMapResource>,
    mut query: Query<(&TileSprite, &mut Sprite)>,
) {
    if !world_map.is_changed() {
        return;
    }
    let map = &world_map.0;
    for (tile_sprite, mut sprite) in &mut query {
        if let Some(tile) = map.get(WorldPosition::new(tile_sprite.x, tile_sprite.y)) {
            sprite.color = tile_color(tile.terrain, tile.grass);
        }
    }
}

/// Compute the display color for a tile based on terrain and grass level.
///
/// - Water → blue
/// - Rock → grey
/// - Dirt → brown (grass=0) → light green (grass≈0.5) → dark green (grass=1.0)
fn tile_color(terrain: Terrain, grass: f64) -> Color {
    match terrain {
        Terrain::Water => Color::srgb(0.2, 0.4, 0.85),
        Terrain::Rock => Color::srgb(0.6, 0.6, 0.6),
        Terrain::Dirt => grass_gradient(grass),
    }
}

/// Interpolate dirt color: brown → light green → dark green based on grass level.
fn grass_gradient(grass: f64) -> Color {
    let g = grass as f32;
    if g <= 0.5 {
        // Brown (0.55, 0.35, 0.15) → Light green (0.4, 0.72, 0.2)
        let t = g * 2.0;
        Color::srgb(lerp(0.55, 0.4, t), lerp(0.35, 0.72, t), lerp(0.15, 0.2, t))
    } else {
        // Light green (0.4, 0.72, 0.2) → Dark green (0.1, 0.45, 0.1)
        let t = (g - 0.5) * 2.0;
        Color::srgb(lerp(0.4, 0.1, t), lerp(0.72, 0.45, t), lerp(0.2, 0.1, t))
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Create a white diamond-shaped image (32×16 px).
///
/// Pixels inside the diamond are opaque white; outside are transparent.
/// The `Sprite::color` field tints this to the desired terrain color.
fn create_diamond_image() -> Image {
    let width = 32u32;
    let height = 16u32;
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 + 0.5 - cx).abs() / cx;
            let dy = (y as f32 + 0.5 - cy).abs() / cy;
            if dx + dy <= 1.0 {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
