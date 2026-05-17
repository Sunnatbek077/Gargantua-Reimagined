// ============================================================
// FILE: shaders/postfx/bloom_up.wgsl
// LINES: ~100
// CATEGORY: Shader — Bloom upsample pass
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Second pass of the bloom effect. Upsamples the downsampled bloom
//   chain back to full resolution using a 9-tap tent filter and
//   additively blends each level. Creates the characteristic soft
//   glow around bright areas (accretion disk inner edge, photon ring).
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var bloom_mip:    texture_2d<f32>;  // current mip level
//   @group(0) @binding(1) var prev_upsample:texture_2d<f32>;  // previously upsampled
//   @group(0) @binding(2) var tex_sampler:  sampler;
//   @group(0) @binding(3) var<uniform>      params: UpsampleParams;
//
//   struct UpsampleParams {
//       filter_radius: f32,   // tent filter radius in texels (default 1.0)
//       blend_factor:  f32,   // blend between mip and prev_upsample [0,1]
//       intensity:     f32,   // final bloom intensity scale
//       _pad:          f32,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 9-tap tent filter (3×3 bilinear samples at half-pixel offsets):
//     // a b c
//     // d e f   weights: corners=1/16, edges=2/16, center=4/16
//     // g h i
//     // up = tent_filter(bloom_mip, uv, filter_radius)
//     //
//     // Blend with previous upsample level:
//     // return mix(prev_upsample_sample, up, blend_factor) * intensity
//
// HELPER FUNCTIONS:
//   fn tent_filter(tex: texture_2d<f32>, uv: vec2<f32>, r: f32, s: sampler) -> vec4<f32>
//     // 9-sample weighted average with tent weights
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/postfx/bloom.rs
//     → second pass: upsample 1/8 → 1/4 → 1/2 → full (3 upsample passes)
//     → final upsample result added to HDR frame before tonemap
//
// NOTE FOR AI:
//   Upsample runs in REVERSE order: 1/8 → 1/4 → 1/2 → full.
//   blend_factor: controls how much each level contributes.
//   intensity: applied once at the final upsample (to full resolution).
//   tent_filter radius: 1.0 texel in destination space (not source space).
//   Fragment shader — uses fullscreen.wgsl vertex shader.
//   Output is additively blended with the main HDR frame in tonemap.wgsl.
// ============================================================
