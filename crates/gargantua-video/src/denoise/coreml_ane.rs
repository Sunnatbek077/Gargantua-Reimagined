// =============================================================================
// FILE: crates/gargantua-video/src/denoise/coreml_ane.rs
// CRATE: gargantua-video
// LINES: ~280
// PLATFORM: Mac only
// =============================================================================
//
// PURPOSE:
//   High-quality neural network denoiser using Apple's Neural Engine (ANE)
//   via Core ML. Runs the OIDN-equivalent network (KPN or UNet architecture)
//   5–10× faster than CPU OIDN because ANE is purpose-built for ML inference.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct CoreMlAneDenoiser`:
//       denoiser:     NeuralEngineDenoiser   — from platform/macos/compute/neural_engine.rs
//       model_loaded: bool
//   - `impl CoreMlAneDenoiser`:
//       `pub fn new() -> Result<Self, VideoError>`
//             Calls NeuralEngineDenoiser::new() with the bundled .mlmodelc path.
//             Returns VideoError::CodecUnsupported if ANE is unavailable
//             (Intel Mac or WASM build).
//       `pub fn denoise(&self, noisy: &wgpu::Texture,
//                        albedo: Option<&wgpu::Texture>,
//                        output: &wgpu::Texture) -> Result<(), VideoError>`
//             1. Calls NeuralEngineDenoiser::denoise(noisy, output).
//             2. If albedo is provided, runs a second albedo-modulated pass
//                to restore fine texture details (feature-guided denoising).
//             Returns Ok(()) or VideoError on failure.
//       `pub fn is_available() -> bool`
//             Wraps NeuralEngineDenoiser::is_ane_available().
//
// OUTBOUND DEPENDENCIES:
//   - platform/macos/compute/neural_engine.rs → NeuralEngineDenoiser
//   - wgpu (external)                          → Texture
//   - errors.rs                                → VideoError
//
// INBOUND (who uses CoreMlAneDenoiser):
//   - video/denoise/mod.rs → selected first on Mac (highest quality, lowest power)
//
// NOTES:
//   - The Core ML model file (gargantua_denoise.mlmodelc) is pre-compiled for
//     ANE and ships in the macOS app bundle under Resources/.
//   - On M1/M2 ANE throughput: ~11 TOPS → can denoise 4K frame in ~30 ms.
//   - On M4 Max ANE: ~38 TOPS → 4K denoise in ~9 ms (real-time capable).
// =============================================================================
