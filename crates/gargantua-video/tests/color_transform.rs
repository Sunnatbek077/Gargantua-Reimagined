// =============================================================================
// FILE: crates/gargantua-video/tests/color_transform.rs
// CRATE: gargantua-video
// TYPE: Integration test
// LINES: ~100
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Verifies the mathematical correctness of the colour space transformation
//   pipeline: XYZ → sRGB matrix multiplication, LUT application, and
//   round-trip accuracy. All tests are pure CPU arithmetic — no GPU required.
//
// WHAT THIS FILE CONTAINS:
//
//   --- IMPORTS ---
//   use gargantua_video::color::transform::{ColorTransform, OutputColorSpace};
//   use gargantua_video::color::lut_3d::Lut3d;
//   use std::path::Path;
//
//   --- HELPER CONSTANTS ---
//   const EPSILON: f32 = 1e-4;   — tolerance for floating-point comparisons
//   const D65_WHITE_XYZ: [f64; 3] = [0.95047, 1.00000, 1.08883];
//                                  — CIE D65 illuminant in XYZ
//
//   --- TESTS ---
//
//   #[test]
//   fn test_xyz_to_srgb_white_point()
//         Converts D65_WHITE_XYZ through ColorTransform(OutputColorSpace::Rec709).
//         Expects result ≈ [1.0, 1.0, 1.0] (white in sRGB).
//         Uses assert!((result[i] - 1.0).abs() < EPSILON) for each channel.
//         Validates that the D65 white point maps correctly to sRGB white.
//
//   #[test]
//   fn test_xyz_to_srgb_black_point()
//         Converts [0.0, 0.0, 0.0] through the Rec709 transform.
//         Expects result == [0.0, 0.0, 0.0] exactly (no black lift).
//
//   #[test]
//   fn test_rec2020_identity_transform()
//         Creates ColorTransform with OutputColorSpace::Rec2020 (identity matrix).
//         Converts [0.5, 0.3, 0.8] scene-linear input.
//         Expects result == [0.5, 0.3, 0.8] (no change — identity path).
//         Validates that the Rec2020 → Rec2020 matrix is truly an identity.
//
//   #[test]
//   fn test_display_p3_wider_than_srgb()
//         Creates a highly saturated red: [1.0, 0.0, 0.0] in sRGB primaries.
//         Converts through both Rec709 and DisplayP3 transforms.
//         Asserts that the P3-transformed red has a *higher* R value than sRGB
//         (because P3 primaries are wider — the same physical colour needs a
//         smaller coordinate value in P3 space).
//         This validates that the P3 matrix is not identical to the Rec709 matrix.
//
//   #[test]
//   fn test_matrix_preserves_luminance()
//         Creates ColorTransform for Rec709.
//         Converts a mid-grey: [0.18, 0.18, 0.18] (18% grey card).
//         Asserts that output[0] ≈ output[1] ≈ output[2] (neutral grey stays neutral).
//         Asserts output values ≈ 0.18 ± EPSILON (luminance preserved).
//         Validates that the gamut conversion matrix has no hue or luminance bias
//         for neutral colours.
//
//   #[test]
//   fn test_apply_frame_processes_all_pixels()
//         Creates a Vec<[f32; 3]> of 100 pixels, all set to [0.5, 0.25, 0.75].
//         Calls color_transform.apply_frame(&mut pixels).
//         Asserts all 100 pixels were transformed (none left at original value).
//         Asserts all pixels transformed to the *same* output value
//         (uniform input → uniform output).
//
//   #[test]
//   fn test_lut_3d_identity_cube_no_op()
//         Builds a synthetic identity .cube LUT in memory:
//           LUT_SIZE 2 → 2³ = 8 entries
//           Each entry = its own normalised position: (0,0,0)→(0,0,0),
//           (1,0,0)→(1,0,0), etc.
//         Saves to a tempfile as a valid .cube text file.
//         Loads via Lut3d::load_cube(path).
//         Applies [0.3, 0.6, 0.9] through the identity LUT.
//         Asserts result ≈ [0.3, 0.6, 0.9] ± EPSILON.
//         Validates that the tetrahedral interpolation is correct for an identity LUT.
//
//   #[test]
//   fn test_lut_3d_clamping_out_of_range()
//         Applies [1.5, -0.2, 0.5] (out-of-range values) through a 2³ identity LUT.
//         Asserts that the result is clamped to [1.0, 0.0, 0.5] (domain [0,1]).
//         Validates that the LUT apply() function does not panic on OOB input.
//
//   #[test]
//   fn test_color_transform_with_lut_chaining()
//         Creates ColorTransform(Rec709, lut_path = Some(identity_cube_path)).
//         Converts [0.5, 0.3, 0.7] through the combined matrix + LUT pipeline.
//         Computes the expected result manually: apply matrix first, then LUT.
//         Asserts combined result matches manual calculation ± EPSILON.
//         Validates that the matrix and LUT are applied in the correct order.
//
// OUTBOUND DEPENDENCIES (imports used in tests):
//   - gargantua_video::color::transform → ColorTransform, OutputColorSpace
//   - gargantua_video::color::lut_3d    → Lut3d
//   - tempfile (dev-dependency)          → NamedTempFile for .cube file creation
//   - std::io::Write                     → writing the temp .cube file content
//
// INBOUND (who runs these tests):
//   - cargo test -p gargantua-video           → runs on every platform
//   - .github/workflows/ci.yml               → runs on Mac, Windows, WASM runners
//
// NOTES:
//   - All tests in this file are pure CPU arithmetic with no GPU or
//     platform-specific code. They run identically on Mac, Windows, and WASM.
//   - Tests do NOT validate perceptual colour accuracy (ΔE values) — that is
//     done via the offline baking pipeline (bake/spectrum/blackbody.rs tests).
//     These tests only validate mathematical correctness of the matrix and LUT code.
//   - The EPSILON = 1e-4 tolerance accounts for f32 rounding errors in
//     matrix multiplication. Tighter tolerances would require f64 throughout.
// =============================================================================