// ============================================================
// FILE: crates/gargantua-bake/src/noise/curl_noise.rs
// LINES: ~220
// CATEGORY: Bake — 3D curl noise texture baker
// PLATFORM: cross-platform (Mac + Windows, requires wgpu)
// ============================================================
//
// PURPOSE:
//   Bakes a 3D curl noise texture (divergence-free vector field)
//   using the GPU compute shader (bake.wgsl entry "curl_noise_main").
//   Used in the render shader for disk turbulence, gas swirl effects,
//   and volumetric accretion disk animation.
//   Output: assets/baked/curl_noise_128.exr  (128³, RGB32F, 3 channels)
//
// CONTENTS (~220 lines):
//   pub fn bake(
//       params: &BakeParams,
//       device: &wgpu::Device,
//       queue:  &wgpu::Queue,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. Upload NoiseUniforms (noise_type=1, size=curl_noise_size, octaves=4)
//     // 2. Dispatch curl_noise_main with workgroup_size(8,8,8)
//     //    Dispatch: (size/8, size/8, size/8)
//     // 3. Readback 3D output buffer (size³ × 3 floats)
//     // 4. Verify divergence ≈ 0 (curl field sanity check)
//     // 5. Write 3D EXR: each z-slice as one scanline layer
//
//   // Verify that the output vector field is approximately divergence-free
//   // div(F) = ∂Fx/∂x + ∂Fy/∂y + ∂Fz/∂z ≈ 0
//   pub fn verify_divergence_free(
//       data: &[f32], size: usize, tolerance: f32,
//   ) -> bool
//     // Finite differences on interior voxels
//     // Returns true if max |div(F)| < tolerance (default 0.01)
//
//   // Write 3D EXR (each z-slice as a separate scanline layer)
//   fn write_3d_exr(
//       data: &[f32], size: usize, path: &std::path::Path
//   ) -> BakeResult<()>
//     // Uses exr crate's Layer API
//     // Layer name pattern: "z_{index}" for each z-slice
//
// USES (imports from):
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   wgpu
//   exr
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()  → bake step 5
//
// NOTE FOR AI:
//   Default size: 128×128×128 = 2M voxels × 3 floats × 4 bytes = 24MB GPU buffer.
//   On-disk EXR: ~24MB (no compression) or ~6MB (ZIP compressed EXR).
//   The render shader reads this as a 3D texture (wgpu Texture3d).
//   divergence-free check: tolerance=0.01 (1% max divergence allowed).
//   3D EXR format: multi-layer, each z-slice is one layer.
//   wgpu Texture3d loading in render crate: see pipelines/accretion.rs.
// ============================================================