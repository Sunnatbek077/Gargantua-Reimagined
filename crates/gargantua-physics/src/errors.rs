// ============================================================
// FILE: crates/gargantua-physics/src/errors.rs
// LINES: ~80
// CATEGORY: Physics — error types
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Defines PhysicsError — the single error enum for the
//   entire gargantua-physics crate, using the `thiserror`
//   crate for automatic Display + Error trait impls.
//
// CONTENTS (~80 lines):
//   #[derive(Debug, thiserror::Error)]
//   pub enum PhysicsError {
//       #[error("spin parameter |a| must be < 1, got {0}")]
//       InvalidSpin(f64),
//
//       #[error("mass must be positive, got {0}")]
//       NegativeMass(f64),
//
//       #[error("geodesic integration diverged after {steps} steps at r={r}")]
//       GeodesicDiverged { steps: u32, r: f64 },
//
//       #[error("impact parameter b={0} is below photon sphere")]
//       BelowPhotonSphere(f64),
//
//       #[error("ISCO computation failed: {0}")]
//       IscoFailed(String),
//   }
//
//   pub type PhysicsResult<T> = Result<T, PhysicsError>;
//
// USES (imports from):
//   Only std and thiserror — no internal crate imports.
//
// USED BY:
//   metric/kerr.rs           → returns PhysicsError::InvalidSpin
//   geodesic/rk4.rs          → returns PhysicsError::GeodesicDiverged
//   accretion/isco.rs        → returns PhysicsError::IscoFailed
//   lib.rs                   → re-exports via `pub mod errors`
//
// NOTE FOR AI:
//   All fallible functions in this crate return PhysicsResult<T>.
//   Do NOT create new error enums — add variants here instead.
//   thiserror v1.x is pinned in Cargo.toml.
// ============================================================