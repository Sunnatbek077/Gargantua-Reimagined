// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/amf_h264.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Windows only (AMD)
// =============================================================================
//
// PURPOSE:
//   AMD AMF H.264 hardware encoder adapter. Wraps AmfEncoder from
//   platform/windows/video/amf.rs with H.264-specific profile and level settings.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct AmfH264 { inner: AmfEncoder }`
//   - `impl AmfH264`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             AmfEncoder::new(AmfCodec::H264, config).
//             Sets AMF_VIDEO_ENCODER_PROFILE_HIGH, level 5.2 for 4K.
//   - `impl Encoder for AmfH264`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/amf.rs → AmfEncoder
//   - errors.rs                      → VideoError
//
// INBOUND:
//   - encode/mod.rs → Windows + AMD + H264
// =============================================================================
