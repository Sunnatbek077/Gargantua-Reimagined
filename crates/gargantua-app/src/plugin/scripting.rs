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
