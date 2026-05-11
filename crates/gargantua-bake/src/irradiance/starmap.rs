// ============================================================
// FILE: crates/gargantua-bake/src/irradiance/starmap.rs
// LINES: ~220
// CATEGORY: Bake — Starmap HDR loading and SH projection
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Loads the source HDR starmap (equirectangular EXR), preprocesses
//   it (exposure normalization, gamma removal), projects it onto SH
//   coefficients via sh_coeffs.rs, and saves the result.
//   Input:  assets/raw/starmap_8k.exr  (8K equirectangular HDR)
//   Output: assets/baked/starmap_sh.bin  (SH coefficient binary)
//           assets/baked/starmap_512.exr (downsampled for runtime sampling)
//
// CONTENTS (~220 lines):
//   pub fn bake(
//       params: &BakeParams,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. Load assets/raw/starmap_8k.exr with exr crate
//     //    → Vec<[f32;3]> linear RGB (8192 × 4096 pixels)
//     //
//     // 2. Normalize exposure: scale so median star brightness = 1.0
//     //
//     // 3. Project onto SH basis (order = params.starmap_sh_order)
//     //    → ShCoefficients via sh_coeffs::ShCoefficients::project()
//     //
//     // 4. Save ShCoefficients to assets/baked/starmap_sh.bin
//     //
//     // 5. Downsample 8K → 512×256 (Lanczos or box filter)
//     //    Save to assets/baked/starmap_512.exr
//     //    (Runtime shader samples this for direct star hits)
//     //
//     // 6. Report progress via tx at each major step
//
//   // Downsample equirectangular image from (src_w, src_h) to (dst_w, dst_h)
//   fn downsample(
//       src: &[[f32;3]], src_w: usize, src_h: usize,
//       dst_w: usize, dst_h: usize,
//   ) -> Vec<[f32;3]>
//     // Box filter: average all source pixels that map to each dest pixel
//     // Preserves energy (does NOT clip or tone-map)
//
//   // Normalize image so median non-zero luminance = target
//   fn normalize_exposure(img: &mut [[f32;3]], target_median: f32)
//     // Computes median luminance of non-black pixels
//     // Scales all pixels by (target_median / median_luminance)
//
// USES (imports from):
//   crate::irradiance::sh_coeffs::ShCoefficients
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   exr  (external)  → load_first_rgba_layer_from_file (EXR reading)
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()  → bake step 6: starmap
//
// NOTE FOR AI:
//   Source file: assets/raw/starmap_8k.exr — must exist before baking.
//   If it does not exist: return BakeError::LutIo with helpful message.
//   The 8K EXR is ~200MB — loading it takes ~2–3 seconds (expected).
//   normalize_exposure target_median: 0.01 (stars are dim in linear space).
//   Downsampled 512×256 EXR is ~1MB and loaded at runtime for star sampling.
//   SH .bin is ~1KB (36 coefficients × 3 floats × 4 bytes = 432 bytes for L=5).
// ============================================================