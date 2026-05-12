// =============================================================================
// crates/gargantua-camera/src/modes/orbit.rs
// =============================================================================
//
// PURPOSE:

//   Arcball / spherical orbit camera around the black hole.
//   Mouse drag rotates the camera around the origin; scroll wheel
//   changes the orbital radius. Maintains a fixed up vector (Vec3::Y).
//   Supports smooth damping (spring-based) for inertial feel.
//
//   FIELDS:
//     radius:   f32    — distance from black hole center (min = r_isco)
//     theta:    f32    — polar angle (0..π)
//     phi:      f32    — azimuthal angle (0..2π)
//     target:   Vec3   — look-at target (default black hole center)
//     damping:  f32    — velocity damping (0.85 = 15% decay per frame)
//     vel_theta: f32, vel_phi: f32  — angular velocities (for inertia)
//
//   KEY FUNCTIONS:
//     pub fn new(radius: f32, theta: f32, phi: f32) -> Self
//     pub fn handle_drag(&mut self, dx: f32, dy: f32, sensitivity: f32)
//     pub fn handle_scroll(&mut self, delta: f32, r_min: f32)
//     pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat)
//       — applies damping to velocities, updates angles, returns pose.
//     pub fn world_position(&self) -> Vec3  — current camera position
//     pub fn set_radius(&mut self, r: f32, r_min: f32)
//
//   CALLED BY:
//     - crate::world_camera::WorldCamera — default startup mode
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - glam::{Vec3, Quat}
//     - crate::world::coord_system::spherical_to_cartesian
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

pub struct OrbitMode {
    // Fields documented above — implement per purpose spec
    _placeholder: (),
}

impl OrbitMode {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat) {
        todo!()
    }
}

impl Default for OrbitMode {
    fn default() -> Self { Self::new() }
}