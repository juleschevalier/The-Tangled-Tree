//! Camera system — pan with keyboard/middle mouse, zoom with scroll wheel.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

/// Plugin providing a 2D camera with pan (WASD / arrow keys / middle-drag)
/// and zoom (scroll wheel).
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_pan, camera_zoom));
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

/// Pan the camera with WASD / arrow keys.
fn camera_pan(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = query.single_mut();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
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
