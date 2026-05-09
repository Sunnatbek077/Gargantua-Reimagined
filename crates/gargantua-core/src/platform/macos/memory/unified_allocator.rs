// =============================================================================
// crates/gargantua-core/src/platform/macos/memory/unified_allocator.rs
// =============================================================================
//
// PURPOSE:
//   Manages GPU memory allocation strategy for Apple Silicon's unified
//   memory architecture, where CPU and GPU share the same physical RAM.
//   Unlike discrete GPU systems, there is no PCIe transfer penalty — but
//   there IS contention: every byte the GPU uses is unavailable to the CPU
//   and OS.
//
//   This allocator wraps wgpu's buffer/texture creation and tracks total
//   GPU memory usage, enforces a configurable budget, and coordinates
//   with MemoryPressureWatcher to respond to OS memory pressure events.
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::macos::gpu::memory_pressure::{MemoryPressureWatcher, MemoryPressureLevel}
//     - crate::platform::macos::memory::pressure_response::PressureResponse
//     - crate::platform::macos::memory::zero_copy_readback::ZeroCopyReadback
//     - crate::frame::resource::ResourcePool
//     - crate::platform::macos::gpu::chip_detect::ChipInfo
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Buffer, Texture, BufferDescriptor, TextureDescriptor}
//     - std::sync::{Arc, Mutex}
//
// CALLED BY:
//   - crates/gargantua-app/src/app.rs::App::new()
//       — creates UnifiedAllocator at startup, passing ChipInfo
//   - crates/gargantua-core/src/frame/resource::ResourcePool
//       — calls allocate_texture/allocate_buffer instead of device directly
//
// PUBLIC TYPES:
//
//   pub struct UnifiedAllocator {
//     device:           Arc<wgpu::Device>,
//     watcher:          MemoryPressureWatcher,
//     pressure_resp:    PressureResponse,
//     budget_bytes:     u64,    // total GPU memory budget
//     allocated_bytes:  u64,    // current GPU memory in use
//   }
//
//   pub struct AllocationStats {
//     pub budget_mb:    f32,
//     pub allocated_mb: f32,
//     pub free_mb:      f32,
//     pub pressure:     MemoryPressureLevel,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:    Arc<wgpu::Device>,
//     chip_info: &ChipInfo,
//   ) -> Result<Self, CoreError>
//     — sets budget_bytes based on total system memory and chip tier:
//         16 GB system: budget = 8 GB  (50% — leave room for OS + apps)
//         32 GB system: budget = 20 GB (62%)
//         64 GB system: budget = 44 GB (68%)
//         96 GB+ system: budget = 72 GB (75%)
//     — system memory is read via sysctl hw.memsize.
//     — creates MemoryPressureWatcher and registers a callback that calls
//       pressure_resp.respond_to_pressure() automatically.
//     — returns CoreError::PlatformError on sysctl failure (rare).
//
//   pub fn allocate_texture(
//     &mut self,
//     desc: &wgpu::TextureDescriptor<'_>,
//   ) -> Result<wgpu::Texture, CoreError>
//     — estimates texture size in bytes:
//         width * height * depth * bytes_per_texel(format) * mip_factor
//     — checks if allocated_bytes + size <= budget_bytes.
//       If over budget: calls evict_until_budget() before allocating.
//     — calls device.create_texture(desc).
//     — increments allocated_bytes by estimated size.
//     — returns CoreError::OutOfMemory if eviction cannot free enough space.
//
//   pub fn allocate_buffer(
//     &mut self,
//     desc: &wgpu::BufferDescriptor<'_>,
//   ) -> Result<wgpu::Buffer, CoreError>
//     — same pattern as allocate_texture but for buffers.
//     — buffer size = desc.size (exact).
//
//   pub fn deallocate_texture(&mut self, size_bytes: u64)
//     — decrements allocated_bytes when a texture is dropped.
//     — called by ResourcePool::release() with the known texture size.
//
//   pub fn deallocate_buffer(&mut self, size_bytes: u64)
//     — same for buffers.
//
//   pub fn stats(&self) -> AllocationStats
//     — returns current memory stats for the stats bar overlay.
//
//   pub fn poll(&mut self, pool: &mut ResourcePool)
//     — called once per frame by App.
//     — checks MemoryPressureWatcher::current_level().
//     — if level changed: calls pressure_resp.respond_to_pressure().
//     — if should_restore_quality(): gradually restores quality_factor.
//
// PRIVATE FUNCTIONS:
//
//   fn evict_until_budget(&mut self, pool: &mut ResourcePool, needed: u64)
//     — called when allocation would exceed budget.
//     — first: pool.release_all_transient() (free reusable textures).
//     — if still over budget: log a warning and attempt to continue
//       (wgpu will fail at create_texture if truly out of VRAM).
//
//   fn system_memory_bytes() -> u64
//     — reads hw.memsize via sysctl and returns total RAM in bytes.
//
// NOTES FOR AI:
//   - On Apple Silicon, "VRAM" and "RAM" are the same physical pool.
//     wgpu does not expose a Metal memory heap API — budget is enforced
//     by this module, not by the GPU driver.
//   - bytes_per_texel(TextureFormat) must handle:
//       Rgba8Unorm:  4 bytes
//       Rgba16Float: 8 bytes
//       Rgba32Float: 16 bytes
//       Bgra8Unorm:  4 bytes
//   - Mip factor for a full mip chain ≈ 1.333 (geometric series sum).
//     Use 1.0 for single-mip textures (most Gargantua textures).
//   - The 50% budget on 16GB leaves 8GB for the OS, Metal driver, browser,
//     and other apps. Do not increase beyond 60% on 16GB machines.
// =============================================================================

