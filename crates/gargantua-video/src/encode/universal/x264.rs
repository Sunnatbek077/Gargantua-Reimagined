// =============================================================================
// FILE: crates/gargantua-video/src/encode/universal/x264.rs
// CRATE: gargantua-video
// LINES: ~160
// PLATFORM: Mac + Windows + WASM (CPU software, always available)
// =============================================================================
//
// PURPOSE:
//   Software H.264 encoder using the x264 library. CPU-based fallback used
//   when no hardware H.264 encoder is available, or when the user explicitly
//   selects software encoding for maximum compatibility.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct X264Encoder`:
//       encoder: *mut x264_t      — raw x264 encoder pointer (unsafe FFI)
//       params:  x264_param_t
//       width:   u32
//       height:  u32
//   - `impl X264Encoder`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             Calls x264_param_default_preset(b"medium\0", b"film\0").
//             Sets width, height, fps, bitrate from config.
//             Calls x264_encoder_open(&params).
//       `pub fn encode_frame(&mut self, yuv: &[u8], pts: i64) -> Result<Vec<u8>, VideoError>`
//             Wraps yuv bytes in x264_picture_t.
//             Calls x264_encoder_encode() → returns NAL units.
//             Concatenates all NAL units into a single annex-B Vec<u8>.
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//             Calls x264_encoder_encode() with null input to drain B-frames.
//   - `impl Drop for X264Encoder`:
//             Calls x264_encoder_close(self.encoder).
//
// OUTBOUND DEPENDENCIES:
//   - x264-sys (external) → raw C FFI to libx264
//   - errors.rs           → VideoError
//
// INBOUND:
//   - encode/mod.rs → final H264 fallback when hardware encoders fail
//   - platform/windows/video/software.rs → SoftwareEncoder uses X264Encoder
//
// NOTES:
//   - x264 preset "medium" balances speed and quality for offline renders.
//   - Input must be in YUV I420 format; the renderer converts RGBA16F → YUV
//     via color/transform.rs before calling encode_frame.
// =============================================================================
