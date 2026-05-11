// =============================================================================
// FILE: crates/gargantua-video/src/encode/universal/x265.rs
// CRATE: gargantua-video
// LINES: ~160
// PLATFORM: Mac + Windows (CPU software)
// =============================================================================
//
// PURPOSE:
//   Software H.265 (HEVC) encoder using the x265 library. CPU fallback for
//   HEVC when no hardware H.265 encoder is available.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct X265Encoder`:
//       encoder:  *mut x265_encoder
//       params:   *mut x265_param
//   - `impl X265Encoder`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             x265_param_default_preset(b"medium\0", b"film\0").
//             Sets bit depth (8 or 10 from config.bit_depth).
//             x265_encoder_open(params).
//       `pub fn encode_frame(&mut self, yuv: &[u8], pts: i64) -> Result<Vec<u8>, VideoError>`
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//   - `impl Drop for X265Encoder`:
//             x265_encoder_close + x265_param_free.
//
// OUTBOUND DEPENDENCIES:
//   - x265-sys or ffmpeg-sys-next (external FFI)
//   - errors.rs → VideoError
//
// INBOUND:
//   - encode/mod.rs → H265 software fallback
// =============================================================================
