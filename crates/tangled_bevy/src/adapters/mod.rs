//! Adapter implementations for Bevy.
//!
//! Each adapter implements an outbound port from `tangled_core`
//! using Bevy's ECS and associated crates.

pub mod renderer;
pub mod stats_reporter;
pub mod world_generator;
