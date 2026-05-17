// ============================================================
// FILE: shaders/render/lensing.wgsl
// LINES: ~180
// CATEGORY: Shader — Gravitational lensing overlay and photon ring
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Renders the gravitational lensing effect on the starfield (Einstein
//   ring, photon ring glow, multiple image arcs) using the pre-baked
//   geodesic LUT. Also draws the event horizon shadow (black disk)
//   and the photon ring bright arc. Composited on top of starfield.wgsl.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var geodesic_lut:  texture_2d<f32>;  // baked LUT
//   @group(0) @binding(1) var starfield_tex: texture_2d<f32>;  // rendered starfield
//   @group(0) @binding(2) var lut_sampler:   sampler;
//   @group(0) @binding(3) var<uniform>       params: LensingParams;
//
//   struct LensingParams {
//       kerr_spin:   f32,
//       r_plus:      f32,   // event horizon
//       r_ph:        f32,   // photon sphere radius
//       b_crit:      f32,   // critical impact parameter
//       photon_ring_brightness: f32,   // glow intensity multiplier
//       ergosphere_glow:        u32,   // 1 = show ergosphere shading
//       _pad:        vec2<f32>,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Compute impact parameter b from pixel position:
//     //    uv    = frag_pos.xy / screen_size
//     //    angle = pixel_to_angle(uv)  // angular position from BH center
//     //    b     = angle_to_impact_param(angle)
//     //
//     // 2. Sample geodesic LUT at (spin, b):
//     //    lut_uv = vec2(b_normalized, spin_normalized)
//     //    lut_val = textureSample(geodesic_lut, lut_sampler, lut_uv)
//     //    deflection_angle = lut_val.r
//     //    disk_hit         = lut_val.g  // 0 or 1
//     //    redshift_factor  = lut_val.b
//     //
//     // 3. Event horizon shadow:
//     //    if b < params.b_crit: return vec4(0.0)  // pure black
//     //
//     // 4. Photon ring glow (b ≈ b_crit):
//     //    ring_dist = abs(b - b_crit) / b_crit
//     //    if ring_dist < 0.05:
//     //        glow = exp(-ring_dist * 80.0) * photon_ring_brightness
//     //        ring_color = vec3(1.0, 0.9, 0.7) * glow  // warm white
//     //
//     // 5. Sample lensed starfield at deflected direction:
//     //    deflected_uv = apply_deflection(uv, deflection_angle)
//     //    star_color   = textureSample(starfield_tex, lut_sampler, deflected_uv)
//     //    star_color  *= (1.0 - redshift_factor * 0.3)  // gravitational dimming
//     //
//     // 6. Composite: return star_color + ring_color
//
// HELPER FUNCTIONS:
//   fn pixel_to_angle(uv: vec2<f32>) -> f32
//     // Maps pixel position to angular distance from BH center in radians
//
//   fn apply_deflection(uv: vec2<f32>, angle: f32) -> vec2<f32>
//     // Applies angular deflection to UV for starfield lookup
//
// USES (imports from):
//   No WGSL imports.
//   Geodesic LUT: assets/baked/geodesic_lut.exr
//
// USED BY:
//   crates/gargantua-render/src/pipelines/lensing.rs
//
// NOTE FOR AI:
//   This shader uses the BAKED geodesic LUT (not real-time RK4).
//   Real-time RK4 is in compute/geodesic_rk4.wgsl — both run per frame.
//   LUT gives faster approximate lensing for the BACKGROUND starfield.
//   RK4 compute gives accurate lensing for the accretion disk.
//   b_crit varies with spin: b_crit ≈ 3√3 M for a=0, less for prograde spin.
//   Photon ring glow: exponential falloff from b=b_crit (sharp, bright ring).
// ============================================================
