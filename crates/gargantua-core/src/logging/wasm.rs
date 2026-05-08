// =============================================================================
// crates/gargantua-core/src/logging/wasm.rs
// =============================================================================
//
// PURPOSE:
//   WASM-specific tracing subscriber setup. Routes all tracing events
//   (info!, warn!, error!, debug!) to the browser's developer console
//   via console.log / console.warn / console.error.
//
//   Only compiled when target_arch = "wasm32". On native builds this
//   file is not included (see logging/mod.rs cfg gate).
//
// SIZE: ~40 lines
//
// DEPENDENCIES:
//   External:
//     - tracing_wasm::{set_as_global_default, WASMLayerConfigBuilder}
//     - tracing::{Level}
//     - wasm_bindgen::prelude::*
//
// CALLED BY:
//   - super::mod::init()  — only on WASM target
//
// PUBLIC FUNCTIONS:
//
//   pub fn init_wasm_logging(level: tracing::Level)
//     — builds a WASMLayerConfig:
//         .set_max_level(level)
//         .set_report_logs_in_timings(true)
//           — logs appear in browser Performance timeline (DevTools)
//         .set_console_config(ConsoleConfig::ReportWithConsoleColor)
//           — colored output in browser console
//     — calls tracing_wasm::set_as_global_default_with_config(config).
//     — safe to call multiple times (subsequent calls are no-ops).
//
//   pub fn console_log(msg: &str)
//     — thin wrapper around web_sys::console::log_1().
//     — used for raw console output before tracing is initialized
//       (e.g., very early startup errors in wasm_bindgen entry point).
//
//   pub fn console_error(msg: &str)
//     — thin wrapper around web_sys::console::error_1().
//     — used for fatal errors that must always appear in console
//       regardless of log level filter.
//
// NOTES FOR AI:
//   - tracing_wasm crate must be in Cargo.toml with:
//       [target.'cfg(target_arch = "wasm32")'.dependencies]
//       tracing-wasm = "X.X"
//   - web_sys must have the "console" feature enabled in Cargo.toml.
//   - This file must not be compiled on native targets. The
//     #[cfg(target_arch = "wasm32")] in mod.rs ensures this.
//   - Browser console colors: INFO=blue, WARN=orange, ERROR=red.
//     TRACE and DEBUG appear as plain console.log entries.
// =============================================================================

#![cfg(target_arch = "wasm32")]

use tracing::Level;

pub fn init_wasm_logging(level: Level) {
    todo!()
}

pub fn console_log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

pub fn console_error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}