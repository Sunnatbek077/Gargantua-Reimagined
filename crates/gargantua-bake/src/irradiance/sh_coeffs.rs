// ============================================================
// FILE: crates/gargantua-bake/src/irradiance/sh_coeffs.rs
// LINES: ~260
// CATEGORY: Bake — Spherical Harmonics coefficient computation
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Projects a starmap (HDR equirectangular image) onto Spherical
//   Harmonics basis functions up to order L (L=3..9 configurable).
//   The resulting SH coefficients are used in the render shader for
//   fast ambient sky lighting without sampling the full starmap.
//
// CONTENTS (~260 lines):
//   // SH coefficient array for RGB lighting
//   pub struct ShCoefficients {
//       pub order:  u32,               // max L order
//       pub coeffs: Vec<[f32; 3]>,     // (L+1)² RGB coefficients
//   }
//
//   impl ShCoefficients {
//       pub fn zeros(order: u32) -> Self
//         // (L+1)² coefficients, all [0.0, 0.0, 0.0]
//
//       // Project equirectangular HDR image onto SH basis
//       // img_data: linear RGB f32 pixels, width × height
//       pub fn project(
//           img_data: &[[f32; 3]],
//           width: usize, height: usize,
//           order: u32,
//       ) -> Self
//         // Monte Carlo integration over sphere:
//         // For each pixel (θ, φ):
//         //   weight = sin(θ)  (Jacobian of spherical → equirect mapping)
//         //   dir = (sin(θ)cos(φ), cos(θ), sin(θ)sin(φ))
//         //   For each SH basis Y_l^m(dir):
//         //     coeffs[l*(l+1)+m] += color * Y_l^m * weight * dΩ
//
//       // Reconstruct lighting at direction dir from SH coefficients
//       // Used for validation only (render shader does this in WGSL)
//       pub fn evaluate(&self, dir: [f32; 3]) -> [f32; 3]
//         // sum over l,m: coeffs[l*(l+1)+m] * Y_l^m(dir)
//
//       // Serialize to flat Vec<f32> for GPU buffer upload
//       // Layout: [R0,G0,B0, R1,G1,B1, ... , Rn,Gn,Bn]
//       pub fn to_gpu_buffer(&self) -> Vec<f32>
//
//       // Save to binary .bin file
//       pub fn save(&self, path: &std::path::Path) -> BakeResult<()>
//
//       // Load from binary .bin file
//       pub fn load(path: &std::path::Path) -> BakeResult<Self>
//   }
//
//   // Real spherical harmonics basis Y_l^m(θ, φ)
//   // Returns value for given l, m at direction (theta, phi)
//   pub fn sh_basis(l: u32, m: i32, theta: f32, phi: f32) -> f32
//     // Uses associated Legendre polynomials P_l^|m|
//     // Real SH convention (cos/sin series for m≠0)
//     // l=0: Y_0^0 = 1/sqrt(4π)
//     // l=1: Y_1^{-1} = sqrt(3/4π) sin(θ) sin(φ)
//     //       Y_1^0  = sqrt(3/4π) cos(θ)
//     //       Y_1^1  = sqrt(3/4π) sin(θ) cos(φ)
//     // Higher orders computed recursively
//
// USES (imports from):
//   crate::errors     → BakeResult, BakeError::LutIo
//   std::{path, fs, io}
//
// USED BY:
//   crate::irradiance::starmap
//     → calls ShCoefficients::project() on loaded HDR starmap
//   crates/gargantua-render/src/pipelines/starfield.rs
//     → loads ShCoefficients from .bin file for ambient lighting
//
// NOTE FOR AI:
//   SH order 3 → (3+1)² = 16 coefficients (good quality, fast).
//   SH order 9 → (9+1)² = 100 coefficients (high quality, larger file).
//   Default bake order: 5 → 36 coefficients (good quality/size tradeoff).
//   project() should normalize by total weight sum (not just pixel count)
//   to account for non-uniform solid angle of equirectangular mapping.
//   .bin file format: [u32 order, u32 n_coeffs, f32×3×n_coeffs data]
// ============================================================