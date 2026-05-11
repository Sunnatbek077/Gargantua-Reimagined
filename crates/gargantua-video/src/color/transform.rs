// =============================================================================
// FILE: crates/gargantua-video/src/color/transform.rs
// CRATE: gargantua-video
// LINES: ~220
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Applies the scene-linear colour space transform from the internal
//   rendering colour space (scene-linear Rec.2020) to the output colour space
//   (Rec.709 / Display P3 / Rec.2020) using a 3×3 matrix multiplication,
//   optionally followed by a 3D LUT.
//
// WHAT THIS FILE CONTAINS:
//   - `pub enum OutputColorSpace { Rec709, DisplayP3, Rec2020 }`
//   - `pub struct ColorTransform`:
//       matrix:       [[f32; 3]; 3]   — colour gamut conversion matrix
//       lut:          Option<Lut3d>   — optional 3D LUT (e.g. ACES RRT)
//       output_space: OutputColorSpace
//   - `impl ColorTransform`:
//       `pub fn new(output: OutputColorSpace, lut_path: Option<&Path>)
//                  -> Result<Self, VideoError>`
//             Selects the appropriate primaries matrix:
//               Rec709:      Rec2020→Rec709 3×3 matrix (Bradford adaptation)
//               DisplayP3:   Rec2020→P3-D65 matrix
//               Rec2020:     identity matrix
//             Optionally loads the .cube LUT from lut_path via Lut3d::load_cube().
//       `pub fn apply(&self, linear_rgb: [f32; 3]) -> [f32; 3]`
//             1. Matrix multiply: linear_rgb × self.matrix → gamut-converted RGB.
//             2. If self.lut.is_some(): apply LUT on top of matrix result.
//             3. Returns final display-referred RGB (still linear; gamma is
//                applied later by the codec or the tonemap pass).
//       `pub fn apply_frame(&self, frame: &mut Vec<[f32; 3]>)`
//             Applies apply() to every pixel of the frame in-place.
//             Called from renderer.rs once per offline frame.
//
// OUTBOUND DEPENDENCIES:
//   - color/lut_3d.rs → Lut3d for optional 3D LUT application
//   - errors.rs       → VideoError
//
// INBOUND (who uses ColorTransform):
//   - video/offline/renderer.rs → instantiated from OfflineConfig.output_space,
//                                  called per frame before encoding
//
// NOTES:
//   - All matrices are pre-computed from CIE chromaticity coordinates; see
//     docs/physics.md for derivation details.
//   - For Rec.2020 output (HDR), no matrix is needed — scene space = output space.
//     The identity path skips the multiplication entirely.
// =============================================================================
