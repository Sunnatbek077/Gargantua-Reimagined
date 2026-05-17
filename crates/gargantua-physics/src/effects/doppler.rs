// ============================================================
// FILE: crates/gargantua-physics/src/effects/doppler.rs
// LINES: ~280
// CATEGORY: Physics — Relativistic Doppler shift and beaming
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Relativistic Doppler shift + intensity beaming for orbiting disk
//   material. Approaching side (blueshifted) is brighter; receding
//   side (redshifted) is dimmer. This asymmetry is the most visually
//   distinctive feature of the Interstellar Gargantua render.
//   Also builds the GPU Doppler LUT baked to doppler_lut.exr.
//
// CONTENTS (~280 lines):
//   // Relativistic Doppler factor:
//   // D = 1 / (γ (1 − β cosθ_obs))
//   // γ = 1/sqrt(1−β²),  β = v/c
//   pub fn doppler_factor(beta: f64, cos_angle: f64) -> f64
//
//   // Keplerian orbital speed at r in Kerr metric:
//   // v_K = r Ω_K / (1 + a*(M/r³)^(1/2)),  clamped to [0, 0.999c]
//   pub fn orbital_beta(r: f64, mass: f64, spin: f64) -> f64
//
//   // Doppler-shifted wavelength: λ_obs = λ_emit / D
//   pub fn shift_wavelength(lambda_nm: f64, beta: f64, cos_angle: f64) -> f64
//
//   // Relativistic beaming: I_obs = I_emit * D^4
//   // (D^3 for flux, D^4 for specific intensity)
//   pub fn beam_intensity(i_emit: f64, beta: f64, cos_angle: f64) -> f64
//
//   // Full color transform: shifts blackbody T by Doppler factor
//   // T_obs = T_emit * D → hotter/cooler → color shifts blue/red
//   pub fn doppler_color_shift(
//       rgb: [f32; 3], temp_k: f64, beta: f64, cos_angle: f64,
//   ) -> [f32; 3]
//
//   // Build 2D Doppler LUT: β × λ_emit → wavelength shift factor
//   //   Axis 0: β  ∈ [0.0, 0.99], n_beta steps (default 256)
//   //   Axis 1: λ  ∈ [360, 780] nm, n_lambda steps (default 256)
//   //   Output: f32 shift factor per cell
//   //   Saved to: assets/baked/doppler_lut.exr
//   pub fn build_doppler_lut(n_beta: usize, n_lambda: usize) -> Vec<f32>
//
// USES (imports from):
//   crate::units           → c_si (for β normalization)
//   crate::accretion::isco → keplerian_freq() for orbital_beta()
//
// USED BY:
//   crates/gargantua-bake/src/spectrum/doppler_lut.rs
//     → build_doppler_lut() → saves doppler_lut.exr
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → per-fragment orbital β uploaded to accretion_disk.wgsl
//   shaders/render/accretion_disk.wgsl
//     → samples doppler_lut.exr at runtime for per-fragment color shift
//   crates/gargantua-physics/tests/doppler.rs
//     → D(β=0.5, cosθ=+1) ≈ 1.732,  D(β=0.5, cosθ=−1) ≈ 0.577
//
// NOTE FOR AI:
//   cosθ = +1: approaching (blueshift, D > 1, brighter)
//   cosθ = −1: receding   (redshift,  D < 1, dimmer)
//   D^4 beaming: at β=0.99, approaching side is ~10^4× brighter than receding.
//   For disk at r=6M  (Schwarzschild ISCO): β ≈ 0.408c
//   For disk at r=2.3M (M87* prograde ISCO): β ≈ 0.610c
//   cos_angle is computed from geodesic velocity direction (state[4..7])
//   at the disk intersection point — obtained from termination.rs.
//   build_doppler_lut() runs once offline during the bake step.
// ============================================================