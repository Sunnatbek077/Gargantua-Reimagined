// =============================================================================
// FILE: crates/gargantua-app/src/lib.rs
// CRATE: gargantua-app
// LINES: ~50
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Crate root for gargantua-app. This is the topmost application crate —
//   it wires together gargantua-core, gargantua-physics, gargantua-render,
//   gargantua-camera, gargantua-video, and gargantua-ui into a runnable binary.
//   All module declarations and top-level re-exports live here.
//
// MODULES DECLARED:
//   pub mod errors;          → AppError enum
//   pub mod plugin;          → plugin system (mod, registry, scripting)
//   pub mod state;           → sim_state, event_bus, undo, url_serde
//   pub mod systems;         → input, physics_sync, replay
//
// RE-EXPORTS:
//   pub use errors::AppError;
//   pub use state::sim_state::SimState;
//   pub use state::event_bus::EventBus;
//
// BINARY ENTRY POINT (PLANNED: crates/gargantua-app/src/main.rs):
//   fn main() {
//       let event_loop = winit::event_loop::EventLoop::new();
//       let app = gargantua_core::app::App::new(&event_loop).unwrap();
//       app.run(event_loop);
//   }
//
// OUTBOUND DEPENDENCIES:
//   - gargantua-core    → App, GpuContext, FrameGraph, Clock
//   - gargantua-physics → KerrMetric, geodesic integrator
//   - gargantua-render  → all render pipelines
//   - gargantua-camera  → camera modes and path
//   - gargantua-video   → offline renderer, realtime capturer
//   - gargantua-ui      → HUD, menus, overlays
//
// INBOUND:
//   - PLANNED: crates/gargantua-app/src/main.rs → constructs App, runs event loop
//
// NOTES:
//   - This crate contains NO rendering logic itself; it only orchestrates.
//   - On WASM, main.rs is replaced by a #[wasm_bindgen(start)] entry function.
// =============================================================================
