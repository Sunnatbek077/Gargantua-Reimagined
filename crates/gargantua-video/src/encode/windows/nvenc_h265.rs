// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/nvenc_h265.rs
// CRATE: gargantua-video
// LINES: ~240
// PLATFORM: Windows only (NVIDIA, RTX 30+)
// =============================================================================
//
// PURPOSE:
//   NVIDIA NVENC H.265 10-bit HDR hardware encoder. RTX 30+ series GPUs have
//   a dedicated NVENC engine for HEVC with 10-bit + HDR10 metadata injection.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct NvencH265 { inner: NvencEncoder, hdr_sei: Option<HdrSei> }`
//   - `impl NvencH265`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             NvencEncoder::new(device, NvencCodec::H265, config).
//             If config.bit_depth == 10, sets NV_ENC_PROFILE_HEVC_MAIN10.
//             If config.hdr, prepares HDR10 SEI data (MasterDisplayInfo + MaxCLL).
//       `pub fn encode_frame(&mut self, frame: &[u8], pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//             Prepends HDR10 SEI NAL unit if hdr_sei is set before the frame.
//   - `impl Encoder for NvencH265`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/nvenc.rs → NvencEncoder
//   - errors.rs                        → VideoError
//
// INBOUND:
//   - encode/mod.rs → Windows + NVIDIA + H265
// =============================================================================
