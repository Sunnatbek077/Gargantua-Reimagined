// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/amf_h265.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Windows only (AMD)
// =============================================================================
//
// PURPOSE:
//   AMD AMF H.265 (HEVC) 10-bit HDR hardware encoder. Available on RX 5000
//   and later. RX 6000+ supports 10-bit output with HDR10 metadata.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct AmfH265 { inner: AmfEncoder }`
//   - `impl AmfH265`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             AmfEncoder::new(AmfCodec::H265, config).
//             Sets HEVC Main10 profile if config.bit_depth == 10.
//   - `impl Encoder for AmfH265`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/amf.rs → AmfEncoder
//
// INBOUND:
//   - encode/mod.rs → Windows + AMD + H265
// =============================================================================
