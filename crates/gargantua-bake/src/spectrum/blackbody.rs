// ============================================================
// FILE: crates/gargantua-bake/src/spectrum/blackbody.rs
// LINES: ~160
// CATEGORY: Bake — Blackbody temperature → RGB LUT baker
// PLATFORM: cross-platform (Mac + Windows, CPU only)
// ============================================================
//
// PURPOSE:
//   Bakes the 1D blackbody temperature → linear sRGB LUT.
//   Uses gargantua-physics spectrum module for physically correct
//   color computation (Planck → CIE XYZ → linear RGB).
//   Output: assets/baked/blackbody_lut.exr  (1×N, RGB32F)
//
// CONTENTS (~160 lines):
//   pub fn bake(
//       params: &BakeParams,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. Load CIE 1931 CMF data from cie_cmf.rs
//     // 2. Call gargantua_physics::accretion::spectrum::build_blackbody_lut(
//     //        &cmf, params.blackbody_lut_size)
//     //    → Vec<[f32;3]> linear sRGB values (1000K – 1e9K, log scale)
//     // 3. Write 1D EXR: width=blackbody_lut_size, height=1, RGB32F
//     //    Path: assets/baked/blackbody_lut.exr
//     // 4. Report progress (single step, fast: ~100ms CPU)
//
//   // Write 1D LUT to EXR (1 row, N columns, RGB32F)
//   fn write_lut_exr(
//       data: &[[f32;3]], path: &std::path::Path
//   ) -> BakeResult<()>
//
// USES (imports from):
//   gargantua_physics::accretion::spectrum::build_blackbody_lut
//   crate::spectrum::cie_cmf::CieCmfData
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   exr
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()  → bake step 2
//   tests/lut_baker.rs → validates LUT values at known temperatures
//
// NOTE FOR AI:
//   LUT width = params.blackbody_lut_size (default 1024).
//   Temperature axis: log10-spaced from 1000K to 1e9K.
//   T_i = 10^(3.0 + i * 6.0 / (n-1))  for i in 0..n
//   The render shader samples this LUT with: t_sample = (log10(T) - 3) / 6
//   to get normalized UV coordinate.
//   EXR format: scanlineimage, 1 scanline, RGB32F channels.
//   File size: 1024 × 3 × 4 = 12KB (tiny — fast to load at startup).
// ============================================================