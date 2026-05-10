// =============================================================================
// crates/gargantua-core/src/platform/windows/memory/staging_pool.rs
// =============================================================================
//
// PURPOSE:
//   Manages a pool of CPU-visible staging buffers for efficient CPU→GPU
//   data uploads on Windows discrete GPU systems (where CPU and GPU have
//   separate physical memory). Reuses staging buffers across frames to
//   avoid repeated allocation/deallocation overhead.
//
//   On systems without ReBAR (most gaming PCs), uploading data to a GPU
//   buffer requires: write to staging buffer → copy staging → GPU buffer.
//   This pool amortizes the staging buffer allocation cost by maintaining
//   a free list of reusable buffers, matched by size.
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Buffer, BufferDescriptor, BufferUsages,
//              CommandEncoder, Queue}
//     - std::sync::Arc
//     - std::collections::HashMap
//
// CALLED BY:
//   - crate::platform::windows::memory::upload_heap::UploadHeap
//       — uses staging pool when ReBAR is unavailable
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — calls StagingPool::upload_buffer() for per-frame uniform updates
//
// PUBLIC TYPES:
//
//   pub struct StagingPool {
//     device:    Arc<wgpu::Device>,
//     free_list: HashMap<u64, Vec<wgpu::Buffer>>,  // size → list of free buffers
//     in_use:    Vec<(wgpu::Buffer, u64)>,          // (buffer, size) currently in use
//   }
//
//   pub struct StagingHandle {
//     pub buffer: wgpu::Buffer,
//     pub size:   u64,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(device: Arc<wgpu::Device>) -> Self
//     — creates an empty pool. No GPU allocations at this point.
//
//   pub fn acquire(&mut self, size: u64) -> StagingHandle
//     — finds a compatible buffer in free_list[size] or creates a new one:
//         wgpu::BufferDescriptor {
//           size:  size,
//           usage: BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
//           mapped_at_creation: true,  // pre-mapped for immediate write
//         }
//     — moves the buffer from free_list to in_use.
//     — returns StagingHandle { buffer, size }.
//     — size matching: exact match first, then smallest buffer >= size.
//
//   pub fn write_and_copy(
//     &mut self,
//     data:    &[u8],
//     dst:     &wgpu::Buffer,
//     offset:  u64,
//     encoder: &mut wgpu::CommandEncoder,
//   )
//     — acquires a staging buffer of size data.len() as u64.
//     — writes data into the staging buffer via buffer.slice(..).get_mapped_range_mut().
//     — unmaps the staging buffer.
//     — encodes encoder.copy_buffer_to_buffer(&staging, 0, dst, offset, size).
//     — marks the staging buffer as in_use (will be released in recycle()).
//
//   pub fn recycle(&mut self)
//     — called once per frame after queue.submit().
//     — moves all in_use buffers back to free_list (GPU is done reading them).
//     — GPU-CPU sync is guaranteed by the frame timeline:
//         frame N staging buffers are submitted in frame N's CommandBuffer.
//         frame N+1 recycle() is called after frame N submit().
//         wgpu guarantees the GPU has finished with frame N's commands
//         before the next frame's submit() completes on the CPU timeline.
//
//   pub fn stats(&self) -> StagingPoolStats
//     — returns current pool state for debug overlay.
//
//   pub struct StagingPoolStats {
//     pub free_buffers:   usize,
//     pub in_use_buffers: usize,
//     pub total_mb:       f32,
//   }
//
// NOTES FOR AI:
//   - mapped_at_creation = true allows writing to the buffer before
//     any GPU work. The buffer is pre-mapped on the CPU side.
//     buffer.unmap() must be called before the GPU can read it.
//   - recycle() must be called AFTER queue.submit() but BEFORE the next
//     frame's write_and_copy() calls. The correct place is at the start
//     of App::render_frame(), after the previous frame's submit.
//   - free_list key is the exact buffer size in bytes. Size matching
//     uses exact match first to avoid wasting larger buffers on small uploads.
//   - The pool grows monotonically — buffers are never dropped until the
//     pool itself is dropped. This is intentional: steady-state memory use
//     is reached quickly (usually after 2-3 frames).
//   - On systems with ReBAR (shared_mem.rs), this pool is bypassed entirely:
//     queue.write_buffer() handles uploads without staging buffers.
// =============================================================================

#![cfg(target_os = "windows")]

use std::{collections::HashMap, sync::Arc};
use crate::errors::CoreError;

pub struct StagingHandle {
    pub buffer: wgpu::Buffer,
    pub size:   u64,
}

pub struct StagingPoolStats {
    pub free_buffers:   usize,
    pub in_use_buffers: usize,
    pub total_mb:       f32,
}

pub struct StagingPool {
    device:    Arc<wgpu::Device>,
    free_list: HashMap<u64, Vec<wgpu::Buffer>>,
    in_use:    Vec<(wgpu::Buffer, u64)>,
}

impl StagingPool {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            free_list: HashMap::new(),
            in_use:    Vec::new(),
        }
    }

    pub fn acquire(&mut self, size: u64) -> StagingHandle {
        todo!()
    }

    pub fn write_and_copy(
        &mut self,
        data:    &[u8],
        dst:     &wgpu::Buffer,
        offset:  u64,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        todo!()
    }

    pub fn recycle(&mut self) {
        for (buf, size) in self.in_use.drain(..) {
            self.free_list.entry(size).or_default().push(buf);
        }
    }

    pub fn stats(&self) -> StagingPoolStats {
        let free  = self.free_list.values().map(|v| v.len()).sum();
        let total = self.free_list.iter()
            .map(|(sz, v)| *sz * v.len() as u64)
            .sum::<u64>()
            + self.in_use.iter().map(|(_, sz)| sz).sum::<u64>();
        StagingPoolStats {
            free_buffers:   free,
            in_use_buffers: self.in_use.len(),
            total_mb:       total as f32 / (1024.0 * 1024.0),
        }
    }
}