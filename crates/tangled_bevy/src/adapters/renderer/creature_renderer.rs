//! Creature renderer — spawns Bevy entities for domain creatures on the isometric map.
//!
//! Visual encoding:
//! - **Size**  → `SizeGene` (circle radius 4–12 px)
//! - **Color** → `DietGene` (green/yellow/red) × age (pale → saturated)
//! - **Vitality bar** → single energy gauge (green→red), shown only when low

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::EguiContexts;
use tangled_core::domain::creatures::{Creature, CreatureConfig, CreatureId, CreatureSpawner};
use tangled_core::domain::genetics::Diet;
use tangled_core::domain::world::WorldConfig;

use super::tilemap_renderer::{
    TilemapInfo, WorldMapResource, diamond_tile_to_world, setup_terrain_sprites,
};
use crate::plugins::camera::MainCamera;

// ─── Constants ──────────────────────────────────────────────────

/// Z layer for creature sprites (above tiles and trees).
const CREATURE_Z: f32 = 1.0;

/// Circle size range in pixels, mapped from SizeGene (0.5–2.5).
const MIN_CREATURE_PX: f32 = 4.0;
const MAX_CREATURE_PX: f32 = 12.0;

/// SizeGene value range (mirrors `SizeGene::MIN` / `SizeGene::MAX`).
const SIZE_GENE_MIN: f32 = 0.5;
const SIZE_GENE_MAX: f32 = 2.5;

/// Number of creatures to spawn initially.
const INITIAL_POPULATION: usize = 40;

/// Vitality bar dimensions.
const BAR_WIDTH: f32 = 14.0;
const BAR_HEIGHT: f32 = 2.0;
/// Vertical offset above the creature center.
const BAR_Y_OFFSET: f32 = 8.0;

/// Click-detection radius in world pixels.
const CLICK_RADIUS: f32 = 12.0;

/// Thresholds below which the vitality bar becomes visible.
const DANGER_ENERGY: f32 = 50.0;

// ─── Plugin ─────────────────────────────────────────────────────

/// Plugin that handles spawning and rendering creatures.
pub struct CreatureRendererPlugin;

impl Plugin for CreatureRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedCreature(None))
            .add_systems(
                Startup,
                spawn_initial_creatures
                    .run_if(resource_exists::<WorldMapResource>)
                    .after(setup_terrain_sprites),
            )
            .add_systems(
                Update,
                (
                    creature_click_system,
                    sync_creature_sprites.run_if(resource_exists::<PopulationResource>),
                ),
            );
    }
}

// ─── Components & Resources ─────────────────────────────────────

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

/// Resource holding the shared white circle texture handle (tinted per creature).
#[derive(Resource)]
struct CreatureTextureHandle(Handle<Image>);

/// Resource holding a 1×1 white pixel texture for the vitality bars.
#[derive(Resource)]
struct BarTextureHandle(Handle<Image>);

/// Marker for the energy vitality bar.
#[derive(Component)]
struct EnergyBar;

/// Resource tracking the currently selected creature (clicked by user).
#[derive(Resource, Default)]
pub struct SelectedCreature(pub Option<CreatureId>);

// ─── Spawn system ───────────────────────────────────────────────

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

    // Shared white circle texture — tinted per-creature via Sprite::color
    let circle_image = create_white_circle(16);
    let circle_handle = images.add(circle_image);
    commands.insert_resource(CreatureTextureHandle(circle_handle.clone()));

    // Shared 1×1 white pixel for vitality bars
    let bar_image = create_white_pixel();
    let bar_handle = images.add(bar_image);
    commands.insert_resource(BarTextureHandle(bar_handle.clone()));

    for creature in &creatures {
        spawn_creature_entity(
            &mut commands,
            creature,
            grid_size,
            offset,
            &circle_handle,
            &bar_handle,
        );
    }

    info!("Spawned {} creatures on the map", creatures.len());
    commands.insert_resource(PopulationResource { creatures });
}

/// Spawn a single creature entity with child bar entities.
fn spawn_creature_entity(
    commands: &mut Commands,
    creature: &Creature,
    grid_size: Vec2,
    offset: Vec2,
    circle_handle: &Handle<Image>,
    bar_handle: &Handle<Image>,
) {
    let world_pos =
        diamond_tile_to_world(creature.position.x, creature.position.y, grid_size, offset);
    let size_px = creature_size_px(creature);
    let color = creature_color(creature);

    commands
        .spawn((
            Sprite {
                image: circle_handle.clone(),
                color,
                custom_size: Some(Vec2::splat(size_px)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(CREATURE_Z)),
            Visibility::Inherited,
            CreatureMarker {
                creature_id: creature.id,
            },
        ))
        .with_children(|parent| {
            // Energy bar (green→red gradient via color, full width)
            parent.spawn((
                Sprite {
                    image: bar_handle.clone(),
                    color: Color::srgb(0.2, 0.85, 0.2),
                    custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                Transform::from_translation(Vec3::new(-BAR_WIDTH / 2.0, BAR_Y_OFFSET, 0.1)),
                Visibility::Hidden,
                EnergyBar,
            ));
        });
}

// ─── Click-to-select system ─────────────────────────────────────

/// System: detect left-click on a creature sprite to select it.
fn creature_click_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    population: Option<Res<PopulationResource>>,
    mut selected: ResMut<SelectedCreature>,
    mut egui_contexts: EguiContexts,
    tilemap_info: Option<Res<TilemapInfo>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't select when clicking on egui panels
    let ctx = egui_contexts.ctx_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        return;
    }

    let Some(population) = population else { return };
    let Some(tilemap_info) = tilemap_info else {
        return;
    };

    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera_q.single();
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let grid_size = tilemap_info.grid_size;
    let offset = tilemap_info.offset;

    // Find the closest living creature to the click point
    let mut best: Option<(CreatureId, f32)> = None;
    for creature in &population.creatures {
        if !creature.is_alive() {
            continue;
        }
        let cpos = diamond_tile_to_world(
            creature.position.x,
            creature.position.y,
            grid_size,
            offset,
        );
        let dist = world_pos.distance(cpos);
        if dist < CLICK_RADIUS && (best.is_none() || dist < best.unwrap().1) {
            best = Some((creature.id, dist));
        }
    }

    selected.0 = best.map(|(id, _)| id);
}

