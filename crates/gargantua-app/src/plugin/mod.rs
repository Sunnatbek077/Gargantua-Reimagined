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
//   - gargantua-app main loop         → calls registry.tick_all(ctx, dt) each frame
//   - ui/menu/tabs/plugin_tab.rs      → lists plugins, shows on_load errors
//
// NOTES:
//   - Plugins are Rust-native (compiled into the binary) in this version.
//     Dynamic .so / .dll loading is a future milestone (tracked in ROADMAP.md).
//   - WASM builds support plugins compiled to wasm32; scripting.rs uses
//     a wasm_bindgen bridge so JavaScript plugins can also be registered.
// =============================================================================


// =============================================================================
// FILE: crates/gargantua-app/src/plugin/registry.rs
// CRATE: gargantua-app
// LINES: ~200
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Owns and manages the lifecycle of all registered plugins. Handles ordered
//   load/unload, per-frame tick dispatch, error isolation (one failing plugin
//   does not crash the application), and dependency ordering between plugins.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct PluginRegistry`:
//       plugins:    IndexMap<String, Box<dyn Plugin>>
//                   — ordered map: insertion order = tick order
//       failed:     HashSet<String>
//                   — plugins whose on_load or on_frame returned Err;
//                     skipped in subsequent ticks
//       ctx:        PluginContext
//   - `impl PluginRegistry`:
//       `pub fn new(ctx: PluginContext) -> Self`
//             Initialises empty plugin map.
//       `pub fn register<P: Plugin + 'static>(&mut self, plugin: P)
//                        -> AppResult<()>`
//             Calls plugin.on_load(&self.ctx).
//             On Ok: inserts into self.plugins with plugin.name() as key.
//             On Err: returns AppError::Plugin without inserting.
//       `pub fn unregister(&mut self, name: &str)`
//             Calls plugin.on_unload(), removes from map.
//             No-op if name is not found.
//       `pub fn tick_all(&mut self, dt: DeltaTime)`
//             Iterates plugins in insertion order.
//             Skips any plugin in self.failed.
//             Calls plugin.on_frame(&self.ctx, dt).
//             On Err: moves plugin name to self.failed, logs tracing::error!.
//       `pub fn get<P: Plugin>(&self, name: &str) -> Option<&P>`
//             Returns a downcasted reference to a specific plugin type.
//       `pub fn list(&self) -> Vec<(&str, (u32,u32,u32))>`
//             Returns (name, version) pairs for the UI plugin list.
//       `pub fn failed_plugins(&self) -> &HashSet<String>`
//             Returns the set of plugin names that have errored.
//
// OUTBOUND DEPENDENCIES:
//   - plugin/mod.rs   → Plugin trait, PluginContext
//   - errors.rs       → AppResult, AppError::Plugin
//   - indexmap (ext)  → IndexMap for ordered iteration
//   - tracing (ext)   → error! macro for failed plugin logging
//
// INBOUND:
//   - systems/input.rs       → calls registry.tick_all() each frame
//   - lib.rs                 → calls registry.register() for built-in plugins
//   - plugin/scripting.rs    → registers itself into the registry on startup
//   - ui/menu/tabs/plugin_tab.rs → calls registry.list() and failed_plugins()
//
// NOTES:
//   - IndexMap preserves insertion order, which matters for plugins that
//     have implicit dependencies (e.g. a post-process plugin must tick after
//     the shader parameter plugin that feeds it).
//   - Failed plugins stay in self.failed permanently; users must restart the
//     app or manually re-register the plugin via the UI to retry on_load.
// =============================================================================


// =============================================================================
// FILE: crates/gargantua-app/src/plugin/scripting.rs
// CRATE: gargantua-app
// LINES: ~280
// PLATFORM: Mac + Windows (Lua); WASM (JavaScript via wasm_bindgen)
// =============================================================================
//
// PURPOSE:
//   Embeds a Lua 5.4 scripting engine (via mlua) that lets users write
//   runtime scripts to control camera paths, animate physics parameters,
//   and respond to simulation events — without recompiling the application.
//   On WASM, JavaScript callbacks replace Lua via a wasm_bindgen bridge.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct ScriptingPlugin`:
//       lua:          mlua::Lua              — Lua VM instance (native only)
//       scripts:      HashMap<String, mlua::Function>
//                     — loaded script functions, keyed by script name
//       error_log:    Vec<String>            — runtime Lua errors accumulate here
//   - `impl Plugin for ScriptingPlugin`:
//       `fn name(&self) -> &str { "scripting" }`
//       `fn on_load(&mut self, ctx: &PluginContext) -> AppResult<()>`
//             Initialises the Lua VM:
//               Loads standard safe libraries (string, table, math).
//               Does NOT load io or os (sandboxed — no file system access).
//               Registers Gargantua Lua API:
//                 gargantua.set_spin(a)    → writes to SimState.spin
//                 gargantua.set_mass(m)    → writes to SimState.mass
//                 gargantua.camera_pos()   → returns (x,y,z) table
//                 gargantua.time()         → returns sim_time as number
//                 gargantua.emit(event)    → sends event to EventBus
//       `fn on_frame(&mut self, ctx: &PluginContext, dt: DeltaTime)`
//             For each script in self.scripts:
//               Calls script_fn.call::<_, ()>(dt.sim).
//               On Lua error: appends error string to self.error_log,
//               removes script from self.scripts (prevents repeated errors).
//   - `impl ScriptingPlugin`:
//       `pub fn load_script(&mut self, name: &str, source: &str) -> AppResult<()>`
//             Compiles Lua source via self.lua.load(source).eval::<mlua::Function>().
//             Stores compiled function in self.scripts.
//             Returns AppError::Plugin with Lua compile error on failure.
//       `pub fn unload_script(&mut self, name: &str)`
//             Removes script from self.scripts.
//       `pub fn error_log(&self) -> &[String]`
//             Returns runtime error messages; used by ui/menu/tabs/plugin_tab.rs.
//
// OUTBOUND DEPENDENCIES:
//   - plugin/mod.rs           → Plugin trait, PluginContext
//   - state/sim_state.rs      → SimState (read/write from Lua API)
//   - state/event_bus.rs      → EventBus::emit() exposed to Lua
//   - mlua (external)         → Lua 5.4 embedding (native builds only)
//   - wasm_bindgen (ext, WASM)→ JS callback bridge (WASM builds only)
//   - errors.rs               → AppResult, AppError::Plugin
//
// INBOUND:
//   - plugin/registry.rs      → registered as a built-in plugin at startup
//   - ui/menu/tabs/plugin_tab.rs → calls load_script() from the script editor UI
//
// NOTES:
//   - The Lua sandbox disables io, os, package, and debug libraries.
//     Scripts cannot read files, spawn processes, or load external modules.
//   - mlua is compiled with the "lua54" feature and "vendored" to bundle
//     Lua 5.4 directly in the binary (no system Lua required).
//   - On WASM, mlua is excluded (#[cfg(not(target_arch="wasm32"))]) and
//     replaced by a JavaScript Promise-based callback registered via web_sys.
// =============================================================================
