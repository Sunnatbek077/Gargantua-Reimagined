// =============================================================================
// FILE: crates/gargantua-core/src/platform/windows/mod.rs
// =============================================================================
//
// PURPOSE:
//   Root module for all Windows-specific platform code. Re-exports
//   sub-modules and provides the top-level WindowsPlatform struct that
//   aggregates all Windows-specific subsystems into one owned handle.
//
//   Used by GpuContext::new() and App::new() to initialize the Windows
//   platform layer before any rendering begins.
//
// SIZE: ~80 lines
//
// SUB-MODULES:
//   compute/
//     shared_mem.rs    — ReBAR detection and direct GPU buffer write
//     workgroup.rs     — optimal compute workgroup dimensions per vendor
//   gpu/
//     adapter_dx12.rs  — DX12 adapter creation and feature detection
//     adapter_vulkan.rs — Vulkan adapter fallback
//     vendor.rs        — GPU vendor/architecture detection (NVIDIA/AMD/Intel)
//   hdr/
//     display_detect.rs — HDR display enumeration and mode detection
//     hdr10.rs          — HDR10 (PQ + BT.2020) output and metadata
//     dolby_vision.rs   — Dolby Vision output (DV over HDMI 2.1)
//   memory/
//     staging_pool.rs  — reusable CPU staging buffer pool
//     upload_heap.rs   — unified CPU→GPU upload abstraction
//     read_heap.rs     — GPU→CPU readback for video capture
//     vram_budget.rs   — DXGI VRAM budget tracking and pressure detection
//   quality/
//     nvidia_presets.rs — render quality presets for NVIDIA GPU tiers
//     amd_presets.rs    — render quality presets for AMD GPU tiers
//     intel_presets.rs  — render quality presets for Intel Arc GPU tiers
//   video/
//     nvenc.rs          — NVIDIA NVENC hardware video encoder (H.264/HEVC/AV1)
//     amf.rs            — AMD AMF hardware video encoder (H.264/HEVC/AV1)
//     qsv.rs            — Intel QuickSync Video encoder (H.264/HEVC/AV1)
//     software.rs       — CPU software encoder fallback (libx264/libx265)
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::context::GpuContext
//     - crate::errors::CoreError
//   External: none (sub-modules carry their own dependencies)
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()  — initializes Windows platform
//   - crates/gargantua-core/src/app.rs — App::new() on Windows builds
//
// PUBLIC TYPES:
//
//   pub struct WindowsPlatform {
//     pub vendor:   gpu::vendor::VendorDetails,
//     pub shared:   compute::shared_mem::SharedMem,
//     pub upload:   memory::upload_heap::UploadHeap,
//     pub readback: memory::read_heap::ReadHeap,   // initialized lazily
//     pub vram:     memory::vram_budget::VramBudget,
//     pub hdr_mode: hdr::display_detect::HdrMode,
//   }
//
// NOTES FOR AI:
//   - All sub-modules are gated with #[cfg(target_os = "windows")].
//   - This mod.rs is also gated — the entire windows/ tree is excluded
//     from macOS and WASM builds.
//   - WindowsPlatform is created once at startup and stored in App.
//     Its fields are accessed by render passes and the video encoder
//     through &App references.
// =============================================================================

#![cfg(target_os = "windows")]

pub mod compute {
    pub mod shared_mem;
    pub mod workgroup;
}

pub mod gpu {
    pub mod adapter_dx12;
    pub mod adapter_vulkan;
    pub mod vendor;
}

pub mod hdr {
    pub mod display_detect;
    pub mod dolby_vision;
    pub mod hdr10;
}

pub mod memory {
    pub mod read_heap;
    pub mod staging_pool;
    pub mod upload_heap;
    pub mod vram_budget;
}

pub mod quality {
    pub mod amd_presets;
    pub mod intel_presets;
    pub mod nvidia_presets;
}

pub mod video {
    pub mod amf;
    pub mod nvenc;
    pub mod qsv;
    pub mod software;
}

use crate::errors::CoreError;

pub struct WindowsPlatform {
    pub vendor:   gpu::vendor::VendorDetails,
    pub shared:   compute::shared_mem::SharedMem,
    pub upload:   memory::upload_heap::UploadHeap,
    pub vram:     memory::vram_budget::VramBudget,
    pub hdr_mode: hdr::display_detect::HdrMode,
}