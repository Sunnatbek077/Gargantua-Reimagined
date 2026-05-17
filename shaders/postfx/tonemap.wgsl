// ============================================================
// FILE: shaders/postfx/tonemap.wgsl
// LINES: ~200
// CATEGORY: Shader — HDR tonemapping and gamma correction
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Final post-processing pass: converts HDR linear light values to
//   display-ready LDR (or EDR on Mac). Applies the selected tonemap
//   operator (ACES/Reinhard/Filmic/None), exposure adjustment, contrast,
//   and sRGB gamma encoding. This is the ONLY shader that applies gamma.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var hdr_texture:  texture_2d<f32>;
//   @group(0) @binding(1) var bloom_texture:texture_2d<f32>;
//   @group(0) @binding(2) var tex_sampler:  sampler;
//   @group(0) @binding(3) var<uniform>      params: TonemapParams;
//
//   struct TonemapParams {
//       exposure:      f32,   // EV offset: color *= pow(2, exposure)
//       contrast:      f32,   // contrast [0.5–2.0], 1.0=neutral
//       bloom_mix:     f32,   // bloom additive blend [0.0–1.0]
//       tonemap_mode:  u32,   // 0=ACES, 1=Reinhard, 2=Filmic, 3=None
//       hdr_enabled:   u32,   // 1=Display P3 EDR (Mac), 0=sRGB
//       vignette:      f32,   // vignette intensity [0.0–1.0]
//       _pad:          vec2<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Load HDR color + add bloom:
//     //    color = textureSample(hdr_texture, ...) + bloom * bloom_mix
//     //
//     // 2. Apply exposure:
//     //    color *= pow(2.0, params.exposure)
//     //
//     // 3. Apply contrast (around 0.18 mid-gray):
//     //    color = pow(color / 0.18, contrast) * 0.18
//     //
//     // 4. Tonemap:
//     //    switch tonemap_mode:
//     //        0 → aces_fitted(color)
//     //        1 → reinhard(color)
//     //        2 → filmic_hejl(color)
//     //        3 → color (linear passthrough for HDR display)
//     //
//     // 5. Apply vignette:
//     //    center_dist = length(uv - 0.5) * 2.0
//     //    vignette_factor = 1.0 - vignette * center_dist^2
//     //    color *= vignette_factor
//     //
//     // 6. Gamma encode:
//     //    if hdr_enabled: return vec4(color, 1.0)  // linear for P3 EDR
//     //    else:           return vec4(srgb_gamma(color), 1.0)
//
// HELPER FUNCTIONS:
//   fn aces_fitted(x: vec3<f32>) -> vec3<f32>
//     // Narkowicz 2015 ACES approximation (fast, film-like):
//     // x = x * 0.6
//     // a=2.51, b=0.03, c=2.43, d=0.59, e=0.14
//     // return saturate((x*(a*x+b)) / (x*(c*x+d)+e))
//
//   fn reinhard(x: vec3<f32>) -> vec3<f32>
//     // x / (1.0 + x)  — simple, preserves hue but clips highlights
//
//   fn filmic_hejl(x: vec3<f32>) -> vec3<f32>
//     // Jim Hejl & Richard Burgess-Dawson (2010):
//     // Slightly filmic S-curve, free of artifacts
//
//   fn srgb_gamma(linear: vec3<f32>) -> vec3<f32>
//     // IEC 61966-2-1 piecewise transfer function:
//     // if c <= 0.0031308: 12.92 * c
//     // else: 1.055 * pow(c, 1/2.4) - 0.055
//
// USES (imports from):
//   No WGSL imports.
//
// USED BY:
//   crates/gargantua-render/src/pipelines/postfx.rs
//     → final pass, renders to swapchain surface
//
// NOTE FOR AI:
//   This is the ONLY shader that applies sRGB gamma — never apply gamma
//   in any earlier shader. All intermediate buffers are linear HDR.
//   hdr_enabled=1 (Mac EDR): output stays linear, wgpu surface is
//   configured as Rgba16Float + Display P3 color space.
//   hdr_enabled=0: sRGB gamma applied, wgpu surface is Bgra8UnormSrgb.
//   bloom is additively blended BEFORE tonemapping (so it toneamps together).
//   vignette: applied AFTER tonemap (in LDR) for consistent darkening.
// ============================================================
