// =============================================================================
// FILE: crates/gargantua-video/src/denoise/mod.rs
// CRATE: gargantua-video
// LINES: ~100
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Denoiser dispatcher: selects the best available denoising backend for the
//   current platform and hardware at runtime. Exposes a single `Denoiser` enum
//   so the rest of the code does not need to know which backend is active.
//
// WHAT THIS FILE CONTAINS:
//   - `pub mod atrous;`
//   - `pub mod coreml_ane;`
//   - `pub mod oidn_cpu;`
//   - `pub mod oidn_cuda;`
//   - `pub enum Denoiser`:
//       Ane(CoreMlAneDenoiser)         — Mac: Neural Engine (best quality+speed)
//       OidnCuda(OidnCudaDenoiser)     — Windows NVIDIA: CUDA OIDN
//       OidnCpu(OidnCpuDenoiser)       — Windows non-NVIDIA: CPU OIDN
//       Atrous(AtrousDenoiser)         — WASM / universal fallback
//   - `impl Denoiser`:
//       `pub fn best_available(ctx: &GpuContext) -> Self`
//             Platform selection logic:
//               #[cfg(target_os="macos")]  && CoreMlAneDenoiser::is_available()
//                   → Denoiser::Ane
//               #[cfg(target_os="windows")] && OidnCudaDenoiser::is_available()
//                   → Denoiser::OidnCuda
//               #[cfg(target_os="windows")]
//                   → Denoiser::OidnCpu
//               fallback (WASM or no match)
//                   → Denoiser::Atrous
//       `pub fn denoise(&self, encoder: &mut CommandEncoder,
//                        noisy: TextureHandle, output: TextureHandle,
//                        albedo: Option<TextureHandle>)`
//             Dispatches to the active backend's denoise() method.
//
// OUTBOUND DEPENDENCIES:
//   - denoise/atrous.rs      → AtrousDenoiser
//   - denoise/coreml_ane.rs  → CoreMlAneDenoiser
//   - denoise/oidn_cpu.rs    → OidnCpuDenoiser
//   - denoise/oidn_cuda.rs   → OidnCudaDenoiser
//   - gpu/context.rs         → GpuContext
//
// INBOUND (who uses Denoiser):
//   - crates/gargantua-video/src/offline/renderer.rs → calls Denoiser::best_available() at startup,
//                                  calls denoiser.denoise() per frame when enabled
//
// NOTES:
//   - best_available() is called once at startup; the result is stored in
//     OfflineRenderer and reused across all frames.
//   - The user can disable denoising entirely from the export tab; in that case
//     best_available() is never called.
// =============================================================================
