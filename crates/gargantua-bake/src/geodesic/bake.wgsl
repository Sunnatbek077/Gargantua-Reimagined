// ============================================================
// FILE: crates/gargantua-bake/src/geodesic/bake.wgsl
// LINES: ~280
// CATEGORY: Bake — GPU compute shader for geodesic LUT baking
// PLATFORM: wgpu (Vulkan/Metal/DX12)
// ============================================================
//
// PURPOSE:
//   Compute shader that bakes geodesic deflection angles in parallel
//   on the GPU. Each invocation traces one (spin, impact_param) cell
//   using a fixed-step RK4 integrator identical to rk4.rs on the CPU.
//
// SHADER INTERFACE:
//   @group(0) @binding(0)
//   var<uniform> params: BakeUniforms;
//     // struct BakeUniforms {
//     //     spin_steps:   u32,
//     //     impact_steps: u32,
//     //     rk4_steps:    u32,
//     //     spin_min:     f32,  // -0.998
//     //     spin_max:     f32,  // +0.998
//     //     b_min:        f32,  // per-spin, passed per dispatch row
//     //     b_max:        f32,
//     //     mass:         f32,  // always 1.0 (G=c=1)
//     // }
//
//   @group(0) @binding(1)
//   var<storage, read_write> output: array<f32>;
//     // Layout: output[y * impact_steps * 4 + x * 4 + channel]
//     // channel 0 = deflection angle (radians)
//     // channel 1 = disk_hit (0.0 or 1.0)
//     // channel 2 = redshift factor (1+z)
//     // channel 3 = unused (padding)
//
// ENTRY POINT:
//   @compute @workgroup_size(16, 16, 1)
//   fn main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // gid.x = impact parameter index
//     // gid.y = spin index
//     // if gid.x >= impact_steps || gid.y >= spin_steps: return
//     //
//     // spin = lerp(spin_min, spin_max, f32(gid.y) / f32(spin_steps - 1))
//     // b    = lerp(b_min, b_max, f32(gid.x) / f32(impact_steps - 1))
//     //
//     // state = initial_state(b, spin)  // r=50M, theta=PI/2
//     // for step in 0..rk4_steps:
//     //     state = rk4_step(state, spin, h=0.1)
//     //     if terminated(state, spin): break
//     //
//     // deflection = compute_deflection(state)
//     // disk_hit   = f32(state.disk_was_hit)
//     // redshift   = compute_redshift(state, spin)
//     //
//     // write to output[...]
//
// HELPER FUNCTIONS (defined in this file):
//   fn kerr_christoffel(r: f32, theta: f32, a: f32) -> array<f32, 64>
//     // Analytic Kerr Christoffel symbols Γ^λ_μν (4×4×4 = 64 components)
//     // Same formula as metric/kerr.rs christoffel() but in f32
//
//   fn rk4_step(state: GeodesicState, a: f32, h: f32) -> GeodesicState
//     // struct GeodesicState { t,r,theta,phi, dt,dr,dtheta,dphi: f32 }
//     // 4th-order Runge-Kutta step (k1..k4)
//
//   fn geodesic_rhs(state: GeodesicState, a: f32) -> GeodesicState
//     // d²x^λ/dλ² = -Γ^λ_μν ẋ^μ ẋ^ν
//
//   fn terminated(state: GeodesicState, a: f32) -> bool
//     // true if r < event_horizon * 1.05  OR  r > 1000.0
//
//   fn compute_deflection(state: GeodesicState) -> f32
//     // total azimuthal deflection: abs(state.phi - initial_phi) - PI
//
//   fn event_horizon(a: f32) -> f32
//     // r_+ = 1.0 + sqrt(1.0 - a*a)  (M=1)
//
// USES (imports from):
//   No WGSL imports — self-contained shader.
//   Must stay mathematically identical to:
//     gargantua-physics/src/geodesic/rk4.rs  (CPU reference)
//     gargantua-render/src/shaders/compute/geodesic_rk4.wgsl  (render shader)
//
// USED BY:
//   crates/gargantua-bake/src/geodesic/let_baker.rs
//     → loaded via include_str!("bake.wgsl") and compiled with wgpu
//
// NOTE FOR AI:
//   Workgroup size (16,16,1): total threads = 256 per workgroup.
//   For 256 spin × 2048 impact: dispatch (128, 16, 1) workgroups.
//   f32 precision is acceptable for baking (LUT has inherent quantization).
//   CPU uses f64 (AdaptiveIntegrator) for the critical-b correction pass.
//   GeodesicState is a WGSL struct — NOT a vec8 (WGSL has no vec8).
//   Kerr Christoffel array: index = lambda*16 + mu*4 + nu.
// ============================================================
