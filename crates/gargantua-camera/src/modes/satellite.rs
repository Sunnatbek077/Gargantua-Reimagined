// =============================================================================
// crates/gargantua-camera/src/modes/satellite.rs
// =============================================================================
//
// PURPOSE:

//   Camera follows a stable circular (or elliptical) orbit around the
//   black hole, computed from orbital mechanics in the Kerr metric.
//   The orbit parameters (semi-major axis, eccentricity, inclination)
//   are set by the user; the camera automatically maintains the orbit
//   while the user can adjust the look direction.
//
//   Useful for B-roll shots: the camera orbits naturally while the user
//   pans/tilts to frame the black hole from different angles.
//
//   FIELDS:
//     semi_major:   f32    — orbital semi-major axis (must be > r_isco)
//     eccentricity: f32    — orbit eccentricity (0=circle, <1=ellipse)
//     inclination:  f32    — orbital plane inclination (radians from equator)
//     phase:        f32    — current orbital phase (0..2π)
//     omega:        f32    — orbital angular velocity (computed from Kepler)
//     look_offset:  Quat   — additional user-controlled rotation
//
//   KEY FUNCTIONS:
//     pub fn new(semi_major: f32, eccentricity: f32, inclination: f32, spin: f32) -> Self
//     pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat)
//       — advances orbital phase, computes world position from Keplerian orbit,
//         adds frame dragging correction for Kerr metric.
//     pub fn handle_look(&mut self, dx: f32, dy: f32)  — adjust look direction
//     pub fn orbital_period(&self) -> f32  — in seconds (geometric units)
//     pub fn orbital_velocity(&self) -> f32  — |v| / c (β parameter)
//
//   CALLED BY:
//     - crate::world_camera::WorldCamera — when in Satellite mode
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - glam::{Vec3, Quat}
//     - crate::world::coord_system::{BoyerLindquist, bl_to_cartesian}
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

pub struct SatelliteMode {
    // Fields documented above — implement per purpose spec
    _placeholder: (),
}

impl SatelliteMode {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn update(&mut self, delta_t: f32) -> (Vec3, Quat) {
        todo!()
    }
}

impl Default for SatelliteMode {
    fn default() -> Self { Self::new() }
}