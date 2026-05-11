// =============================================================================
// FILE: crates/gargantua-video/src/realtime/capturer.rs
// CRATE: gargantua-video
// LINES: ~280
// PLATFORM: Mac + Windows
// =============================================================================
//
// PURPOSE:
//   Real-time GPU frame capturer for the live recording feature.
//   On Mac: uses zero-copy unified memory readback (no PCIe, no stall).
//   On Windows: uses triple-buffered PCIe staging pool to avoid GPU stalls.
//   Feeds raw frames to webcodecs.rs for live hardware encoding.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct RealtimeCapturer`:
//       mac_readback:  Option<ZeroCopyReadback>    — Mac path (zero-copy)
//       win_pool:      Option<StagingPool>         — Windows path (triple buf)
//       frame_tx:      std::sync::mpsc::SyncSender<CapturedFrame>
//       semaphore:     Arc<Semaphore>              — limits frames in flight to 3
//   - `pub struct CapturedFrame`:
//       data:  Vec<u8>     — raw RGBA bytes (or pointer to shared memory on Mac)
//       pts:   u64         — presentation timestamp in microseconds
//       width: u32
//       height: u32
//   - `impl RealtimeCapturer`:
//       `pub fn new(ctx: &GpuContext, width: u32, height: u32) -> Self`
//             On Mac: creates ZeroCopyReadback (unified_allocator.rs).
//             On Windows: creates StagingPool with triple-buffer (staging_pool.rs).
//       `pub fn capture_frame(&mut self, encoder: &mut CommandEncoder,
//                              texture: TextureHandle, pts: u64)`
//             On Mac:
//               Calls zero_copy_readback.copy_from_texture(encoder, texture).
//               On GPU submit, calls read_frame() which reads shared memory
//               and sends CapturedFrame to frame_tx.
//             On Windows:
//               Records copy_texture_to_buffer into the current staging buffer.
//               Calls staging_pool.advance() to rotate triple buffer.
//               Maps the oldest buffer (N-2), sends CapturedFrame to frame_tx.
//       `pub fn frame_receiver(&self) -> &std::sync::mpsc::Receiver<CapturedFrame>`
//             Returns the receiver end; used by webcodecs.rs to pull frames.
//
// OUTBOUND DEPENDENCIES:
//   - platform/macos/memory/zero_copy_readback.rs → Mac path
//   - platform/windows/memory/staging_pool.rs     → Windows path
//   - gpu/context.rs                               → GpuContext
//   - frame/resource.rs                            → TextureHandle
//   - errors.rs                                    → VideoError
//
// INBOUND (who uses RealtimeCapturer):
//   - video/realtime/webcodecs.rs → pulls CapturedFrame from frame_receiver()
//
// NOTES:
//   - The semaphore limits in-flight frames to 3 to prevent the CPU from
//     running too far ahead of the GPU encoder.
//   - On Mac, the zero-copy path means no GPU→CPU data movement at all —
//     the encoder reads directly from the unified memory allocation.
// =============================================================================
