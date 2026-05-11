// ============================================================
// FILE: crates/gargantua-bake/src/noise/bake.wgsl
// LINES: ~220
// CATEGORY: Bake — GPU compute shader for noise texture generation
// PLATFORM: wgpu (Vulkan/Metal/DX12)
// ============================================================
//
// PURPOSE:
//   Compute shader that generates blue noise and 3D curl noise textures
//   on the GPU. Two entry points: one for blue noise (2D),
//   one for curl noise (3D). Selected by pipeline specialization.
//
// SHADER INTERFACE:
//   @group(0) @binding(0)
//   var<uniform> noise_params: NoiseUniforms;
//     // struct NoiseUniforms {
//     //     size:       u32,    // texture width/height (blue) or depth (curl)
//     //     seed:       u32,    // random seed
//     //     noise_type: u32,    // 0 = blue noise, 1 = curl noise
//     //     octaves:    u32,    // curl noise octaves [1–8]
//     //     frequency:  f32,    // curl noise base frequency
//     //     amplitude:  f32,    // curl noise amplitude
//     // }
//
//   @group(0) @binding(1)
//   var<storage, read_write> output: array<f32>;
//     // Blue noise (2D): output[y*size + x] = value ∈ [0, 1]
//     // Curl noise (3D): output[(z*size*size + y*size + x) * 3 + ch]
//     //   ch 0,1,2 = x,y,z components of curl vector field
//
// ENTRY POINTS:
//   @compute @workgroup_size(8, 8, 1)
//   fn blue_noise_main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // Void-and-cluster blue noise generation
//     // Phase 1: place initial binary pattern with energy minimization
//     // Phase 2: rank remaining pixels by energy
//     // Result: spatially uniform noise (no clumping)
//
//   @compute @workgroup_size(8, 8, 8)
//   fn curl_noise_main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // 3D curl noise: divergence-free vector field
//     // F(x) = curl(A(x)) where A is Perlin noise potential
//     // curl(A) = (∂Az/∂y - ∂Ay/∂z, ∂Ax/∂z - ∂Az/∂x, ∂Ay/∂x - ∂Ax/∂y)
//
// HELPER FUNCTIONS:
//   fn pcg_hash(seed: u32) -> u32
//     // PCG32 permutation hash — fast, high quality
//
//   fn perlin3d(p: vec3<f32>, seed: u32) -> f32
//     // Classic Perlin noise with smooth quintic interpolation
//     // fade(t) = 6t⁵ - 15t⁴ + 10t³
//
//   fn fbm3d(p: vec3<f32>, octaves: u32, freq: f32, amp: f32) -> f32
//     // Fractional Brownian Motion: sum of octaves of Perlin noise
//     // lacunarity=2.0, gain=0.5
//
//   fn grad3(hash: u32, p: vec3<f32>) -> f32
//     // Perlin gradient function — maps hash to one of 12 gradient vectors
//
// USES (imports from):
//   No WGSL imports — self-contained.
//
// USED BY:
//   crates/gargantua-bake/src/noise/blue_noise.rs  → entry "blue_noise_main"
//   crates/gargantua-bake/src/noise/curl_noise.rs  → entry "curl_noise_main"
//
// NOTE FOR AI:
//   Two separate entry points in the same shader file.
//   Create two wgpu ComputePipelines from the same shader module,
//   one with entry_point="blue_noise_main", one with "curl_noise_main".
//   Blue noise size 256×256 (default): dispatch (32, 32, 1) @ (8,8,1).
//   Curl noise size 128×128×128 (default): dispatch (16, 16, 16) @ (8,8,8).
//   Void-and-cluster blue noise requires iterative GPU passes —
//   implement as multiple dispatches with atomic operations or
//   fall back to CPU for the iterative phase.
// ============================================================
