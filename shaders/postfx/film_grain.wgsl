// ============================================================
// FILE: shaders/postfx/film_grain.wgsl
// LINES: ~100
// CATEGORY: Shader — Film grain noise overlay
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Adds time-varying film grain noise to the tonemapped image.
//   Applied AFTER tonemapping (in LDR sRGB space). The grain pattern
//   changes every frame (driven by frame_idx) to simulate real film.
//   Grain intensity is luminance-adaptive: brighter areas have less grain.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var ldr_texture:  texture_2d<f32>;  // tonemapped
//   @group(0) @binding(1) var blue_noise:   texture_2d<f32>;  // 256×256
//   @group(0) @binding(2) var tex_sampler:  sampler;
//   @group(0) @binding(3) var<uniform>      params: GrainParams;
//
//   struct GrainParams {
//       strength:  f32,   // grain intensity [0.0–0.5]
//       frame_idx: u32,   // frame counter (for animation)
//       _pad:      vec2<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Sample blue noise with frame-offset UV:
//     //    noise_uv = (frag_pos.xy + vec2(frame_idx * 17u, frame_idx * 13u)) / 256.0
//     //    noise    = textureSample(blue_noise, tex_sampler, noise_uv).r * 2.0 - 1.0
//     //    (centered noise ∈ [-1, 1])
//     //
//     // 2. Luminance-adaptive scale:
//     //    color = textureSample(ldr_texture, tex_sampler, uv)
//     //    lum   = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722))
//     //    grain_scale = strength * (1.0 - lum * 0.5)  // less grain in highlights
//     //
//     // 3. Apply grain:
//     //    return vec4(color.rgb + noise * grain_scale, 1.0)
//
// USES (imports from):
//   No WGSL imports.
//   Blue noise: assets/baked/blue_noise_256.exr (loaded by render pipeline)
//
// USED BY:
//   crates/gargantua-render/src/pipelines/postfx.rs  → last postfx pass
//
// NOTE FOR AI:
//   Applied AFTER tonemap — operates in LDR sRGB [0,1] range.
//   frame_idx offsets: prime multipliers (17, 13) to avoid tiling patterns.
//   Luminance-adaptive: dark areas get more grain (matches film behavior).
//   Blue noise grain: much better than white noise (no clumping/banding).
//   strength=0.0 → identity. strength=0.05 is subtle, 0.2 is heavy grain.
// ============================================================
