//! # tangled_core
//!
//! Pure domain logic for **The Tangled Tree** — a Darwinian evolution simulation.
//!
//! This crate contains the simulation's core business logic with **zero external
//! dependencies**. It defines the genetic model, creature lifecycle, world
//! representation, and simulation rules.
//!
//! ## Architecture (Hexagonal / Ports & Adapters)
//!
//! - **domain/** — Core types and logic (genetics, creatures, world, simulation)
//! - **ports/** — Trait interfaces for external interactions
//!   - `inbound` — Commands the simulation accepts (driving side)
//!   - `outbound` — Services the simulation needs (driven side)
//!
//! The domain never depends on any game engine, rendering library, or I/O framework.

pub mod domain;
pub mod ports;
