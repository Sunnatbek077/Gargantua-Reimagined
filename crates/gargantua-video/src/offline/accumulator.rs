// =============================================================================
// FILE: crates/gargantua-video/src/offline/accumulator.rs
// CRATE: gargantua-video
// LINES: ~180
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   HDR floating-point frame accumulation buffer for offline rendering.
//   When SPP (samples per pixel) > 1, multiple sub-frames are rendered and
//   averaged together by this accumulator before passing to the encoder.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct FrameAccumulator`:
//       buffer:       Vec<[f32; 4]>    — RGBA32F accumulation buffer (CPU-side)
//       sample_count: u32              — how many sub-frames have been added
//       width:        u32
//       height:       u32
//   - `impl FrameAccumulator`:
//       `pub fn new(width: u32, height: u32) -> Self`
//             Allocates width × height × 4 f32 values, zero-initialised.
//       `pub fn add_frame(&mut self, frame: &[f32])`
//             Adds all pixel values of frame to self.buffer in-place
//             (simple sum accumulation: buffer[i] += frame[i]).
//       `pub fn average(&self) -> Vec<[f32; 4]>`
//             Returns a new Vec where each pixel = buffer[i] / sample_count.
//             The result is the final averaged HDR frame for encoding.
//       `pub fn reset(&mut self)`
//             Zeroes buffer and sets sample_count = 0.
//             Called between frames in the renderer loop.
//       `pub fn is_complete(&self, target_spp: u32) -> bool`
//             Returns sample_count >= target_spp.
//
// OUTBOUND DEPENDENCIES:
//   - None (pure CPU arithmetic, no external crates)
//
// INBOUND (who uses FrameAccumulator):
//   - crates/gargantua-video/src/offline/renderer.rs → creates one accumulator, calls add_frame()
//                                   target_spp times, then average() for output
//
// NOTES:
//   - The accumulator is CPU-side; the GPU ray marcher renders sub-frames
//     into a GPU texture (accumulate.wgsl handles GPU-side accumulation for
//     realtime mode). For offline, frames are read back to CPU first.
//   - Using f32 avoids quantisation error from repeated addition that would
//     occur with u8 or f16 buffers.
// =============================================================================
