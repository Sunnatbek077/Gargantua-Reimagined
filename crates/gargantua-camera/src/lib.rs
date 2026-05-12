// =============================================================================
// crates/gargantua-camera/src/lib.rs
// =============================================================================
//
// PURPOSE:
//   Crate root for gargantua-camera. Declares all public modules. This crate
//   owns all camera logic: modes, paths, world coordinates, LOD, and
//   camera-specific visual effects. It does NOT own GPU resources — it only
//   produces CPU-side matrices and scalars consumed by gargantua-render.
//
// SIZE: ~60 lines
//
// CRATE-WIDE LINTS:
//   #![deny(clippy::all)]
//   #![deny(clippy::pedantic)]
//   #![allow(clippy::module_name_repetitions)]
//   #![allow(clippy::cast_precision_loss)]
//
// PUBLIC MODULES:
//
//   pub mod errors
//     — CameraError: all camera-specific errors
//
//   pub mod world_camera
//     — WorldCamera: main camera struct, owns mode + all sub-systems
//     — CameraPose: position + rotation + fov snapshot
//     — InputState: keyboard/mouse input for camera modes
//     — CameraMode: enum of active mode
//
//   pub mod modes
//     — modes::orbit::OrbitMode          — arcball orbit around black hole
//     — modes::free_flight::FreeFlightMode — WASD + mouse free camera
//     — modes::cinematic::CinematicMode  — spline playback
//     — modes::gravity::GravityMode      — geodesic fall / orbit
//     — modes::satellite::SatelliteMode  — stable orbital mechanics
//
//   pub mod path
//     — path::easing::{EasingType, apply} — easing curves for keyframes
//     — path::keyframe::Keyframe          — single time-stamped camera pose
//     — path::spline::CameraSpline        — Catmull-Rom spline over keyframes
//     — path::recorder::PathRecorder      — records live camera path
//
//   pub mod fx
//     — fx::horizon_fx::HorizonFx             — event horizon approach effects
//     — fx::relativistic_fov::RelativisticFov — aberration-corrected FOV
//     — fx::time_warp::TimeWarp               — simulation time scaling
//
//   pub mod world
//     — world::coord_system::{BoyerLindquist, cartesian_to_bl, bl_to_cartesian, ...}
//     — world::chunk_manager::ChunkManager    — spatial LOD management
//     — world::floating_origin::FloatingOrigin — f32 precision management
//     — world::lod::LodLevel                  — LOD level enum + params
//
// RE-EXPORTS:
//   pub use errors::CameraError;
//   pub use world_camera::{WorldCamera, CameraPose, CameraMode, InputState};
//   pub use world::coord_system::{BoyerLindquist, cartesian_to_bl, bl_to_cartesian};
//
// DEPENDENCY GRAPH (within gargantua-camera):
//   world/ (no internal deps)
//     ↓
//   path/ ← world/coord_system
//     ↓
//   fx/   ← world/coord_system
//     ↓
//   modes/ ← world, path, fx
//     ↓
//   world_camera ← modes, path, fx, world
//
// EXTERNAL CRATE DEPENDENCIES:
//   glam     — Vec2/3/4, Quat, Mat4 (all math)
//   thiserror — error derives
//   serde    — Keyframe serialization (optional feature)
//   winit    — KeyCode for InputState
//
// NOTES FOR AI:
//   - gargantua-camera has NO dependency on wgpu or gargantua-core.
//     It is a pure CPU/math crate. This keeps it testable without GPU.
//   - The test files (tests/coord_system.rs, tests/spline.rs) test the
//     pure math functions without any GPU setup.
// =============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]

pub mod errors;
pub mod world_camera;

pub mod modes {
    pub mod cinematic;
    pub mod free_flight;
    pub mod gravity;
    pub mod orbit;
    pub mod satellite;
}

pub mod path {
    pub mod easing;
    pub mod keyframe;
    pub mod recorder;
    pub mod spline;
}

pub mod fx {
    pub mod horizon_fx;
    pub mod relativistic_fov;
    pub mod time_warp;
}

pub mod world {
    pub mod chunk_manager;
    pub mod coord_system;
    pub mod floating_origin;
    pub mod lod;
}

// Convenience re-exports
pub use errors::CameraError;
pub use world_camera::{CameraMode, CameraPose, InputState, WorldCamera};
pub use world::coord_system::{BoyerLindquist, bl_to_cartesian, cartesian_to_bl};