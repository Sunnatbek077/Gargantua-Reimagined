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
