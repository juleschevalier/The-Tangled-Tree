//! Renderer adapter — draws the simulation world using Bevy sprites and tilemap.

pub mod creature_renderer;
pub mod tilemap_renderer;

pub use creature_renderer::CreatureRendererPlugin;
pub use tilemap_renderer::TilemapRendererPlugin;
