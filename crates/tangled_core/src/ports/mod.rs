//! Port interfaces for the hexagonal architecture.
//!
//! Ports define the contracts between the domain and the outside world.
//! They are pure Rust traits with no external dependencies.

pub mod inbound;
pub mod outbound;
