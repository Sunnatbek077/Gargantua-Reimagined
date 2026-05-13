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
// BINARY ENTRY POINT (main.rs, not listed here):
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
//   - Binary crate (src/main.rs) → calls gargantua_app::run()
//
// NOTES:
//   - This crate contains NO rendering logic itself; it only orchestrates.
//   - On WASM, main.rs is replaced by a #[wasm_bindgen(start)] entry function.
// =============================================================================


// =============================================================================
// FILE: crates/gargantua-app/src/errors.rs
// CRATE: gargantua-app
// LINES: ~80
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Top-level AppError enum that aggregates errors from all sub-crates.
//   Any error that propagates to the application's top level is wrapped here,
//   providing a unified error type for the main event loop.
//
// WHAT THIS FILE CONTAINS:
//   - `#[derive(thiserror::Error, Debug)] pub enum AppError`:
//       Core(#[from] gargantua_core::errors::CoreError)
//             Transparent forwarding of GPU / frame graph errors.
//       Physics(#[from] gargantua_physics::errors::PhysicsError)
//             Transparent forwarding of physics computation errors.
//       Video(#[from] gargantua_video::errors::VideoError)
//             Transparent forwarding of encoder / capture errors.
//       Plugin(String)
//             Plugin load, registration, or scripting failure.
//       StateDeserialize(String)
//             URL state deserialisation failure (malformed share link).
//       Io(#[from] std::io::Error)
//             File I/O errors (config file, LUT load, EXR output).
//   - `pub type AppResult<T> = Result<T, AppError>;`
//
// OUTBOUND DEPENDENCIES:
//   - thiserror (external)                     → derive macros
//   - gargantua_core::errors::CoreError        → #[from] impl
//   - gargantua_physics::errors::PhysicsError  → #[from] impl
//   - gargantua_video::errors::VideoError      → #[from] impl
//
// INBOUND:
//   - All systems/*.rs and state/*.rs files that propagate AppResult<T>
//   - plugin/mod.rs  → wraps plugin failures into AppError::Plugin
//   - state/url_serde.rs → wraps decode failures into AppError::StateDeserialize
// =============================================================================