// =============================================================================
// crates/gargantua-camera/src/path/easing.rs
// =============================================================================
//
// PURPOSE:
//   Defines easing functions and the EasingType enum used by keyframe
//   interpolation in spline.rs. Easing controls the acceleration profile
//   of camera movement between keyframes — linear, smooth, or physically
//   motivated curves.
//
//   Each EasingType maps t ∈ [0,1] → eased_t ∈ [0,1] where eased_t
//   describes the interpolation progress with the desired motion feel.
//
// SIZE: ~160 lines
//
// DEPENDENCIES: none
//
// CALLED BY:
//   - crate::path::keyframe::Keyframe     — stores easing_in / easing_out
//   - crate::path::spline::CameraSpline   — applies easing during eval()
//   - crates/gargantua-ui/src/panel::path_editor.rs
//       — displays easing type selector per keyframe
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum EasingType {
//     Linear,        // constant velocity — use for pure motion blur tests
//     SmoothStep,    // cubic S-curve: 3t² - 2t³ — most common default
//     SmootherStep,  // quintic S-curve: 6t⁵ - 15t⁴ + 10t³ — C² continuous
//     EaseIn,        // slow start, fast end: t² (quadratic)
//     EaseOut,       // fast start, slow end: 1-(1-t)² (quadratic)
//     EaseInOut,     // combined ease in+out: cubic Hermite
//     CatmullRom,    // no remapping — Catmull-Rom spline handles continuity
//     Spring,        // slight overshoot then settle: physical spring feel
//     Bounce,        // 3 bounces at end — used for dramatic arrival
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn apply(easing: EasingType, t: f32) -> f32
//     — maps t ∈ [0,1] → eased value ∈ [0,1] for most types.
//       (Spring and Bounce may slightly exceed [0,1] for the overshoot.)
//     — clamps input t to [0,1] before applying.
//     — implementations:
//
//       Linear:       t
//
//       SmoothStep:   t * t * (3.0 - 2.0 * t)
//
//       SmootherStep: t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
//
//       EaseIn:       t * t
//
//       EaseOut:      1.0 - (1.0 - t) * (1.0 - t)
//
//       EaseInOut:    if t < 0.5 { 2t² } else { 1 - (-2t+2)²/2 }
//
//       CatmullRom:   t  (identity — spline handles continuity natively)
//
//       Spring:       spring_ease(t)  — see below
//
//       Bounce:       bounce_ease(t)  — see below
//
//   pub fn spring_ease(t: f32) -> f32
//     — models a damped spring with:
//         frequency = 2π × 1.5  (1.5 oscillations at t=1)
//         damping   = 0.4       (slight underdamping — visible overshoot)
//     — formula: 1 - exp(-damping × t × 8) × cos(frequency × t)
//     — overshoot: peaks at ~1.08 before settling to 1.0.
//     — use for dramatic camera arrivals (approach to photon sphere).
//
//   pub fn bounce_ease(t: f32) -> f32
//     — piecewise quadratic bounce with 3 bounces:
//         if t < 1/2.75:     7.5625 × t²
//         elif t < 2/2.75:   7.5625 × (t-1.5/2.75)² + 0.75
//         elif t < 2.5/2.75: 7.5625 × (t-2.25/2.75)² + 0.9375
//         else:              7.5625 × (t-2.625/2.75)² + 0.984375
//     — creates a bouncy landing effect. Use sparingly (can feel cartoonish).
//
//   pub fn blend_easing(
//     out_type: EasingType,  // easing_out of the previous keyframe
//     in_type:  EasingType,  // easing_in of the current keyframe
//     t:        f32,
//   ) -> f32
//     — combines two easing curves across a keyframe boundary.
//     — blends: apply(out_type, t * 0.5) * 2.0 for t < 0.5
//               apply(in_type, (t-0.5) * 2.0) * 0.5 + 0.5 for t >= 0.5
//     — used by spline.rs for inter-keyframe interpolation.
//
// NOTES FOR AI:
//   - All easing functions are pure mathematical functions — no state.
//     Thread-safe; call from any thread.
//   - EasingType::CatmullRom returns identity (t) because Catmull-Rom
//     spline interpolation already provides C1 continuity at keyframes
//     without additional easing. Adding easing would double-apply smoothing.
//   - Spring and Bounce may return values slightly outside [0,1].
//     Position interpolation handles this naturally (overshoot in world space).
//     FOV and rotation interpolation should clamp their easing output if needed.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingType {
    Linear,
    SmoothStep,
    SmootherStep,
    EaseIn,
    EaseOut,
    EaseInOut,
    CatmullRom,
    Spring,
    Bounce,
}

pub fn apply(easing: EasingType, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        EasingType::Linear       => t,
        EasingType::SmoothStep   => t * t * (3.0 - 2.0 * t),
        EasingType::SmootherStep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
        EasingType::EaseIn       => t * t,
        EasingType::EaseOut      => 1.0 - (1.0 - t) * (1.0 - t),
        EasingType::EaseInOut    => {
            if t < 0.5 { 2.0 * t * t }
            else       { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
        },
        EasingType::CatmullRom   => t,
        EasingType::Spring       => spring_ease(t),
        EasingType::Bounce       => bounce_ease(t),
    }
}

pub fn spring_ease(t: f32) -> f32 {
    let damping   = 0.4_f32;
    let frequency = std::f32::consts::TAU * 1.5;
    1.0 - (-damping * t * 8.0).exp() * (frequency * t).cos()
}

pub fn bounce_ease(t: f32) -> f32 {
    let n1: f32 = 7.5625;
    let d1: f32 = 2.75;
    let mut t = t;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        t -= 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        t -= 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        t -= 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

pub fn blend_easing(out_type: EasingType, in_type: EasingType, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        apply(out_type, t * 2.0) * 0.5
    } else {
        apply(in_type, (t - 0.5) * 2.0) * 0.5 + 0.5
    }
}