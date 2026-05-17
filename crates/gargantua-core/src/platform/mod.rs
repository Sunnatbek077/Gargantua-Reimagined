// =============================================================================
// FILE: crates/gargantua-core/src/platform/mod.rs
// =============================================================================
//
// PURPOSE:
//   Platform HAL dispatch root. Re-exports macOS or Windows sub-trees per
//   `target_os`. WASM builds omit both; GpuContext uses WebGPU defaults.
//
// SUB-MODULES:
//   platform/macos/   — Metal, EDR, unified memory, Apple Silicon tiers
//   platform/windows/ — DX12/Vulkan, HDR10, VRAM budget, vendor encoders
//
// NOTES FOR AI:
//   Do not put OS-specific logic in this file — only `cfg` module declarations.
// =============================================================================

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;
