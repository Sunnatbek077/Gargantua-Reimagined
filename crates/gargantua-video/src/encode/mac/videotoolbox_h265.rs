// =============================================================================
// FILE: crates/gargantua-video/src/encode/mac/videotoolbox_h265.rs
// CRATE: gargantua-video
// LINES: ~220
// PLATFORM: Mac only
// =============================================================================
//
// PURPOSE:
//   Apple VideoToolbox hardware HEVC (H.265) encoder with 10-bit HDR support.
//   Supports HDR10 metadata and HLG (Hybrid Log-Gamma) for broadcast HDR.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct VideoToolboxH265`:
//       session:    VTCompressionSession
//       hdr_config: Option<HdrMetadata>   — HDR10/HLG metadata if enabled
//   - `impl VideoToolboxH265`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             VTCompressionSessionCreate with:
//               Codec: kCMVideoCodecType_HEVC
//               kVTCompressionPropertyKey_ProfileLevel:
//                 if config.bit_depth == 10 → HEVC_Main10_AutoLevel
//                 else → HEVC_Main_AutoLevel
//               kVTCompressionPropertyKey_HDRMetadataInsertionMode:
//                 if config.hdr → Auto (inserts HDR10 SEI NALs)
//       `pub fn set_hdr_metadata(&mut self, master_display: MasterDisplayInfo,
//                                 max_cll: u16, max_fall: u16)`
//             Sets kVTCompressionPropertyKey_MasteringColorVolume and
//             kVTCompressionPropertyKey_ContentLightLevelInfo.
//             These values appear as HDR10 SEI messages in the HEVC stream.
//       `pub fn encode_frame(&mut self, buf: CVPixelBufferRef, pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//
// OUTBOUND DEPENDENCIES:
//   - videotoolbox-sys / core-media (Apple frameworks)
//   - errors.rs → VideoError
//
// INBOUND:
//   - encode/mod.rs → selected for Mac + H265
//
// NOTES:
//   - 10-bit HEVC is the standard container for HDR10 content on Apple platforms.
//   - HLG support allows backward-compatible HDR: the same stream displays
//     correctly on both SDR and HDR monitors.
// =============================================================================
