// =============================================================================
// crates/gargantua-core/src/platform/macos/gpu/adapter_metal.rs
// =============================================================================
//
// PURPOSE:
//   Creates a wgpu Adapter specifically for the Metal backend on macOS.
//   Bypasses wgpu's default adapter selection to ensure the discrete GPU
//   is selected on Macs with both integrated and discrete GPUs (e.g., older
//   MacBook Pros), and to set up the CAMetalLayer correctly for HDR output.
//
//   On Apple Silicon (M1-M5), there is only one GPU — this module still
//   runs to configure the CAMetalLayer for EDR (Extended Dynamic Range)
//   support and to set the correct colorspace.
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::chip_detect::ChipInfo              — detect Apple Silicon tier
//     - super::super::hdr::edr::configure_edr_layer  — EDR Metal layer setup
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Instance, Adapter, RequestAdapterOptions, PowerPreference,
//              Surface, Backends}
//     - objc2::runtime::Object
//     - objc2_foundation::NSString
//     - metal (wgpu's metal backend types, accessed via wgpu::hal::metal)
//     - raw_window_handle::{HasWindowHandle, HasDisplayHandle}
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()  — macOS branch of adapter creation
//
// PUBLIC FUNCTIONS:
//
//   pub async fn create_metal_adapter(
//     instance: &wgpu::Instance,
//     surface:  &wgpu::Surface<'_>,
//   ) -> Result<wgpu::Adapter, CoreError>
//     — requests an adapter from wgpu with:
//         power_preference: PowerPreference::HighPerformance
//         compatible_surface: Some(surface)
//         force_fallback_adapter: false
//     — on Apple Silicon: the only adapter is the unified GPU — select it.
//     — on Intel Mac with discrete GPU: HighPerformance selects the dGPU.
//     — after adapter selection, calls configure_metal_layer(surface, &adapter)
//       to set up CAMetalLayer properties:
//         pixelFormat     = MTLPixelFormatRGBA16Float (for EDR)
//         colorspace      = kCGColorSpaceExtendedLinearDisplayP3
//         wantsExtendedDynamicRangeContent = YES (macOS 10.11+)
//     — returns CoreError::NoSuitableAdapter if Metal is unavailable.
//
//   pub fn configure_metal_layer(
//     surface: &wgpu::Surface<'_>,
//     adapter: &wgpu::Adapter,
//   ) -> Result<(), CoreError>
//     — accesses the underlying CAMetalLayer via wgpu's unsafe Metal HAL API:
//         unsafe { adapter.as_hal::<wgpu::hal::api::Metal, _, _>(|metal_adapter| { ... }) }
//     — sets CAMetalLayer.pixelFormat = MTLPixelFormatRGBA16Float
//     — sets CAMetalLayer.colorspace to extended P3 (EDR colorspace)
//     — sets CAMetalLayer.wantsExtendedDynamicRangeContent = true
//     — these settings enable pixel values > 1.0 (EDR white above SDR white)
//       which Gargantua uses for the bright accretion disk emission.
//     — safe to call on non-EDR displays — values > 1.0 are clamped.
//
//   pub fn metal_device_name(adapter: &wgpu::Adapter) -> String
//     — extracts the MTLDevice.name string via the Metal HAL.
//     — returns e.g. "Apple M1 Pro" or "AMD Radeon Pro 5500M".
//     — used by GpuContext for logging.
//
// NOTES FOR AI:
//   - All wgpu HAL access is unsafe. Every unsafe block must have a
//     // SAFETY: comment.
//   - CAMetalLayer configuration must happen BEFORE the first frame is
//     rendered. Call configure_metal_layer() inside GpuContext::new(),
//     not lazily.
//   - EDR (Extended Dynamic Range) on macOS requires:
//       1. MTLPixelFormatRGBA16Float pixel format
//       2. Extended P3 colorspace on the CAMetalLayer
//       3. wantsExtendedDynamicRangeContent = YES
//       4. The display must support EDR (Pro Display XDR, M1 Pro/Max screens)
//   - On non-EDR displays, the RGBA16Float framebuffer still works but
//     values > 1.0 are clamped at the display compositor level.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::errors::CoreError;

pub async fn create_metal_adapter(
    instance: &wgpu::Instance,
    surface:  &wgpu::Surface<'_>,
) -> Result<wgpu::Adapter, CoreError> {
    todo!()
}

pub fn configure_metal_layer(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
) -> Result<(), CoreError> {
    todo!()
}

pub fn metal_device_name(adapter: &wgpu::Adapter) -> String {
    todo!()
}