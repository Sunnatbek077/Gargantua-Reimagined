// ============================================================
// FILE: crates/gargantua-core/src/frame/barrier.rs
// LINES: ~180
// CATEGORY: Core — Frame synchronization barrier
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Synchronization primitive that ensures all per-frame GPU work
//   (compute shaders, render passes, readbacks) completes before
//   the next frame begins. Acts as a CPU-side fence that waits
//   for the GPU timeline to reach a specific point.
//   Prevents frame overlap that would cause visual tearing or
//   incorrect accumulation results.
//
// CONTENTS (~180 lines):
//   pub struct FrameBarrier {
//       device:               Arc<wgpu::Device>,
//       frame_idx:            u64,   // monotonically increasing frame counter
//       max_frames_in_flight: u32,   // how many frames GPU may lead CPU by
//   }
//
//   impl FrameBarrier {
//       pub fn new(device: Arc<wgpu::Device>, max_in_flight: u32) -> Self
//         // max_in_flight=2 (double-buffering): GPU can be at most 1 frame ahead
//         // max_in_flight=1 (strict): CPU waits every frame (lowest latency)
//
//       // Submit the current frame's command encoder to the GPU queue
//       // and stall the CPU if the GPU is too far ahead.
//       pub fn submit(
//           &mut self,
//           queue: &wgpu::Queue,
//           encoder: wgpu::CommandEncoder,
//       )
//         // queue.submit([encoder.finish()])
//         // frame_idx += 1
//         // If frame_idx % max_frames_in_flight == 0:
//         //     self.wait()  // stall CPU until GPU catches up
//
//       // Block until all pending GPU work is complete.
//       // Required before: buffer readback, pipeline rebuild, shutdown.
//       pub fn wait(&self)
//         // device.poll(wgpu::Maintain::Wait)
//
//       // Non-blocking poll — call each frame to advance async callbacks.
//       pub fn poll(&self)
//         // device.poll(wgpu::Maintain::Poll)
//
//       // Returns true if all submitted GPU work has completed.
//       pub fn is_idle(&self) -> bool
//
//       // Current frame index (used by shaders as per-frame seed).
//       pub fn frame_index(&self) -> u64
//
//       // Reset frame counter (e.g. after pipeline rebuild).
//       // Calls wait() first to drain the GPU.
//       pub fn reset(&mut self)
//   }
//
//   impl Drop for FrameBarrier {
//       fn drop(&mut self)
//         // self.wait()  — ensures GPU is idle before wgpu resources are freed
//   }
//
// USES (imports from):
//   wgpu      → Device, Queue, CommandEncoder, Maintain
//   std::sync::Arc
//
// USED BY:
//   crates/gargantua-app/src/app.rs
//     → barrier.submit(queue, encoder) each frame in the render loop
//     → barrier.wait() before window resize or app shutdown
//   crates/gargantua-render/src/pipelines/accumulate.rs
//     → barrier.wait() before reading back the accumulated pixel buffer
//   crates/gargantua-bake/src/geodesic/let_baker.rs
//     → barrier.wait() before GPU→CPU buffer readback after bake dispatch
//
// NOTE FOR AI:
//   max_frames_in_flight=2 is standard for smooth real-time rendering
//   (GPU stays 1 frame ahead of CPU for pipeline utilization).
//   max_frames_in_flight=1: lower GPU utilization, lower input latency.
//   wgpu::Maintain::Wait blocks the calling thread — do NOT call on the
//   main thread in WASM (freezes the browser). On WASM: use Poll +
//   is_idle() inside the requestAnimationFrame loop instead.
//   frame_index() is consumed by:
//     accumulate.wgsl  → EMA frame counter (1/frame_idx blending)
//     film_grain.wgsl  → animated noise offset per frame
//     geodesic_rk4.wgsl → blue noise jitter for anti-aliasing
//   Drop impl drain is mandatory — skipping it causes wgpu validation
//   errors ("Buffer X destroyed while still in use") on app exit.
// ============================================================