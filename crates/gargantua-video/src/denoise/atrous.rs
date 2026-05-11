// =============================================================================
// FILE: crates/gargantua-video/src/denoise/atrous.rs
// CRATE: gargantua-video
// LINES: ~240
// PLATFORM: Mac + Windows + WASM (universal fallback)
// =============================================================================
//
// PURPOSE:
//   À-trous ("with holes") wavelet denoiser implemented as a GPU compute shader
//   dispatch. Serves as the universal denoiser fallback when hardware-specific
//   denoisers (ANE on Mac, CUDA OIDN on Windows NVIDIA) are unavailable.
//   Also used as the WASM denoiser since ANE and CUDA are not available in browsers.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct AtrousDenoiser`:
//       pipeline:    wgpu::ComputePipeline   — compiled from atrous.wgsl (inline WGSL)
//       bind_group:  wgpu::BindGroup
//       sigma_c:     f32     — colour sigma (spatial extent of denoising)
//       sigma_n:     f32     — normal sigma (feature preservation)
//       passes:      u32     — number of À-trous iterations (default: 5)
//   - `impl AtrousDenoiser`:
//       `pub fn new(ctx: &GpuContext, width: u32, height: u32) -> Self`
//             Compiles the inline WGSL À-trous compute shader.
//             Creates bind groups for input (noisy), output (denoised),
//             and optional albedo/normal buffers for feature-guided filtering.
//       `pub fn denoise(&self, encoder: &mut wgpu::CommandEncoder,
//                        noisy: TextureHandle, output: TextureHandle,
//                        albedo: Option<TextureHandle>)`
//             Dispatches 5 À-trous passes with increasing hole sizes:
//               Pass 0: kernel stride 1  (finest detail)
//               Pass 1: kernel stride 2
//               Pass 2: kernel stride 4
//               Pass 3: kernel stride 8
//               Pass 4: kernel stride 16 (coarsest)
//             Each pass reads the previous output as input (ping-pong).
//       `pub fn set_strength(&mut self, sigma_c: f32, sigma_n: f32)`
//             Adjusts denoising strength via push constants.
//
// OUTBOUND DEPENDENCIES:
//   - wgpu (external)          → ComputePipeline, BindGroup, CommandEncoder
//   - frame/resource.rs        → TextureHandle
//   - gpu/context.rs           → GpuContext
//
// INBOUND (who uses AtrousDenoiser):
//   - video/denoise/mod.rs → selected when ANE and CUDA OIDN are unavailable,
//                             or when platform is WASM
//
// NOTES:
//   - À-trous is a 5-level wavelet transform; each pass runs in O(n) time
//     (not O(n²) like a full Gaussian) making it GPU-efficient.
//   - Quality is lower than OIDN CUDA but higher than a simple Gaussian blur.
//   - WGSL source is embedded as a string literal (include_str!) rather than
//     loaded from disk, so it works in WASM without a file system.
// =============================================================================
