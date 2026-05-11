// ============================================================
// FILE: crates/gargantua-bake/src/spectrum/doppler_lut.rs
// LINES: ~160
// CATEGORY: Bake — Doppler shift LUT baker
// PLATFORM: cross-platform (Mac + Windows, CPU only)
// ============================================================
//
// PURPOSE:
//   Bakes the 2D Doppler LUT: β × λ_emit → wavelength shift factor.
//   Uses gargantua-physics doppler module.
//   Output: assets/baked/doppler_lut.exr  (n_beta × n_lambda, R32F)
//
// CONTENTS (~160 lines):
//   pub fn bake(
//       params: &BakeParams,
//       tx:     &std::sync::mpsc::Sender<BakeProgressEvent>,
//       cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
//   ) -> BakeResult<()>
//     // 1. Call gargantua_physics::effects::doppler::build_doppler_lut(
//     //        params.doppler_n_beta, params.doppler_n_lambda)
//     //    → Vec<f32> of length n_beta * n_lambda
//     //
//     // 2. Write 2D EXR: width=n_lambda, height=n_beta, R32F single channel
//     //    Path: assets/baked/doppler_lut.exr
//     //
//     // 3. Report progress (fast: ~200ms CPU)
//
//   fn write_doppler_exr(
//       data: &[f32], n_beta: usize, n_lambda: usize,
//       path: &std::path::Path,
//   ) -> BakeResult<()>
//
// USES (imports from):
//   gargantua_physics::effects::doppler::build_doppler_lut
//   crate::errors::{BakeResult, BakeError}
//   crate::scheduler::{BakeParams, BakeProgressEvent}
//   exr
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()  → bake step 3
//   tests/lut_baker.rs → validates Doppler LUT shift values
//
// NOTE FOR AI:
//   LUT axis 0 (Y): β ∈ [0.0, 0.99], n_beta steps (default 256).
//   LUT axis 1 (X): λ ∈ [360nm, 780nm], n_lambda steps (default 256).
//   Each cell stores: λ_observed / λ_emitted = 1/D(β, cosθ=+1) shift factor.
//   (cosθ=+1 means approaching — disk approaching side.)
//   The render shader uses UV coords: u = (lambda - 360) / 420, v = beta / 0.99.
//   File size: 256 × 256 × 4 bytes = 256KB (small).
// ============================================================