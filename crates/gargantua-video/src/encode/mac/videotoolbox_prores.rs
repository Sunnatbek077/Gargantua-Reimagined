// =============================================================================
// FILE: crates/gargantua-video/src/encode/mac/videotoolbox_prores.rs
// CRATE: gargantua-video
// LINES: ~280
// PLATFORM: Mac only
// =============================================================================
//
// PURPOSE:
//   Apple VideoToolbox ProRes hardware encoder. Supports ProRes RAW,
//   ProRes 4444 XQ, ProRes 422 HQ — the professional editing formats used
//   by Final Cut Pro and DaVinci Resolve.
//
// WHAT THIS FILE CONTAINS:
//   - `pub enum ProResVariant { Raw, Xq4444, Hq422, Lt422, Proxy422 }`
//   - `pub struct VideoToolboxProRes`:
//       session:  VTCompressionSession
//       variant:  ProResVariant
//   - `impl VideoToolboxProRes`:
//       `pub fn new(variant: ProResVariant, config: &EncodeConfig)
//                  -> Result<Self, VideoError>`
//             VTCompressionSessionCreate with:
//               kCMVideoCodecType_AppleProRes4444XQ  (for Xq4444)
//               kCMVideoCodecType_AppleProRes422HQ   (for Hq422)
//               kCMVideoCodecType_AppleProResRAW     (for Raw)
//             RequireHardwareAcceleratedVideoEncoder = true.
//             ProRes RAW requires an Apple Silicon M1+ media engine.
//       `pub fn encode_frame(&mut self, buf: CVPixelBufferRef, pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//             Returns a ProRes frame wrapped in a QuickTime Movie Atom header
//             (not annex-B; ProRes uses QuickTime container framing).
//       `pub fn bitrate_for_variant(variant: &ProResVariant, width: u32, height: u32) -> u64`
//             Returns the theoretical uncompressed-ish bitrate for each variant
//             (ProRes 4444 XQ 4K = ~1.1 GB/s).
//
// OUTBOUND DEPENDENCIES:
//   - videotoolbox-sys, core-media (Apple frameworks)
//   - errors.rs → VideoError
//
// INBOUND:
//   - encode/mod.rs → selected for Mac + ProRes codec choice
//
// NOTES:
//   - ProRes is a LOSSLESS-grade codec (visually lossless, not mathematically).
//     File sizes are very large; use only for intermediate editing masters.
//   - On Windows, ProRes encoding is not hardware-accelerated; the UI widget
//     (codec_selector.rs) greys out ProRes when running on Windows.
// =============================================================================
