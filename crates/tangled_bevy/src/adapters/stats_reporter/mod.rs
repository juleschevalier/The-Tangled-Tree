//! Stats reporter adapter — displays simulation metrics via `bevy_egui`.
//!
//! Renders a lightweight HUD panel (top-left) showing live simulation state:
//! tick number, alive/dead counts, births and deaths per tick.
//! Also shows a scrollable event log for births and deaths.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use tangled_core::domain::creatures::CreatureConfig;
use tangled_core::domain::genetics::Diet;
use tangled_core::domain::simulation::SimulationEvent;
use tangled_core::domain::world::FruitTreeState;

use crate::adapters::renderer::creature_renderer::{PopulationResource, SelectedCreature};
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
            .add_systems(Update, event_log_system)
            .add_systems(Update, creature_detail_panel);
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

/// Display a detail panel for the currently selected creature (right side).
fn creature_detail_panel(
    mut contexts: EguiContexts,
    selected: Option<Res<SelectedCreature>>,
    population: Option<Res<PopulationResource>>,
) {
    let Some(selected) = selected else { return };
    let Some(creature_id) = selected.0 else { return };
    let Some(population) = population else { return };

    let Some(creature) = population.creatures.iter().find(|c| c.id == creature_id) else {
        return;
    };

    let config = CreatureConfig::default();

    egui::Window::new("Creature Details")
        .default_pos([800.0, 10.0])
        .default_width(240.0)
        .resizable(false)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            // ── Header ──
            let status_icon = if creature.is_alive() { "🟢" } else { "💀" };
            ui.heading(format!("{status_icon} Creature #{}", creature.id.0));
            ui.separator();

            // ── Vitals ──
            egui::Grid::new("vitals_grid")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Position:");
                    ui.label(format!("({}, {})", creature.position.x, creature.position.y));
                    ui.end_row();

                    ui.label("Age:");
                    let age_pct = creature.age_ticks as f32 / config.max_age_ticks as f32 * 100.0;
                    ui.label(format!(
                        "{} ticks ({:.0}%)",
                        creature.age_ticks, age_pct
                    ));
                    ui.end_row();

                    ui.label("Energy:");
                    let bar_color = if creature.energy > 50.0 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if creature.energy > 20.0 {
                        egui::Color32::from_rgb(220, 180, 30)
                    } else {
                        egui::Color32::from_rgb(220, 40, 40)
                    };
                    ui.horizontal(|ui| {
                        let bar = egui::ProgressBar::new(creature.energy / 100.0)
                            .text(format!("{:.0}", creature.energy))
                            .fill(bar_color);
                        ui.add(bar);
                    });
                    ui.end_row();

                    if let Some(cause) = creature.death_cause {
                        ui.label("Death:");
                        let cause_str = match cause {
                            tangled_core::domain::creatures::DeathCause::Starvation => {
                                "⚠ Starvation"
                            }
                            tangled_core::domain::creatures::DeathCause::Age => "🕐 Old age",
                        };
                        ui.label(cause_str);
                        ui.end_row();
                    }
                });

            ui.separator();
            ui.label(egui::RichText::new("Genetics").strong());

            // ── Genetics ──
            egui::Grid::new("genetics_grid")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    // Diet
                    let diet = creature.genome.expressed_diet();
                    let (diet_str, diet_color) = match diet {
                        Diet::Herbivore => ("🌿 Herbivore", egui::Color32::from_rgb(60, 180, 60)),
                        Diet::Omnivore => ("🍽 Omnivore", egui::Color32::from_rgb(200, 170, 40)),
                        Diet::Carnivore => ("🥩 Carnivore", egui::Color32::from_rgb(200, 50, 50)),
                    };
                    ui.label("Diet:");
                    ui.label(egui::RichText::new(diet_str).color(diet_color));
                    ui.end_row();

                    let diet_m = creature.genome.diet.maternal.value();
                    let diet_p = creature.genome.diet.paternal.value();
                    ui.label("");
                    ui.label(
                        egui::RichText::new(format!("  ♀ {:?}  ♂ {:?}", diet_m, diet_p))
                            .small()
                            .weak(),
                    );
                    ui.end_row();

                    // Speed
                    let speed = creature.genome.expressed_speed();
                    ui.label("Speed:");
                    ui.label(format!("{:.2}", speed));
                    ui.end_row();

                    let sp_m = creature.genome.speed.maternal.value();
                    let sp_p = creature.genome.speed.paternal.value();
                    ui.label("");
                    ui.label(
                        egui::RichText::new(format!("  ♀ {:.2}  ♂ {:.2}", sp_m, sp_p))
                            .small()
                            .weak(),
                    );
                    ui.end_row();

                    // Size
                    let size = creature.genome.expressed_size();
                    ui.label("Size:");
                    ui.label(format!("{:.2}", size));
                    ui.end_row();

                    let sz_m = creature.genome.size.maternal.value();
                    let sz_p = creature.genome.size.paternal.value();
                    ui.label("");
                    ui.label(
                        egui::RichText::new(format!("  ♀ {:.2}  ♂ {:.2}", sz_m, sz_p))
                            .small()
                            .weak(),
                    );
                    ui.end_row();

                    // Metabolism (computed)
                    let metabolism = speed * size;
                    let drain = config.energy_drain_per_tick + metabolism * config.metabolism_factor;
                    ui.label("Metabolism:");
                    ui.label(format!("{:.2}/tick", drain));
                    ui.end_row();
                });
        });
}
