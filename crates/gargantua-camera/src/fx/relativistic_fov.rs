// =============================================================================
// crates/gargantua-camera/src/fx/relativistic_fov.rs
// =============================================================================
//
// PURPOSE:

//   Computes relativistic aberration of the camera's field of view
//   when moving at significant fractions of the speed of light.
//   Used by FreeFlight mode when β = |v|/c > 0.05.
//
//   Special relativistic aberration formula:
//     cos(θ') = (cos(θ) - β) / (1 - β × cos(θ))
//   where θ is the angle from the direction of motion in the rest frame
//   and θ' is the apparent angle in the moving frame.
//
//   Effect: stars in the direction of motion appear compressed into a
//   smaller solid angle (headlight effect). Stars behind appear spread out.
//
//   Practical implementation: the effective FOV changes with β.
//   This module computes the aberration-corrected FOV that is passed
//   to WorldCamera and then to SceneUniforms.fov_y_deg.
//
//   FIELDS:
//     base_fov_deg: f32   — rest-frame FOV in degrees
//     beta:         f32   — |v|/c (0.0 = stationary, <1.0 always)
//
//   KEY FUNCTIONS:
//     pub fn new(base_fov_deg: f32) -> Self
//     pub fn aberrated_fov(&self, beta: f32) -> f32
//       — returns apparent FOV in degrees for given β.
//     pub fn aberration_factor(&self) -> f32
//       — ratio of apparent FOV to rest-frame FOV.
//
//   CALLED BY:
//     - crate::modes::free_flight::FreeFlightMode::update()
//     - crates/gargantua-render/src/bindgroups::scene.rs::build_uniforms()
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CameraError
//   External:
//     - glam::Vec3
//
// NOTES FOR AI:
//   - All fx structs are pure data + math — no GPU resources.
//     They compute scalar/vector parameters that are uploaded to
//     SceneUniforms or used in CPU logic.
//   - Results are consumed by world_camera.rs and passed downstream
//     to gargantua-render/src/bindgroups/scene.rs::build_uniforms().
// =============================================================================

pub struct RelativisticFov {
    _placeholder: (),
}

impl RelativisticFov {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for RelativisticFov {
    fn default() -> Self { Self::new() }
}