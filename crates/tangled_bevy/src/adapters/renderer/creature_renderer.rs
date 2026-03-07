//! Creature renderer — spawns Bevy entities for domain creatures on the isometric map.

use bevy::prelude::*;
use tangled_core::domain::creatures::{Creature, CreatureId, CreatureSpawner};
use tangled_core::domain::world::WorldConfig;

use super::tilemap_renderer::{WorldMapResource, HALF_TILE};

/// Plugin that handles spawning and rendering creatures.
pub struct CreatureRendererPlugin;

impl Plugin for CreatureRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_initial_creatures
                .run_if(resource_exists::<WorldMapResource>)
                .after(super::tilemap_renderer::setup_terrain_tilemap),
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

/// Number of creatures to spawn initially.
const INITIAL_POPULATION: usize = 20;

/// System: spawn domain creatures then create Bevy sprite entities.
fn spawn_initial_creatures(
    mut commands: Commands,
    world_map: Res<WorldMapResource>,
    world_config: Res<WorldConfigResource>,
    asset_server: Res<AssetServer>,
) {
    let creatures = CreatureSpawner::spawn_initial(
        &world_map.0,
        INITIAL_POPULATION,
        world_config.0.seed.wrapping_add(0xCAFE),
    );

    let texture: Handle<Image> = asset_server.load("sprites/creature.png");

    for creature in &creatures {
        let (px, py) = world_to_iso_pixel(creature.position.x, creature.position.y);

        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(px, py, 1.0)),
            CreatureMarker {
                creature_id: creature.id,
            },
        ));
    }

    info!("Spawned {} creatures", creatures.len());

    commands.insert_resource(PopulationResource { creatures });
}

/// Resource wrapping WorldConfig for Bevy access.
#[derive(Resource)]
pub struct WorldConfigResource(pub WorldConfig);

/// Convert grid coordinates to isometric pixel position (Diamond layout).
///
/// Matches bevy_ecs_tilemap's Diamond coordinate system.
fn world_to_iso_pixel(x: u32, y: u32) -> (f32, f32) {
    let x = x as f32;
    let y = y as f32;
    let px = (x - y) * HALF_TILE;
    let py = (x + y) * HALF_TILE * 0.5;
    (px, py)
}
