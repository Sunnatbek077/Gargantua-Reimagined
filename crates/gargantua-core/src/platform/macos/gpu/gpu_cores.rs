// =============================================================================
// crates/gargantua-core/src/platform/macos/gpu/gpu_cores.rs
// =============================================================================
//
// PURPOSE:
//   Maps the detected chip model to the exact GPU core count and derives
//   the optimal render quality settings (SPP, max steps, workgroup size)
//   that fit within the GPU's computational budget.
//
//   The GPU core count directly determines how many concurrent SIMD groups
//   the GPU can execute, which sets the ceiling for ray march throughput.
//   This module provides the authoritative core count and derived limits
//   used by the quality tier modules (quality/*.rs).
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::chip_detect::{ChipInfo, ChipVariant}
//     - super::super::quality::ChipTier
//   External: none
//
// CALLED BY:
//   - crate::platform::macos::quality::m1_pro_tier::M1ProTier::new()
//   - crate::platform::macos::quality::m1_max_tier::M1MaxTier::new()
//   - crate::platform::macos::quality::m2_series::M2Tier::new()
//   - ... all quality tier modules
//   - crates/gargantua-ui/src/overlay/stats_bar.rs  — displays core count
//
// PUBLIC TYPES:
//
//   pub struct GpuCoreProfile {
//     pub core_count:           u32,    // physical GPU cores
//     pub max_concurrent_waves: u32,    // core_count * waves_per_core (4 on AS)
//     pub tflops_fp32:          f32,    // theoretical FP32 throughput (TFLOPS)
//     pub tflops_fp16:          f32,    // theoretical FP16 throughput (TFLOPS)
//     pub recommended_spp:      u32,    // samples per pixel for real-time 60fps
//     pub recommended_steps:    u32,    // geodesic integration steps
//     pub max_offline_spp:      u32,    // max SPP for offline (non-real-time) render
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn profile_from_chip(info: &ChipInfo) -> GpuCoreProfile
//     — maps ChipInfo to GpuCoreProfile using the known GPU core table.
//     — known profiles:
//
//       M1 Base (7-core GPU):
//         tflops_fp32 = 2.6, tflops_fp16 = 5.2
//         recommended_spp = 4,  steps = 64,  max_offline_spp = 128
//
//       M1 Base (8-core GPU):
//         tflops_fp32 = 2.6, tflops_fp16 = 5.2
//         recommended_spp = 4,  steps = 64,  max_offline_spp = 256
//
//       M1 Pro (14-core GPU):
//         tflops_fp32 = 5.2, tflops_fp16 = 10.4
//         recommended_spp = 8,  steps = 128, max_offline_spp = 512
//
//       M1 Pro (16-core GPU):
//         tflops_fp32 = 6.1, tflops_fp16 = 12.2
//         recommended_spp = 8,  steps = 128, max_offline_spp = 512
//
//       M1 Max (24-core GPU):
//         tflops_fp32 = 10.4, tflops_fp16 = 20.8
//         recommended_spp = 16, steps = 256, max_offline_spp = 1024
//
//       M1 Max (32-core GPU):
//         tflops_fp32 = 13.6, tflops_fp16 = 27.2
//         recommended_spp = 16, steps = 256, max_offline_spp = 2048
//
//       M1 Ultra (64-core GPU):
//         tflops_fp32 = 27.2, tflops_fp16 = 54.4
//         recommended_spp = 32, steps = 512, max_offline_spp = 4096
//
//       M2 Base (8-core), M2 Base (10-core):
//         tflops_fp32 = 3.6 / 4.5
//         recommended_spp = 8,  steps = 128, max_offline_spp = 256
//
//       M2 Pro (16-core), M2 Pro (19-core):
//         tflops_fp32 = 6.8 / 8.5
//         recommended_spp = 12, steps = 192, max_offline_spp = 768
//
//       M2 Max (30-core), M2 Max (38-core):
//         tflops_fp32 = 13.6 / 17.8
//         recommended_spp = 16, steps = 256, max_offline_spp = 2048
//
//       M3 Base (10-core):
//         tflops_fp32 = 4.6  (hardware ray tracing available but unused by wgpu)
//         recommended_spp = 8,  steps = 128, max_offline_spp = 512
//
//       M3 Pro (18-core):
//         tflops_fp32 = 8.2
//         recommended_spp = 16, steps = 256, max_offline_spp = 1024
//
//       M3 Max (30-core), M3 Max (40-core):
//         tflops_fp32 = 13.6 / 18.0
//         recommended_spp = 24, steps = 384, max_offline_spp = 2048
//
//       M4 Base (10-core):
//         tflops_fp32 = 4.6, tflops_fp16 = 9.2
//         recommended_spp = 12, steps = 192, max_offline_spp = 768
//
//       M4 Pro (20-core):
//         tflops_fp32 = 10.9, tflops_fp16 = 21.8
//         recommended_spp = 20, steps = 320, max_offline_spp = 2048
//
//       M4 Max (32-core), M4 Max (40-core):
//         tflops_fp32 = 17.8 / 22.2
//         recommended_spp = 32, steps = 512, max_offline_spp = 4096
//
//       M5 (speculative — treat same as M4 with 10% boost):
//         same tier as M4 until hardware confirmed
//
//     — returns a safe default for unknown chips:
//         recommended_spp = 4, steps = 64, max_offline_spp = 128
//
//   pub fn max_concurrent_waves(core_count: u32) -> u32
//     — Apple Silicon: 4 waves per GPU core (warp occupancy)
//     — returns core_count * 4
//     — used internally to compute max_concurrent_waves field.
//
// NOTES FOR AI:
//   - TFLOPS figures are Apple's announced numbers for marketing purposes.
//     Real-world shader throughput is typically 60-80% of peak.
//   - recommended_spp targets 60fps on the given GPU at 1440p.
//     For 4K real-time, halve the recommended_spp.
//   - max_offline_spp is a soft limit — the user can override it in the UI.
//     It is provided as a sensible default for the offline render dialog.
//   - max_concurrent_waves = core_count * 4 is an Apple Silicon constant.
//     On AMD/NVIDIA, the waves-per-CU/SM ratio is different.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::platform::macos::gpu::chip_detect::ChipInfo;

pub struct GpuCoreProfile {
    pub core_count:           u32,
    pub max_concurrent_waves: u32,
    pub tflops_fp32:          f32,
    pub tflops_fp16:          f32,
    pub recommended_spp:      u32,
    pub recommended_steps:    u32,
    pub max_offline_spp:      u32,
}

pub fn profile_from_chip(info: &ChipInfo) -> GpuCoreProfile {
    todo!()
}

pub fn max_concurrent_waves(core_count: u32) -> u32 {
    core_count * 4
}