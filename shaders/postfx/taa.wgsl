// ============================================================
// FILE: shaders/postfx/taa.wgsl
// LINES: ~200
// CATEGORY: Shader — Temporal Anti-Aliasing (TAA)
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Temporal Anti-Aliasing: blends the current frame with a reprojected
//   history frame to reduce aliasing. Uses neighbourhood clamping to
//   prevent ghosting when the scene changes. Runs after ray_march.wgsl
//   and before postfx. Complements accumulate.wgsl for still frames.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var current_frame:  texture_2d<f32>;
//   @group(0) @binding(1) var history_frame:  texture_2d<f32>;
//   @group(0) @binding(2) var tex_sampler:    sampler;
//   @group(0) @binding(3) var<uniform>        params: TaaParams;
//
//   struct TaaParams {
//       blend_alpha:   f32,   // history blend [0.05–0.2], default 0.1
//       variance_gamma:f32,   // neighbourhood clamp tightness [0.5–2.0]
//       _pad:          vec2<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Sample current frame at pixel
//     //    curr = textureLoad(current_frame, coord, 0)
//     //
//     // 2. Sample history (no reprojection — static camera optimization)
//     //    hist = textureSample(history_frame, tex_sampler, uv)
//     //
//     // 3. Neighbourhood variance clamping (anti-ghosting):
//     //    Compute min/max of 3×3 neighbourhood in current frame
//     //    hist_clamped = clamp(hist, neighbourhood_min, neighbourhood_max)
//     //    (prevents ghosting from old history when scene changes)
//     //
//     // 4. Blend: return mix(hist_clamped, curr, blend_alpha)
//     //    (blend_alpha=0.1: 90% history, 10% current → heavy temporal smoothing)
//
// HELPER FUNCTIONS:
//   fn neighbourhood_aabb(tex: texture_2d<f32>, coord: vec2<i32>)
//       -> (vec3<f32>, vec3<f32>)   // (min_color, max_color)
//     // Samples 3×3 window around coord, returns AABB of color values
//     // Used for variance clamping
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/postfx/taa.rs
//     → run once per frame after ray marching, before bloom
//
// NOTE FOR AI:
//   TAA requires two history textures (ping-pong each frame).
//   blend_alpha=0.1: converges in ~10 frames, good ghosting/noise tradeoff.
//   variance_gamma=1.0: standard neighbourhood clamp width.
//   No velocity buffer here: static camera assumed (black holes are static).
//   For camera motion: disable TAA and use accumulate.wgsl instead.
//   3×3 neighbourhood: 9 texture reads — acceptable at fullscreen resolution.
// ============================================================
