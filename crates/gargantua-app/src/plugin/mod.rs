// =============================================================================
// FILE: crates/gargantua-app/src/plugin/mod.rs
// CRATE: gargantua-app
// LINES: ~120
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Defines the core `Plugin` trait and the top-level plugin module structure.
//   Every feature that can be dynamically loaded or toggled at runtime
//   (custom shaders, scripted camera paths, third-party colour pipelines)
//   implements this trait and registers itself through the PluginRegistry.
//
// WHAT THIS FILE CONTAINS:
//   - `pub mod registry;`
//   - `pub mod scripting;`
//   - `pub trait Plugin: Send + Sync`:
//       `fn name(&self) -> &str`
//             Unique identifier string, e.g. "custom_lut" or "lua_camera_path".
//             Used as key in PluginRegistry.
//       `fn version(&self) -> (u32, u32, u32)`
//             Semantic version tuple (major, minor, patch).
//       `fn on_load(&mut self, ctx: &PluginContext) -> AppResult<()>`
//             Called once when the plugin is registered. Should initialise
//             GPU resources, load files, or register event listeners.
//             Returns AppError::Plugin on failure; the registry rolls back
//             the registration if this returns Err.
//       `fn on_unload(&mut self)`
//             Called before the plugin is removed. Must release all GPU
//             resources (pipelines, bind groups, textures) and deregister
//             any event listeners added in on_load.
//       `fn on_frame(&mut self, ctx: &PluginContext, dt: DeltaTime)`
//             Called every frame between physics update and render submission.
//             Optional (default: no-op). Plugins that need per-frame updates
//             (e.g. animated shader parameters) override this.
//   - `pub struct PluginContext`:
//       event_bus: Arc<EventBus>       — send/receive app-wide events
//       sim_state: Arc<RwLock<SimState>>  — read physics params
//       gpu_ctx:   Arc<GpuContext>     — create GPU resources
//   - `pub use registry::PluginRegistry;`
//   - `pub use scripting::ScriptingPlugin;`
//
// OUTBOUND DEPENDENCIES:
//   - plugin/registry.rs              → PluginRegistry
//   - plugin/scripting.rs             → ScriptingPlugin
//   - state/event_bus.rs              → EventBus
//   - state/sim_state.rs              → SimState
//   - gargantua_core::gpu::context    → GpuContext
//   - gargantua_core::clock           → DeltaTime
//   - errors.rs                       → AppResult
//
// INBOUND:
//   - gargantua_core::app::App event loop         → calls registry.tick_all(ctx, dt) each frame
//   - PLANNED: crates/gargantua-ui/src/menu/tabs/plugin_tab.rs      → lists plugins, shows on_load errors
//
// NOTES:
//   - Plugins are Rust-native (compiled into the binary) in this version.
//     Dynamic .so / .dll loading is a future milestone (tracked in ROADMAP.md).
//   - WASM builds support plugins compiled to wasm32; scripting.rs uses
//     a wasm_bindgen bridge so JavaScript plugins can also be registered.
// =============================================================================
