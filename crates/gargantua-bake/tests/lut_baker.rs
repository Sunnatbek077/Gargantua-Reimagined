// ============================================================
// FILE: crates/gargantua-bake/tests/lut_baker.rs
// LINES: ~240
// CATEGORY: Integration test — LUT bake output validation
// RUN: cargo test --package gargantua-bake --test lut_baker
// ============================================================
//
// PURPOSE:
//   Validates that baked LUT files contain physically correct values.
//   Tests the blackbody LUT, Doppler LUT, and (optionally) the
//   geodesic LUT for known reference points.
//   Some tests write actual EXR files to a temp directory.
//
// TESTED FUNCTIONS:
//   spectrum::blackbody::bake()       → blackbody LUT output validation
//   spectrum::doppler_lut::bake()     → Doppler LUT output validation
//   noise::blue_noise::validate_blue_noise()  → noise quality check
//
// NOTE:
//   Geodesic LUT tests are marked #[ignore] by default (slow: ~30s)
//   Run with: cargo test -- --include-ignored
//
// SETUP:
//   fn minimal_params(dir: &tempfile::TempDir) -> BakeParams
//     // BakeParams with minimal sizes for fast tests:
//     // blackbody_lut_size=64, doppler_n_beta=32, doppler_n_lambda=32
//     // geo_spin_steps=8, geo_impact_steps=64
//
//   fn load_exr_1d(path) -> Vec<[f32;3]>
//     // Loads 1D EXR (1 row) into Vec of RGB triplets
//
//   fn load_exr_2d_single(path) -> (Vec<f32>, usize, usize)
//     // Loads 2D single-channel EXR into (data, width, height)
//
// TEST CASES (~240 lines):
//
//   #[test]
//   fn test_blackbody_lut_creates_file()
//     // bake(&params, &tx, &cancel) → Ok(())
//     // assets/baked/blackbody_lut.exr exists and size > 0
//
//   #[test]
//   fn test_blackbody_lut_first_entry_red()
//     // LUT[0] corresponds to T=1000K
//     // 1000K is deep red: R > G > B, all ∈ [0, 1]
//     // assert!(lut[0][0] > lut[0][1])   // R > G
//     // assert!(lut[0][1] > lut[0][2])   // G > B
//
//   #[test]
//   fn test_blackbody_lut_peak_entry_bluewhite()
//     // LUT[last] corresponds to T≈1e9K (blue-white hot)
//     // B ≥ R (blue dominant at extreme temperature)
//     // All channels near-equal (approaches white at very high T)
//
//   #[test]
//   fn test_blackbody_lut_monotonic_blue_channel()
//     // Blue channel should increase monotonically with temperature
//     // (hotter = more blue relative to red)
//     // lut[i+1][2] >= lut[i][2] for most i (allow small non-monotonic)
//
//   #[test]
//   fn test_blackbody_reference_6500k()
//     // T=6500K is approximately D65 daylight (white)
//     // Find the LUT entry closest to 6500K
//     // Assert R ≈ G ≈ B (white/neutral within 20% relative deviation)
//
//   #[test]
//   fn test_doppler_lut_creates_file()
//     // doppler_lut::bake() → Ok(())
//     // assets/baked/doppler_lut.exr exists
//
//   #[test]
//   fn test_doppler_lut_zero_beta_no_shift()
//     // Row 0 (β=0): all shift factors ≈ 1.0
//     // (no velocity → no Doppler shift)
//     // assert_relative_eq!(lut[0][x], 1.0, epsilon=0.01) for all x
//
//   #[test]
//   fn test_doppler_lut_high_beta_blueshift()
//     // Row at β≈0.9 (approaching): shift factors < 1.0 (blueshift)
//     // λ_obs / λ_emit < 1.0 means light shifted to shorter wavelength
//     // assert!(lut[high_beta_row][mid_lambda] < 0.5)
//
//   #[test]
//   fn test_doppler_lut_dimensions()
//     // EXR width == doppler_n_lambda, height == doppler_n_beta
//     // (32 × 32 for minimal params)
//
//   #[test]
//   fn test_blue_noise_quality()
//     // Generate small 64×64 blue noise (CPU fallback if no GPU)
//     // validate_blue_noise(data, 64) == true
//     // (verifies high-frequency dominance in power spectrum)
//
//   #[test]
//   #[ignore]  // slow: ~10s
//   fn test_geodesic_lut_creates_file()
//     // geodesic::let_baker::bake() with minimal params → Ok(())
//     // assets/baked/geodesic_lut.exr exists, size > 0
//
//   #[test]
//   #[ignore]  // slow
//   fn test_geodesic_lut_schwarzschild_deflection()
//     // At spin=0 (Schwarzschild), b >> b_crit:
//     // deflection angle → 4M/b  (Einstein ring formula, weak field)
//     // Check LUT row for spin≈0, large b gives small deflection
//
// USES (imports from):
//   gargantua_bake::spectrum::{blackbody, doppler_lut}
//   gargantua_bake::noise::blue_noise
//   gargantua_bake::scheduler::BakeParams
//   gargantua_bake::errors::BakeError
//   tempfile::TempDir
//   approx::assert_relative_eq
//   exr
// ============================================================