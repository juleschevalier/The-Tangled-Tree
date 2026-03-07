//! Creature renderer — spawns Bevy entities for domain creatures on the isometric map.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tangled_core::domain::creatures::{Creature, CreatureId, CreatureSpawner};
use tangled_core::domain::world::WorldConfig;

use super::tilemap_renderer::{WorldMapResource, setup_terrain_tilemap};

/// Plugin that handles spawning and rendering creatures.
pub struct CreatureRendererPlugin;

impl Plugin for CreatureRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_initial_creatures
                .run_if(resource_exists::<WorldMapResource>)
                .after(setup_terrain_tilemap),
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

/// Number of creatures to spawn initially.
const INITIAL_POPULATION: usize = 20;

/// System: spawn domain creatures then create Bevy sprite entities.
///
/// Queries the actual tilemap entity's [`TilemapGridSize`] and [`Transform`] so
/// creature world positions are computed with the exact same Diamond basis as
/// `bevy_ecs_tilemap` uses internally.
fn spawn_initial_creatures(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    world_config: Res<WorldConfigResource>,
    asset_server: Res<AssetServer>,
    tilemap_query: Query<(&TilemapGridSize, &Transform), With<TilemapSize>>,
) {
    let creatures = CreatureSpawner::spawn_initial(
        &world_map.0,
        INITIAL_POPULATION,
        world_config.0.seed.wrapping_add(0xCAFE),
    );

    let (grid_size, tilemap_transform) = match tilemap_query.get_single() {
        Ok(v) => v,
        Err(e) => {
            error!("Could not find tilemap entity for creature projection: {e}");
            return;
        }
    };
    let tilemap_offset = tilemap_transform.translation.truncate();

    let texture: Handle<Image> = asset_server.load("sprites/creature.png");

    for creature in &creatures {
        let world_pos = diamond_tile_to_world(
            creature.position.x,
            creature.position.y,
            grid_size,
            tilemap_offset,
        );

        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::new(30.0, 20.0)),
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

/// Project a Diamond isometric grid position to world space.
///
/// Replicates `bevy_ecs_tilemap`'s internal `DIAMOND_BASIS` matrix:
/// ```text
/// DIAMOND_BASIS = | 0.5   0.5 |     unscaled.x = 0.5*(x + y)
///                 |-0.5   0.5 |     unscaled.y = 0.5*(y - x)
/// ```
/// Then scaled by `grid_size` and offset by the tilemap's world transform.
fn diamond_tile_to_world(
    x: u32,
    y: u32,
    grid_size: &TilemapGridSize,
    tilemap_offset: Vec2,
) -> Vec2 {
    let xf = x as f32;
    let yf = y as f32;
    let local = Vec2::new(grid_size.x * 0.5 * (xf + yf), grid_size.y * 0.5 * (yf - xf));
    tilemap_offset + local
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
    tilemap_query: Query<
        (&TilemapGridSize, &Transform),
        (With<TilemapSize>, Without<CreatureMarker>),
    >,
    asset_server: Res<AssetServer>,
) {
    let Ok((grid_size, tilemap_tf)) = tilemap_query.get_single() else {
        return;
    };
    let tilemap_offset = tilemap_tf.translation.truncate();

    // Track which creature IDs already have a sprite entity
    let mut existing_ids: HashSet<u64> = HashSet::new();

    for (marker, mut transform, mut visibility) in &mut query {
        existing_ids.insert(marker.creature_id.0);

        // Find the matching domain creature
        let Some(creature) = population
            .creatures
            .iter()
            .find(|c| c.id == marker.creature_id)
        else {
            // Creature removed from domain — hide it
            *visibility = Visibility::Hidden;
            continue;
        };

        if !creature.is_alive() {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
            let world_pos = diamond_tile_to_world(
                creature.position.x,
                creature.position.y,
                grid_size,
                tilemap_offset,
            );
            transform.translation = world_pos.extend(1.0);
        }
    }

    // Spawn sprites for newborns not yet represented
    let texture: Handle<Image> = asset_server.load("sprites/creature.png");
    for creature in &population.creatures {
        if existing_ids.contains(&creature.id.0) || !creature.is_alive() {
            continue;
        }
        let world_pos = diamond_tile_to_world(
            creature.position.x,
            creature.position.y,
            grid_size,
            tilemap_offset,
        );
        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::new(30.0, 20.0)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
            CreatureMarker {
                creature_id: creature.id,
            },
        ));
    }
}
