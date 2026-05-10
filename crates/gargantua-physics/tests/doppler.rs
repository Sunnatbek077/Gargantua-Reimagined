// ============================================================
// FILE: crates/gargantua-physics/tests/doppler.rs
// LINES: ~180
// CATEGORY: Integration test — Relativistic Doppler & beaming
// RUN: cargo test --package gargantua-physics --test doppler
// ============================================================
//
// PURPOSE:
//   Validates doppler.rs functions against known analytic values.
//   Tests: Doppler factor formula, orbital velocity, beaming law,
//   wavelength shift, and 2D LUT shape.
//
// TESTED FUNCTIONS (from crate::effects::doppler):
//   doppler_factor(beta, cos_angle) -> f64
//   orbital_beta(r, mass, spin)     -> f64
//   beam_intensity(i_emit, beta, cos_angle) -> f64
//   shift_wavelength(lambda_nm, beta, cos_angle) -> f64
//   build_doppler_lut(n_beta, n_lambda) -> Vec<f32>
//
// TEST CASES (~180 lines):
//
//   #[test]
//   fn test_doppler_factor_zero_velocity()
//     // beta=0.0, any cos_angle → D = 1.0 (no motion, no shift)
//     // assert_relative_eq!(doppler_factor(0.0, 1.0), 1.0, epsilon=1e-10)
//     // assert_relative_eq!(doppler_factor(0.0, -1.0), 1.0, epsilon=1e-10)
//
//   #[test]
//   fn test_doppler_factor_approaching()
//     // beta=0.5, cos_angle=+1.0 (head-on approach)
//     // D = 1/(γ(1-β)) = 1/(0.866 * 0.5) ≈ 1/(√(3)/2 * 1/2) = ... 
//     // exact: γ = 1/√(1-0.25) = 1/√0.75 ≈ 1.1547
//     // D = 1/(1.1547 * (1 - 0.5)) = 1/0.5774 ≈ 1.7321
//     // assert_relative_eq!(result, 1.7320508, epsilon=1e-5)
//
//   #[test]
//   fn test_doppler_factor_receding()
//     // beta=0.5, cos_angle=-1.0 (moving away)
//     // D = 1/(γ(1+β)) = 1/(1.1547 * 1.5) ≈ 0.5774
//     // assert_relative_eq!(result, 0.57735, epsilon=1e-5)
//     // Symmetry check: D_approach * D_recede ≈ 1.0
//
//   #[test]
//   fn test_doppler_factor_transverse()
//     // beta=0.5, cos_angle=0.0 (transverse motion, 90°)
//     // D = 1/γ = √(1-β²) = √0.75 ≈ 0.8660
//     // (pure transverse Doppler — time dilation only)
//     // assert_relative_eq!(result, 0.86603, epsilon=1e-5)
//
//   #[test]
//   fn test_beam_intensity_d4_law()
//     // Beaming: I_obs = I_emit * D^4
//     // beta=0.5, cos_angle=+1.0: D≈1.732, D^4≈9.0
//     // assert_relative_eq!(beam_intensity(1.0, 0.5, 1.0), 9.0, epsilon=0.01)
//     // At beta=0.0: beam_intensity = 1.0 (no beaming)
//
//   #[test]
//   fn test_orbital_beta_schwarzschild_isco()
//     // At r=6M (Schwarzschild ISCO), spin=0:
//     // v_K = sqrt(M / (r - 3M)) ... exact: β = 1/√12 ≈ 0.2887... wait
//     // Actually v_ISCO(Schw) = c / sqrt(6) * 1/sqrt(1 - 3M/r) simplified
//     // Reference: β_ISCO ≈ 0.5c for proper velocity, ~0.408c coordinate
//     // Use coordinate velocity: v_K = sqrt(M/r) / (1 - 2M/r) simplified
//     // assert!(result > 0.38 && result < 0.45)
//
//   #[test]
//   fn test_orbital_beta_high_spin_isco()
//     // At r≈2.3M (M87* spin=0.9 prograde ISCO):
//     // β is significantly higher than Schwarzschild ISCO
//     // assert!(orbital_beta(2.3, 1.0, 0.9) > 0.55)
//
//   #[test]
//   fn test_shift_wavelength_blueshift()
//     // Approaching: λ_obs = λ_emit / D < λ_emit (blueshift)
//     // 550nm, beta=0.5, cos=+1: λ_obs ≈ 550/1.732 ≈ 317nm
//     // assert_relative_eq!(result, 317.5, epsilon=1.0)
//
//   #[test]
//   fn test_shift_wavelength_redshift()
//     // Receding: λ_obs = λ_emit / D > λ_emit (redshift)
//     // 550nm, beta=0.5, cos=-1: λ_obs ≈ 550/0.577 ≈ 953nm
//     // assert_relative_eq!(result, 952.6, epsilon=1.0)
//
//   #[test]
//   fn test_build_doppler_lut_dimensions()
//     // build_doppler_lut(256, 256) → Vec of length 256*256 = 65536
//     // All values > 0 (wavelength shift factors are positive)
//     // LUT[0][*]: β=0 row → shift factor ≈ 1.0 for all λ
//     // LUT[255][*]: β=0.99 row → large blueshift at cosθ=+1
//
//   #[test]
//   fn test_beaming_asymmetry_magnitude()
//     // At β=0.9: ratio D_approach^4 / D_recede^4 should be very large
//     // D_approach ≈ 4.36, D_recede ≈ 0.229 → ratio ≈ 1625
//     // assert!(ratio > 1000.0)
//
// USES (imports from):
//   gargantua_physics::effects::doppler::*
//   approx::assert_relative_eq    // dev-dependency in Cargo.toml
//
// DOES NOT TEST (out of scope for this file):
//   GPU LUT sampling (tested in gargantua-render integration tests)
//   Combined Doppler+redshift (tested in geodesic.rs)
// ============================================================