// =============================================================================
// FILE: crates/gargantua-video/src/encode/mac/videotoolbox_h264.rs
// CRATE: gargantua-video
// LINES: ~200
// PLATFORM: Mac only
// =============================================================================
//
// PURPOSE:
//   Apple VideoToolbox hardware H.264 encoder. Uses the M1+ media engine for
//   real-time 4K encoding without taxing the GPU or CPU.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct VideoToolboxH264`:
//       session:   VTCompressionSession   — VideoToolbox encode session
//       callback:  Arc<Mutex<Vec<Vec<u8>>>>  — encoded packets from async callback
//   - `impl VideoToolboxH264`:
//       `pub fn new(config: &EncodeConfig) -> Result<Self, VideoError>`
//             Calls VTCompressionSessionCreate with:
//               kVTVideoEncoderSpecification_RequireHardwareAcceleratedVideoEncoder = true
//               Codec: kCMVideoCodecType_H264
//               kVTCompressionPropertyKey_ProfileLevel = H264_High_AutoLevel
//               kVTCompressionPropertyKey_AverageBitRate = config.bitrate_bps
//               kVTCompressionPropertyKey_AllowFrameReordering = false (for live)
//               kVTCompressionPropertyKey_RealTime = true
//       `pub fn encode_frame(&mut self, pixel_buffer: CVPixelBufferRef, pts: u64)
//                            -> Result<Vec<u8>, VideoError>`
//             Calls VTCompressionSessionEncodeFrame(session, pixelBuffer, pts, ...).
//             The completion callback appends annex-B H.264 NAL units to self.callback.
//             Returns the accumulated packets since the last call.
//       `pub fn flush(&mut self) -> Result<Vec<Vec<u8>>, VideoError>`
//             Calls VTCompressionSessionCompleteFrames(kCMTimeIndefinite).
//             Drains remaining frames from the callback queue.
//
// OUTBOUND DEPENDENCIES:
//   - videotoolbox-sys or corevideo-sys (external FFI)
//   - core-media / core-foundation (Apple frameworks)
//   - errors.rs → VideoError
//
// INBOUND (who uses VideoToolboxH264):
//   - encode/mod.rs → best_encoder() returns this for Mac + H264
//
// NOTES:
//   - VideoToolbox H.264 hardware encoder is available on all Apple Silicon
//     (M1 and later) and produces broadcast-quality output.
//   - Annex-B format (start codes) is used for compatibility with ffmpeg/mp4box.
//   - On Intel Macs, hardware H.264 is still available via Quick Sync via
//     VideoToolbox, but performance is lower than Apple Silicon's media engine.
// =============================================================================
