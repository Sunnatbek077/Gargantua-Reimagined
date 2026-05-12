// =============================================================================
// crates/gargantua-camera/src/modes/gravity.rs
// =============================================================================
//
// PURPOSE:

//   Camera follows a geodesic trajectory under the Kerr metric.
//   The camera is treated as a test particle with initial position
//   and velocity in Boyer-Lindquist coordinates; the geodesic equation
//   is integrated on the CPU (RK4) to evolve the trajectory.
//
//   Simulates the experience of falling into a black hole or orbiting
//   along a physically accurate path. Unlike OrbitMode (which forces
//   circular orbits), GravityMode integrates the full equations of motion
//   including radial infall, orbital precession, and frame dragging.
//
//   FIELDS:
//     pos_bl:      BoyerLindquist  — current position in BL coords
//     vel_bl:      [f32; 4]        — 4-velocity (t, r, θ, φ components)
//     spin:        f32             — black hole a/M
//     mass:        f32             — black hole M
//     rk4_dt:      f32             — integration step size
//
//   KEY FUNCTIONS:
//     pub fn new(pos: BoyerLindquist, vel: [f32;4], spin: f32, mass: f32) -> Self
//     pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat)
//       — integrates geodesic by delta_t seconds.
//       — converts BL position to world Cartesian.
//       — returns (world_position, rotation_looking_at_black_hole).
//     pub fn proper_time(&self) -> f32  — elapsed proper time of the observer
//     pub fn is_inside_horizon(&self) -> bool
//
//   CALLED BY:
//     - crate::world_camera::WorldCamera — when in GravityMode
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::world::coord_system::{BoyerLindquist, bl_to_cartesian}
//     - glam::{Vec3, Quat}
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

pub struct GravityMode {
    // Fields documented above — implement per purpose spec
    _placeholder: (),
}

impl GravityMode {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat) {
        todo!()
    }
}

impl Default for GravityMode {
    fn default() -> Self { Self::new() }
}