# 🌳 The Tangled Tree

> *"There is grandeur in this view of life [...] from so simple a beginning endless forms most beautiful and most wonderful have been, and are being, evolved."*
> — Charles Darwin, On the Origin of Species

**The Tangled Tree** is a Darwinian evolution sandbox simulation where creatures evolve through genetics in a procedurally generated isometric pixel art world. Observe natural selection at work as dominant traits emerge over generations.

## Architecture

The project follows a **hexagonal architecture** (Ports & Adapters) for maximum testability and separation of concerns:

```
crates/
├── tangled_core          → Pure domain logic (genetics, simulation, world)
├── tangled_bevy          → Bevy engine adapter (rendering, input, tilemap)
├── tangled_persistence   → Save/load adapter (serde + RON)
└── tangled_app           → Application entry point (wires everything together)
```

**Key principle**: `tangled_core` has zero external dependencies — the simulation logic runs without any game engine.

## Tech Stack

- **Rust** (2024 edition)
- **Bevy 0.15** — ECS game engine
- **Isometric pixel art** — `bevy_ecs_tilemap`
- **UI** — `bevy_egui` + `egui_plot`
- **Procedural generation** — `noise` crate (Perlin, Simplex)
- **Deterministic** — seeded RNG via `rand_chacha`

## Getting Started

```bash
# Build
cargo build

# Run
cargo run -p tangled_app

# Test (domain logic, no game engine needed)
cargo test -p tangled_core

# Test all
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

## Project Status

🚧 **Early development** — MVP in progress.

## License

MIT
