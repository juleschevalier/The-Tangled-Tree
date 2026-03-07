//! Simulation engine — tick loop, rules, metrics.
//!
//! Orchestrates the simulation: advances time, applies natural selection
//! pressure, handles reproduction and death cycles, and collects metrics.

mod tick;

pub use tick::{SimulationState, SimulationTick};
