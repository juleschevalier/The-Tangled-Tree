//! # tangled_bevy
//!
//! Bevy engine adapter for **The Tangled Tree**.
//!
//! This crate implements the outbound ports defined in `tangled_core` using
//! the Bevy game engine. It handles rendering, input, tilemap display,
//! and UI via egui.
//!
//! ## Structure
//!
//! - **adapters/** — Concrete implementations of domain ports
//! - **plugins/** — Bevy plugins that wire adapters into the ECS

pub mod adapters;
pub mod plugins;
