# ADR-001: Architecture Hexagonale (Ports & Adapters)

**Date**: 2026-03-06
**Status**: Accepted
**Deciders**: jchevalier

## Context

The Tangled Tree is a Darwinian evolution simulation. The core logic (genetics, simulation rules, world model) is complex and must be:

1. **Testable** without a game engine
2. **Deterministic** — same seed must produce same results
3. **Extensible** — adding genes, behaviors, biomes should not require modifying existing code
4. **Engine-agnostic** — the domain should survive a Bevy major version upgrade

## Decision

We adopt a **Hexagonal Architecture** (Ports & Adapters, Alistair Cockburn) with the following structure:

### Crate layout

| Crate | Role | Dependencies |
|---|---|---|
| `tangled_core` | Pure domain (genetics, simulation, world) | **None** (stdlib only) |
| `tangled_bevy` | Bevy adapter (rendering, input, tilemap, UI) | `tangled_core`, `bevy`, `bevy_ecs_tilemap`, `bevy_egui`, `noise` |
| `tangled_persistence` | Persistence adapter (save/load) | `tangled_core`, `serde`, `ron` |
| `tangled_app` | Entry point, wiring | All crates |

### Rules

1. **`tangled_core` has zero external dependencies** — no `use bevy`, no `use serde`, no `use noise`
2. **Ports are traits** defined in `tangled_core::ports`
3. **Adapters are structs** in adapter crates that implement those traits
4. **Bevy is an adapter**, not the center of the architecture
5. **Domain types are plain Rust structs/enums** — never `#[derive(Component)]` in core

### Ports

- **Inbound** (driving): `SimulationController`, `WorldConfigurator`
- **Outbound** (driven): `WorldGenerator`, `Renderer`, `StatsReporter`, `Persistence`

## Consequences

### Positive
- Domain logic is testable in milliseconds (no ECS, no renderer)
- Bevy version upgrades only affect `tangled_bevy`, not core logic
- The simulation can run headless (no rendering) for benchmarks and CI
- New genes/traits can be added by extending domain types — open/closed principle

### Negative
- Slight indirection cost (trait dispatch) — negligible for a simulation game
- Need discipline to avoid "just adding bevy as a dep to core"
- Some data mapping between domain types and Bevy Components

### Risks
- Over-abstraction if ports proliferate — keep them focused and minimal
- Performance: if trait objects cause issues, can switch to generics later

## Notes

The name `tangled_` prefix was chosen for brevity while remaining unique and evocative of the project's theme (tangled tree of life / phylogenetics).
