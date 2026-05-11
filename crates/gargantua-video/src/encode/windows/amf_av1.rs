// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/amf_av1.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Windows only (AMD RX 6000+)
// =============================================================================
//
// PURPOSE:
//   AMD AMF AV1 hardware encoder. Available on RDNA 2 (RX 6000) and later.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct AmfAv1 { inner: AmfEncoder }`
//   - `impl AmfAv1`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             AmfEncoder::new(AmfCodec::Av1, config).
//       `pub fn is_available() -> bool`
//             Calls AmfEncoder::is_available() and checks AV1 GUID presence.
//   - `impl Encoder for AmfAv1`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/amf.rs → AmfEncoder
//
// INBOUND:
//   - encode/mod.rs → Windows + AMD + AV1
// =============================================================================
