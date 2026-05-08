// =============================================================================
// crates/gargantua-core/src/gpu/surface.rs
// =============================================================================
//
// PURPOSE:
//   Manages the wgpu Surface (the OS window's drawable surface), the
//   swap chain configuration, and surface resize events.
//
//   Provides the current frame's TextureView that the final tonemapping
//   pass renders into. Handles format negotiation (preferring Rgba16Float
//   for HDR, falling back to Bgra8Unorm for SDR).
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::context::GpuContext          — device + adapter
//     - crate::render::hdr::HdrSwapchain         — HDR surface wrapper (gargantua-render)
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Surface, SurfaceConfiguration, TextureFormat, TextureView,
//              PresentMode, CompositeAlphaMode, SurfaceTexture}
//     - winit::dpi::PhysicalSize
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()     — creates GpuSurface
//   - crate::app::App::handle_resize()           — calls GpuSurface::resize()
//   - crate::app::App::render_frame()            — calls acquire_frame() each frame
//
// PUBLIC TYPES:
//
//   pub struct GpuSurface {
//     surface:    wgpu::Surface<'static>,
//     config:     wgpu::SurfaceConfiguration,
//     format:     wgpu::TextureFormat,    // actual format in use
//     hdr:        bool,                   // true if Rgba16Float is active
//     width:      u32,
//     height:     u32,
//   }
//
//   pub struct FrameOutput {
//     pub texture: wgpu::SurfaceTexture,
//     pub view:    wgpu::TextureView,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     surface:  wgpu::Surface<'static>,
//     adapter:  &wgpu::Adapter,
//     device:   &wgpu::Device,
//     size:     winit::dpi::PhysicalSize<u32>,
//   ) -> Result<Self, CoreError>
//     — queries adapter.get_texture_format_features() for the surface.
//     — preferred format priority:
//         1. wgpu::TextureFormat::Rgba16Float  (HDR — Mac EDR / Win HDR10)
//         2. wgpu::TextureFormat::Bgra8UnormSrgb (SDR sRGB)
//         3. wgpu::TextureFormat::Bgra8Unorm   (SDR linear, fallback)
//     — configures the surface with:
//         present_mode:    PresentMode::Mailbox  (low latency, no tearing)
//                          fallback: PresentMode::Fifo (VSync, always supported)
//         alpha_mode:      CompositeAlphaMode::Opaque
//         usage:           TextureUsages::RENDER_ATTACHMENT
//     — calls surface.configure(device, &config).
//     — sets self.hdr = true if Rgba16Float was selected.
//
//   pub fn acquire_frame(&self) -> Result<FrameOutput, CoreError>
//     — calls surface.get_current_texture().
//     — creates a TextureView from the SurfaceTexture.
//     — returns FrameOutput { texture, view }.
//     — returns CoreError::SurfaceLost if the surface is lost (window minimized).
//       Caller (App::render_frame) should call resize() and retry.
//
//   pub fn resize(
//     &mut self,
//     device: &wgpu::Device,
//     new_size: winit::dpi::PhysicalSize<u32>,
//   )
//     — updates self.config.width and .height.
//     — calls surface.configure(device, &self.config) to recreate the swap chain.
//     — must be called when winit emits WindowEvent::Resized or
//       WindowEvent::ScaleFactorChanged.
//     — silently ignores resize to (0, 0) (minimized window).
//
//   pub fn format(&self) -> wgpu::TextureFormat { self.format }
//   pub fn is_hdr(&self) -> bool { self.hdr }
//   pub fn width(&self) -> u32  { self.width }
//   pub fn height(&self) -> u32 { self.height }
//
// NOTES FOR AI:
//   - On macOS with EDR (Extended Dynamic Range), the surface format
//     Rgba16Float enables pixel values > 1.0 which map to brightness
//     above SDR white. This is handled by edr.rs in platform/macos/hdr/.
//   - PresentMode::Mailbox is preferred over Fifo for the real-time viewer
//     to minimize input latency. For the offline renderer (gargantua-video),
//     the surface is not used at all — frames are written directly to a buffer.
//   - SurfaceTexture must be presented (texture.present()) after encoding,
//     or the frame will not appear. This is called in App::render_frame()
//     after queue.submit().
//   - wgpu::Surface has a lifetime tied to the window. Use 'static by
//     transmuting or using Arc<Window> to ensure the window outlives the surface.
// =============================================================================

use winit::dpi::PhysicalSize;
use crate::errors::CoreError;

pub struct FrameOutput {
    pub texture: wgpu::SurfaceTexture,
    pub view:    wgpu::TextureView,
}

pub struct GpuSurface {
    surface: wgpu::Surface<'static>,
    config:  wgpu::SurfaceConfiguration,
    format:  wgpu::TextureFormat,
    hdr:     bool,
    width:   u32,
    height:  u32,
}

impl GpuSurface {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device:  &wgpu::Device,
        size:    PhysicalSize<u32>,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn acquire_frame(&self) -> Result<FrameOutput, CoreError> {
        todo!()
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_size: PhysicalSize<u32>) {
        todo!()
    }

    pub fn format(&self) -> wgpu::TextureFormat { self.format }
    pub fn is_hdr(&self)  -> bool               { self.hdr    }
    pub fn width(&self)   -> u32                { self.width  }
    pub fn height(&self)  -> u32                { self.height }
}