// =============================================================================
// FILE: crates/gargantua-video/src/realtime/webcodecs.rs
// CRATE: gargantua-video
// LINES: ~240
// PLATFORM: Mac + Windows (WebCodecs API for WASM; native hw for desktop)
// =============================================================================
//
// PURPOSE:
//   Encodes real-time captured frames to H.264/VP9/AV1 using the WebCodecs API
//   (in WASM/browser) or hardware encoders on desktop. Bridges the capturer's
//   raw frame output to a compressed bytestream suitable for live recording
//   or streaming.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct WebCodecsEncoder`:
//       codec:     RealtimeCodec
//       encoder:   RealtimeEncoderImpl    — enum: WebCodecs (WASM), HW (desktop)
//       bitrate:   u32
//   - `pub enum RealtimeCodec { H264, Vp9, Av1 }`
//   - `impl WebCodecsEncoder`:
//       `pub fn new(codec: RealtimeCodec, bitrate: u32, width: u32, height: u32)
//                  -> Result<Self, VideoError>`
//             On WASM: initialises the browser VideoEncoder API via web-sys.
//             On native Mac: creates VideoToolboxH264/H265 session.
//             On native Windows: creates NvencH264 / AmfH264 etc.
//       `pub fn encode(&mut self, frame: CapturedFrame)`
//             Calls the active encoder's encode_frame().
//             On WASM: creates a VideoFrame from the CapturedFrame bytes and
//             calls videoEncoder.encode(videoFrame).
//       `pub fn on_chunk<F: Fn(Vec<u8>)>(&mut self, callback: F)`
//             On WASM: sets the VideoEncoder.output callback.
//             On native: polls the encoder for output packets synchronously.
//
// OUTBOUND DEPENDENCIES:
//   - realtime/capturer.rs          → CapturedFrame input
//   - encode/mac/videotoolbox_h264.rs etc. → native hw encoder backends
//   - web-sys (external, WASM only) → VideoEncoder, VideoFrame (browser APIs)
//   - errors.rs                     → VideoError
//
// INBOUND (who uses WebCodecsEncoder):
//   - gargantua-app (recording system) → creates WebCodecsEncoder when user
//                                         starts recording, feeds CapturedFrames
//
// NOTES:
//   - WebCodecs API (WASM path) is available in Chrome 94+ and Firefox 130+.
//   - The native path re-uses the same encode/mac and encode/windows modules
//     as the offline renderer, but with lower latency settings (CBR, no B-frames).
//   - For streaming (future feature), the encoded packets would be sent via
//     WebRTC or WebSocket; currently they are written to a local MP4 file.
// =============================================================================
