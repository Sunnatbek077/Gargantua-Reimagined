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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] gargantua_core::errors::CoreError),
    #[error(transparent)]
    Physics(#[from] gargantua_physics::errors::PhysicsError),
    #[error(transparent)]
    Video(#[from] gargantua_video::errors::VideoError),
    #[error("plugin: {0}")]
    Plugin(String),
    #[error("state deserialize: {0}")]
    StateDeserialize(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
