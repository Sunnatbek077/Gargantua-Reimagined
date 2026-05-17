// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/qsv.h265.rs
// CRATE: gargantua-video
// LINES: ~180
// PLATFORM: Windows only (Intel)
// =============================================================================
//
// PURPOSE:
//   Intel Quick Sync Video H.265 hardware encoder.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct QsvH265 { inner: QsvEncoder }`
//   - `impl QsvH265`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             QsvEncoder::new(QsvCodec::H265, config).
//   - `impl Encoder for QsvH265`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/qsv.rs → QsvEncoder
//
// INBOUND:
//   - encode/mod.rs → Windows + Intel + H265
// =============================================================================
