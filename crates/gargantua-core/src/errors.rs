// =============================================================================
// crates/gargantua-core/src/errors.rs
// =============================================================================
//
// PURPOSE:
//   Defines CoreError — the unified error type for the entire gargantua-core
//   crate. All public functions that can fail return Result<T, CoreError>.
//   Uses thiserror for ergonomic #[derive(Error)] derives.
//
//   Downstream crates (gargantua-render, gargantua-physics, gargantua-app)
//   may define their own error types that wrap CoreError via #[from].
//
// SIZE: ~120 lines
//
// DEPENDENCIES:
//   External:
//     - thiserror::Error    — #[derive(Error)] for Display + Error impls
//     - wgpu::RequestDeviceError
//     - wgpu::SurfaceError
//
// CALLED BY:
//   - Every module in gargantua-core that returns Result<T, CoreError>
//   - crates/gargantua-app/src/errors.rs — wraps CoreError in AppError
//   - crates/gargantua-render/src/errors.rs — wraps CoreError in RenderError
//
// PUBLIC TYPES:
//
//   #[derive(Debug, thiserror::Error)]
//   pub enum CoreError {
//
//     #[error("No suitable GPU adapter found: {0}")]
//     NoSuitableAdapter(String),
//       — returned by adapter_metal.rs, adapter_dx12.rs, adapter_vulkan.rs
//         when no compatible GPU is found on the system.
//       — String contains a human-readable description of what was tried.
//
//     #[error("GPU device creation failed: {0}")]
//     DeviceCreationFailed(#[from] wgpu::RequestDeviceError),
//       — returned by GpuContext::new() if adapter.request_device() fails.
//       — #[from] enables ? operator from wgpu::RequestDeviceError.
//
//     #[error("GPU surface lost")]
//     SurfaceLost,
//       — returned by GpuSurface::acquire_frame() when the swapchain is lost.
//       — App::render_frame() catches this and calls handle_resize().
//       — recoverable: App should resize and retry.
//
//     #[error("GPU surface outdated")]
//     SurfaceOutdated,
//       — returned when the swapchain is outdated (window resized before
//         the app processed the resize event). Same recovery as SurfaceLost.
//
//     #[error("GPU surface timeout")]
//     SurfaceTimeout,
//       — returned by acquire_frame() when waiting for the next swapchain
//         image times out. Rare on desktop. Retry next frame.
//
//     #[error("Frame graph cycle detected")]
//     CyclicDependency,
//       — returned by FrameGraph::topological_sort() if a dependency cycle
//         exists in the registered render passes. Indicates a programming error.
//
//     #[error("GPU out of memory")]
//     OutOfMemory,
//       — returned by ResourcePool, UnifiedAllocator, VramBudget when
//         allocation would exceed budget or GPU signals OOM.
//
//     #[error("Insufficient GPU features: {0}")]
//     InsufficientGpuFeatures(String),
//       — returned by limits::negotiate_limits() if required GPU features
//         (compute shaders, storage textures) are not supported.
//       — String lists the missing features.
//
//     #[error("Platform error: {0}")]
//     PlatformError(String),
//       — returned by macOS/Windows platform code for OS API failures:
//         sysctl failure, dispatch_source_create failure, DXGI error.
//
//     #[error("CoreML model load failed: {path}")]
//     CoreMLLoadFailed { path: String },
//       — returned by neural_engine.rs if the .mlpackage file is missing
//         or fails to compile.
//
//     #[error("Shader compilation failed: {shader}: {message}")]
//     ShaderCompilationFailed { shader: String, message: String },
//       — returned if wgpu reports a pipeline creation error (WGSL syntax
//         error that slipped past naga validation in CI).
//
//     #[error("Encoder initialization failed: {0}")]
//     EncoderInitFailed(String),
//       — returned by nvenc.rs, amf.rs, qsv.rs if hardware encoder fails
//         to initialize. App falls back to software.rs.
//
//     #[error("IO error: {0}")]
//     Io(#[from] std::io::Error),
//       — returned by any file I/O operation (LUT loading, video output).
//
//     #[error("Other error: {0}")]
//     Other(String),
//       — catch-all for one-off errors. Prefer specific variants above.
//   }
//
// IMPL NOTES:
//
//   impl From<wgpu::SurfaceError> for CoreError
//     — maps wgpu::SurfaceError variants to CoreError:
//         SurfaceError::Lost      → CoreError::SurfaceLost
//         SurfaceError::Outdated  → CoreError::SurfaceOutdated
//         SurfaceError::Timeout   → CoreError::SurfaceTimeout
//         SurfaceError::OutOfMemory → CoreError::OutOfMemory
//
// NOTES FOR AI:
//   - CoreError must implement both Debug and Display (via thiserror).
//   - All CoreError variants must be Send + Sync (required by anyhow and
//     tokio/async contexts). thiserror derives ensure this.
//   - Use the most specific variant possible. Other(String) is a last resort.
//   - SurfaceLost and SurfaceOutdated are NOT fatal — handle in App::render_frame().
//   - EncoderInitFailed is NOT fatal — fall back to SoftwareEncoder.
//   - OutOfMemory IS fatal on macOS if UnifiedAllocator cannot evict enough.
//     On Windows, wgpu returns it from device.create_texture() which panics
//     by default — catch with std::panic::catch_unwind if needed.
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("No suitable GPU adapter found: {0}")]
    NoSuitableAdapter(String),

    #[error("GPU device creation failed: {0}")]
    DeviceCreationFailed(#[from] wgpu::RequestDeviceError),

    #[error("GPU surface lost")]
    SurfaceLost,

    #[error("GPU surface outdated")]
    SurfaceOutdated,

    #[error("GPU surface timeout")]
    SurfaceTimeout,

    #[error("Frame graph cycle detected")]
    CyclicDependency,

    #[error("GPU out of memory")]
    OutOfMemory,

    #[error("Insufficient GPU features: {0}")]
    InsufficientGpuFeatures(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("CoreML model load failed: {path}")]
    CoreMLLoadFailed { path: String },

    #[error("Shader compilation failed: {shader}: {message}")]
    ShaderCompilationFailed { shader: String, message: String },

    #[error("Encoder initialization failed: {0}")]
    EncoderInitFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<wgpu::SurfaceError> for CoreError {
    fn from(e: wgpu::SurfaceError) -> Self {
        match e {
            wgpu::SurfaceError::Lost      => CoreError::SurfaceLost,
            wgpu::SurfaceError::Outdated  => CoreError::SurfaceOutdated,
            wgpu::SurfaceError::Timeout   => CoreError::SurfaceTimeout,
            wgpu::SurfaceError::OutOfMemory => CoreError::OutOfMemory,
        }
    }
}