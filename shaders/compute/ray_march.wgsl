// ============================================================
// FILE: shaders/compute/ray_march.wgsl
// LINES: ~320
// CATEGORY: Shader — Volumetric ray marching through accretion disk
// STAGE: Compute  (@compute @workgroup_size(8, 8, 1))
// ============================================================
//
// PURPOSE:
//   After geodesic_rk4.wgsl traces ray paths, this shader marches
//   through the volumetric accretion disk along those paths to
//   compute emission and absorption. Handles photon ring (multiple
//   disk crossings), disk emission profile, and disk glow.
//   Also applies gravitational redshift and Doppler beaming per fragment.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var<uniform>          kerr:       KerrParams;
//   @group(0) @binding(1) var<uniform>          disk:       DiskParams;
//   @group(0) @binding(2) var<storage, read>    ray_results:array<RayResult>;
//   @group(0) @binding(3) var                   blackbody_lut: texture_2d<f32>;
//   @group(0) @binding(4) var                   doppler_lut:   texture_2d<f32>;
//   @group(0) @binding(5) var                   curl_noise_3d: texture_3d<f32>;
//   @group(0) @binding(6) var                   lut_sampler:   sampler;
//   @group(0) @binding(7) var<storage, read_write> color_out: array<vec4<f32>>;
//
//   struct DiskParams {
//       r_inner:        f32,   // ISCO radius
//       r_outer:        f32,   // outer disk radius
//       temp_peak_k:    f32,   // peak disk temperature in Kelvin
//       accretion_rate: f32,   // Eddington fraction
//       beta_disk:      f32,   // plasma β (MHD turbulence)
//       frame_idx:      u32,   // for time-varying turbulence
//       doppler_on:     u32,   // 1 = apply Doppler, 0 = skip
//       redshift_on:    u32,   // 1 = apply gravitational redshift
//   }
//
// ENTRY POINT:
//   @compute @workgroup_size(8, 8, 1)
//   fn main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // 1. Load RayResult for this pixel
//     //    if hit_horizon: output black (absorbed), return
//     //
//     // 2. If disk_r > 0 (primary disk crossing):
//     //    temp = disk_temperature(disk_r, kerr.spin)  // NT model
//     //    temp = apply_mhd_turbulence(temp, disk_r, disk_phi, frame_idx)
//     //
//     //    if disk.redshift_on:
//     //        z = gravitational_redshift(disk_r, PI/2.0, kerr.spin)
//     //        temp = temp / (1.0 + z)
//     //
//     //    if disk.doppler_on:
//     //        beta = orbital_beta(disk_r, kerr.spin)
//     //        d4   = doppler_d4(beta, disk_cos_angle)
//     //        temp = temp * doppler_factor(beta, disk_cos_angle)
//     //        intensity_scale = d4
//     //    else:
//     //        intensity_scale = 1.0
//     //
//     //    color = sample_blackbody_lut(temp, blackbody_lut, lut_sampler)
//     //    color *= intensity_scale
//     //
//     // 3. Photon ring (secondary crossings):
//     //    Not handled in this pass (separate photon_ring pass in pipeline)
//     //
//     // 4. If no disk hit: color = vec4(0.0) (starfield handled by render pass)
//     //
//     // 5. Write to color_out[idx]
//
// HELPER FUNCTIONS:
//   fn disk_temperature(r: f32, a: f32) -> f32
//     // Novikov-Thorne temperature profile:
//     // T(r) = T_peak * (r_isco/r)^(3/4) * nt_correction(r, a)
//     // Returns 0.0 for r < r_isco
//
//   fn gravitational_redshift(r: f32, theta: f32, a: f32) -> f32
//     // 1+z = 1/sqrt(-g_tt - 2*g_tphi*Omega - g_phiphi*Omega^2)
//     // Omega = keplerian_freq(r, a)
//
//   fn orbital_beta(r: f32, a: f32) -> f32
//     // v_K = r * Omega_K / (1 + a*sqrt(M/r^3)),  clamp to [0, 0.999]
//
//   fn doppler_factor(beta: f32, cos_angle: f32) -> f32
//     // D = 1 / (gamma * (1 - beta * cos_angle))
//     // gamma = 1 / sqrt(1 - beta^2)
//
//   fn doppler_d4(beta: f32, cos_angle: f32) -> f32
//     // D^4 beaming factor
//
//   fn sample_blackbody_lut(temp_k: f32, lut: texture_2d<f32>, s: sampler) -> vec4<f32>
//     // u = (log10(temp_k) - 3.0) / 6.0   (maps 1000K–1e9K to [0,1])
//     // return textureSampleLevel(lut, s, vec2(u, 0.5), 0.0)
//
//   fn apply_mhd_turbulence(temp: f32, r: f32, phi: f32, seed: u32) -> f32
//     // temp * (1.0 + turbulence_amplitude(r, phi, seed))
//     // turbulence_amplitude uses pcg_hash — same as mhd.rs CPU version
//
// USES (imports from):
//   No WGSL imports.
//   CPU counterpart: crates/gargantua-physics/src/effects/{doppler,redshift}.rs
//   Blackbody LUT: assets/baked/blackbody_lut.exr
//   Doppler LUT:   assets/baked/doppler_lut.exr (used via sample, not direct)
//   Curl noise:    assets/baked/curl_noise_128.exr (MHD turbulence)
//
// USED BY:
//   crates/gargantua-render/src/pipelines/ray_march.rs
//     → dispatched after geodesic_rk4.wgsl each frame
//
// NOTE FOR AI:
//   Reads RayResult from geodesic_rk4.wgsl output — order matters.
//   disk_temperature() must use the SAME formula as novikov_thorne.rs.
//   gravitational_redshift() uses Kerr g_tt, g_tφ — NOT Schwarzschild approx.
//   apply_mhd_turbulence(): pcg_hash seed = (frame_idx * 1234567u + pixel_idx).
//   Photon ring (secondary crossings): handled in a separate pipeline pass,
//   not in this shader — do NOT add secondary crossing logic here.
// ============================================================
