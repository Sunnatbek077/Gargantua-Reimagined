// ============================================================
// FILE: crates/gargantua-physics/src/lib.rs
// LINES: ~40
// CATEGORY: Physics — crate entry point
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Public interface of the gargantua-physics crate.
//   Re-exports all sub-modules so external crates only
//   need to import from this single entry point.
//   Contains zero computation logic.
//
// CONTENTS (~40 lines):
//   pub mod errors;      // PhysicsError enum (thiserror)
//   pub mod units;       // G=c=1 natural unit helpers
//   pub mod metric;      // MetricTensor trait + Kerr/Schwarzschild
//   pub mod geodesic;    // RK4 integrator + adaptive step
//   pub mod accretion;   // ISCO, Novikov-Thorne, MHD, spectrum
//   pub mod effects;     // Doppler, redshift, aberration, Penrose
//   Crate-level attributes: #![deny(unsafe_code)], #![warn(missing_docs)]
//
// USES (imports from):
//   errors.rs, units.rs, metric/mod.rs, geodesic/rk4.rs,
//   accretion/isco.rs, effects/doppler.rs (all via pub mod)
//
// USED BY:
//   crates/gargantua-render/src/pipelines/geodesic_gpu.rs
//   crates/gargantua-render/src/pipelines/accretion.rs
//   crates/gargantua-ui/src/menu/tabs/physics_tab.rs
//   crates/gargantua-app/src/systems/physics_sync.rs
//
// NOTE FOR AI:
//   Pure module declarations + re-exports only.
//   No structs, no functions, no math here.
//   Every new sub-module must be registered here with `pub mod`.
// ============================================================

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod accretion;
pub mod effects;
pub mod errors;
pub mod geodesic;
pub mod metric;
pub mod units;