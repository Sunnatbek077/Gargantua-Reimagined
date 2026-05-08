// =============================================================================
// crates/gargantua-core/src/gpu/limits.rs
// =============================================================================
//
// PURPOSE:
//   Negotiates the wgpu Features and Limits required by Gargantua's shaders
//   and render pipelines against what the selected GPU adapter actually
//   supports. Called once during GpuContext::new() before device creation.
//
//   The negotiation strategy is: request all desired features, then fall back
//   gracefully for optional ones. Hard-required features (compute shaders,
//   storage textures) cause an error if unsupported. Optional features
//   (f16, timestamp queries) are silently disabled.
//
// SIZE: ~150 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Adapter, Features, Limits, AdapterInfo}
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()  — before device creation
//
// PUBLIC FUNCTIONS:
//
//   pub fn negotiate_limits(
//     adapter: &wgpu::Adapter,
//   ) -> Result<(wgpu::Features, wgpu::Limits), CoreError>
//     — queries adapter.features() and adapter.limits().
//     — builds the Features set:
//
//         REQUIRED (return Err if not supported):
//           TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
//             — needed for Rgba16Float storage texture in ray_march.wgsl
//           BGRA8UNORM_STORAGE
//             — needed for swapchain texture as storage target
//
//         OPTIONAL (enabled if supported, silently skipped otherwise):
//           SHADER_F16
//             — half-precision in WGSL shaders (faster on Apple Silicon)
//           TIMESTAMP_QUERY
//             — GPU timing for GpuProfiler (profiler.rs)
//           TIMESTAMP_QUERY_INSIDE_PASSES
//             — fine-grained per-pass timing
//           PUSH_CONSTANTS
//             — small per-draw constants without a bind group
//           POLYGON_MODE_LINE
//             — wireframe debug rendering
//
//     — builds the Limits struct:
//         max_texture_dimension_2d:        min(adapter, 8192)
//           — 8192 = 8K resolution support
//         max_storage_textures_per_shader_stage: min(adapter, 8)
//         max_compute_workgroup_size_x:    min(adapter, 256)
//         max_compute_workgroup_size_y:    min(adapter, 256)
//         max_compute_invocations_per_workgroup: min(adapter, 1024)
//           — ray_march.wgsl uses @workgroup_size(8, 8, 1) = 64 invocations
//           — well within limits on all supported GPUs
//         max_buffer_size:                 min(adapter, 1 << 30)
//           — 1 GiB max buffer (geodesic LUT can be large)
//         max_bindings_per_bind_group:     min(adapter, 16)
//         max_bind_groups:                 min(adapter, 4)
//           — Gargantua uses bind groups 0,1,2,3 (scene, physics, textures, postfx)
//
//     — returns (features, limits) tuple consumed by DeviceDescriptor.
//     — returns CoreError::InsufficientGpuFeatures with a message listing
//       which required features are missing.
//
//   pub fn log_adapter_info(adapter: &wgpu::Adapter)
//     — logs adapter name, vendor, backend, driver version via tracing::info!
//     — called from GpuContext::new() for diagnostics.
//     — example output:
//         [INFO] GPU: Apple M1 Pro | Metal | driver: 0.2.0
//         [INFO] GPU: NVIDIA RTX 4090 | Dx12 | driver: 537.13
//
// NOTES FOR AI:
//   - Use min(adapter_limit, desired_limit) pattern — never request more
//     than the adapter supports or device creation will fail.
//   - wgpu::Limits::downlevel_defaults() is a safe baseline for WASM/WebGPU.
//     For native builds, use wgpu::Limits::default() as the starting point.
//   - SHADER_F16 requires the "shader-f16" wgpu feature flag in Cargo.toml
//     AND the adapter to support it. On M1 Pro it is always available.
//     On older Windows GPUs it may not be.
//   - The 4 bind group slots (0-3) are a hard architectural constraint:
//       group(0) = scene uniforms (camera, time, physics params)
//       group(1) = baked textures (LUTs, blue noise, starmap)
//       group(2) = frame textures (framebuffer, TAA history, velocity)
//       group(3) = post-fx params (tonemap, bloom, etc.)
//     All WGSL shaders must be written with this layout in mind.
// =============================================================================

use wgpu::{Adapter, Features, Limits};

use crate::errors::CoreError;

pub fn negotiate_limits(
    adapter: &Adapter,
) -> Result<(Features, Limits), CoreError> {
    todo!()
}

pub fn log_adapter_info(adapter: &Adapter) {
    let info = adapter.get_info();
    tracing::info!(
        "GPU: {} | {:?} | driver: {}",
        info.name,
        info.backend,
        info.driver
    );
}