#![cfg(target_os = "macos")]

use std::sync::Arc;

use crate::{
    errors::CoreError,
    frame::resource::ResourcePool,
    platform::macos::{
        gpu::{
            chip_detect::ChipInfo,
            memory_pressure::{MemoryPressureLevel, MemoryPressureWatcher},
        },
        memory::pressure_response::PressureResponse,
    },
};

pub struct AllocationStats {
    pub budget_mb:    f32,
    pub allocated_mb: f32,
    pub free_mb:      f32,
    pub pressure:     MemoryPressureLevel,
}

pub struct UnifiedAllocator {
    device:          Arc<wgpu::Device>,
    watcher:         MemoryPressureWatcher,
    pressure_resp:   PressureResponse,
    budget_bytes:    u64,
    allocated_bytes: u64,
}

impl UnifiedAllocator {
    pub fn new(
        device:    Arc<wgpu::Device>,
        chip_info: &ChipInfo,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn allocate_texture(
        &mut self,
        desc: &wgpu::TextureDescriptor<'_>,
    ) -> Result<wgpu::Texture, CoreError> {
        todo!()
    }

    pub fn allocate_buffer(
        &mut self,
        desc: &wgpu::BufferDescriptor<'_>,
    ) -> Result<wgpu::Buffer, CoreError> {
        todo!()
    }

    pub fn deallocate_texture(&mut self, size_bytes: u64) {
        self.allocated_bytes = self.allocated_bytes.saturating_sub(size_bytes);
    }

    pub fn deallocate_buffer(&mut self, size_bytes: u64) {
        self.allocated_bytes = self.allocated_bytes.saturating_sub(size_bytes);
    }

    pub fn stats(&self) -> AllocationStats {
        let budget_mb    = self.budget_bytes    as f32 / (1024.0 * 1024.0);
        let allocated_mb = self.allocated_bytes as f32 / (1024.0 * 1024.0);
        AllocationStats {
            budget_mb,
            allocated_mb,
            free_mb:   budget_mb - allocated_mb,
            pressure:  self.watcher.current_level(),
        }
    }

    pub fn poll(&mut self, pool: &mut ResourcePool) {
        todo!()
    }

    fn evict_until_budget(&mut self, pool: &mut ResourcePool, needed: u64) {
        todo!()
    }

    fn system_memory_bytes() -> u64 {
        todo!()
    }
}