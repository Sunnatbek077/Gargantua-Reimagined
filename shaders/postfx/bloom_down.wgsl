// ============================================================
// FILE: shaders/postfx/bloom_down.wgsl
// LINES: ~120
// CATEGORY: Shader — Bloom downsample pass
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   First pass of the two-pass bloom effect. Downsamples the HDR
//   frame buffer by 2× with a 13-tap Kawase filter while applying
//   a luminance threshold to extract only the bright (blooming) areas.
//   Output feeds into bloom_up.wgsl for the upsample/blur pass.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var hdr_texture: texture_2d<f32>;
//   @group(0) @binding(1) var tex_sampler: sampler;
//   @group(0) @binding(2) var<uniform> bloom: BloomParams;
//
//   struct BloomParams {
//       threshold:  f32,   // luminance threshold (default 1.0)
//       knee:       f32,   // soft knee width (default 0.5)
//       intensity:  f32,   // bloom intensity multiplier
//       _pad:       f32,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 13-tap Kawase filter (dual Kawase blur — better than gaussian):
//     // Samples at center + 4 corners at offset 0.5px + 4 corners at 1.5px
//     // + 4 corners at 2.5px (weighted average)
//     //
//     // Apply threshold with soft knee:
//     //   rq = clamp(lum - threshold + knee, 0, 2*knee) / (2*knee)
//     //   weight = rq * rq * 0.25 + step(threshold, lum) * (1 - 0.25)
//     //   return sample * weight
//     //
//     // Returns: downsampled, threshold-filtered HDR color
//
// HELPER FUNCTIONS:
//   fn luminance(c: vec3<f32>) -> f32
//     // dot(c, vec3(0.2126, 0.7152, 0.0722))  // BT.709 coefficients
//
//   fn soft_threshold(lum: f32, threshold: f32, knee: f32) -> f32
//     // Smooth ramp: 0 below threshold, 1 above threshold+knee
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/postfx/bloom.rs
//     → first pass of bloom: downsample to half resolution
//     → chained: full → 1/2 → 1/4 → 1/8 (3 downsample passes)
//
// NOTE FOR AI:
//   This is a FRAGMENT shader (not compute) — runs on a fullscreen quad.
//   Vertex shader: fullscreen.wgsl (draws a clip-space triangle).
//   Output resolution: input_width/2 × input_height/2.
//   Run 3 times in sequence: 1/2, 1/4, 1/8 resolution.
//   Kawase filter: much cheaper than separable Gaussian at similar quality.
//   threshold default 1.0: only pixels brighter than white bloom.
//   All values are in LINEAR light space — NO gamma here.
// ============================================================
