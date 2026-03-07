//! Camera system — pan with keyboard / left-mouse drag, zoom with scroll wheel.

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Plugin providing a 2D camera with pan (WASD / arrow keys / left-click drag)
/// and zoom (scroll wheel).
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (camera_keyboard_pan, camera_mouse_drag, camera_zoom),
        );
    }
}

/// Marker component for the main game camera.
#[derive(Component)]
pub struct MainCamera;

/// Camera movement speed in pixels per second.
const PAN_SPEED: f32 = 500.0;

/// Zoom speed factor per scroll tick.
const ZOOM_SPEED: f32 = 0.1;

/// Minimum zoom (most zoomed in).
const MIN_ZOOM: f32 = 0.2;

/// Maximum zoom (most zoomed out).
const MAX_ZOOM: f32 = 5.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Pan the camera with WASD / ZQSD / arrow keys.
fn camera_keyboard_pan(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = query.single_mut();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::KeyZ)
        || keyboard.pressed(KeyCode::ArrowUp)
    {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyQ)
        || keyboard.pressed(KeyCode::ArrowLeft)
    {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        // Scale pan speed by current zoom level so it feels natural
        let zoom = transform.scale.x;
        transform.translation.x += direction.x * PAN_SPEED * zoom * time.delta_secs();
        transform.translation.y += direction.y * PAN_SPEED * zoom * time.delta_secs();
    }
}

/// Pan the camera by dragging with left mouse button held.
///
/// Skips dragging when the pointer is over an egui window so HUD clicks
/// don't move the map.
fn camera_mouse_drag(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    mut egui_contexts: EguiContexts,
) {
    // Consume events even if we skip, to avoid stale deltas
    let deltas: Vec<Vec2> = motion_events.read().map(|e| e.delta).collect();

    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    // Don't pan when interacting with the egui HUD
    if egui_contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let total_delta: Vec2 = deltas.into_iter().sum();
    if total_delta == Vec2::ZERO {
        return;
    }

    let mut transform = query.single_mut();
    let zoom = transform.scale.x;

    // Mouse moves in screen space; invert Y for world space, scale by zoom
    transform.translation.x -= total_delta.x * zoom;
    transform.translation.y += total_delta.y * zoom;
}

/// Zoom the camera with the scroll wheel.
fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = query.single_mut();

    for event in scroll_events.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };

        let zoom_factor = 1.0 - scroll_amount * ZOOM_SPEED;
        let new_scale = (transform.scale.x * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
        transform.scale = Vec3::splat(new_scale);
    }
}
