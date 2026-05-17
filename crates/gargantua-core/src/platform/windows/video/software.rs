// =============================================================================
// crates/gargantua-core/src/platform/windows/video/software.rs
// =============================================================================
// PURPOSE:
//   CPU software video encoder fallback using libx264 (H.264) and libx265
//   (HEVC). Used when no hardware encoder is available or when the user
//   explicitly selects software encoding for maximum quality/compatibility.
//   Runs on all Windows systems regardless of GPU vendor.
//
//   Also used on WASM (via a stub) and in test environments where hardware
//   encoders cannot be initialized.
//
// SIZE: ~160 lines
// DEPENDENCIES:
//   Internal:  crate::errors::CoreError
//   External:  x264_sys (FFI to x264.dll): x264_t, x264_param_t, x264_nal_t
//              x265_sys (FFI to x265.dll): x265_encoder, x265_param, x265_picture
// CALLED BY:
//   crates/gargantua-video/src/encode/mod.rs — fallback encoder
//   crates/gargantua-video/src/encode/mod.rs    — WASM always uses software
//
// CODEC SUPPORT:
//   H.264:  libx264 — CRF mode (constant rate factor 0-51, default 18)
//   HEVC:   libx265 — CRF mode, ~40% better compression than H.264
//   (No AV1 — libaom is too slow for practical use as a fallback)
//
// KEY FUNCTIONS:
//   pub fn new(config: SoftwareConfig, output: PathBuf) -> Result<Self, CoreError>
//     — x264_param_default_preset("slow", "film") for H.264.
//     — x265_param_default_preset("slow", "film") for HEVC.
//     — Opens output file and muxes to raw Annex B bitstream or MP4.
//
//   pub fn encode_frame(&mut self, frame: &[u8], pts: i64) -> Result<(), CoreError>
//     — Converts RGBA input to YUV420P (libswscale or manual conversion).
//     — x264_encoder_encode / x265_encoder_encode.
//     — Writes NALs to output file.
//
//   pub fn finish(&mut self) -> Result<(), CoreError>
//     — Flushes delayed frames (x264/x265 B-frame pipeline).
//     — Closes encoder and output file.
//
// PERFORMANCE (M1 Pro comparison):
//   1080p 24fps H.264 software: ~15fps encode (realtime possible)
//   4K    24fps H.264 software: ~3fps encode   (slower than realtime)
//   4K    24fps HEVC  software: ~1fps encode   (offline only)
//
// NOTES FOR AI:
//   - libx264 and libx265 are LGPL licensed. Bundle as DLLs with the app
//     or link statically (check license terms for static linking).
//   - CRF 18 for H.264 and CRF 22 for HEVC produce visually lossless output
//     for the Gargantua render pipeline (HDR is stored in 16-bit).
//   - RGBA → YUV420P conversion loses color precision. For HDR output,
//     use YUV420P10 (10-bit) with HEVC. libx265 supports 10-bit natively.
// =============================================================================

use std::path::PathBuf;
use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftwareCodec { H264, Hevc }

#[derive(Debug, Clone, Copy)]
pub struct SoftwareConfig {
    pub codec:   SoftwareCodec,
    pub width:   u32,
    pub height:  u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub crf:     u32,    // 0-51, lower = better quality
    pub preset:  &'static str,  // "slow", "medium", "fast"
    pub hdr10:   bool,
}

pub struct SoftwareEncoder {
    config: SoftwareConfig,
    output: PathBuf,
}

impl SoftwareEncoder {
    pub fn new(config: SoftwareConfig, output: PathBuf) -> Result<Self, CoreError> {
        todo!()
    }
    pub fn encode_frame(&mut self, frame: &[u8], pts: i64) -> Result<(), CoreError> { todo!() }
    pub fn finish(&mut self) -> Result<(), CoreError> { todo!() }
}

impl Drop for SoftwareEncoder {
    fn drop(&mut self) { let _ = self.finish(); }
}