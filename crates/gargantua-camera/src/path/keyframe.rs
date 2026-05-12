// =============================================================================
// crates/gargantua-camera/src/path/keyframe.rs
// =============================================================================
//
// PURPOSE:
//   Defines a camera keyframe — a single time-stamped pose (position +
//   orientation) on a camera animation path. Keyframes are interpolated
//   by spline.rs to produce smooth camera motion for cinematic shots
//   and offline renders.
//
//   Supports both simple position/rotation keyframes and physics-anchored
//   keyframes (e.g., "at Boyer-Lindquist coordinates (r=8M, θ=π/3, φ=0)
//   looking at the black hole").
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::world::coord_system::{BoyerLindquist, bl_to_cartesian}
//   External:
//     - glam::{Vec3, Quat}
//     - serde::{Serialize, Deserialize}
//
// CALLED BY:
//   - crate::path::spline::CameraSpline     — stores Vec<Keyframe>
//   - crate::path::recorder::PathRecorder  — records live Keyframes
//   - crate::modes::cinematic::CinematicMode — plays back Keyframe sequences
//   - crates/gargantua-app/src/preset/load.rs — loads Keyframes from .toml
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct Keyframe {
//     pub time:       f32,         // animation time in seconds (monotonic)
//     pub position:   Vec3,        // world-space camera position
//     pub rotation:   Quat,        // world-space camera orientation (normalized)
//     pub fov_y_deg:  f32,         // field of view in degrees (default 60.0)
//     pub easing_in:  EasingType,  // easing from previous keyframe to this one
//     pub easing_out: EasingType,  // easing from this keyframe to next one
//     pub label:      Option<String>, // optional name for UI display ("Photon sphere approach")
//   }
//
//   impl Keyframe:
//
//     pub fn new(time: f32, position: Vec3, rotation: Quat) -> Self
//       — creates with default fov_y_deg=60.0, EasingType::CatmullRom for both.
//
//     pub fn from_bl(
//       time:      f32,
//       bl:        BoyerLindquist,
//       target:    Vec3,          // point camera looks at (usually Vec3::ZERO)
//       up:        Vec3,          // up vector (usually Vec3::Y)
//       spin:      f32,
//       fov_y_deg: f32,
//     ) -> Self
//       — converts BL coordinates to world position via bl_to_cartesian().
//       — computes rotation from look_at_matrix(position, target, up) → Quat.
//       — used by preset files to specify keyframes in physics coordinates.
//
//     pub fn lerp(a: &Keyframe, b: &Keyframe, t: f32) -> Keyframe
//       — linear interpolation between two keyframes.
//       — position: Vec3::lerp(a.position, b.position, t)
//       — rotation: Quat::slerp(a.rotation, b.rotation, t) — spherical lerp
//       — fov_y_deg: f32::lerp(a.fov_y_deg, b.fov_y_deg, t)
//       — time: interpolated time value
//       — easing_in/out: taken from b (the target keyframe)
//
//     pub fn look_direction(&self) -> Vec3
//       — returns the camera's forward direction vector from self.rotation.
//       — forward = self.rotation * (-Vec3::Z)  (right-handed, Z = back)
//
// NOTES FOR AI:
//   - Keyframe.rotation is ALWAYS a normalized Quat.
//     Assert/normalize in constructors: rotation = rotation.normalize().
//   - fov_y_deg defaults to 60.0 degrees (typical cinematic FOV).
//     FOV interpolation in lerp() smoothly changes field of view
//     for "zoom" effects during playback.
//   - serde: Keyframe is serialized to/from TOML in preset files.
//     Vec3 and Quat are not serde-compatible by default — use glam's
//     "serde" feature flag (add to Cargo.toml: glam = { features=["serde"] }).
//   - time values must be monotonically increasing in a keyframe sequence.
//     spline.rs sorts keyframes by time on insertion.
// =============================================================================

use glam::{Quat, Vec3};
use crate::{
    path::easing::EasingType,
    world::coord_system::{bl_to_cartesian, look_at_matrix, BoyerLindquist},
};

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time:       f32,
    pub position:   Vec3,
    pub rotation:   Quat,
    pub fov_y_deg:  f32,
    pub easing_in:  EasingType,
    pub easing_out: EasingType,
    pub label:      Option<String>,
}

impl Keyframe {
    pub fn new(time: f32, position: Vec3, rotation: Quat) -> Self {
        Self {
            time,
            position,
            rotation: rotation.normalize(),
            fov_y_deg:  60.0,
            easing_in:  EasingType::CatmullRom,
            easing_out: EasingType::CatmullRom,
            label:      None,
        }
    }

    pub fn from_bl(
        time:      f32,
        bl:        BoyerLindquist,
        target:    Vec3,
        up:        Vec3,
        spin:      f32,
        fov_y_deg: f32,
    ) -> Self {
        let position = bl_to_cartesian(bl, spin);
        let view_mat = look_at_matrix(position, target, up);
        // Extract rotation from the 3×3 upper-left of the inverse view matrix
        let rotation = Quat::from_mat4(&view_mat.inverse()).normalize();
        Self { time, position, rotation, fov_y_deg, ..Self::new(time, position, rotation) }
    }

    pub fn lerp(a: &Keyframe, b: &Keyframe, t: f32) -> Keyframe {
        Keyframe {
            time:       a.time + (b.time - a.time) * t,
            position:   a.position.lerp(b.position, t),
            rotation:   a.rotation.slerp(b.rotation, t).normalize(),
            fov_y_deg:  a.fov_y_deg + (b.fov_y_deg - a.fov_y_deg) * t,
            easing_in:  b.easing_in,
            easing_out: b.easing_out,
            label:      b.label.clone(),
        }
    }

    pub fn look_direction(&self) -> Vec3 {
        self.rotation * (-Vec3::Z)
    }
}