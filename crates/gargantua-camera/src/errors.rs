// =============================================================================
// crates/gargantua-camera/src/errors.rs
// =============================================================================
//
// PURPOSE:
//   Defines CameraError — the unified error type for the gargantua-camera
//   crate. All public functions that can fail return Result<T, CameraError>.
//
// SIZE: ~60 lines
//
// DEPENDENCIES:
//   External:
//     - thiserror::Error
//
// CALLED BY:
//   - crate::path::spline::CameraSpline::new()
//   - crate::path::recorder::PathRecorder::stop_recording()
//   - crate::modes::gravity::GravityMode (geodesic integration errors)
//   - crates/gargantua-app/src/errors.rs — wraps CameraError in AppError
//
// PUBLIC TYPES:
//
//   #[derive(Debug, thiserror::Error)]
//   pub enum CameraError {
//
//     #[error("Insufficient keyframes: got {got}, need at least {need}")]
//     InsufficientKeyframes { got: usize, need: usize },
//       — returned by CameraSpline::new() if < 2 keyframes provided.
//       — returned by PathRecorder::stop_recording() if < 2 frames recorded.
//
//     #[error("Camera is inside event horizon (r={r:.3} < r_horizon={r_horizon:.3})")]
//     InsideHorizon { r: f32, r_horizon: f32 },
//       — returned by GravityMode::update() if the geodesic integrator
//         places the camera inside the event horizon.
//       — recoverable: App can switch to OrbitMode at a safe distance.
//
//     #[error("Invalid orbital parameters: {message}")]
//     InvalidOrbit { message: String },
//       — returned by SatelliteMode::new() if semi_major < r_isco.
//       — also if eccentricity >= 1.0 (non-bound orbit).
//
//     #[error("Spline time out of range: t={t:.3} not in [{t_min:.3}, {t_max:.3}]")]
//     TimeOutOfRange { t: f32, t_min: f32, t_max: f32 },
//       — returned by CameraSpline::eval() in strict mode (clamping disabled).
//       — default: clamp rather than error.
//
//     #[error("Mode transition failed: {from} -> {to}: {reason}")]
//     ModeTransitionFailed { from: String, to: String, reason: String },
//       — returned by WorldCamera::set_mode() if the transition is invalid.
//       — e.g., entering CinematicMode with no spline set.
//   }
//
// NOTES FOR AI:
//   - CameraError is lightweight — no GPU resources, no async.
//   - InsideHorizon should be caught at the App level and handled gracefully
//     (switch to safe orbit) rather than crashing.
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CameraError {
    #[error("Insufficient keyframes: got {got}, need at least {need}")]
    InsufficientKeyframes { got: usize, need: usize },

    #[error("Camera inside event horizon (r={r:.3} < r_horizon={r_horizon:.3})")]
    InsideHorizon { r: f32, r_horizon: f32 },

    #[error("Invalid orbital parameters: {message}")]
    InvalidOrbit { message: String },

    #[error("Spline time out of range: t={t:.3} not in [{t_min:.3}, {t_max:.3}]")]
    TimeOutOfRange { t: f32, t_min: f32, t_max: f32 },

    #[error("Mode transition failed: {from} -> {to}: {reason}")]
    ModeTransitionFailed { from: String, to: String, reason: String },
}