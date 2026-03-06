//! # tangled_persistence
//!
//! Persistence adapter for **The Tangled Tree**.
//!
//! Implements the `Persistence` outbound port from `tangled_core` using
//! `serde` + `ron` for serialization. Handles saving/loading of:
//! - Simulation seeds and configuration
//! - World state snapshots
//! - Creature population data

pub mod adapters;
