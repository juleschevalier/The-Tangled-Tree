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

/// Grass regeneration rate per tick.
/// Slightly higher than before — grass is now a weaker food source so it
/// must replenish faster to keep the baseline pressure steady.
const GRASS_REGEN_RATE: f64 = 0.005;

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

    // Save cumulative death-cause totals before the tick overwrites the state
    let prev_by_starvation = state.0.deaths_by_starvation;
    let prev_by_exhaustion = state.0.deaths_by_exhaustion;
    let prev_by_age = state.0.deaths_by_age;

    let mut new_state = SimulationTick::step(
        &mut population.creatures,
        &mut world_map.0,
        CreatureConfig::default(),
        MutationConfig::default(),
        current_tick,
        GRASS_REGEN_RATE,
    );

    // Accumulate death-cause counters across ticks
    new_state.deaths_by_starvation += prev_by_starvation;
    new_state.deaths_by_exhaustion += prev_by_exhaustion;
    new_state.deaths_by_age += prev_by_age;

    state.0 = new_state;
}
