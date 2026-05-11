// =============================================================================
// FILE: crates/gargantua-video/src/encode/mac/videotoolbox_av1.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Mac only
// =============================================================================
//
// PURPOSE:
//   Apple VideoToolbox AV1 hardware encoder (available on M3, M4, M5).
//   On M1/M2 which lack hardware AV1 encode, falls back to the software
//   rav1e encoder from encode/universal/rav1e.rs.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct VideoToolboxAv1`:
//       session:   Option<VTCompressionSession>  — None if hardware unavailable
//       rav1e_fallback: Option<Rav1eEncoder>     — used when session is None
//   - `impl VideoToolboxAv1`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             Attempts VTCompressionSessionCreate with kCMVideoCodecType_AV1.
//             If hardware AV1 is unavailable (M1/M2), creates Rav1eEncoder fallback.
//       `pub fn encode_frame(&mut self, buf: CVPixelBufferRef, pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//             Dispatches to VT session or rav1e based on which is active.
//       `pub fn is_hardware_available() -> bool`
//             Checks VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1).
//             (VideoToolbox uses the same check for encode capability on M3+.)
//
// OUTBOUND DEPENDENCIES:
//   - encode/universal/rav1e.rs   → Rav1eEncoder (software fallback)
//   - videotoolbox-sys            → VT session API
//   - errors.rs                   → VideoError
//
// INBOUND:
//   - encode/mod.rs → selected for Mac + AV1
//
// NOTES:
//   - Hardware AV1 encode was added in M3 (2023). M1/M2 can decode but not encode.
//   - AV1 produces ~30% better quality than H.265 at the same bitrate, making
//     it excellent for streaming-quality offline renders.
// =============================================================================
