// =============================================================================
// crates/gargantua-core/src/platform/macos/memory/zero_copy_readback.rs
// =============================================================================
//
// PURPOSE:
//   Implements zero-copy GPU→CPU readback of rendered frames using Apple
//   Silicon's unified memory architecture. Since GPU and CPU share the same
//   physical RAM, a Metal StorageMode::Shared buffer can be written by the
//   GPU and read by the CPU without any copy — unlike discrete GPU systems
//   where readback requires an explicit PCIe DMA transfer.
//
//   Used by gargantua-video for frame capture during offline rendering:
//   the GPU renders into a shared buffer, the CPU reads it directly to
//   encode the video frame, with zero memcpy overhead.
//
//   Also used by GpuProfiler to read timestamp query results back to CPU.
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::macos::gpu::chip_detect::is_apple_silicon
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Device, Buffer, BufferDescriptor, BufferUsages,
//              CommandEncoder, ImageCopyBuffer, ImageCopyTexture,
//              ImageDataLayout, Extent3d, Maintain}
//     - std::sync::Arc
//
// CALLED BY:
//   - crates/gargantua-video/src/capture/frame_capture.rs
//       — creates ZeroCopyReadback for the offline render pipeline
//   - crates/gargantua-core/src/gpu/profiler.rs
//       — uses zero-copy for timestamp readback (faster than map + unmap)
//
// PUBLIC TYPES:
//
//   pub struct ZeroCopyReadback {
//     shared_buffer: wgpu::Buffer,   // StorageMode::Shared (Metal) / MAP_READ (wgpu)
//     buffer_size:   u64,            // bytes: width * height * bytes_per_pixel
//     width:         u32,
//     height:        u32,
//     format:        wgpu::TextureFormat,
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
//     — calculates buffer_size = width * height * bytes_per_texel(format).
//       Rounds up to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT per row.
//     — creates the shared buffer with:
//         BufferUsages::COPY_DST | BufferUsages::MAP_READ
//         mapped_at_creation: false
//     — on Apple Silicon with Metal backend, wgpu creates this buffer
//       with MTLStorageMode::Shared automatically for MAP_READ buffers.
//       No explicit Metal HAL access needed.
//     — returns CoreError::OutOfMemory if allocation fails.
//
//   pub fn encode_copy(
//     &self,
//     encoder: &mut wgpu::CommandEncoder,
//     source:  &wgpu::Texture,
//   )
//     — encodes a copy from source texture → self.shared_buffer.
//     — uses encoder.copy_texture_to_buffer() with correct ImageDataLayout:
//         bytes_per_row: Some(aligned_bytes_per_row)
//         rows_per_image: Some(height)
//     — call this inside the frame's CommandEncoder before submit().
//
//   pub fn read_frame<F>(&self, device: &wgpu::Device, queue: &wgpu::Queue, f: F)
//     where F: FnOnce(&[u8])
//     — maps the shared buffer for reading (async → sync via poll).
//     — on Apple Silicon, this is zero-copy: the CPU reads directly from
//       the same physical memory the GPU wrote to.
//     — calls device.poll(Maintain::Wait) to ensure GPU has finished writing.
//     — maps the buffer, calls f(&data), then unmaps.
//     — the slice passed to f is width * height * bytes_per_texel bytes.
//     — after f returns, the buffer is unmapped and ready for the next frame.
//
//   pub fn bytes_per_row(&self) -> u32
//     — returns the aligned bytes per row (may be larger than width * bpp).
//     — used by video encoders (gargantua-video) to correctly interpret
//       the row stride when encoding pixels into a video frame.
//
//   pub fn buffer_size(&self) -> u64 { self.buffer_size }
//   pub fn width(&self)       -> u32 { self.width }
//   pub fn height(&self)      -> u32 { self.height }
//
// NOTES FOR AI:
//   - On Apple Silicon, MTLStorageMode::Shared is the key to zero-copy.
//     wgpu automatically uses Shared for MAP_READ buffers on Metal backend.
//     On discrete GPU (non-Apple-Silicon), this falls back to a regular
//     PCIe readback with an extra copy — still correct, just slower.
//   - bytes_per_row MUST be a multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
//     (256 bytes). If width * bpp is not a multiple of 256, pad it:
//       aligned = (width * bpp + 255) & !255
//   - device.poll(Maintain::Wait) blocks the CPU until the GPU is done.
//     For real-time capture, this adds 1-2ms latency. For offline rendering,
//     this is acceptable. For real-time display, never use MAP_READ on the
//     display path — only use it for video capture and profiler readback.
//   - The closure pattern (FnOnce(&[u8])) ensures the buffer is always
//     unmapped after use, preventing the common bug of forgetting to unmap.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::errors::CoreError;

pub struct ZeroCopyReadback {
    shared_buffer: wgpu::Buffer,
    buffer_size:   u64,
    width:         u32,
    height:        u32,
    format:        wgpu::TextureFormat,
}

impl ZeroCopyReadback {
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

    pub fn bytes_per_row(&self) -> u32 {
        let bpp = bytes_per_texel(self.format);
        let raw = self.width * bpp;
        (raw + wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1)
            & !(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1)
    }

    pub fn buffer_size(&self) -> u64 { self.buffer_size }
    pub fn width(&self)       -> u32 { self.width       }
    pub fn height(&self)      -> u32 { self.height      }
}

fn bytes_per_texel(format: wgpu::TextureFormat) -> u32 {
    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
        wgpu::TextureFormat::Rgba16Float       => 8,
        wgpu::TextureFormat::Rgba32Float       => 16,
        _ => 4, // safe default
    }
}