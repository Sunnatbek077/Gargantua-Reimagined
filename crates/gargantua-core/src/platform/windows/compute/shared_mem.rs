// =============================================================================
// crates/gargantua-core/src/platform/windows/compute/shared_mem.rs
// =============================================================================
//
// PURPOSE:
//   Manages GPU shared memory (ESRAM / resizable BAR / ReBAR) configuration
//   on Windows. On systems with ReBAR enabled, the entire GPU VRAM is
//   directly CPU-addressable, enabling zero-copy uploads similar to Apple
//   Silicon's unified memory — but only for buffer writes (not reads).
//
//   Detects ReBAR availability and configures wgpu buffer creation strategy:
//   - ReBAR available: use UPLOAD heap for all vertex/uniform buffers
//     (direct CPU write, no staging copy needed)
//   - ReBAR unavailable: use staging_pool.rs for CPU→GPU transfers
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::windows::memory::staging_pool::StagingPool
//     - crate::platform::windows::gpu::vendor::GpuVendor
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Buffer, BufferDescriptor, BufferUsages, MemoryHints}
//     - windows_sys::Win32::Graphics::Dxgi::{
//         IDXGIAdapter4, DXGI_QUERY_VIDEO_MEMORY_INFO,
//         DXGI_MEMORY_SEGMENT_GROUP_LOCAL }
//
// CALLED BY:
//   - crate::platform::windows::memory::upload_heap::UploadHeap::new()
//       — checks rebar_available() before choosing heap strategy
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — calls SharedMem::write_uniforms() each frame
//
// PUBLIC TYPES:
//
//   pub struct SharedMem {
//     rebar_available:   bool,      // true if ReBAR / Smart Access Memory active
//     rebar_size_bytes:  u64,       // bytes of CPU-visible VRAM (0 if no ReBAR)
//     device:            Arc<wgpu::Device>,
//   }
//
//   pub struct ReBarInfo {
//     pub available:    bool,
//     pub size_mb:      u64,     // CPU-accessible VRAM in MB
//     pub full_rebar:   bool,    // true if entire VRAM is accessible (not partial)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device: Arc<wgpu::Device>,
//     vendor: GpuVendor,
//   ) -> Result<Self, CoreError>
//     — detects ReBAR via DXGI adapter query:
//         IDXGIAdapter4::QueryVideoMemoryInfo(
//           0,
//           DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
//           &mut info
//         )
//       — compares AvailableForReservation vs Budget to infer ReBAR.
//     — on AMD (Smart Access Memory / SAM): ReBAR is marketed as SAM.
//     — on NVIDIA (RTX 30/40 series): marketed as Resizable BAR.
//     — on Intel Arc: always has ReBAR (unified driver architecture).
//     — stores rebar_available and rebar_size_bytes.
//
//   pub fn rebar_info(&self) -> ReBarInfo
//     — returns ReBarInfo struct for display in the stats overlay.
//
//   pub fn rebar_available(&self) -> bool { self.rebar_available }
//
//   pub fn write_buffer(
//     &self,
//     buffer: &wgpu::Buffer,
//     offset: u64,
//     data:   &[u8],
//     queue:  &wgpu::Queue,
//   )
//     — if rebar_available: writes data directly to the buffer via
//         queue.write_buffer(buffer, offset, data)
//         wgpu uses MTL_RESOURCE_STORAGE_MODE_SHARED equivalent on DX12
//         (D3D12_HEAP_TYPE_UPLOAD) for MAP_WRITE buffers.
//     — if no ReBAR: same queue.write_buffer() path but wgpu will use
//         a staging buffer internally (less efficient, still correct).
//     — used for per-frame uniform updates (scene params, physics params).
//
//   pub fn create_upload_buffer(
//     &self,
//     desc: &wgpu::BufferDescriptor<'_>,
//   ) -> wgpu::Buffer
//     — creates a buffer with optimal usage flags for this system:
//         ReBAR:    BufferUsages::UNIFORM | COPY_DST (direct write)
//         No ReBAR: BufferUsages::UNIFORM | COPY_DST (staging path)
//     — MemoryHints::Prefer32BitAlignedFloat for uniform buffers.
//
// NOTES FOR AI:
//   - ReBAR requires both: (1) GPU support (RTX 30/40, RX 6000+, Arc)
//     AND (2) BIOS/UEFI with "Resizable BAR" / "Above 4G Decoding" enabled.
//     Many users have capable GPUs but ReBAR disabled in BIOS.
//   - On DX12, D3D12_HEAP_TYPE_UPLOAD maps to write-combined CPU memory
//     that the GPU can read from directly. This is NOT zero-copy for reads —
//     only CPU→GPU writes benefit. GPU→CPU readback still requires READBACK heap.
//   - wgpu abstracts this: queue.write_buffer() on COPY_DST buffers
//     automatically uses the most efficient path for the backend.
//   - DXGI QueryVideoMemoryInfo requires IDXGIAdapter4 (DXGI 1.4+),
//     available on Windows 10 1607+ with WDDM 2.1 drivers. Safe assumption.
// =============================================================================

#![cfg(target_os = "windows")]

use std::sync::Arc;
use crate::{errors::CoreError, platform::windows::gpu::vendor::GpuVendor};

pub struct ReBarInfo {
    pub available:  bool,
    pub size_mb:    u64,
    pub full_rebar: bool,
}

pub struct SharedMem {
    rebar_available:  bool,
    rebar_size_bytes: u64,
    device:           Arc<wgpu::Device>,
}

impl SharedMem {
    pub fn new(
        device: Arc<wgpu::Device>,
        vendor: GpuVendor,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn rebar_info(&self) -> ReBarInfo {
        ReBarInfo {
            available:  self.rebar_available,
            size_mb:    self.rebar_size_bytes / (1024 * 1024),
            full_rebar: self.rebar_available,
        }
    }

    pub fn rebar_available(&self) -> bool { self.rebar_available }

    pub fn write_buffer(
        &self,
        buffer: &wgpu::Buffer,
        offset: u64,
        data:   &[u8],
        queue:  &wgpu::Queue,
    ) {
        queue.write_buffer(buffer, offset, data);
    }

    pub fn create_upload_buffer(
        &self,
        desc: &wgpu::BufferDescriptor<'_>,
    ) -> wgpu::Buffer {
        self.device.create_buffer(desc)
    }
}