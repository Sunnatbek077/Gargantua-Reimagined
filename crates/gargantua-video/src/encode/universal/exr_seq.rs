// =============================================================================
// FILE: crates/gargantua-video/src/encode/universal/exr_seq.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Lossless OpenEXR frame sequence exporter. Writes each rendered frame as
//   a separate .exr file (frame_00001.exr, frame_00002.exr, ...) for use in
//   professional compositing pipelines (Nuke, After Effects, DaVinci Resolve).
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct ExrSeqEncoder`:
//       output_dir:  PathBuf
//       frame_count: u64
//       compression: exr::meta::attribute::Compression
//   - `impl ExrSeqEncoder`:
//       `pub fn new(output_dir: &Path, config: &EncodeConfig) -> Self`
//             Creates the output directory. Sets compression from config:
//               Lossless → exr::Compression::PIZ (wavelet, ~2:1 lossless)
//               Fast     → exr::Compression::ZIP  (deflate, per-scanline)
//       `pub fn encode_frame(&mut self, pixels: &[[f32; 4]], width: u32, height: u32,
//                             pts: u64) -> Result<Vec<u8>, VideoError>`
//             Formats filename as frame_{pts:08}.exr.
//             Calls exr::write_rgba_file() with HALF (f16) precision channels.
//             Returns the raw file bytes (also written to disk).
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//             No-op for EXR sequences (each frame is written immediately).
//       `pub fn is_always_available() -> bool { true }`
//
// OUTBOUND DEPENDENCIES:
//   - exr (external crate)    → OpenEXR 2.x read/write (pure Rust)
//   - std::fs                 → directory creation and file writing
//   - errors.rs               → VideoError
//
// INBOUND:
//   - encode/mod.rs → always available; selected for EXR_SEQ codec choice
//
// NOTES:
//   - EXR sequences are the preferred output for VFX pipelines.
//     Each frame is full 32-bit or 16-bit HDR float with no generation loss.
//   - Disk speed (not CPU) is the bottleneck; NVMe SSD recommended for 4K.
//   - The sequence can be assembled into a video using ffmpeg:
//       ffmpeg -r 24 -i frame_%08d.exr -c:v prores_ks output.mov
// =============================================================================
