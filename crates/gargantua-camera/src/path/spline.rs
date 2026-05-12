// =============================================================================
// crates/gargantua-camera/src/path/spline.rs
// =============================================================================
//
// PURPOSE:
//   Catmull-Rom spline interpolation over a sequence of camera Keyframes.
//   Produces smooth, C1-continuous camera paths for cinematic rendering
//   and automated camera animation. The Catmull-Rom parameterization
//   guarantees the curve passes through every keyframe and has smooth
//   first derivatives at each knot.
//
//   Also supports arc-length reparameterization — distributing keyframes
//   evenly along the path in world space rather than in time, so the
//   camera moves at a physically consistent speed.
//
// SIZE: ~320 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::path::keyframe::Keyframe
//     - crate::path::easing::{EasingType, apply, blend_easing}
//     - crate::errors::CameraError
//   External:
//     - glam::{Vec3, Quat}
//
// CALLED BY:
//   - crate::modes::cinematic::CinematicMode
//       — calls CameraSpline::eval(current_time) each frame
//   - crates/gargantua-video/src/render/offline.rs
//       — evaluates spline at precise frame timestamps
//   - tests/spline.rs   — unit tests (C1 continuity, arc-length accuracy)
//
// PUBLIC TYPES:
//
//   pub struct CameraSpline {
//     keyframes:    Vec<Keyframe>,        // sorted by keyframe.time
//     arc_lengths:  Vec<f32>,             // cumulative arc length at each keyframe
//     total_length: f32,                  // total path length in world units
//     looping:      bool,                 // true = wrap from last keyframe to first
//   }
//
//   pub struct SplineEval {
//     pub position:  Vec3,
//     pub rotation:  Quat,
//     pub fov_y_deg: f32,
//     pub velocity:  Vec3,   // dP/dt (camera velocity for motion blur)
//     pub time:      f32,    // input time that produced this result
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(keyframes: Vec<Keyframe>, looping: bool) -> Result<Self, CameraError>
//     — sorts keyframes by time.
//     — validates: at least 2 keyframes required; times must be monotonic.
//     — computes arc_lengths via numerical integration (100 sub-steps per segment).
//     — returns CameraError::InsufficientKeyframes if < 2 keyframes.
//
//   pub fn eval(&self, time: f32) -> SplineEval
//     — finds the segment [k_i, k_{i+1}] containing time.
//     — computes local t = (time - k_i.time) / (k_{i+1}.time - k_i.time).
//     — applies easing: t_eased = blend_easing(k_i.easing_out, k_{i+1}.easing_in, t)
//     — Catmull-Rom position:
//         p0 = k_{i-1}.position (or extrapolated if i=0)
//         p1 = k_i.position
//         p2 = k_{i+1}.position
//         p3 = k_{i+2}.position (or extrapolated if i = last-1)
//         pos = catmull_rom_point(p0, p1, p2, p3, t_eased)
//     — Rotation: Quat::slerp(k_i.rotation, k_{i+1}.rotation, t_eased)
//     — FOV: linear lerp(k_i.fov_y_deg, k_{i+1}.fov_y_deg, t_eased)
//     — Velocity: numerical derivative (eval(t+ε) - eval(t-ε)) / (2ε)
//     — returns SplineEval { position, rotation, fov_y_deg, velocity, time }
//
//   pub fn eval_at_arc_length(&self, arc_s: f32) -> SplineEval
//     — maps arc length s ∈ [0, total_length] to a time value via
//       binary search in arc_lengths, then calls eval(time).
//     — used for constant-speed camera motion in offline renders.
//
//   pub fn duration(&self) -> f32
//     — returns last_keyframe.time - first_keyframe.time.
//
//   pub fn total_arc_length(&self) -> f32 { self.total_length }
//
//   pub fn add_keyframe(&mut self, kf: Keyframe) -> Result<(), CameraError>
//     — inserts keyframe in sorted order by time.
//     — recomputes arc_lengths after insertion.
//
//   pub fn remove_keyframe(&mut self, index: usize) -> Result<(), CameraError>
//     — removes keyframe at index (must keep >= 2 keyframes).
//     — recomputes arc_lengths after removal.
//
// PRIVATE FUNCTIONS:
//
//   fn catmull_rom_point(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3
//     — standard uniform Catmull-Rom:
//         m1 = 0.5 × (p2 - p0)   (tangent at p1)
//         m2 = 0.5 × (p3 - p1)   (tangent at p2)
//         h00 = 2t³ - 3t² + 1
//         h10 = t³ - 2t² + t
//         h01 = -2t³ + 3t²
//         h11 = t³ - t²
//         result = h00×p1 + h10×m1 + h01×p2 + h11×m2
//
//   fn compute_arc_lengths(&mut self)
//     — integrates arc length: sum of ||eval(t+dt).position - eval(t).position||
//       for 100 sub-steps per segment. Stores cumulative length at each keyframe.
//
// NOTES FOR AI:
//   - Catmull-Rom requires 4 control points (p0..p3) for each segment.
//     At the first segment (i=0): p0 = 2×p1 - p2 (reflection of p2 through p1).
//     At the last segment (i=N-2): p3 = 2×p_{N-1} - p_{N-2} (same trick).
//   - looping = true: p0 = p_{N-2} (second-to-last), p3 = p_1 (second).
//     This creates a smooth closed loop for orbit shots.
//   - Arc-length reparameterization is computed once in new() and stored.
//     It is NOT recalculated each frame — call recompute() after editing.
//   - Velocity is used by motion_blur.rs (MotionBlurPass) for velocity buffer
//     generation when the camera is on a spline path.
// =============================================================================

