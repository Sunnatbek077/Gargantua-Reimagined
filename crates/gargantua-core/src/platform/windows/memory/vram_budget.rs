// =============================================================================
// crates/gargantua-core/src/platform/windows/memory/vram_budget.rs
// =============================================================================
//
// PURPOSE:
//   Tracks VRAM usage and budget on Windows discrete GPU systems using
//   DXGI's QueryVideoMemoryInfo API. Provides accurate VRAM stats and
//   enforces a configurable budget to prevent GPU out-of-memory crashes.
//
//   Unlike macOS unified memory (where RAM budget = GPU budget), Windows
//   discrete GPUs have dedicated VRAM. When VRAM is exhausted, the driver
//   pages textures to system RAM (very slow) or crashes.
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::windows::gpu::vendor::VendorDetails
//     - crate::frame::resource::ResourcePool
//     - crate::errors::CoreError
//   External:
//     - windows_sys::Win32::Graphics::Dxgi::{
//         IDXGIAdapter4, DXGI_QUERY_VIDEO_MEMORY_INFO,
//         DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
//         DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL }
//     - std::sync::Arc
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs::App::new()  — creates at startup
//   - crates/gargantua-core/src/app.rs::App::render_frame()
//       — calls poll() each frame to check budget
//   - crates/gargantua-ui/src/overlay/stats_bar.rs
//       — calls stats() to display VRAM usage
//
// PUBLIC TYPES:
//
//   pub struct VramBudget {
//     local_budget:    u64,    // VRAM budget (OS-assigned, may be < total VRAM)
//     local_usage:     u64,    // current VRAM in use by this process
//     nonlocal_budget: u64,    // system RAM budget for GPU overflow
//     nonlocal_usage:  u64,    // system RAM currently used as VRAM overflow
//     total_vram:      u64,    // physical VRAM (from DXGI_ADAPTER_DESC3.DedicatedVideoMemory)
//   }
//
//   pub struct VramStats {
//     pub total_mb:       f32,
//     pub budget_mb:      f32,   // OS-assigned budget (may be < total)
//     pub used_mb:        f32,
//     pub free_mb:        f32,
//     pub overflow_mb:    f32,   // amount using system RAM as overflow
//     pub pressure:       VramPressure,
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum VramPressure {
//     Normal,    // usage < 80% of budget
//     Warning,   // usage 80-95% of budget
//     Critical,  // usage > 95% of budget (imminent overflow/crash)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(adapter: &wgpu::Adapter) -> Result<Self, CoreError>
//     — queries IDXGIAdapter4::QueryVideoMemoryInfo for:
//         DXGI_MEMORY_SEGMENT_GROUP_LOCAL    (dedicated VRAM)
//         DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL (system RAM overflow)
//     — stores Budget and CurrentUsage from DXGI_QUERY_VIDEO_MEMORY_INFO.
//     — queries total VRAM from DXGI_ADAPTER_DESC3.DedicatedVideoMemory.
//     — accesses IDXGIAdapter4 via wgpu DX12 HAL:
//         unsafe { adapter.as_hal::<wgpu::hal::api::Dx12, _, _>(|dx12| { ... }) }
//
//   pub fn poll(&mut self, adapter: &wgpu::Adapter) -> VramPressure
//     — re-queries QueryVideoMemoryInfo (cheap DXGI call, ~microseconds).
//     — updates local_budget, local_usage, nonlocal_budget, nonlocal_usage.
//     — returns the current VramPressure level.
//     — called every frame by App::render_frame().
//
//   pub fn stats(&self) -> VramStats
//     — returns VramStats struct for the UI overlay.
//
//   pub fn pressure(&self) -> VramPressure
//     — returns pressure based on local_usage / local_budget:
//         < 80%:  Normal
//         80-95%: Warning
//         > 95%:  Critical
//
//   pub fn should_evict(&self) -> bool
//     — returns true if pressure is Warning or Critical.
//     — called by frame_graph.rs to trigger ResourcePool::release_all_transient().
//
// NOTES FOR AI:
//   - DXGI_QUERY_VIDEO_MEMORY_INFO.Budget is the OS-assigned budget, NOT total VRAM.
//     On a 12GB GPU, Budget might be 10GB if the OS reserves some for other processes.
//     Always use Budget as the ceiling, not the physical total.
//   - NonLocal segment (system RAM) usage > 0 means the GPU is paging textures
//     to system RAM. This is a severe performance warning — reduce quality immediately.
//   - IDXGIAdapter4::QueryVideoMemoryInfo requires IDXGIAdapter4 (DXGI 1.4+).
//     This is available on Windows 10 1607+ with WDDM 2.1. Safe assumption.
//   - On NVIDIA: NVML (NVIDIA Management Library) provides more detailed stats
//     but requires an additional SDK. DXGI is sufficient for budget enforcement.
//   - poll() is cheap enough to call every frame. The OS updates these values
//     in real-time as allocations and deallocations occur.
// =============================================================================

#![cfg(target_os = "windows")]

use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VramPressure { Normal, Warning, Critical }

pub struct VramStats {
    pub total_mb:   f32,
    pub budget_mb:  f32,
    pub used_mb:    f32,
    pub free_mb:    f32,
    pub overflow_mb: f32,
    pub pressure:   VramPressure,
}

pub struct VramBudget {
    local_budget:    u64,
    local_usage:     u64,
    nonlocal_budget: u64,
    nonlocal_usage:  u64,
    total_vram:      u64,
}

impl VramBudget {
    pub fn new(adapter: &wgpu::Adapter) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn poll(&mut self, adapter: &wgpu::Adapter) -> VramPressure {
        todo!()
    }

    pub fn stats(&self) -> VramStats {
        let mb = |b: u64| b as f32 / (1024.0 * 1024.0);
        VramStats {
            total_mb:    mb(self.total_vram),
            budget_mb:   mb(self.local_budget),
            used_mb:     mb(self.local_usage),
            free_mb:     mb(self.local_budget.saturating_sub(self.local_usage)),
            overflow_mb: mb(self.nonlocal_usage),
            pressure:    self.pressure(),
        }
    }

    pub fn pressure(&self) -> VramPressure {
        if self.local_budget == 0 { return VramPressure::Normal; }
        let ratio = self.local_usage as f64 / self.local_budget as f64;
        if ratio > 0.95      { VramPressure::Critical }
        else if ratio > 0.80 { VramPressure::Warning  }
        else                 { VramPressure::Normal    }
    }

    pub fn should_evict(&self) -> bool {
        self.pressure() != VramPressure::Normal
    }
}