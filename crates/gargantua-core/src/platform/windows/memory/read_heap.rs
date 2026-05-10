// =============================================================================
// crates/gargantua-core/src/platform/windows/memory/read_heap.rs
// =============================================================================
//
// PURPOSE:
//   Manages GPU→CPU readback (download) of rendered frames on Windows.
//   On discrete GPU systems, GPU→CPU transfer requires a READBACK heap
//   (D3D12_HEAP_TYPE_READBACK): GPU copies result to READBACK buffer,
//   then CPU maps it for reading. This is the Windows counterpart to
//   macOS's zero_copy_readback.rs (but NOT zero-copy — requires a copy).
//
//   Used by gargantua-video for frame capture in the offline render pipeline.
//   Also used by GpuProfiler::read_back() for timestamp readback.
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Buffer, BufferDescriptor, BufferUsages,
//              CommandEncoder, ImageCopyTexture, ImageCopyBuffer,
//              ImageDataLayout, Extent3d, Maintain, MapMode}
//     - std::sync::Arc
//
// CALLED BY:
//   - crates/gargantua-video/src/capture/frame_capture.rs
//   - crates/gargantua-core/src/gpu/profiler.rs
//
// PUBLIC TYPES:
//
//   pub struct ReadHeap {
//     readback_buffer: wgpu::Buffer,   // COPY_DST | MAP_READ
//     buffer_size:     u64,
//     bytes_per_row:   u32,            // aligned to COPY_BYTES_PER_ROW_ALIGNMENT
//     width:           u32,
//     height:          u32,
//     format:          wgpu::TextureFormat,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device: &wgpu::Device,
//     width:  u32,
//     height: u32,
//     format: wgpu::TextureFormat,
//   ) -> Result<Self, CoreError>
//     — calculates bytes_per_row (aligned to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT).
//     — buffer_size = bytes_per_row * height.
//     — creates readback_buffer with:
//         size:  buffer_size
//         usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ
//     — returns CoreError::OutOfMemory on allocation failure.
//
//   pub fn encode_copy(
//     &self,
//     encoder: &mut wgpu::CommandEncoder,
//     source:  &wgpu::Texture,
//   )
//     — encodes texture→buffer copy for GPU execution:
//         encoder.copy_texture_to_buffer(
//           source.as_image_copy(),
//           ImageCopyBuffer {
//             buffer: &self.readback_buffer,
//             layout: ImageDataLayout {
//               offset: 0,
//               bytes_per_row: Some(self.bytes_per_row),
//               rows_per_image: Some(self.height),
//             },
//           },
//           Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
//         )
//
//   pub fn read_frame<F>(&self, device: &wgpu::Device, queue: &wgpu::Queue, f: F)
//     where F: FnOnce(&[u8])
//     — polls device until GPU is done: device.poll(Maintain::Wait).
//     — maps readback_buffer: buffer.slice(..).map_async(MapMode::Read, callback).
//     — device.poll(Maintain::Wait) again to drive the map to completion.
//     — calls f(&mapped_view) — closure receives the raw pixel data.
//     — unmaps the buffer after the closure returns.
//
//   pub fn bytes_per_row(&self) -> u32  { self.bytes_per_row }
//   pub fn buffer_size(&self)   -> u64  { self.buffer_size   }
//   pub fn width(&self)         -> u32  { self.width         }
//   pub fn height(&self)        -> u32  { self.height        }
//
// NOTES FOR AI:
//   - Unlike macOS zero_copy_readback.rs, this IS a copy — the GPU writes
//     to the READBACK buffer, then the CPU reads from it. Two transfers occur:
//       1. GPU renders to RGBA texture
//       2. GPU copies texture → READBACK buffer (encode_copy)
//       3. CPU reads READBACK buffer (read_frame)
//     On discrete GPU, step 2 is over PCIe (~16 GB/s for x16 PCIe 4.0).
//     A 4K RGBA16Float frame = ~63MB → ~4ms transfer on PCIe 4.0 x16.
//   - map_async requires a callback. Use a channel or AtomicBool to wait
//     for the async map to complete synchronously in read_frame().
//   - device.poll(Maintain::Wait) blocks the CPU thread. For offline
//     rendering this is acceptable. Never use in the real-time display path.
//   - bytes_per_row MUST be aligned to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT (256).
//     Unaligned rows cause validation errors.
// =============================================================================

#![cfg(target_os = "windows")]

use crate::errors::CoreError;

pub struct ReadHeap {
    readback_buffer: wgpu::Buffer,
    buffer_size:     u64,
    bytes_per_row:   u32,
    width:           u32,
    height:          u32,
    format:          wgpu::TextureFormat,
}

impl ReadHeap {
    pub fn new(
        device: &wgpu::Device,
        width:  u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn encode_copy(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source:  &wgpu::Texture,
    ) {
        todo!()
    }

    pub fn read_frame<F>(&self, device: &wgpu::Device, queue: &wgpu::Queue, f: F)
    where
        F: FnOnce(&[u8]),
    {
        todo!()
    }

    pub fn bytes_per_row(&self) -> u32  { self.bytes_per_row }
    pub fn buffer_size(&self)   -> u64  { self.buffer_size   }
    pub fn width(&self)         -> u32  { self.width         }
    pub fn height(&self)        -> u32  { self.height        }
}