// =============================================================================
// crates/gargantua-core/src/platform/windows/video/qsv.rs
// =============================================================================
// PURPOSE:
//   Intel QuickSync Video (QSV) hardware encoder for H.264, HEVC, and AV1
//   on Intel Arc and Intel integrated graphics. Uses the media engine
//   (fixed-function hardware separate from the shader execution units).
//   On Arc GPUs, QSV runs on the Xe Media Engine — significantly faster
//   than software encoding and parallel with GPU render work.
//
// SIZE: ~180 lines
// DEPENDENCIES:
//   Internal:  crate::platform::windows::gpu::vendor::VendorDetails, crate::errors::CoreError
//   External:  mfx_sys (FFI to libmfx.dll / Intel VPL): MFXVideoENCODE,
//              mfxSession, mfxVideoParam, mfxBitstream, mfxFrameSurface1
// CALLED BY:
//   crates/gargantua-video/src/encode/mod.rs — Intel encoder branch
//
// CODEC SUPPORT:
//   H.264:  MFX_CODEC_AVC   — all Intel GPU generations
//   HEVC:   MFX_CODEC_HEVC  — Skylake (Gen9) integrated and Arc
//   AV1:    MFX_CODEC_AV1   — Arc Alchemist and newer (Battlemage preferred)
//
// KEY FUNCTIONS:
//   pub fn is_available() -> bool
//     — MFXLoad() probe via Intel VPL (Video Processing Library).
//
//   pub fn new(config: QsvConfig, output: PathBuf) -> Result<Self, CoreError>
//     — MFXInit → MFXVideoENCODE_Init with ICQ (intelligent constant quality).
//     — Sets MFX_TARGETUSAGE_BEST_QUALITY for offline, BALANCED for real-time.
//
//   pub fn encode_frame(&mut self, frame: &[u8], pts: i64) -> Result<(), CoreError>
//     — AllocFrames → lock surface → copy → MFXVideoENCODE_EncodeFrameAsync
//     — SyncOperation → write bitstream.
//
//   pub fn finish(&mut self) -> Result<(), CoreError>
//     — MFXVideoENCODE_EncodeFrameAsync(null surface) → drain → MFXClose.
//
// NOTES FOR AI:
//   - Intel VPL replaces the legacy Intel Media SDK (libmfx). Use VPL API.
//   - AV1 encoding requires Arc Alchemist or newer. Check IntelArch >= AlchemistArc.
//   - QSV quality mode: ICQ (Intelligent Constant Quality) is recommended for
//     offline render output. CQP (Constant QP) for fixed quality.
//   - MFX_TARGETUSAGE_BEST_QUALITY trades speed for quality — use for offline export.
//     MFX_TARGETUSAGE_BALANCED for real-time preview capture.
// =============================================================================

#![cfg(target_os = "windows")]

use std::{ffi::c_void, path::PathBuf};
use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsvCodec { H264, Hevc, Av1 }

#[derive(Debug, Clone, Copy)]
pub struct QsvConfig {
    pub codec:   QsvCodec,
    pub width:   u32,
    pub height:  u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub bitrate: u32,
    pub quality: u32,
    pub hdr10:   bool,
}

pub struct QsvEncoder {
    session: *mut c_void,
    config:  QsvConfig,
    output:  PathBuf,
}

unsafe impl Send for QsvEncoder {}

impl QsvEncoder {
    pub fn is_available() -> bool { todo!() }
    pub fn new(config: QsvConfig, output: PathBuf) -> Result<Self, CoreError> { todo!() }
    pub fn encode_frame(&mut self, frame: &[u8], pts: i64) -> Result<(), CoreError> { todo!() }
    pub fn finish(&mut self) -> Result<(), CoreError> { todo!() }
}

impl Drop for QsvEncoder {
    fn drop(&mut self) { let _ = self.finish(); }
}