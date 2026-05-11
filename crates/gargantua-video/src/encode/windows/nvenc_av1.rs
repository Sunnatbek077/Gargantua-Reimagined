// =============================================================================
// FILE: crates/gargantua-video/src/encode/windows/nvenc_av1.rs
// CRATE: gargantua-video
// LINES: ~220
// PLATFORM: Windows only (NVIDIA RTX 40+, dual engine on RTX 50)
// =============================================================================
//
// PURPOSE:
//   NVIDIA NVENC AV1 hardware encoder. AV1 NVENC was introduced in RTX 40
//   (Ada Lovelace). RTX 50 (Blackwell) has two NVENC engines, enabling
//   dual-stream AV1 or double-speed single-stream encoding.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct NvencAv1 { inner: NvencEncoder }`
//   - `impl NvencAv1`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             NvencEncoder::new(device, NvencCodec::Av1, config).
//             Requires NV_ENC_AV1 GUID support (RTX 40+).
//       `pub fn supports_dual_nvenc(device: &ID3D12Device) -> bool`
//             Checks NVENC session count > 1 via NvEncGetEncodeGUIDCount.
//             Used to enable parallel encode for even faster throughput on RTX 50.
//   - `impl Encoder for NvencAv1`
//
// OUTBOUND DEPENDENCIES:
//   - platform/windows/video/nvenc.rs → NvencEncoder
//   - errors.rs                        → VideoError
//
// INBOUND:
//   - encode/mod.rs → Windows + NVIDIA RTX40+ + AV1
// =============================================================================
