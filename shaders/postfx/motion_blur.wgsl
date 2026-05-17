// ============================================================
// FILE: shaders/postfx/motion_blur.wgsl
// LINES: ~160
// CATEGORY: Shader — Per-object motion blur (camera motion)
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Screen-space motion blur based on the camera velocity vector.
//   Samples the HDR texture along the reprojected motion direction
//   for each pixel. Applied only when camera moves (beta > 0 or
//   camera animation is active). Simulates camera shutter motion.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var hdr_texture:    texture_2d<f32>;
//   @group(0) @binding(1) var depth_texture:  texture_2d<f32>;  // optional
//   @group(0) @binding(2) var tex_sampler:    sampler;
//   @group(0) @binding(3) var<uniform>        params: MotionBlurParams;
//
//   struct MotionBlurParams {
//       velocity_vec:  vec2<f32>,  // screen-space motion vector (pixels/frame)
//       num_samples:   u32,        // tap count [4–16], default 8
//       strength:      f32,        // blur scale [0.0–1.0]
//       max_blur_px:   f32,        // max blur radius in pixels (default 32)
//       _pad:          vec3<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // blur_dir = velocity_vec * strength / screen_size
//     // clamp blur_dir length to max_blur_px / screen_size
//     //
//     // Accumulate num_samples along blur_dir:
//     //   for i in 0..num_samples:
//     //     t = (f32(i) / f32(num_samples - 1)) - 0.5   // centered [-0.5, 0.5]
//     //     sample_uv = uv + blur_dir * t
//     //     accum += textureSample(hdr_texture, tex_sampler, sample_uv)
//     //   return accum / f32(num_samples)
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/postfx/motion_blur.rs
//     → applied before bloom when camera velocity > threshold
//
// NOTE FOR AI:
//   velocity_vec is computed in Rust from camera delta position per frame.
//   strength=0.0 or velocity near zero → skip this pass entirely in Rust.
//   num_samples=8 default: good quality at low cost.
//   max_blur_px=32: clamps extreme velocities (prevents smearing full screen).
//   This is CAMERA motion blur only — not per-object (no velocity buffer).
//   For relativistic camera speeds: motion blur is expected to be extreme.
// ============================================================
