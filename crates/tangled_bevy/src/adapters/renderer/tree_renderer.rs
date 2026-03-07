//! Tree renderer — draws fruit trees on the isometric map.
//!
//! Each tree is rendered as a small colored circle placed on its tile:
//! - **Growing**: dark green (building up energy)
//! - **Fruiting**: orange-yellow (ripe fruit available)
//! - **Resting**: warm brown (recovering after harvest)
//!
//! Tree sprites use z = 0.5 (above terrain tiles, below creatures).

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use tangled_core::domain::world::FruitTreeState;

use super::tilemap_renderer::{WorldMapResource, diamond_tile_to_world, setup_terrain_sprites};
use crate::adapters::renderer::tilemap_renderer::TilemapInfo;

/// Z layer for tree sprites (above tiles at 0.0, below creatures at 1.0).
const TREE_Z: f32 = 0.5;

/// Diameter of the tree circle sprite in pixels.
const TREE_SIZE: f32 = 8.0;

/// Color for a tree in the Growing state.
const COLOR_GROWING: Color = Color::srgb(0.1, 0.45, 0.12);
/// Color for a tree in the Fruiting state (ripe fruit).
const COLOR_FRUITING: Color = Color::srgb(1.0, 0.6, 0.05);
/// Color for a tree in the Resting state (bare / recovering).
const COLOR_RESTING: Color = Color::srgb(0.42, 0.27, 0.12);

/// Plugin that renders fruit trees as colored circles on the isometric map.
pub struct TreeRendererPlugin;

impl Plugin for TreeRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_tree_sprites
                .run_if(resource_exists::<WorldMapResource>)
                .after(setup_terrain_sprites),
        )
        .add_systems(
            Update,
            update_tree_colors.run_if(resource_exists::<WorldMapResource>),
        );
    }
}

/// Marker component linking a sprite entity to a fruit tree by its index in
/// [`WorldMap::trees()`].
#[derive(Component)]
pub struct TreeSprite {
    /// Index into `WorldMap::trees()`.
    pub tree_index: usize,
}

/// System: spawn a circle sprite entity for each fruit tree at startup.
fn spawn_tree_sprites(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    tilemap_info: Res<TilemapInfo>,
    mut images: ResMut<Assets<Image>>,
) {
    let map = &world_map.0;
    let trees = map.trees();

    if trees.is_empty() {
        return;
    }

    let grid_size = tilemap_info.grid_size;
    let offset = tilemap_info.offset;

    // Create a shared white circle texture — tinted per-tree via Sprite::color
    let circle_handle = images.add(create_white_circle(12));

    for (index, tree) in trees.iter().enumerate() {
        let world_pos = diamond_tile_to_world(tree.position.x, tree.position.y, grid_size, offset);
        let color = state_color(tree.state);

        commands.spawn((
            Sprite {
                image: circle_handle.clone(),
                color,
                custom_size: Some(Vec2::splat(TREE_SIZE)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(TREE_Z)),
            TreeSprite { tree_index: index },
        ));
    }

    info!("Spawned {} tree sprites on the map", trees.len());
}

/// System: update tree sprite colors when the world map changes (tree state transitions).
fn update_tree_colors(
    world_map: Res<WorldMapResource>,
    mut query: Query<(&TreeSprite, &mut Sprite)>,
) {
    if !world_map.is_changed() {
        return;
    }
    let trees = world_map.0.trees();
    for (tree_sprite, mut sprite) in &mut query {
        if let Some(tree) = trees.get(tree_sprite.tree_index) {
            sprite.color = state_color(tree.state);
        }
    }
}

/// Map a tree state to its display color.
fn state_color(state: FruitTreeState) -> Color {
    match state {
        FruitTreeState::Growing => COLOR_GROWING,
        FruitTreeState::Fruiting => COLOR_FRUITING,
        FruitTreeState::Resting => COLOR_RESTING,
    }
}

/// Create a white circle image with the given diameter.
///
/// The circle is tinted at render time by `Sprite::color`.
fn create_white_circle(diameter: u32) -> Image {
    let cx = diameter as f32 / 2.0;
    let cy = diameter as f32 / 2.0;
    let r = cx;
    let mut data = vec![0u8; (diameter * diameter * 4) as usize];

    for y in 0..diameter {
        for x in 0..diameter {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * diameter + x) * 4) as usize;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    Image::new(
        Extent3d {
            width: diameter,
            height: diameter,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
