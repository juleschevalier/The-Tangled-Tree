//! Stats reporter adapter — displays simulation metrics via `bevy_egui`.
//!
//! Renders a lightweight HUD panel (top-left) showing live simulation state:
//! tick number, alive/dead counts, births and deaths per tick.
//! Also shows a scrollable event log for births and deaths.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use tangled_core::domain::simulation::SimulationEvent;
use tangled_core::domain::world::FruitTreeState;

use crate::adapters::renderer::tilemap_renderer::WorldMapResource;
use crate::plugins::simulation_plugin::{SimulationRunning, SimulationStateResource};

/// Maximum number of events retained in the log.
const MAX_EVENT_LOG: usize = 200;

/// Plugin that overlays an egui HUD with simulation metrics.
pub struct StatsHudPlugin;

impl Plugin for StatsHudPlugin {
    fn build(&self, app: &mut App) {
        // EguiPlugin is idempotent — safe to add even if another adapter adds it too.
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.insert_resource(EventLog::default())
            .add_systems(Update, collect_events)
            .add_systems(Update, stats_hud_system)
            .add_systems(Update, event_log_system);
    }
}

/// Persistent event log resource — accumulates events across ticks.
#[derive(Resource, Default)]
struct EventLog {
    entries: Vec<String>,
}

/// Collect simulation events into the persistent event log.
fn collect_events(state: Option<Res<SimulationStateResource>>, mut log: ResMut<EventLog>) {
    let Some(state) = state else { return };
    if !state.is_changed() {
        return;
    }

    for event in &state.0.events {
        let entry = match event {
            SimulationEvent::Birth { id, tick } => {
                format!("🐣 Tick {tick}: Creature #{} born", id.0)
            }
            SimulationEvent::Death {
                id,
                tick,
                age_ticks,
                cause,
            } => {
                let cause_str = match cause {
                    tangled_core::domain::creatures::DeathCause::Starvation => "starvation",
                    tangled_core::domain::creatures::DeathCause::Exhaustion => "exhaustion",
                    tangled_core::domain::creatures::DeathCause::Age => "old age",
                };
                format!(
                    "💀 Tick {tick}: Creature #{} died ({cause_str}, age: {age_ticks} ticks)",
                    id.0
                )
            }
        };
        log.entries.push(entry);
    }

    // Trim old entries
    if log.entries.len() > MAX_EVENT_LOG {
        let excess = log.entries.len() - MAX_EVENT_LOG;
        log.entries.drain(0..excess);
    }
}

/// Display a non-intrusive stats panel in the top-left corner.
fn stats_hud_system(
    mut contexts: EguiContexts,
    state: Option<Res<SimulationStateResource>>,
    world_map: Option<Res<WorldMapResource>>,
    mut running: ResMut<SimulationRunning>,
) {
    let Some(state) = state else { return };
    let state = &state.0;

    // Compute food stats from the world map
    let (total_grass, fruiting_trees, total_trees) = if let Some(ref wm) = world_map {
        let grass: f64 = wm.0.iter().map(|(_, t)| t.grass).sum();
        let fruiting =
            wm.0.trees()
                .iter()
                .filter(|t| t.state == FruitTreeState::Fruiting)
                .count();
        let total = wm.0.trees().len();
        (grass, fruiting, total)
    } else {
        (0.0, 0, 0)
    };

    egui::Window::new("Simulation")
        .default_pos([10.0, 10.0])
        .default_width(200.0)
        .resizable(false)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("The Tangled Tree");
            ui.separator();

            egui::Grid::new("stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Tick:");
                    ui.label(format!("{}", state.tick));
                    ui.end_row();

                    ui.label("Alive:");
                    ui.label(format!("{}", state.alive_count));
                    ui.end_row();

                    ui.label("Dead:");
                    ui.label(format!("{}", state.dead_count));
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Births/tick:");
                    ui.label(format!("{}", state.births_this_tick));
                    ui.end_row();

                    ui.label("Deaths/tick:");
                    ui.label(format!("{}", state.deaths_this_tick));
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Starvation:");
                    ui.label(format!("⊕ {}", state.deaths_by_starvation));
                    ui.end_row();

                    ui.label("Exhaustion:");
                    ui.label(format!("⊕ {}", state.deaths_by_exhaustion));
                    ui.end_row();

                    ui.label("Old age:");
                    ui.label(format!("⊕ {}", state.deaths_by_age));
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Grass:");
                    ui.label(format!("{:.1}", total_grass));
                    ui.end_row();

                    ui.label("Fruits:");
                    ui.label(format!("{} / {}", fruiting_trees, total_trees));
                    ui.end_row();
                });

            ui.separator();
            let label = if running.0 { "⏸ Pause" } else { "▶ Play" };
            if ui.button(label).clicked() {
                running.0 = !running.0;
            }
        });
}

/// Display a scrollable event log window (bottom-left).
fn event_log_system(mut contexts: EguiContexts, log: Res<EventLog>) {
    egui::Window::new("Event Log")
        .default_pos([10.0, 350.0])
        .default_width(320.0)
        .default_height(200.0)
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if log.entries.is_empty() {
                        ui.label("No events yet…");
                    } else {
                        for entry in &log.entries {
                            ui.label(entry);
                        }
                    }
                });
        });
}