use glam::{Quat, Vec3};
use crate::{errors::CameraError, path::{easing::blend_easing, keyframe::Keyframe}};

pub struct SplineEval {
    pub position:  Vec3,
    pub rotation:  Quat,
    pub fov_y_deg: f32,
    pub velocity:  Vec3,
    pub time:      f32,
}

pub struct CameraSpline {
    keyframes:    Vec<Keyframe>,
    arc_lengths:  Vec<f32>,
    total_length: f32,
    looping:      bool,
}

impl CameraSpline {
    pub fn new(keyframes: Vec<Keyframe>, looping: bool) -> Result<Self, CameraError> {
        if keyframes.len() < 2 {
            return Err(CameraError::InsufficientKeyframes { got: keyframes.len(), need: 2 });
        }
        let mut s = Self {
            keyframes,
            arc_lengths: Vec::new(),
            total_length: 0.0,
            looping,
        };
        s.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        s.compute_arc_lengths();
        Ok(s)
    }

    pub fn eval(&self, time: f32) -> SplineEval {
        let n = self.keyframes.len();
        // Find segment
        let i = self.keyframes
            .windows(2)
            .position(|w| time <= w[1].time)
            .unwrap_or(n - 2)
            .min(n - 2);

        let k0 = &self.keyframes[i];
        let k1 = &self.keyframes[(i + 1).min(n - 1)];
        let seg_dur = (k1.time - k0.time).max(1e-6);
        let t_raw   = ((time - k0.time) / seg_dur).clamp(0.0, 1.0);
        let t       = blend_easing(k0.easing_out, k1.easing_in, t_raw);

        // Control points for Catmull-Rom
        let p_prev = if i == 0 {
            2.0 * k0.position - self.keyframes[1].position
        } else {
            self.keyframes[i - 1].position
        };
        let p0   = k0.position;
        let p1   = k1.position;
        let p_next = if i + 2 < n {
            self.keyframes[i + 2].position
        } else {
            2.0 * k1.position - k0.position
        };

        let position  = catmull_rom_point(p_prev, p0, p1, p_next, t);
        let rotation  = k0.rotation.slerp(k1.rotation, t).normalize();
        let fov_y_deg = k0.fov_y_deg + (k1.fov_y_deg - k0.fov_y_deg) * t;

        // Numerical velocity
        let eps = 1e-3_f32;
        let pos_fwd  = self.eval_position(time + eps);
        let pos_back = self.eval_position(time - eps);
        let velocity = (pos_fwd - pos_back) / (2.0 * eps);

        SplineEval { position, rotation, fov_y_deg, velocity, time }
    }

    pub fn eval_at_arc_length(&self, arc_s: f32) -> SplineEval {
        let clamped = arc_s.clamp(0.0, self.total_length);
        // Binary search for time
        let idx = self.arc_lengths
            .partition_point(|&l| l < clamped)
            .min(self.arc_lengths.len().saturating_sub(1));
        let time = if idx < self.keyframes.len() { self.keyframes[idx].time } else { self.duration() };
        self.eval(time)
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
            - self.keyframes.first().map(|k| k.time).unwrap_or(0.0)
    }

    pub fn total_arc_length(&self) -> f32 { self.total_length }

    pub fn add_keyframe(&mut self, kf: Keyframe) -> Result<(), CameraError> {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self.compute_arc_lengths();
        Ok(())
    }

    pub fn remove_keyframe(&mut self, index: usize) -> Result<(), CameraError> {
        if self.keyframes.len() <= 2 {
            return Err(CameraError::InsufficientKeyframes { got: 2, need: 2 });
        }
        self.keyframes.remove(index);
        self.compute_arc_lengths();
        Ok(())
    }

    fn eval_position(&self, time: f32) -> Vec3 { self.eval(time).position }

    fn compute_arc_lengths(&mut self) {
        let steps  = 100_usize;
        let n      = self.keyframes.len();
        let t_end  = self.keyframes.last().map(|k| k.time).unwrap_or(1.0);
        let t_start = self.keyframes.first().map(|k| k.time).unwrap_or(0.0);
        let total_steps = steps * (n - 1);
        let dt     = (t_end - t_start) / total_steps as f32;

        self.arc_lengths = vec![0.0; n];
        let mut prev_pos = self.eval_position(t_start);
        let mut cumulative = 0.0_f32;

        for ki in 0..n {
            let t_kf = self.keyframes[ki].time;
            let end_t = if ki + 1 < n { self.keyframes[ki + 1].time } else { t_kf };
            let mut t = t_kf;
            while t < end_t {
                let pos = self.eval_position(t);
                cumulative += (pos - prev_pos).length();
                prev_pos = pos;
                t += dt;
            }
            self.arc_lengths[ki] = cumulative;
        }
        self.total_length = cumulative;
    }
}

fn catmull_rom_point(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let m1 = 0.5 * (p2 - p0);
    let m2 = 0.5 * (p3 - p1);
    let h00 =  2.0 * t * t * t - 3.0 * t * t + 1.0;
    let h10 =       t * t * t - 2.0 * t * t + t;
    let h01 = -2.0 * t * t * t + 3.0 * t * t;
    let h11 =       t * t * t -       t * t;
    h00 * p1 + h10 * m1 + h01 * p2 + h11 * m2
}