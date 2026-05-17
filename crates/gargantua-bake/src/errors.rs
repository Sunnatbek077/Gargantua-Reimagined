// ============================================================
// FILE: crates/gargantua-bake/src/errors.rs
// LINES: ~90
// CATEGORY: Bake — Error types
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines BakeError — the single error enum for gargantua-bake.
//   Covers geodesic integration failures, file I/O errors,
//   GPU compute shader errors, and cache corruption.
//
// CONTENTS (~90 lines):
//   #[derive(Debug, thiserror::Error)]
//   pub enum BakeError {
//       #[error("geodesic integration failed at spin={spin}, b={b}: {source}")]
//       GeodesicFailed {
//           spin: f64, b: f64,
//           source: gargantua_physics::errors::PhysicsError,
//       },
//
//       #[error("LUT file I/O error writing '{path}': {source}")]
//       LutIo { path: String, source: std::io::Error },
//
//       #[error("EXR write error: {0}")]
//       ExrWrite(String),
//
//       #[error("wgpu compute error: {0}")]
//       GpuCompute(String),
//
//       #[error("cache is corrupt or incompatible version: {0}")]
//       CacheCorrupt(String),
//
//       #[error("bake cancelled by user")]
//       Cancelled,
//   }
//
//   pub type BakeResult<T> = Result<T, BakeError>;
//
// USES (imports from):
//   thiserror (external)
//   gargantua_physics::errors::PhysicsError
//   std::io
//
// USED BY:
//   All bake sub-modules return BakeResult<T>
//   crates/gargantua-app/src/lib.rs
//     → matches BakeError variants to show error messages in UI
//
// NOTE FOR AI:
//   BakeError::Cancelled is returned when the user clicks "Cancel"
//   in the bake progress overlay. Not a real error — app handles it
//   separately from actual failures.
//   GpuCompute wraps wgpu error strings (wgpu errors are not Clone/Send
//   so they are converted to String before storing).
// ============================================================