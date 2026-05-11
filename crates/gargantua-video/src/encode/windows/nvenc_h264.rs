// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/nvenc_h264.rs
// CRATE: gargantua-video
// LINES: ~220
// PLATFORM: Windows only (NVIDIA)
// =============================================================================
//
// PURPOSE:
//   NVIDIA NVENC H.264 hardware encoder wrapper. Thin adapter between the
//   Encoder trait and the NvencEncoder in platform/windows/video/nvenc.rs.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct NvencH264 { inner: NvencEncoder }`
//   - `impl NvencH264`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             Calls NvencEncoder::new(device, NvencCodec::H264, config).
//             Sets H.264-specific params:
//               Profile: High 4.2 (for 4K 60fps support)
//               Rate control: CBR or VBR from config.rate_control
//               Ref frames: 1 (low latency) or 4 (quality)
//       `pub fn is_available(device: &ID3D12Device) -> bool`
//             Calls NvencEncoder::is_available(device).
//   - `impl Encoder for NvencH264`:
//       `fn encode_frame(...)`, `fn flush(...)`, `fn codec_name() → "NVENC H.264"`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/nvenc.rs → NvencEncoder, NvencCodec
//   - errors.rs                        → VideoError
//
// INBOUND:
//   - encode/mod.rs → best_encoder() returns NvencH264 for Windows+NVIDIA+H264
// =============================================================================
