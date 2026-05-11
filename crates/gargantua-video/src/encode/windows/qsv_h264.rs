// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/qsv_h264.rs
// CRATE: gargantua-video
// LINES: ~180
// PLATFORM: Windows only (Intel)
// =============================================================================
//
// PURPOSE:
//   Intel Quick Sync Video H.264 hardware encoder. Available on all modern Intel
//   CPUs with integrated graphics, and on Arc discrete GPUs.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct QsvH264 { inner: QsvEncoder }`
//   - `impl QsvH264`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             QsvEncoder::new(QsvCodec::H264, config).
//             Sets MFX_PROFILE_AVC_HIGH, MFX_LEVEL_AVC_52.
//   - `impl Encoder for QsvH264`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/qsv.rs → QsvEncoder
//
// INBOUND:
//   - encode/mod.rs → Windows + Intel + H264
// =============================================================================
