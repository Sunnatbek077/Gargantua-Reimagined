// =============================================================================
// FILE: crates/gargantua-video/src/encode/universal/rav1e.rs
// CRATE: gargantua-video
// LINES: ~160
// PLATFORM: Mac + Windows + WASM (pure Rust AV1 encoder)
// =============================================================================
//
// PURPOSE:
//   Software AV1 encoder using the rav1e crate — a pure-Rust AV1 encoder.
//   Used on M1/M2 Macs (no hardware AV1 encode), on WASM, and as the
//   Windows AV1 fallback when NVENC/AMF are unavailable.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct Rav1eEncoder`:
//       ctx:  rav1e::Context<u16>     — 10-bit encoder context
//   - `impl Rav1eEncoder`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             rav1e::Config::new()
//               .with_width(config.width)
//               .with_height(config.height)
//               .with_bit_depth(config.bit_depth)
//               .with_speed_setting(SpeedSetting::Six)  // balanced speed/quality
//               .create_context()?
//       `pub fn encode_frame(&mut self, frame: rav1e::Frame<u16>, pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//             ctx.send_frame(frame)?; ctx.receive_packet()?.data.
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//             ctx.flush(); drain all remaining packets.
//
// OUTBOUND DEPENDENCIES:
//   - rav1e (external, pure Rust)  → AV1 encoder
//   - errors.rs                    → VideoError
//
// INBOUND:
//   - encode/mac/videotoolbox_av1.rs → rav1e_fallback field for M1/M2
//   - encode/mod.rs                  → AV1 software fallback
//
// NOTES:
//   - rav1e SpeedSetting::Six is a good balance: ~2–3× faster than Speed 10
//     (slowest, highest quality) while producing comparable SSIM scores.
//   - WASM builds use rav1e compiled to wasm32-unknown-unknown (no SIMD
//     acceleration in browsers without WASM-SIMD experimental flag).
// =============================================================================
