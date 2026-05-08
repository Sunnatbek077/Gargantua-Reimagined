// =============================================================================
// crates/gargantua-core/src/frame/resource.rs
// =============================================================================
//
// PURPOSE:
//   Implements the ResourcePool — a handle-based memory reuse system for
//   GPU textures and buffers. Instead of allocating and deallocating GPU
//   resources every frame (which is expensive), the pool keeps a free list
//   of previously allocated resources and hands them out by handle.
//
//   Transient resources (used within a single frame) are returned to the
//   pool at frame end via FrameGraph::reset(). Persistent resources
//   (swap chain, baked LUTs, accumulation buffer) are allocated once and
//   never returned to the pool.
//
// SIZE: ~260 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Texture, TextureDescriptor, TextureView,
//              Buffer, BufferDescriptor, TextureUsages, BufferUsages}
//     - std::collections::HashMap
//
// CALLED BY:
//   - frame_graph.rs::FrameGraph  — allocates/releases transient resources
//   - All Pass implementors       — resolve handles to TextureView/Buffer
//   - crate::app::App             — allocates persistent resources at startup
//
// PUBLIC TYPES:
//
//   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//   pub struct ResourceHandle(u32);
//     — opaque handle to a GPU resource.
//     — u32 index into ResourcePool's internal slab.
//     — two handles are equal iff they refer to the same slot.
//
//   pub enum ResourceKind {
//     Texture(wgpu::TextureDescriptor<'static>),
//     Buffer(wgpu::BufferDescriptor<'static>),
//   }
//
//   pub struct ResourceEntry {
//     kind:      ResourceKind,
//     texture:   Option<wgpu::Texture>,      // Some if kind is Texture
//     buffer:    Option<wgpu::Buffer>,       // Some if kind is Buffer
//     in_use:    bool,                       // false = in the free list
//     persistent: bool,                     // true = never returned to pool
//   }
//
//   pub struct ResourcePool {
//     entries:   Vec<ResourceEntry>,         // slab allocator
//     free_list: Vec<u32>,                   // indices of free slots
//     device:    Arc<wgpu::Device>,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(device: Arc<wgpu::Device>) -> Self
//     — creates an empty pool. No GPU allocations at this point.
//
//   pub fn allocate_texture(
//     &mut self,
//     desc: wgpu::TextureDescriptor<'static>,
//     persistent: bool,
//   ) -> ResourceHandle
//     — checks the free list for a compatible texture (same format, size,
//       usage flags). If found, marks it in_use and returns the handle.
//     — if not found, calls device.create_texture(&desc), stores it,
//       marks in_use = true, returns a new handle.
//     — persistent = true: the resource is never added to the free list.
//     — used for: framebuffer, HDR accumulation buffer, TAA history buffer,
//       motion blur velocity buffer, baked LUT textures.
//
//   pub fn allocate_buffer(
//     &mut self,
//     desc: wgpu::BufferDescriptor<'static>,
//     persistent: bool,
//   ) -> ResourceHandle
//     — same logic as allocate_texture but for wgpu::Buffer.
//     — used for: scene uniform buffer, physics params buffer,
//       indirect dispatch buffer, staging/readback buffers.
//
//   pub fn get_texture(&self, handle: ResourceHandle) -> &wgpu::Texture
//     — resolves a handle to a wgpu::Texture reference.
//     — panics if the handle is invalid or the resource is a Buffer.
//     — called by passes to get the texture for bind group creation.
//
//   pub fn get_texture_view(
//     &self,
//     handle: ResourceHandle,
//   ) -> wgpu::TextureView
//     — convenience: calls get_texture().create_view(&Default::default()).
//     — most passes use this instead of get_texture() directly.
//
//   pub fn get_buffer(&self, handle: ResourceHandle) -> &wgpu::Buffer
//     — resolves a handle to a wgpu::Buffer reference.
//     — panics if handle is invalid or resource is a Texture.
//
//   pub fn release(&mut self, handle: ResourceHandle)
//     — marks the resource as not in_use and adds its index to free_list.
//     — called by FrameGraph::reset() for all transient resources.
//     — does nothing for persistent resources (persistent flag = true).
//
//   pub fn release_all_transient(&mut self)
//     — iterates all entries, calls release() on every non-persistent entry.
//     — called once per frame by FrameGraph::reset().
//
// COMPATIBILITY MATCHING (for texture reuse):
//   A free texture is compatible with a descriptor if:
//     - format matches exactly
//     - size (width, height, depth_or_array_layers) matches exactly
//     - mip_level_count matches
//     - sample_count matches
//     - usage flags are a superset of the requested flags
//   Buffer compatibility: size >= requested size AND usage is superset.
//
// NOTES FOR AI:
//   - ResourceHandle(0) is reserved as a null/invalid handle.
//     Always check handle.0 != 0 before resolving.
//   - The pool never shrinks — entries are reused but never dropped.
//     This is intentional: GPU allocations are expensive and the pool
//     reaches steady state after the first few frames.
//   - TextureDescriptor has a lifetime parameter. Use 'static by storing
//     the label as a &'static str or by using Option<String> separately.
//   - For the TAA history buffer: allocate_texture with persistent=true
//     and TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT.
// =============================================================================

use std::sync::Arc;

use wgpu::{Buffer, BufferDescriptor, Device, Texture, TextureDescriptor, TextureView};

use crate::errors::CoreError;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceHandle(pub u32);

impl ResourceHandle {
    pub const INVALID: Self = Self(0);
    pub fn is_valid(self) -> bool { self.0 != 0 }
}

pub enum ResourceKind {
    Texture(wgpu::TextureDescriptor<'static>),
    Buffer(wgpu::BufferDescriptor<'static>),
}

pub struct ResourceEntry {
    pub kind:       ResourceKind,
    pub texture:    Option<wgpu::Texture>,
    pub buffer:     Option<wgpu::Buffer>,
    pub in_use:     bool,
    pub persistent: bool,
}

pub struct ResourcePool {
    entries:   Vec<ResourceEntry>,
    free_list: Vec<u32>,
    device:    Arc<Device>,
}

impl ResourcePool {
    pub fn new(device: Arc<Device>) -> Self {
        todo!()
    }

    pub fn allocate_texture(
        &mut self,
        desc: wgpu::TextureDescriptor<'static>,
        persistent: bool,
    ) -> ResourceHandle {
        todo!()
    }

    pub fn allocate_buffer(
        &mut self,
        desc: wgpu::BufferDescriptor<'static>,
        persistent: bool,
    ) -> ResourceHandle {
        todo!()
    }

    pub fn get_texture(&self, handle: ResourceHandle) -> &wgpu::Texture {
        todo!()
    }

    pub fn get_texture_view(&self, handle: ResourceHandle) -> wgpu::TextureView {
        todo!()
    }

    pub fn get_buffer(&self, handle: ResourceHandle) -> &wgpu::Buffer {
        todo!()
    }

    pub fn release(&mut self, handle: ResourceHandle) {
        todo!()
    }

    pub fn release_all_transient(&mut self) {
        todo!()
    }
}