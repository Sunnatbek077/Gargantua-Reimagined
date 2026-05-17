// ============================================================
// FILE: shaders/compute/geodesic_rk4.wgsl
// LINES: ~380
// CATEGORY: Shader — GPU geodesic RK4 integrator (real-time)
// STAGE: Compute  (@compute @workgroup_size(8, 8, 1))
// ============================================================
//
// PURPOSE:
//   Real-time GPU compute shader that traces null geodesics (photon
//   paths) through the Kerr spacetime for every screen pixel.
//   Each invocation traces one ray from the camera through the pixel,
//   using fixed-step RK4. Outputs: ray endpoint direction (for
//   starfield sampling) and disk intersection data (for accretion color).
//   This is the core shader of the entire render pipeline.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var<uniform> kerr:    KerrParams;
//   @group(0) @binding(1) var<uniform> camera:  CameraParams;
//   @group(0) @binding(2) var<uniform> render:  RenderParams;
//   @group(0) @binding(3) var<storage, read_write> ray_output: array<RayResult>;
//
//   struct KerrParams {
//       mass:   f32, spin:   f32, charge: f32, r_s:  f32,
//       r_plus: f32, r_isco: f32, r_ph:   f32, _pad: f32,
//   }
//   // Must byte-match KerrGpuParams in crates/gargantua-physics/src/metric/kerr.rs
//
//   struct CameraParams {
//       pos:      vec4<f32>,   // (r, theta, phi, unused) Boyer-Lindquist
//       forward:  vec4<f32>,   // camera forward direction (Cartesian)
//       up:       vec4<f32>,   // camera up direction
//       right:    vec4<f32>,   // camera right direction
//       fov_tan:  f32,         // tan(fov/2)
//       aspect:   f32,         // width / height
//       beta:     f32,         // camera velocity β = v/c (for aberration)
//       _pad:     f32,
//   }
//
//   struct RenderParams {
//       width:       u32,
//       height:      u32,
//       rk4_steps:   u32,    // max integration steps (default 256)
//       step_size:   f32,    // RK4 step size h (default 0.1)
//       r_disk_outer:f32,    // outer disk radius in M
//       frame_idx:   u32,    // for blue noise offset
//       _pad:        vec2<u32>,
//   }
//
//   struct RayResult {
//       exit_dir:     vec4<f32>,  // direction when ray escaped (for starfield)
//       disk_r:       f32,        // r at disk crossing (0 if no crossing)
//       disk_phi:     f32,        // φ at disk crossing
//       disk_cos_angle: f32,      // cosine of ray angle to disk (for Doppler)
//       hit_horizon:  u32,        // 1 if ray fell into BH, 0 otherwise
//   }
//
// ENTRY POINT:
//   @compute @workgroup_size(8, 8, 1)
//   fn main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // 1. Compute ray direction from pixel (gid.x, gid.y):
//     //    uv = (gid.xy / vec2(width, height)) * 2.0 - 1.0
//     //    ray_dir = normalize(forward + uv.x*right*fov_tan*aspect + uv.y*up*fov_tan)
//     //
//     // 2. Apply relativistic aberration if beta > 0.001:
//     //    ray_dir = aberrate(ray_dir, camera.beta)
//     //
//     // 3. Convert ray to Boyer-Lindquist initial state:
//     //    state = bl_initial_state(camera.pos, ray_dir, kerr.spin)
//     //    Sets initial (t,r,θ,φ,ṫ,ṙ,θ̇,φ̇) from camera position + direction
//     //
//     // 4. RK4 integration loop (rk4_steps iterations):
//     //    for step in 0..rk4_steps:
//     //        state = rk4_step(state, kerr.spin, render.step_size)
//     //        if r < kerr.r_plus * 1.05: hit_horizon=1; break
//     //        if r > 1000.0: break  (escaped)
//     //        if disk_crossing(prev_state, state, kerr.r_isco, r_disk_outer):
//     //            record disk_r, disk_phi, disk_cos_angle; break
//     //
//     // 5. Write RayResult to ray_output[gid.y * width + gid.x]
//
// HELPER FUNCTIONS:
//   fn rk4_step(state: GeodesicState, a: f32, h: f32) -> GeodesicState
//     // k1 = h * geodesic_rhs(state, a)
//     // k2 = h * geodesic_rhs(state + k1/2, a)
//     // k3 = h * geodesic_rhs(state + k2/2, a)
//     // k4 = h * geodesic_rhs(state + k3, a)
//     // return state + (k1 + 2k2 + 2k3 + k4) / 6
//
//   fn geodesic_rhs(state: GeodesicState, a: f32) -> GeodesicState
//     // Computes d²x^λ/dλ² = -Γ^λ_μν ẋ^μ ẋ^ν
//     // Calls kerr_christoffel(state.r, state.theta, a)
//
//   fn kerr_christoffel(r: f32, theta: f32, a: f32) -> array<f32, 64>
//     // Analytic Kerr Γ^λ_μν (same formula as metric/kerr.rs christoffel)
//     // 64 = 4³ components, indexed as [lambda*16 + mu*4 + nu]
//
//   fn disk_crossing(prev: GeodesicState, curr: GeodesicState,
//       r_isco: f32, r_outer: f32) -> bool
//     // true if sign(prev.theta - PI/2) != sign(curr.theta - PI/2)
//     // AND r_isco < r_at_crossing < r_outer
//
//   fn aberrate(dir: vec3<f32>, beta: f32) -> vec3<f32>
//     // Special relativistic stellar aberration
//     // cos_obs = (cos_src - beta) / (1 - beta * cos_src)
//
//   fn bl_initial_state(cam_pos: vec4<f32>, ray_dir: vec3<f32>, a: f32) -> GeodesicState
//     // Convert camera position (r,θ,φ) and Cartesian ray direction
//     // to Boyer-Lindquist initial velocities (ṫ, ṙ, θ̇, φ̇)
//
// USES (imports from):
//   No WGSL imports. Self-contained.
//   CPU counterpart: crates/gargantua-physics/src/geodesic/rk4.rs
//   Bake counterpart: crates/gargantua-bake/src/geodesic/bake.wgsl
//   ALL THREE must implement IDENTICAL geodesic_rhs() and kerr_christoffel().
//
// USED BY:
//   crates/gargantua-render/src/pipelines/geodesic_gpu.rs
//     → dispatched once per frame: (width/8, height/8, 1)
//     → RayResult buffer fed into ray_march.wgsl and accretion_disk.wgsl
//
// NOTE FOR AI:
//   KerrParams layout MUST match KerrGpuParams in kerr.rs byte-for-byte.
//   GeodesicState in WGSL: struct with 8 f32 fields (not vec8 — no such type).
//   step_size: default 0.1, near horizon (r < 2*r_plus): use 0.01.
//   workgroup_size(8,8,1): 64 threads per workgroup, good for warp alignment.
//   Blue noise offset: jitter ray direction by blue_noise[pixel] * 0.5px
//   to remove aliasing (sample from blue_noise_256.exr using frame_idx).
//   This is the most performance-critical shader — profile before optimizing.
// ============================================================
