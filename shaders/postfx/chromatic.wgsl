// ============================================================
// FILE: shaders/postfx/chromatic.wgsl
// LINES: ~100
// CATEGORY: Shader — Chromatic aberration post-process
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Simulates optical chromatic aberration: R, G, B channels are
//   sampled at slightly different UV offsets, causing color fringing
//   at high-contrast edges (like a real camera lens).
//   Applied before tonemapping. Strength controlled by PostFxTab.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var hdr_texture: texture_2d<f32>;
//   @group(0) @binding(1) var tex_sampler: sampler;
//   @group(0) @binding(2) var<uniform>    params: ChromaticParams;
//
//   struct ChromaticParams {
//       strength:    f32,   // aberration strength [0.0–1.0], 0=off
//       radial_only: u32,   // 1=radial falloff, 0=uniform
//       _pad:        vec2<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // center = vec2(0.5, 0.5)
//     // uv     = frag_pos.xy / screen_size
//     // offset = (uv - center) * strength * 0.01
//     //
//     // if radial_only: offset *= length(uv - center)  (stronger at edges)
//     //
//     // R = textureSample(hdr_texture, tex_sampler, uv + offset * 1.0).r
//     // G = textureSample(hdr_texture, tex_sampler, uv).g
//     // B = textureSample(hdr_texture, tex_sampler, uv - offset * 1.0).b
//     //
//     // return vec4(R, G, B, 1.0)
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/pipelines/postfx.rs
//     → applied after bloom, before tonemap
//
// NOTE FOR AI:
//   strength=0.0 → identity pass (no performance cost, just sampling).
//   For strength>0: R channel shifts outward, B shifts inward (barrel).
//   radial_only=1 gives more realistic lens behavior (stronger at edges).
//   UV offsets are in normalized [0,1] space — multiply by 0.01 for subtlety.
//   Fragment shader — fullscreen quad via fullscreen.wgsl vertex.
// ============================================================
