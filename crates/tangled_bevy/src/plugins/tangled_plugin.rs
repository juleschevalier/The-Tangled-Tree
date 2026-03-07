//! Main Tangled Tree plugin — wires all sub-plugins together.

use bevy::prelude::*;

use crate::adapters::renderer::TilemapRendererPlugin;
use crate::adapters::renderer::creature_renderer::{CreatureRendererPlugin, WorldConfigResource};
use crate::adapters::renderer::tilemap_renderer::WorldMapResource;
use crate::adapters::stats_reporter::StatsHudPlugin;
use crate::adapters::world_generator::PerlinWorldGenerator;
use crate::plugins::camera::CameraPlugin;
use crate::plugins::simulation_plugin::SimulationPlugin;
use tangled_core::domain::world::WorldConfig;
use tangled_core::ports::outbound::WorldGenerator;

/// The main plugin for The Tangled Tree.
///
/// Generates the world, sets up rendering, and configures the camera.
#[derive(Default)]
pub struct TangledPlugin {
    /// World generation configuration.
    pub world_config: WorldConfig,
}

impl Plugin for TangledPlugin {
    fn build(&self, app: &mut App) {
        // Generate the world (domain logic via adapter)
        let generator = PerlinWorldGenerator::default();
        let world_map = generator.generate(&self.world_config);

        info!(
            "World generated: {}x{} tiles (seed: {})",
            world_map.width(),
            world_map.height(),
            self.world_config.seed
        );

        // Insert as Bevy resources
        app.insert_resource(WorldMapResource(world_map));
        app.insert_resource(WorldConfigResource(self.world_config.clone()));

        // Register sub-plugins
        app.add_plugins(TilemapRendererPlugin)
            .add_plugins(CreatureRendererPlugin)
            .add_plugins(SimulationPlugin)
            .add_plugins(StatsHudPlugin)
            .add_plugins(CameraPlugin);
    }
}
