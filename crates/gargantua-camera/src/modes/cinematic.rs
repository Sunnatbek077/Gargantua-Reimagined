// =============================================================================
// crates/gargantua-camera/src/modes/cinematic.rs
// =============================================================================
//
// PURPOSE:

//   Plays back a CameraSpline path for cinematic shots and offline renders.
//   Drives camera position and rotation by evaluating the spline at the
//   current playback time. Supports play/pause/seek and loop modes.
//
//   Used by gargantua-video for offline rendering: the video encoder requests
//   frames at specific timestamps, and CinematicMode provides the exact camera
//   pose for each frame.
//
//   FIELDS:
//     spline:      CameraSpline    — the path to follow
//     time:        f32             — current playback time in seconds
//     speed:       f32             — playback speed multiplier (1.0 = realtime)
//     looping:     bool            — loop at end of path
//     is_playing:  bool
//
//   KEY FUNCTIONS:
//     pub fn new(spline: CameraSpline) -> Self
//     pub fn update(&mut self, delta_t: f32) -> SplineEval
//       — advances time, evaluates spline, returns camera pose.
//     pub fn seek(&mut self, time: f32)  — jump to time
//     pub fn play(&mut self)  / pub fn pause(&mut self)
//     pub fn duration(&self) -> f32
//     pub fn progress(&self) -> f32  — time / duration ∈ [0,1]
//
//   CALLED BY:
//     - crate::world_camera::WorldCamera  — when in CinematicMode
//     - crates/gargantua-video/src/render/offline.rs — frame-by-frame eval
//
// SIZE: ~240 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::path::spline::{CameraSpline, SplineEval}
//     - crate::errors::CameraError
//   External:
//     - glam::{Vec3, Quat}
//
// NOTES FOR AI:
//   - All camera modes implement the CameraMode trait (defined in world_camera.rs):
//       fn update(&mut self, delta_t: f32, input: &InputState) -> CameraPose
//       fn mode_name(&self) -> &'static str
//   - CameraPose { position: Vec3, rotation: Quat, fov_y_deg: f32 }
//   - Modes are stored as Box<dyn CameraMode> in WorldCamera.
//   - Input handling (keyboard/mouse) is passed via InputState from gargantua-app.
// =============================================================================

use glam::{Vec3, Quat};
use crate::errors::CameraError;

pub struct CinematicMode {
    // Fields documented above — implement per purpose spec
    _placeholder: (),
}

impl CinematicMode {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat) {
        todo!()
    }
}

impl Default for CinematicMode {
    fn default() -> Self { Self::new() }
}