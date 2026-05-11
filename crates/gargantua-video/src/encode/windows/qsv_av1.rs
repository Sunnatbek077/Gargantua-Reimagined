// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/qsv_av1.rs
// CRATE: gargantua-video
// LINES: ~180
// PLATFORM: Windows only (Intel Arc GPU)
// =============================================================================
//
// PURPOSE:
//   Intel Arc GPU AV1 hardware encoder via Quick Sync / oneVPL.
//   AV1 encode is available on Intel Arc B580 and A770, not on iGPUs.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct QsvAv1 { inner: QsvEncoder }`
//   - `impl QsvAv1`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             QsvEncoder::new(QsvCodec::AV1, config).
//       `pub fn is_arc_gpu_available() -> bool`
//             Checks MFXQueryVersion() returns oneVPL 2.6+ (Arc requirement).
//   - `impl Encoder for QsvAv1`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/qsv.rs → QsvEncoder
//
// INBOUND:
//   - encode/mod.rs → Windows + Intel Arc + AV1
// =============================================================================
