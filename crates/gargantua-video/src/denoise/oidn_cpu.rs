// =============================================================================
// FILE: crates/gargantua-video/src/denoise/oidn_cpu.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Windows + Mac (CPU fallback, no GPU required)
// =============================================================================
//
// PURPOSE:
//   CPU-based denoiser using Intel Open Image Denoise (OIDN) 2.x.
//   Used on Windows machines without NVIDIA GPU, or as a fallback when
//   CUDA OIDN initialisation fails.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct OidnCpuDenoiser`:
//       device:  oidn::Device    — OIDN CPU device handle
//       filter:  oidn::Filter    — "RT" ray tracing filter
//   - `impl OidnCpuDenoiser`:
//       `pub fn new() -> Result<Self, VideoError>`
//             Calls oidn::Device::new(DeviceType::CPU).
//             Creates filter with oidn::FilterBuilder for "RT" type.
//             Sets hdrScale = 1.0, quality = OIDN_QUALITY_HIGH.
//       `pub fn denoise(&self, noisy: &[f32], albedo: Option<&[f32]>,
//                        normal: Option<&[f32]>, output: &mut [f32],
//                        width: u32, height: u32)`
//             Sets filter inputs (color, optionally albedo + normal buffers).
//             Calls filter.execute() — blocks until CPU denoising is complete.
//             Performance: ~2–8 seconds for 4K on a modern CPU (not real-time).
//       `pub fn is_available() -> bool`
//             Always true on supported platforms (OIDN is a pure CPU library).
//
// OUTBOUND DEPENDENCIES:
//   - oidn (external crate)  → Intel OIDN 2.x Rust bindings
//   - errors.rs              → VideoError
//
// INBOUND (who uses OidnCpuDenoiser):
//   - denoise/mod.rs → selected on Windows when CUDA OIDN is unavailable
//
// NOTES:
//   - OIDN 2.x ships its own neural network weights; no separate model file needed.
//   - CPU denoising is sequential; it blocks renderer.rs's frame loop.
//     For long offline renders this is acceptable (each frame waits for denoising).
//   - The noisy/albedo/normal buffers are CPU-side f32 slices; the renderer
//     maps the GPU readback buffer before calling this.
// =============================================================================
