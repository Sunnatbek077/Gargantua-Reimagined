// =============================================================================
// FILE: crates/gargantua-video/src/color/lut_3d.rs
// CRATE: gargantua-video
// LINES: ~180
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Loads and applies a .cube format 3D Look-Up Table (LUT) for colour space
//   transforms in the video export pipeline. Uses tetrahedral interpolation
//   (the same method DaVinci Resolve uses) for highest accuracy.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct Lut3d`:
//       size:   u32                  — LUT grid dimension (typically 33 or 65)
//       data:   Vec<[f32; 3]>        — flattened R×G×B → RGB grid
//       domain_min: [f32; 3]         — input range lower bound (usually [0,0,0])
//       domain_max: [f32; 3]         — input range upper bound (usually [1,1,1])
//   - `impl Lut3d`:
//       `pub fn load_cube(path: &Path) -> Result<Self, VideoError>`
//             Parses the Adobe .cube text format:
//               Reads LUT_SIZE, DOMAIN_MIN, DOMAIN_MAX header lines.
//               Reads size³ lines of "R G B" float triplets into self.data.
//             Returns VideoError if the file is malformed.
//       `pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3]`
//             Clamps rgb to [domain_min, domain_max], normalises to [0,1].
//             Performs tetrahedral interpolation:
//               Finds the enclosing cube cell in the LUT grid.
//               Decomposes cell into 6 tetrahedra (Sakamoto decomposition).
//               Interpolates within the correct tetrahedron using barycentric coords.
//             Returns the transformed [R, G, B] triple.
//       `pub fn apply_slice(&self, pixels: &mut [[f32; 3]])`
//             Batch version — applies the LUT to a slice of pixels in-place.
//             Called from renderer.rs for each completed offline frame.
//
// OUTBOUND DEPENDENCIES:
//   - std::fs, std::io          — file reading
//   - errors.rs                 — VideoError
//
// INBOUND (who uses Lut3d):
//   - video/src/color/transform.rs → loads a .cube LUT and chains it after
//                                     the colour matrix transform
//   - crates/gargantua-video/src/offline/renderer.rs    → applies lut_3d to each finished frame
//                                     before encoding
//
// NOTES:
//   - Tetrahedral interpolation preserves achromatic axis (no hue shift on
//     neutrals) unlike trilinear interpolation which can show banding.
//   - The assets/luts/ directory contains rec2020_to_p3.cube and aces_rrt.cube.
//     Users can also load custom .cube files via the export tab.
//   - apply_slice() processes pixels sequentially on CPU; a GPU version would
//     live in tonemap.wgsl (the 3D LUT is already sampled there for realtime).
// =============================================================================
