//! Simulation engine — tick loop, rules, metrics.
//!
//! Orchestrates the simulation: advances time, applies natural selection
//! pressure, handles reproduction and death cycles, and collects metrics.

pub mod movement;
mod tick;

pub use tick::{SimulationEvent, SimulationState, SimulationTick};
