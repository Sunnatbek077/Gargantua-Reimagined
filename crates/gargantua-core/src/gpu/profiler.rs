// =============================================================================
// crates/gargantua-core/src/gpu/profiler.rs
// =============================================================================
//
// PURPOSE:
//   Records GPU-side timing for each render and compute pass using wgpu
//   timestamp queries. Results are read back to CPU and exposed as
//   per-pass millisecond durations displayed in the stats bar overlay
//   (gargantua-ui/src/overlay/stats_bar.rs).
//
//   If the GPU does not support TIMESTAMP_QUERY (checked via limits.rs),
//   the profiler becomes a no-op and all timings return 0.0.
//
// SIZE: ~120 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::context::GpuContext   — checks supports_timestamp_queries()
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Queue, QuerySet, QueryType, Buffer, BufferUsages,
//              CommandEncoder, BufferView}
//     - std::collections::HashMap
//
// CALLED BY:
//   - crate::app::App::render_frame()             — begin/end frame profiling
//   - crate::frame::frame_graph::FrameGraph       — wraps each pass with queries
//   - crates/gargantua-ui/src/overlay/stats_bar.rs — reads frame_timings()
//
// PUBLIC TYPES:
//
//   pub struct GpuProfiler {
//     query_set:      Option<wgpu::QuerySet>,    // None if timestamps unsupported
//     resolve_buffer: Option<wgpu::Buffer>,      // GPU-side resolved timestamps
//     readback_buf:   Option<wgpu::Buffer>,      // CPU-mapped readback buffer
//     pass_names:     Vec<String>,               // pass name per query pair
//     num_passes:     u32,                       // number of registered passes
//     timings:        HashMap<String, f32>,      // last frame's ms per pass
//     timestamp_period: f32,                     // nanoseconds per tick (from adapter)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(device: &wgpu::Device, ctx: &GpuContext, max_passes: u32) -> Self
//     — if ctx.supports_timestamp_queries() is false, returns a no-op profiler.
//     — creates a QuerySet with QueryType::Timestamp and capacity = max_passes * 2
//       (start + end timestamp per pass).
//     — creates resolve_buffer: BufferUsages::QUERY_RESOLVE | COPY_SRC
//       size = max_passes * 2 * 8 bytes (u64 timestamp per query).
//     — creates readback_buf: BufferUsages::COPY_DST | MAP_READ
//       same size as resolve_buffer.
//     — timestamp_period = adapter.get_timestamp_period() (nanoseconds per tick).
//
//   pub fn begin_pass(
//     &mut self,
//     encoder: &mut wgpu::CommandEncoder,
//     pass_name: &str,
//   ) -> u32
//     — writes a start timestamp query at index (pass_idx * 2).
//     — encoder.write_timestamp(&self.query_set, query_idx).
//     — returns the pass_idx used to pair with end_pass().
//     — no-op if timestamp queries are unsupported.
//
//   pub fn end_pass(
//     &mut self,
//     encoder: &mut wgpu::CommandEncoder,
//     pass_idx: u32,
//   )
//     — writes an end timestamp query at index (pass_idx * 2 + 1).
//     — no-op if timestamp queries are unsupported.
//
//   pub fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder)
//     — called once per frame after all passes are encoded.
//     — encoder.resolve_query_set(&query_set, 0..num_queries, &resolve_buffer, 0)
//     — encoder.copy_buffer_to_buffer(&resolve_buffer, 0, &readback_buf, 0, size)
//     — no-op if timestamp queries are unsupported.
//
//   pub fn read_back(&mut self, device: &wgpu::Device, queue: &wgpu::Queue)
//     — maps readback_buf for reading (async, polls until ready).
//     — converts raw u64 timestamp pairs to milliseconds:
//         ms = (end_tick - start_tick) as f32 * timestamp_period / 1_000_000.0
//     — stores results in self.timings HashMap keyed by pass name.
//     — unmaps the buffer after reading.
//     — called after queue.submit() + device.poll(Maintain::Wait).
//
//   pub fn frame_timings(&self) -> &HashMap<String, f32>
//     — returns the last frame's per-pass timings in milliseconds.
//     — read by stats_bar.rs to display GPU pass timings in the overlay.
//
//   pub fn total_gpu_ms(&self) -> f32
//     — sums all values in self.timings.
//     — displayed as "GPU: X.Xms" in the stats bar.
//
// NOTES FOR AI:
//   - Timestamp queries require TIMESTAMP_QUERY in Features (limits.rs).
//     Always guard with if self.query_set.is_some() before using.
//   - timestamp_period varies by GPU:
//       Apple M1 Pro Metal: ~41.67 ns/tick (24 MHz timer)
//       NVIDIA RTX 4090 DX12: ~1.0 ns/tick
//   - Buffer mapping is async on wgpu. Use device.poll(Maintain::Wait)
//     to block until the GPU has finished writing before mapping.
//     In the real-time viewer, this adds a 1-frame delay to the timings
//     display (acceptable for a profiling overlay).
//   - max_passes should be set to ~20 (number of passes in the frame graph).
//     Current passes: ray_march, geodesic, accretion, lensing, starfield,
//     taa, bloom_down, bloom_up, chromatic, film_grain, motion_blur, tonemap.
// =============================================================================

use std::collections::HashMap;
use crate::{errors::CoreError, gpu::context::GpuContext};

pub struct GpuProfiler {
    query_set:        Option<wgpu::QuerySet>,
    resolve_buffer:   Option<wgpu::Buffer>,
    readback_buf:     Option<wgpu::Buffer>,
    pass_names:       Vec<String>,
    num_passes:       u32,
    timings:          HashMap<String, f32>,
    timestamp_period: f32,
}

impl GpuProfiler {
    pub fn new(device: &wgpu::Device, ctx: &GpuContext, max_passes: u32) -> Self {
        todo!()
    }

    pub fn begin_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
    ) -> u32 {
        todo!()
    }

    pub fn end_pass(&mut self, encoder: &mut wgpu::CommandEncoder, pass_idx: u32) {
        todo!()
    }

    pub fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder) {
        todo!()
    }

    pub fn read_back(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        todo!()
    }

    pub fn frame_timings(&self) -> &HashMap<String, f32> {
        &self.timings
    }

    pub fn total_gpu_ms(&self) -> f32 {
        self.timings.values().sum()
    }
}