// ============================================================
// FILE: crates/gargantua-physics/src/accretion/spectrum.rs
// LINES: ~360
// CATEGORY: Physics — Blackbody spectrum → physically correct disk color
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Pipeline: T(r) → Planck spectrum → CIE XYZ integration → linear RGB.
//   Builds the baked LUT (blackbody_lut.exr) used by the GPU shader at
//   render time. All computations in linear light — NO gamma here.
//   Supports both sRGB (standard) and Display P3 (Mac EDR/HDR) outputs.
//
// CONTENTS (~360 lines):
//   // Planck spectral radiance [W/m²/sr/m]:
//   // B(λ,T) = (2hc²/λ⁵) / (exp(hc/λkT) − 1)
//   pub fn planck_radiance(lambda_nm: f64, temp_kelvin: f64) -> f64
//     // Normalized at 560 nm to keep values near [0, 1]
//
//   // Integrate Planck × CIE 1931 CMF → XYZ tristimulus
//   // Range: 360–780 nm, step: 5 nm → 85 samples
//   pub fn blackbody_to_xyz(temp_kelvin: f64, cmf: &CieCmf) -> [f64; 3]
//
//   // CIE XYZ → linear sRGB (BT.709 primaries, D65 white point)
//   pub fn xyz_to_srgb(xyz: [f64; 3]) -> [f32; 3]
//     // Bradford-adapted D65 matrix:
//     // [ 3.2406, -1.5372, -0.4986]
//     // [-0.9689,  1.8758,  0.0415]
//     // [ 0.0557, -0.2040,  1.0570]
//
//   // CIE XYZ → linear Display P3 (wider gamut, Mac HDR path)
//   pub fn xyz_to_display_p3(xyz: [f64; 3]) -> [f32; 3]
//     // DCI-P3 D65 primaries matrix (different from BT.709)
//
//   // Build 1024-point temperature → RGB LUT:
//   // T range: 1000K – 1e8K, log-spaced
//   // Output: Vec of [f32; 3] (linear RGB, no gamma)
//   // Saved to: assets/baked/blackbody_lut.exr
//   pub fn build_blackbody_lut(cmf: &CieCmf, n_points: usize) -> Vec<[f32; 3]>
//
//   pub struct CieCmf {
//       pub wavelengths: Vec<f64>,  // 360..780 nm (85 entries, 5 nm step)
//       pub x_bar: Vec<f64>,        // x̄(λ) — red CMF
//       pub y_bar: Vec<f64>,        // ȳ(λ) — luminosity / green CMF
//       pub z_bar: Vec<f64>,        // z̄(λ) — blue CMF
//   }
//
//   impl CieCmf {
//       pub fn from_raw(data: &[(f64, f64, f64, f64)]) -> Self
//         // data: [(wavelength_nm, x_bar, y_bar, z_bar), ...]
//         // Source: assets/raw/cie_1931_cmf.csv (Stiles & Burch 1955)
//   }
//
// USES (imports from):
//   crates/gargantua-bake/src/spectrum/cie_cmf.rs → CIE 1931 raw tabulated data
//
// USED BY:
//   crates/gargantua-bake/src/spectrum/blackbody.rs
//     → build_blackbody_lut() → EXR write → assets/baked/blackbody_lut.exr
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → CPU preview: blackbody_to_xyz() + xyz_to_srgb() for UI swatches
//   crates/gargantua-ui/src/menu/tabs/accretion_tab.rs
//     → live temperature color swatch (calls xyz_to_srgb directly)
//
// NOTE FOR AI:
//   ALL color math is in LINEAR light space — NO sRGB gamma here.
//   sRGB gamma (^(1/2.2) or piecewise) is applied ONLY at end of
//   shaders/postfx/tonemap.wgsl — never inside this file.
//   planck_radiance() normalization: divide by B(560nm, T) to keep
//   XYZ values in [0, ~3] range (avoids float overflow at high T).
//   LUT temperature axis: t_i = 10^(log10(1000) + i*(6/n))
//   (1000K to 10^9K over n_points=1024 log-spaced entries).
// ============================================================