// ─── Sync system ────────────────────────────────────────────────

/// System: synchronise Bevy sprites with domain creature state each frame.
#[allow(clippy::type_complexity)]
fn sync_creature_sprites(
    mut commands: Commands,
    population: Res<PopulationResource>,
    mut creature_query: Query<(
        Entity,
        &CreatureMarker,
        &mut Transform,
        &mut Visibility,
        &mut Sprite,
        &Children,
    )>,
    mut energy_bars: Query<
        (&mut Visibility, &mut Sprite),
        (With<EnergyBar>, Without<CreatureMarker>),
    >,
    tilemap_info: Res<TilemapInfo>,
    texture: Res<CreatureTextureHandle>,
    bar_texture: Res<BarTextureHandle>,
) {
    let grid_size = tilemap_info.grid_size;
    let offset = tilemap_info.offset;

    let mut existing_ids: HashSet<u64> = HashSet::new();

    for (_, marker, mut transform, mut vis, mut sprite, children) in &mut creature_query {
        existing_ids.insert(marker.creature_id.0);

        let Some(creature) = population
            .creatures
            .iter()
            .find(|c| c.id == marker.creature_id)
        else {
            *vis = Visibility::Hidden;
            continue;
        };

        if !creature.is_alive() {
            *vis = Visibility::Hidden;
            continue;
        }

        // Update position
        *vis = Visibility::Inherited;
        let world_pos =
            diamond_tile_to_world(creature.position.x, creature.position.y, grid_size, offset);
        transform.translation = world_pos.extend(CREATURE_Z);

        // Update color and size
        sprite.color = creature_color(creature);
        sprite.custom_size = Some(Vec2::splat(creature_size_px(creature)));

        // Update vitality bar
        let in_danger = creature.energy < DANGER_ENERGY;

        let energy_ratio = (creature.energy / 100.0).clamp(0.0, 1.0);

        for &child in children.iter() {
            if let Ok((mut bar_vis, mut bar_sprite)) = energy_bars.get_mut(child) {
                *bar_vis = if in_danger {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                bar_sprite.custom_size =
                    Some(Vec2::new(BAR_WIDTH * energy_ratio, BAR_HEIGHT));
                // Color: green when full → red when empty
                bar_sprite.color = Color::srgb(
                    1.0 - energy_ratio,
                    energy_ratio * 0.85,
                    0.1,
                );
            }
        }
    }

    // Spawn sprites for newborns not yet represented
    for creature in &population.creatures {
        if existing_ids.contains(&creature.id.0) || !creature.is_alive() {
            continue;
        }
        spawn_creature_entity(
            &mut commands,
            creature,
            grid_size,
            offset,
            &texture.0,
            &bar_texture.0,
        );
    }
}

// ─── Visual helpers ─────────────────────────────────────────────

/// Map a creature's SizeGene to a pixel diameter.
fn creature_size_px(creature: &Creature) -> f32 {
    let size = creature.genome.expressed_size();
    let t = ((size - SIZE_GENE_MIN) / (SIZE_GENE_MAX - SIZE_GENE_MIN)).clamp(0.0, 1.0);
    MIN_CREATURE_PX + t * (MAX_CREATURE_PX - MIN_CREATURE_PX)
}

/// Compute the creature's display color from Diet and Age.
///
/// - Diet sets the base hue: Herbivore=green, Omnivore=yellow, Carnivore=red
/// - Age modulates saturation: young=pale/pastel → old=rich/saturated
fn creature_color(creature: &Creature) -> Color {
    let config = CreatureConfig::default();
    let age_ratio = (creature.age_ticks as f32 / config.max_age_ticks as f32).clamp(0.0, 1.0);

    // Pale → saturated: interpolate between a muted pastel and a vivid tone.
    // saturation factor: 0.3 (young) → 1.0 (old)
    let sat = 0.3 + 0.7 * age_ratio;

    let (r, g, b) = match creature.genome.expressed_diet() {
        Diet::Herbivore => (0.2 * sat, 0.4 + 0.5 * sat, 0.15 * sat),
        Diet::Omnivore => (0.5 + 0.4 * sat, 0.4 + 0.3 * sat, 0.1 * sat),
        Diet::Carnivore => (0.5 + 0.45 * sat, 0.15 * sat, 0.1 * sat),
    };

    Color::srgb(r, g, b)
}

// ─── Texture generation ─────────────────────────────────────────

/// Create a white circle image with the given diameter (tinted at render time).
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

/// Create a 1×1 white pixel image (used for scalable bar sprites).
fn create_white_pixel() -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
