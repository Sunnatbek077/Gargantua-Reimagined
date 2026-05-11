// =============================================================================
// FILE: crates/gargantua-video/src/denoise/oidn_cuda.rs
// CRATE: gargantua-video
// LINES: ~220
// PLATFORM: Windows only (NVIDIA GPU with CUDA)
// =============================================================================
//
// PURPOSE:
//   GPU-accelerated NVIDIA CUDA OIDN denoiser. Uses albedo and normal buffers
//   for feature-guided denoising that preserves edges and fine details.
//   Runs directly on the GPU, avoiding GPU→CPU readback for the noisy image.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct OidnCudaDenoiser`:
//       device:   oidn::Device    — OIDN CUDA device (wraps a CUDA context)
//       filter:   oidn::Filter    — "RT" filter on CUDA device
//   - `impl OidnCudaDenoiser`:
//       `pub fn new(d3d12_device: &ID3D12Device) -> Result<Self, VideoError>`
//             Creates oidn::Device with DeviceType::CUDA, using a CUDA context
//             derived from the D3D12 device via CUDA/D3D12 interop.
//             Sets filter to "RT" with albedo + normal inputs enabled.
//       `pub fn denoise(&self, noisy: &wgpu::Texture,
//                        albedo: &wgpu::Texture,
//                        normal: &wgpu::Texture,
//                        output: &wgpu::Texture)`
//             Uses D3D12/CUDA interop to pass wgpu textures directly as OIDN
//             input/output buffers without CPU readback.
//             Calls filter.execute() — runs on GPU, returns quickly.
//       `pub fn is_available() -> bool`
//             Tries oidn::Device::new(DeviceType::CUDA); returns false if
//             CUDA driver is absent or GPU is not NVIDIA.
//
// OUTBOUND DEPENDENCIES:
//   - oidn (external)    → OIDN 2.x with CUDA backend
//   - windows-rs (ext)   → ID3D12Device for CUDA interop
//   - wgpu (external)    → Texture
//   - errors.rs          → VideoError
//
// INBOUND (who uses OidnCudaDenoiser):
//   - denoise/mod.rs → selected on Windows NVIDIA after availability check
//
// NOTES:
//   - CUDA/D3D12 interop requires CUDA 11.0+ and Windows 10 2004+.
//   - GPU denoising at 4K: ~50–150 ms on RTX 4070+; acceptable for offline renders.
//   - If CUDA interop fails (older driver), automatically falls back to
//     OidnCpuDenoiser via an error return from OidnCudaDenoiser::new().
// =============================================================================
