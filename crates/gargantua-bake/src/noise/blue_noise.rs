// ============================================================
// FILE: crates/gargantua-bake/src/noise/blue_noise.rs
// LINES: ~200
// CATEGORY: Bake — Blue noise texture baker
// PLATFORM: cross-platform (Mac + Windows, requires wgpu)
// ============================================================
//
// PURPOSE:
//   Bakes a 2D blue noise texture using the void-and-cluster algorithm
//   on the GPU (bake.wgsl entry "blue_noise_main").
//   Blue noise has uniform spatial distribution with no low-frequency
//   clumping — used in the render shader for dithered sampling
//   (avoids banding artifacts in ray march and disk sampling).
//   Output: assets/baked/blue_noise_256.exr  (256×256, R32F single channel)
//
// CONTENTS (~200 lines):
//   pub fn bake(
//       params: &BakeParams,
//       device: &wgpu::Device,
//       queue:  &wgpu::Queue,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. Upload NoiseUniforms to GPU (noise_type=0, size=blue_noise_size)
//     // 2. Dispatch blue_noise_main compute shader
//     // 3. Readback output buffer
//     // 4. Write single-channel EXR to assets/baked/blue_noise_{size}.exr
//
//   // Validate blue noise quality: compute power spectrum, check high-freq dominance
//   pub fn validate_blue_noise(data: &[f32], size: usize) -> bool
//     // Computes 2D DFT (or DCT approximation)
//     // Returns true if low-freq energy < 10% of total energy
//     // Used in crates/gargantua-bake/tests/cache.rs to verify baked output quality
//
// USES (imports from):
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   wgpu
//   exr
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()  → bake step 4
//   crates/gargantua-bake/tests/cache.rs → validate_blue_noise() for quality check
//
// NOTE FOR AI:
//   Blue noise size default: 256×256 = 65536 pixels.
//   Output is a single R32F channel (not RGBA) — 1MB on disk as EXR.
//   The render shader samples this texture with a per-pixel offset based
//   on frame index to achieve temporal blue noise (less aliasing over time).
//   Void-and-cluster on GPU may require iterative passes — see bake.wgsl note.
//   If GPU void-and-cluster is too complex: use CPU implementation instead
//   (size 256×256 at ~0.5s CPU time is acceptable for a one-time bake).
// ============================================================