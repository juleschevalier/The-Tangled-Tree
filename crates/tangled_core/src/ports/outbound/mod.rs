//! Outbound ports (driven side).
//!
//! These traits define the services the domain needs from the outside:
//! world generation, rendering, persistence, statistics reporting, etc.
//! Adapters implement these traits with concrete technologies.

mod world_generator;

pub use world_generator::WorldGenerator;
