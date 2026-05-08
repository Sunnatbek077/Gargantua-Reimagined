// =============================================================================
// crates/gargantua-core/src/gpu/context.rs
// =============================================================================
//
// PURPOSE:
//   Creates and owns the core wgpu objects: Instance, Adapter, Device, and
//   Queue. These are the fundamental GPU handles that every other system
//   in the engine depends on.
//
//   On macOS: selects the Metal backend via the platform adapter
//             (platform/macos/gpu/adapter_metal.rs).
//   On Windows: selects DX12 preferring the high-performance GPU
//              (platform/windows/gpu/adapter_dx12.rs), with Vulkan fallback.
//   On WASM: uses the WebGPU backend (navigator.gpu).
//
//   Also negotiates WebGPU feature flags and limits via limits.rs to ensure
//   the device is created with the correct capabilities for Gargantua's
//   shader requirements (compute shaders, f16, timestamp queries, etc.).
//
// SIZE: ~280 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::limits::negotiate_limits   — feature/limit negotiation
//     - crate::gpu::surface::GpuSurface        — surface creation (uses context)
//     - crate::gpu::profiler::GpuProfiler      — timestamp queries (uses device)
//     - crate::errors::CoreError
//     - crate::platform::mod::Platform         — current platform enum
//     #[cfg(target_os = "macos")]
//     - crate::platform::macos::gpu::adapter_metal::create_metal_adapter
//     #[cfg(target_os = "windows")]
//     - crate::platform::windows::gpu::adapter_dx12::create_dx12_adapter
//     - crate::platform::windows::gpu::adapter_vulkan::create_vulkan_adapter
//   External:
//     - wgpu::{Instance, Adapter, Device, Queue, DeviceDescriptor,
//              RequestAdapterOptions, PowerPreference, Backends,
//              Features, Limits}
//     - std::sync::Arc
//
// CALLED BY:
//   - crate::app::App::new()  — creates GpuContext at application startup
//   - crate::gpu::surface::GpuSurface::new()  — needs the adapter for surface compat
//
// PUBLIC TYPES:
//
//   pub struct GpuContext {
//     pub instance: wgpu::Instance,
//     pub adapter:  wgpu::Adapter,
//     pub device:   Arc<wgpu::Device>,
//     pub queue:    Arc<wgpu::Queue>,
//     pub features: wgpu::Features,   // actually enabled features
//     pub limits:   wgpu::Limits,     // actually negotiated limits
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub async fn new(
//     window: &winit::window::Window,
//   ) -> Result<(Self, GpuSurface), CoreError>
//     — async because wgpu adapter/device request is async.
//     — steps:
//         1. wgpu::Instance::new() with backend selected per platform:
//              macOS   → Backends::METAL
//              Windows → Backends::DX12 | Backends::VULKAN
//              WASM    → Backends::BROWSER_WEBGPU
//         2. create_surface(window) — creates the raw wgpu Surface
//         3. platform-specific adapter selection:
//              macOS   → adapter_metal::create_metal_adapter(&instance, &surface)
//              Windows → adapter_dx12::create_dx12_adapter(&instance, &surface)
//                        falls back to create_vulkan_adapter() on failure
//              WASM    → instance.request_adapter() with default options
//         4. limits::negotiate_limits(&adapter) — compute required features/limits
//         5. adapter.request_device(&DeviceDescriptor { features, limits, .. })
//         6. wrap device and queue in Arc<> for shared ownership
//         7. construct GpuSurface from the raw surface + adapter + device
//         8. return (GpuContext, GpuSurface)
//     — returns CoreError::NoSuitableAdapter if no GPU is found.
//     — returns CoreError::DeviceCreationFailed if device request fails.
//
//   pub fn device(&self) -> &Arc<wgpu::Device>
//     — accessor for the Arc<Device>. Used by ResourcePool, pipelines, etc.
//
//   pub fn queue(&self) -> &Arc<wgpu::Queue>
//     — accessor for the Arc<Queue>.
//
//   pub fn supports_f16(&self) -> bool
//     — returns true if wgpu::Features::SHADER_F16 is in self.features.
//     — used by render pipelines to decide whether to use f16 in shaders.
//
//   pub fn supports_timestamp_queries(&self) -> bool
//     — returns true if TIMESTAMP_QUERY is enabled.
//     — used by GpuProfiler to decide whether to record GPU timings.
//
// NOTES FOR AI:
//   - GpuContext is created once at startup and lives for the entire
//     application lifetime. Do not drop and recreate it.
//   - Arc<Device> and Arc<Queue> are cloned into ResourcePool, all pipelines,
//     and GpuProfiler. This is the standard wgpu ownership pattern.
//   - The async fn is driven by winit's event loop on native, and by
//     wasm_bindgen_futures::spawn_local on WASM.
//   - On macOS, the Metal surface is created via CAMetalLayer in
//     adapter_metal.rs — do not use wgpu's default surface creation on Mac.
//   - Error type CoreError must implement From<wgpu::RequestDeviceError>
//     for the ? operator to work in the device request step.
// =============================================================================

use std::sync::Arc;

use wgpu::{Adapter, Backends, Device, DeviceDescriptor, Features, Instance, Limits, Queue};

use crate::{
    errors::CoreError,
    gpu::{limits::negotiate_limits, surface::GpuSurface},
};

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter:  wgpu::Adapter,
    pub device:   Arc<wgpu::Device>,
    pub queue:    Arc<wgpu::Queue>,
    pub features: wgpu::Features,
    pub limits:   wgpu::Limits,
}

impl GpuContext {
    pub async fn new(
        window: &winit::window::Window,
    ) -> Result<(Self, GpuSurface), CoreError> {
        todo!()
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    pub fn supports_f16(&self) -> bool {
        self.features.contains(wgpu::Features::SHADER_F16)
    }

    pub fn supports_timestamp_queries(&self) -> bool {
        self.features.contains(wgpu::Features::TIMESTAMP_QUERY)
    }
}