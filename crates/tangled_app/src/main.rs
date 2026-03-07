//! # The Tangled Tree
//!
//! Application entry point. Wires together the domain (`tangled_core`),
//! the Bevy adapter (`tangled_bevy`), and the persistence adapter
//! (`tangled_persistence`), then launches the simulation.

use bevy::prelude::*;
use tangled_bevy::plugins::TangledPlugin;
use tangled_core::domain::world::WorldConfig;

fn main() {
    // World generation config — change seed for a different world
    let world_config = WorldConfig::with_seed(42);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Tangled Tree".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TangledPlugin { world_config })
        .run();
}
