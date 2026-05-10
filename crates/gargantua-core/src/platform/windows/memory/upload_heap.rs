// =============================================================================
// crates/gargantua-core/src/platform/windows/memory/upload_heap.rs
// =============================================================================
//
// PURPOSE:
//   Abstracts CPU→GPU buffer upload strategy on Windows. Selects between:
//     1. Direct write (ReBAR available): CPU writes directly to GPU buffer
//        via queue.write_buffer() with D3D12_HEAP_TYPE_UPLOAD buffers.
//     2. Staging pool (no ReBAR): CPU writes to staging buffer, GPU copies
//        to final buffer via CommandEncoder::copy_buffer_to_buffer().
//
//   Provides a unified interface so render passes do not need to know
//   whether ReBAR is active — they just call upload_heap.upload().
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::windows::compute::shared_mem::SharedMem
//     - crate::platform::windows::memory::staging_pool::StagingPool
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Queue, Buffer, CommandEncoder}
//     - std::sync::Arc
//
// CALLED BY:
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//   - crates/gargantua-render/src/pipelines/geodesic.rs
//   - crates/gargantua-core/src/frame/frame_graph.rs
//
// PUBLIC TYPES:
//
//   pub struct UploadHeap {
//     staging: Option<StagingPool>,  // None if ReBAR available
//     shared:  SharedMem,
//     queue:   Arc<wgpu::Queue>,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device: Arc<wgpu::Device>,
//     queue:  Arc<wgpu::Queue>,
//     shared: SharedMem,
//   ) -> Self
//     — if shared.rebar_available(): staging = None (skip staging pool).
//     — else: staging = Some(StagingPool::new(device.clone())).
//
//   pub fn upload(
//     &mut self,
//     data:    &[u8],
//     buffer:  &wgpu::Buffer,
//     offset:  u64,
//     encoder: Option<&mut wgpu::CommandEncoder>,
//   )
//     — if ReBAR: calls queue.write_buffer(buffer, offset, data). encoder unused.
//     — if staging: requires encoder (panics if None).
//       calls staging.write_and_copy(data, buffer, offset, encoder).
//
//   pub fn recycle(&mut self)
//     — calls staging.recycle() if staging pool exists.
//     — no-op if ReBAR active.
//     — called at start of each frame by App::render_frame().
//
//   pub fn is_rebar(&self) -> bool
//     — returns true if staging pool is None (ReBAR path active).
// =============================================================================

#![cfg(target_os = "windows")]

use std::sync::Arc;

use crate::{
    errors::CoreError,
    platform::windows::{
        compute::shared_mem::SharedMem,
        memory::staging_pool::StagingPool,
    },
};

pub struct UploadHeap {
    staging: Option<StagingPool>,
    shared:  SharedMem,
    queue:   Arc<wgpu::Queue>,
}

impl UploadHeap {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue:  Arc<wgpu::Queue>,
        shared: SharedMem,
    ) -> Self {
        let staging = if shared.rebar_available() {
            None
        } else {
            Some(StagingPool::new(device))
        };
        Self { staging, shared, queue }
    }

    pub fn upload(
        &mut self,
        data:    &[u8],
        buffer:  &wgpu::Buffer,
        offset:  u64,
        encoder: Option<&mut wgpu::CommandEncoder>,
    ) {
        if let Some(staging) = &mut self.staging {
            let enc = encoder.expect("encoder required without ReBAR");
            staging.write_and_copy(data, buffer, offset, enc);
        } else {
            self.queue.write_buffer(buffer, offset, data);
        }
    }

    pub fn recycle(&mut self) {
        if let Some(staging) = &mut self.staging {
            staging.recycle();
        }
    }

    pub fn is_rebar(&self) -> bool {
        self.staging.is_none()
    }
}