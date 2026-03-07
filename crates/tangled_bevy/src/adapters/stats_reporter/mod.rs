//! Stats reporter adapter — displays simulation metrics via `bevy_egui`.
//!
//! Renders a lightweight HUD panel (top-left) showing live simulation state:
//! tick number, alive/dead counts, births and deaths per tick.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

use crate::plugins::simulation_plugin::{SimulationRunning, SimulationStateResource};

/// Plugin that overlays an egui HUD with simulation metrics.
pub struct StatsHudPlugin;

impl Plugin for StatsHudPlugin {
    fn build(&self, app: &mut App) {
        // EguiPlugin is idempotent — safe to add even if another adapter adds it too.
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.add_systems(Update, stats_hud_system);
    }
}

/// Display a non-intrusive stats panel in the top-left corner.
fn stats_hud_system(
    mut contexts: EguiContexts,
    state: Option<Res<SimulationStateResource>>,
    mut running: ResMut<SimulationRunning>,
) {
    let Some(state) = state else { return };
    let state = &state.0;

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
                });

            ui.separator();
            let label = if running.0 { "⏸ Pause" } else { "▶ Play" };
            if ui.button(label).clicked() {
                running.0 = !running.0;
            }
        });
}
