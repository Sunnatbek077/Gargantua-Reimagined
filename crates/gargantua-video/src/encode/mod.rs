// =============================================================================
// FILE: crates/gargantua-video/src/encode/mod.rs
// CRATE: gargantua-video
// LINES: ~120
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Encoder dispatcher: selects the best available hardware or software encoder
//   for a given platform and requested codec. Exposes a single `Encoder` trait
//   object so renderer.rs does not need platform-specific codec knowledge.
//
// WHAT THIS FILE CONTAINS:
//   - `pub mod mac;`        → videotoolbox_h264, h265, prores, av1
//   - `pub mod windows;`   → nvenc_h264, h265, av1, amf_h264, h265, av1,
//                             qsv_h264, h265, av1
//   - `pub mod universal;` → x264, x265, rav1e, exr_seq
//   - `pub trait Encoder: Send`:
//       `fn encode_frame(&mut self, frame: &[u8], pts: u64) -> Result<Vec<u8>, VideoError>`
//       `fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//       `fn codec_name(&self) -> &str`
//   - `pub fn best_encoder(config: &OfflineConfig) -> Box<dyn Encoder>`
//             Platform × Codec dispatch matrix:
//               Mac  + H264     → VideoToolboxH264 (hardware)
//               Mac  + H265     → VideoToolboxH265 (hardware)
//               Mac  + ProRes   → VideoToolboxProRes (hardware)
//               Mac  + AV1      → VideoToolboxAv1 if M3+, else Rav1e (software)
//               Win  + H264     + NVIDIA → NvencH264
//               Win  + H264     + AMD    → AmfH264
//               Win  + H264     + Intel  → QsvH264
//               Win  + H264     fallback → X264 (software)
//               Win  + H265     → same dispatch pattern as H264
//               Win  + AV1      → NvencAv1 (RTX40+) / AmfAv1 (RX6000+) / Rav1e
//               Any  + EXR_SEQ  → ExrSeq (lossless sequence)
//             Returns Box<dyn Encoder> wrapping the selected encoder.
//
// OUTBOUND DEPENDENCIES:
//   - encode/mac/videotoolbox_*.rs   → Mac hardware encoders
//   - encode/windows/nvenc_*.rs      → NVIDIA encoders
//   - encode/windows/amf_*.rs        → AMD encoders
//   - encode/windows/qsv_*.rs        → Intel encoders
//   - encode/universal/x264.rs etc.  → CPU software encoders
//   - config.rs                      → OfflineConfig (codec, platform settings)
//   - errors.rs                      → VideoError
//
// INBOUND (who calls best_encoder):
//   - crates/gargantua-video/src/offline/renderer.rs → calls best_encoder() once at render startup
//
// NOTES:
//   - is_available() is called for hardware encoders before selecting them.
//     If hardware init fails, falls through to the next option automatically.
//   - The EXR sequence path always succeeds and never returns an error.
// =============================================================================
