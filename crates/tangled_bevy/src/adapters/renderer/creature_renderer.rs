//! Creature renderer — spawns Bevy entities for domain creatures on the isometric map.
//!
//! Creatures are rendered as red circles positioned using the diamond iso projection.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use tangled_core::domain::creatures::{Creature, CreatureId, CreatureSpawner};
use tangled_core::domain::world::WorldConfig;

use super::tilemap_renderer::{
    TilemapInfo, WorldMapResource, diamond_tile_to_world, setup_terrain_sprites,
};

/// Plugin that handles spawning and rendering creatures.
pub struct CreatureRendererPlugin;

impl Plugin for CreatureRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_initial_creatures
                .run_if(resource_exists::<WorldMapResource>)
                .after(setup_terrain_sprites),
        )
        .add_systems(
            Update,
            sync_creature_sprites.run_if(resource_exists::<PopulationResource>),
        );
    }
}

/// Bevy marker component linking an entity to a domain creature.
#[derive(Component)]
pub struct CreatureMarker {
    pub creature_id: CreatureId,
}

/// Resource holding all domain creatures.
#[derive(Resource)]
pub struct PopulationResource {
    pub creatures: Vec<Creature>,
}

/// Resource wrapping WorldConfig for Bevy access.
#[derive(Resource)]
pub struct WorldConfigResource(pub WorldConfig);

/// Resource holding the shared red circle texture handle.
#[derive(Resource)]
struct CreatureTextureHandle(Handle<Image>);

/// Number of creatures to spawn initially.
const INITIAL_POPULATION: usize = 40;

/// Creature circle diameter in pixels.
const CREATURE_SIZE: f32 = 10.0;

/// System: spawn domain creatures then create Bevy sprite entities.
fn spawn_initial_creatures(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    world_config: Res<WorldConfigResource>,
    tilemap_info: Res<TilemapInfo>,
    mut images: ResMut<Assets<Image>>,
) {
    let creatures = CreatureSpawner::spawn_initial(
        &world_map.0,
        INITIAL_POPULATION,
        world_config.0.seed.wrapping_add(0xCAFE),
    );

    let grid_size = tilemap_info.grid_size;
    let offset = tilemap_info.offset;

    // Create a shared red circle texture
    let circle_image = create_circle_image(16, Color::srgb(0.9, 0.15, 0.15));
    let circle_handle = images.add(circle_image);
    commands.insert_resource(CreatureTextureHandle(circle_handle.clone()));

    for creature in &creatures {
        let world_pos =
            diamond_tile_to_world(creature.position.x, creature.position.y, grid_size, offset);

        commands.spawn((
            Sprite {
                image: circle_handle.clone(),
                custom_size: Some(Vec2::splat(CREATURE_SIZE)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
            CreatureMarker {
                creature_id: creature.id,
            },
        ));
    }

    info!("Spawned {} creatures on the map", creatures.len());
    commands.insert_resource(PopulationResource { creatures });
}

/// System: synchronise Bevy sprites with domain creature state each frame.
///
/// - **Dead creatures** are hidden (`Visibility::Hidden`)
/// - **Live creatures** have their transform updated to match position
/// - **Newborns** (no matching Bevy entity) get a new sprite entity spawned
fn sync_creature_sprites(
    mut commands: Commands,
    population: Res<PopulationResource>,
    mut query: Query<(&CreatureMarker, &mut Transform, &mut Visibility)>,
    tilemap_info: Res<TilemapInfo>,
    texture: Res<CreatureTextureHandle>,
) {
    let grid_size = tilemap_info.grid_size;
    let offset = tilemap_info.offset;

    // Track which creature IDs already have a sprite entity
    let mut existing_ids: HashSet<u64> = HashSet::new();

    for (marker, mut transform, mut visibility) in &mut query {
        existing_ids.insert(marker.creature_id.0);

        let Some(creature) = population
            .creatures
            .iter()
            .find(|c| c.id == marker.creature_id)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if !creature.is_alive() {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
            let world_pos =
                diamond_tile_to_world(creature.position.x, creature.position.y, grid_size, offset);
            transform.translation = world_pos.extend(1.0);
        }
    }

    // Spawn sprites for newborns not yet represented
    for creature in &population.creatures {
        if existing_ids.contains(&creature.id.0) || !creature.is_alive() {
            continue;
        }
        let world_pos =
            diamond_tile_to_world(creature.position.x, creature.position.y, grid_size, offset);
        commands.spawn((
            Sprite {
                image: texture.0.clone(),
                custom_size: Some(Vec2::splat(CREATURE_SIZE)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
            CreatureMarker {
                creature_id: creature.id,
            },
        ));
    }
}

/// Create a circle-shaped image with the given radius and color.
///
/// Generates a (diameter × diameter) RGBA image where pixels inside the circle
/// are filled with the given color and outside are transparent.
fn create_circle_image(radius: u32, color: Color) -> Image {
    let diameter = radius * 2;
    let cx = radius as f32;
    let cy = radius as f32;
    let r = radius as f32;
    let mut data = vec![0u8; (diameter * diameter * 4) as usize];

    let linear = color.to_linear();

    // Convert linear back to sRGB 0-255 for the texture
    let cr = (linear.red.powf(1.0 / 2.2) * 255.0) as u8;
    let cg = (linear.green.powf(1.0 / 2.2) * 255.0) as u8;
    let cb = (linear.blue.powf(1.0 / 2.2) * 255.0) as u8;

    for y in 0..diameter {
        for x in 0..diameter {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * diameter + x) * 4) as usize;
                data[idx] = cr;
                data[idx + 1] = cg;
                data[idx + 2] = cb;
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
