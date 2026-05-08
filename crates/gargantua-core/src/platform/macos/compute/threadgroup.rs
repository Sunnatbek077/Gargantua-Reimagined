// =============================================================================
// crates/gargantua-core/src/platform/macos/compute/threadgroup.rs
// =============================================================================
//
// PURPOSE:
//   Computes optimal Metal threadgroup (workgroup) dimensions for 2D compute
//   dispatch given an image resolution and the chip's SIMD group config.
//
//   Metal performance is sensitive to threadgroup sizing: too small wastes
//   SIMD lanes, too large exceeds the per-threadgroup shared memory limit.
//   This module picks the best dimensions for each chip tier and resolution.
//
// SIZE: ~140 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::simd_group::{SimdGroupConfig, simd_config_for_chip}
//     - super::super::quality::ChipTier
//   External: none
//
// CALLED BY:
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — calls compute_dispatch() to get (groups_x, groups_y) for dispatch
//   - crates/gargantua-render/src/pipelines/geodesic.rs
//       — same
//   - crates/gargantua-bake/src/noise/baker.rs
//       — same pattern for the blue noise bake compute pass
//
// PUBLIC TYPES:
//
//   pub struct DispatchConfig {
//     pub workgroup_x:  u32,   // threads per threadgroup in X
//     pub workgroup_y:  u32,   // threads per threadgroup in Y
//     pub dispatch_x:   u32,   // number of threadgroups to dispatch in X
//     pub dispatch_y:   u32,   // number of threadgroups to dispatch in Y
//     pub padded_width: u32,   // width rounded up to workgroup_x multiple
//     pub padded_height:u32,   // height rounded up to workgroup_y multiple
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn compute_dispatch(
//     width:  u32,
//     height: u32,
//     tier:   ChipTier,
//   ) -> DispatchConfig
//     — gets SimdGroupConfig for the chip via simd_config_for_chip(tier).
//     — workgroup_x and workgroup_y come from the config.
//     — dispatch_x  = width.div_ceil(workgroup_x)
//     — dispatch_y  = height.div_ceil(workgroup_y)
//     — padded_width  = dispatch_x * workgroup_x
//     — padded_height = dispatch_y * workgroup_y
//     — WGSL shaders must guard against out-of-bounds using:
//         if pos.x >= uniforms.width || pos.y >= uniforms.height { return; }
//
//   pub fn threadgroup_shared_memory_bytes(tier: ChipTier) -> u32
//     — returns the per-threadgroup shared memory limit in bytes:
//         M1 series: 32,768 bytes (32 KB)
//         M2 series: 32,768 bytes (32 KB)
//         M3 series: 32,768 bytes (32 KB)
//         M4 series: 65,536 bytes (64 KB) — doubled in M4
//         M5 series: 65,536 bytes (assumed same as M4 until confirmed)
//     — used by shaders that use var<workgroup> arrays to size their
//       shared memory allocations correctly per chip.
//
//   pub fn max_threads_per_threadgroup(tier: ChipTier) -> u32
//     — returns the maximum total invocations per threadgroup:
//         All Apple Silicon: 1024 threads maximum
//     — used to validate that workgroup_x * workgroup_y <= this limit.
//
// NOTES FOR AI:
//   - Always use div_ceil (integer ceiling division) for dispatch sizes,
//     never integer division. div_ceil(a, b) = (a + b - 1) / b.
//     In Rust: u32::div_ceil(a, b) is available from Rust 1.73.
//   - Padding: the dispatched grid may be larger than the image. WGSL shaders
//     must return early for out-of-bounds positions:
//       let pos = vec2u(global_id.x, global_id.y);
//       if pos.x >= scene.width || pos.y >= scene.height { return; }
//   - threadgroup_shared_memory_bytes is queried at pipeline creation time
//     to set the correct wgpu pipeline layout. The WGSL var<workgroup>
//     arrays must fit within this limit.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::platform::macos::quality::ChipTier;

pub struct DispatchConfig {
    pub workgroup_x:   u32,
    pub workgroup_y:   u32,
    pub dispatch_x:    u32,
    pub dispatch_y:    u32,
    pub padded_width:  u32,
    pub padded_height: u32,
}

pub fn compute_dispatch(width: u32, height: u32, tier: ChipTier) -> DispatchConfig {
    todo!()
}

pub fn threadgroup_shared_memory_bytes(tier: ChipTier) -> u32 {
    todo!()
}

pub fn max_threads_per_threadgroup(_tier: ChipTier) -> u32 {
    1024
}