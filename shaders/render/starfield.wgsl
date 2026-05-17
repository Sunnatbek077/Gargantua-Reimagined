// ============================================================
// FILE: shaders/render/starfield.wgsl
// LINES: ~220
// CATEGORY: Shader — Background starfield rendering
// STAGE: Fragment  (fullscreen quad)
// ============================================================
//
// PURPOSE:
//   Renders the background starfield that appears after gravitational
//   lensing by the black hole. Samples the downsampled starmap texture
//   (starmap_512.exr) and reconstructs high-frequency star detail using
//   SH coefficients for ambient sky lighting. Applies relativistic
//   aberration if the camera is moving at high speed.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var starmap:        texture_2d<f32>;  // 512×256 HDR
//   @group(0) @binding(1) var tex_sampler:    sampler;
//   @group(0) @binding(2) var<uniform>        params: StarfieldParams;
//   @group(0) @binding(3) var<storage, read>  sh_coeffs: array<f32>; // SH coefficients
//   @group(0) @binding(4) var<storage, read>  ray_results: array<RayResult>; // exit dirs
//
//   struct StarfieldParams {
//       aberration_mat: mat3x3<f32>,  // 3×3 SR aberration matrix (from aberration.rs)
//       camera_beta:    f32,          // camera speed β = v/c
//       sh_order:       u32,          // SH order (3..9)
//       star_brightness:f32,          // overall starfield brightness scale
//       _pad:           f32,
//   }
//
// ENTRY POINT:
//   @fragment
//   fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32>
//     // 1. Load exit direction from ray_results (from geodesic_rk4.wgsl):
//     //    exit_dir = ray_results[pixel_idx].exit_dir.xyz
//     //    if hit_horizon || disk_r > 0: return vec4(0.0)  // not a sky pixel
//     //
//     // 2. Apply relativistic aberration if beta > 0.001:
//     //    exit_dir = params.aberration_mat * exit_dir
//     //
//     // 3. Convert exit direction to equirectangular UV:
//     //    phi   = atan2(exit_dir.z, exit_dir.x)
//     //    theta = acos(exit_dir.y)
//     //    uv    = vec2(phi / (2*PI) + 0.5, theta / PI)
//     //
//     // 4. Sample starmap texture:
//     //    star_color = textureSample(starmap, tex_sampler, uv) * star_brightness
//     //
//     // 5. Add SH ambient sky contribution:
//     //    sh_ambient = eval_sh(sh_coeffs, exit_dir, sh_order)
//     //    color = star_color + sh_ambient * 0.1  // subtle ambient fill
//     //
//     // 6. return vec4(color, 1.0)
//
// HELPER FUNCTIONS:
//   fn eval_sh(coeffs: ptr<storage, array<f32>, read>, dir: vec3<f32>, order: u32) -> vec3<f32>
//     // Evaluate SH lighting at direction dir
//     // sum over l,m: coeffs[base + l*(l+1)+m] * Y_l^m(dir)
//     // base offsets: R=0, G=n_coeffs, B=2*n_coeffs
//
//   fn sh_basis_y(l: u32, m: i32, theta: f32, phi: f32) -> f32
//     // Real spherical harmonics Y_l^m  (same formula as sh_coeffs.rs)
//
// USES (imports from):
//   No WGSL imports.
//   Starmap: assets/baked/starmap_512.exr
//   SH coeffs: assets/baked/starmap_sh.bin (loaded as storage buffer)
//   RayResult from: compute/geodesic_rk4.wgsl
//
// USED BY:
//   crates/gargantua-render/src/pipelines/starfield.rs
//
// NOTE FOR AI:
//   eval_sh() must implement SAME SH basis as sh_coeffs.rs::sh_basis().
//   aberration_mat is uploaded from aberration.rs::aberration_matrix().
//   star_brightness: scale factor for overall sky luminance (default 1.0).
//   Equirectangular UV: phi=[-π,π]→[0,1], theta=[0,π]→[0,1].
//   Wrap mode for starmap sampler: AddressMode::Repeat (seamless wrap).
//   SH ambient is subtle (×0.1): only fills very dark shadow regions.
// ============================================================
