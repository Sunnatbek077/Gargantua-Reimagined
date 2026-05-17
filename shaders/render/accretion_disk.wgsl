// ============================================================
// FILE: shaders/render/accretion_disk.wgsl
// LINES: ~280
// CATEGORY: Shader — Accretion disk final color composition
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Composites the accretion disk color from ray_march.wgsl output
//   onto the final frame. Applies the Doppler LUT for wavelength-shift
//   based color adjustment, adds the inner glow (photon ring proximity
//   glow), disk edge glow, and computes the final RGBA output for the
//   disk pixels. Starfield pixels are left for starfield.wgsl.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var<storage, read> ray_results:  array<RayResult>;
//   @group(0) @binding(1) var<storage, read> disk_colors:  array<vec4<f32>>;
//   @group(0) @binding(2) var                doppler_lut:  texture_2d<f32>;
//   @group(0) @binding(3) var                blackbody_lut:texture_2d<f32>;
//   @group(0) @binding(4) var                curl_noise:   texture_3d<f32>;
//   @group(0) @binding(5) var                lut_sampler:  sampler;
//   @group(0) @binding(6) var<uniform>        disk_params:  DiskRenderParams;
//
//   struct DiskRenderParams {
//       r_inner:       f32,
//       r_outer:       f32,
//       kerr_spin:     f32,
//       inner_glow:    u32,   // 1 = photon ring proximity glow
//       jet_on:        u32,
//       jet_power_w:   f32,   // BZ jet power in Watts (for intensity scaling)
//       frame_idx:     u32,
//       _pad:          f32,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Load RayResult and disk_colors for this pixel
//     //    if hit_horizon: return vec4(0.0)  (black hole interior)
//     //    if disk_r == 0.0: return vec4(0.0)  (no disk, let starfield through)
//     //
//     // 2. Retrieve base disk color from disk_colors (from ray_march.wgsl)
//     //    Already has Doppler + redshift applied
//     //
//     // 3. Apply Doppler LUT color shift (wavelength-shifted color):
//     //    beta = orbital_beta(disk_r, kerr_spin)
//     //    doppler_uv = vec2(beta / 0.99, (disk_cos_angle + 1.0) / 2.0)
//     //    shift = textureSample(doppler_lut, lut_sampler, doppler_uv).r
//     //    Apply wavelength shift to RGB via blackbody_lut resample
//     //
//     // 4. Inner glow (photon ring proximity):
//     //    if inner_glow && disk_r < r_inner * 1.5:
//     //        glow = exp(-10.0 * (disk_r / r_inner - 1.0)) * 2.0
//     //        color += glow * vec3(1.0, 0.7, 0.3)   // warm orange glow
//     //
//     // 5. Disk edge fade:
//     //    outer_fade = smoothstep(r_outer, r_outer * 0.85, disk_r)
//     //    color *= outer_fade
//     //
//     // 6. Jet contribution (if jet_on):
//     //    jet_z = ... (distance above disk plane along jet axis)
//     //    jet_color = jet_emission(jet_z, jet_power_w, frame_idx)
//     //    color += jet_color
//     //
//     // 7. return vec4(color, 1.0)
//
// HELPER FUNCTIONS:
//   fn orbital_beta(r: f32, a: f32) -> f32
//   fn jet_emission(z: f32, power: f32, seed: u32) -> vec3<f32>
//     // Conical jet: dim, bluish-white, turbulent (uses pcg_hash + curl_noise)
//
// USES (imports from):
//   No WGSL imports.
//   Reads: geodesic_rk4.wgsl output (RayResult), ray_march.wgsl output (disk_colors)
//
// USED BY:
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → final disk composition pass
//
// NOTE FOR AI:
//   disk_colors contains pre-computed colors from ray_march.wgsl.
//   This shader does NOT re-compute temperature — it uses the already-computed color.
//   Doppler LUT resample: shift the effective temperature, then re-sample blackbody.
//   inner_glow: adds warm orange halo near the ISCO (photon ring region).
//   outer_fade: prevents hard disk edge cutoff — smooth exponential fade.
// ============================================================
