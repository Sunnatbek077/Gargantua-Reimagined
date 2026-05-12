// =============================================================================
// crates/gargantua-camera/src/modes/free_flight.rs
// =============================================================================
//
// PURPOSE:

//   First-person free-flight camera controlled by keyboard + mouse.
//   WASD = translate XZ, QE = translate Y, mouse drag = rotate.
//   Supports multiple speed tiers (shift = fast, ctrl = slow).
//   Applies gravitational lensing aberration to the camera FOV when
//   moving at relativistic speeds (β > 0.1c).
//
//   FIELDS:
//     velocity:      Vec3     — current camera velocity (world units/s)
//     acceleration:  f32     — movement acceleration (default 10.0)
//     max_speed:     f32     — speed cap (default 50.0 M/s)
//     sensitivity:   f32     — mouse rotation sensitivity (default 0.003 rad/px)
//     yaw:           f32     — current yaw angle (radians)
//     pitch:         f32     — current pitch angle (radians, clamped ±89°)
//
//   KEY FUNCTIONS:
//     pub fn new(sensitivity: f32, max_speed: f32) -> Self
//     pub fn handle_key(&mut self, key: KeyCode, pressed: bool)
//     pub fn handle_mouse_delta(&mut self, dx: f32, dy: f32)
//     pub fn update(&mut self, delta_t: f32, camera_pos: &mut Vec3) -> Quat
//       — integrates velocity, applies friction, returns new rotation.
//     pub fn beta(&self) -> f32
//       — returns |velocity| / c (relativistic β parameter).
//       — passed to starfield.wgsl for aberration effect.
//
//   CALLED BY:
//     - crate::world_camera::WorldCamera — when in FreeFlight mode
//     - crate::fx::relativistic_fov::RelativisticFov — uses beta()
//
// SIZE: ~260 lines
//
// DEPENDENCIES:
//   Internal:
//     - glam::{Vec3, Quat}
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

pub struct FreeFlightMode {
    // Fields documented above — implement per purpose spec
    _placeholder: (),
}

impl FreeFlightMode {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat) {
        todo!()
    }
}

impl Default for FreeFlightMode {
    fn default() -> Self { Self::new() }
}