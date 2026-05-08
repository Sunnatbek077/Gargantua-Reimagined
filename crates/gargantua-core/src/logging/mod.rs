// =============================================================================
// crates/gargantua-core/src/logging/mod.rs
// =============================================================================
//
// PURPOSE:
//   Initializes the tracing subscriber for structured logging across the
//   entire workspace. Dispatches to the correct backend depending on the
//   compile target:
//     - Native (macOS / Windows): tracing-subscriber with pretty formatter
//       writing to stderr, with ANSI color support.
//     - WASM: routes tracing events to browser console via tracing-wasm
//       (wasm.rs handles this).
//
//   Exposes a single init() function called once at application startup
//   in gargantua-app/src/app.rs before any other system is initialized.
//
// SIZE: ~60 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::wasm::init_wasm_logging   — WASM-specific subscriber (wasm.rs)
//   External:
//     - tracing_subscriber::{EnvFilter, fmt, prelude::*}
//     - tracing::{info, warn, error, debug}
//
// CALLED BY:
//   - crates/gargantua-app/src/app.rs::App::new()  — called first before GPU init
//
// PUBLIC FUNCTIONS:
//
//   pub fn init(level: tracing::Level)
//     — initializes the global tracing subscriber.
//     — on native: installs a fmt subscriber with:
//         EnvFilter from RUST_LOG env var, defaulting to `level`
//         .with_target(true)      — show module path in log output
//         .with_thread_ids(false) — omit thread IDs (single render thread)
//         .with_file(true)        — show source file and line number
//         .compact()              — compact format for readability
//     — on WASM: calls wasm::init_wasm_logging(level).
//     — safe to call multiple times — subsequent calls are no-ops
//       (tracing subscriber can only be set once globally).
//
//   pub fn set_level(level: tracing::Level)
//     — dynamically updates the log filter level at runtime.
//     — used by the debug UI (gargantua-ui/src/panel/debug.rs)
//       to toggle verbose logging without restarting.
//
// LOG LEVELS USED ACROSS THE WORKSPACE:
//   ERROR — unrecoverable GPU errors, shader compile failures
//   WARN  — recoverable issues: surface lost, adapter fallback, cache miss
//   INFO  — startup messages: GPU selected, feature flags, preset loaded
//   DEBUG — per-frame diagnostics: pass timings, resource allocations
//   TRACE — verbose: every RK4 step, every bind group creation (never in release)
//
// NOTES FOR AI:
//   - Call init() exactly once, at the very start of main() or App::new().
//   - tracing macros (info!, warn!, etc.) are used throughout the workspace.
//     Never use println! in non-test code.
//   - In release builds, TRACE and DEBUG are compiled out via the
//     `max_level_info` feature of the tracing crate (set in Cargo.toml).
//   - EnvFilter respects RUST_LOG=gargantua_core=debug,gargantua_physics=trace
//     for selective verbose output during development.
// =============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm;

use tracing::Level;

pub fn init(level: Level) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::{fmt, EnvFilter};
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.as_str()));
        let _ = fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_file(true)
            .compact()
            .try_init();
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm::init_wasm_logging(level);
    }
}

pub fn set_level(_level: Level) {
    todo!()
}