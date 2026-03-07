//! Simulation plugin — drives the domain simulation tick from Bevy's update loop.

use bevy::prelude::*;
use tangled_core::domain::creatures::CreatureConfig;
use tangled_core::domain::genetics::MutationConfig;
use tangled_core::domain::simulation::{SimulationState, SimulationTick};

use crate::adapters::renderer::creature_renderer::PopulationResource;
use crate::adapters::renderer::tilemap_renderer::WorldMapResource;

/// Plugin that runs the simulation tick every [`TICK_INTERVAL`] seconds.
pub struct SimulationPlugin;

/// Seconds between each simulation tick.
const TICK_INTERVAL: f64 = 0.15;

/// Grass regeneration rate per tick (very slow).
const GRASS_REGEN_RATE: f64 = 0.003;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationTimer(Timer::from_seconds(
            TICK_INTERVAL as f32,
            TimerMode::Repeating,
        )))
        .insert_resource(SimulationStateResource(SimulationState::default()))
        .insert_resource(SimulationRunning(true))
        .add_systems(
            Update,
            simulation_tick_system
                .run_if(resource_exists::<PopulationResource>)
                .run_if(resource_exists::<WorldMapResource>),
        );
    }
}

/// Timer controlling simulation tick rate.
#[derive(Resource)]
pub struct SimulationTimer(pub Timer);

/// Bevy resource with the latest simulation state snapshot.
#[derive(Resource)]
pub struct SimulationStateResource(pub SimulationState);

/// Toggle: pause/resume the simulation.
#[derive(Resource)]
pub struct SimulationRunning(pub bool);

/// Bevy system that advances the domain simulation on each timer tick.
fn simulation_tick_system(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut population: ResMut<PopulationResource>,
    mut world_map: ResMut<WorldMapResource>,
    mut state: ResMut<SimulationStateResource>,
    running: Res<SimulationRunning>,
) {
    if !running.0 {
        return;
    }

    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    let current_tick = state.0.tick + 1;

    let new_state = SimulationTick::step(
        &mut population.creatures,
        &mut world_map.0,
        CreatureConfig::default(),
        MutationConfig::default(),
        current_tick,
        GRASS_REGEN_RATE,
    );

    state.0 = new_state;
}
