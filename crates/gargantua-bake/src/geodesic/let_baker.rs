// ============================================================
// FILE: crates/gargantua-bake/src/geodesic/let_baker.rs
// LINES: ~360
// CATEGORY: Bake — Geodesic LUT baker (CPU + GPU hybrid)
// PLATFORM: cross-platform (Mac + Windows, requires wgpu)
// ============================================================
//
// PURPOSE:
//   Bakes the 2D geodesic Look-Up Table:
//     Axes: spin a ∈ [-0.998, +0.998]  (geo_spin_steps entries)
//           impact parameter b ∈ [b_min, b_max]  (geo_impact_steps entries)
//     Value: (deflection_angle, disk_hit_flag, redshift_factor) — 3× f32
//   Output: assets/baked/geodesic_lut.exr  (RGBA16F, width=impact_steps, height=spin_steps)
//
//   Hybrid approach:
//     - CPU path (AdaptiveIntegrator): used for high-accuracy reference cells
//     - GPU path (bake.wgsl compute shader): used for bulk parallel baking
//
// CONTENTS (~360 lines):
//   pub fn bake(
//       params: &BakeParams,
//       device: &wgpu::Device,
//       queue:  &wgpu::Queue,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. For each spin step s: a = lerp(-0.998, +0.998, s / spin_steps)
//     //    Build KerrNewman(mass=1.0, spin=a, charge=0)
//     //    Compute b_min = photon_sphere_b(a), b_max = 30.0 * M
//     //
//     // 2. Dispatch GPU compute shader (bake.wgsl) for bulk cells
//     //    Shader writes deflection angle per (s, b) cell
//     //
//     // 3. CPU post-pass: for cells near b_critical (photon sphere),
//     //    re-compute with AdaptiveIntegrator for higher accuracy
//     //
//     // 4. Composite CPU corrections into GPU output buffer
//     //
//     // 5. Write final buffer to EXR via exr crate
//     //    Channel layout: R=deflection_angle, G=disk_hit(0/1), B=redshift, A=unused
//
//   // Compute critical impact parameter for given spin
//   // Below b_crit: photon falls into BH
//   // Above b_crit: photon escapes
//   fn photon_sphere_b(spin: f64) -> f64
//     // Uses KerrNewman::photon_sphere() and circular orbit formula
//
//   // Setup wgpu compute pipeline for bake.wgsl
//   fn setup_gpu_pipeline(
//       device: &wgpu::Device,
//       params: &BakeParams,
//   ) -> (wgpu::ComputePipeline, wgpu::BindGroup, wgpu::Buffer)
//
//   // Read back GPU output buffer to CPU Vec<f32>
//   fn readback_buffer(
//       device: &wgpu::Device, queue: &wgpu::Queue,
//       buffer: &wgpu::Buffer, size: u64,
//   ) -> BakeResult<Vec<f32>>
//
//   // Write Vec<f32> (3 channels) to EXR file
//   fn write_exr(
//       data: &[f32], width: usize, height: usize,
//       path: &std::path::Path,
//   ) -> BakeResult<()>
//
// USES (imports from):
//   gargantua_physics::geodesic::adaptive::AdaptiveIntegrator
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::mod::MetricTensor
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   wgpu (external)    → Device, Queue, ComputePipeline, Buffer
//   exr  (external)    → write EXR files
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()
//     → bake step 1: geodesic::let_baker::bake(...)
//
// NOTE FOR AI:
//   File is named let_baker.rs (visible in screenshot) but logically
//   it is the "LUT baker" for geodesics — lut_baker.rs conceptually.
//   EXR output: width = geo_impact_steps, height = geo_spin_steps.
//   GPU workgroup size in bake.wgsl: (16, 16, 1).
//   Total dispatch: (impact_steps/16, spin_steps/16, 1).
//   CPU correction pass covers ±5 cells around b_critical for each spin.
//   EXR RGBA16F: 2 bytes per channel × 4 channels × W × H bytes total.
//   For default params (256 spin × 2048 impact): ~4 MB on disk.
// ============================================================