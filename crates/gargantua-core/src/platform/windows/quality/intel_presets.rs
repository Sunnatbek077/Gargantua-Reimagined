// =============================================================================
// crates/gargantua-core/src/platform/windows/quality/intel_presets.rs
// =============================================================================
//
// PURPOSE:
//   Defines render quality presets for Intel Arc GPUs on Windows.
//   Called by the cross-platform quality detector (quality/detector.rs)
//   when the detected GPU vendor is Intel Arc.
//   Returns a QualityPreset tuned to the GPU architecture's compute
//   characteristics, memory bandwidth, and driver optimizations.
//
// SIZE: ~120 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::windows::gpu::vendor::{GpuVendor, VendorDetails}
//     - crate::quality::preset::QualityPreset
//   External: none
//
// CALLED BY:
//   - crate::quality::detector::QualityDetector::detect()
//       — Windows branch, Intel Arc vendor path
//
// VARIANTS AND PRESET VALUES:
//     Arc Alchemist: spp=4/8, steps=64, offline_spp=256, wg=(8,4), taa=true, bloom=true, mb=false, fps=60
//     Arc Battlemage: spp=8/16, steps=128, offline_spp=512, wg=(8,4), taa=true, bloom=true, mb=true, fps=60
//
// NOTES FOR AI:
//   - SPP values target 60fps (or 120fps for newer tiers) at 1440p resolution.
//     At 4K, SPP is automatically halved by the adaptive quality system.
//   - Workgroup dimensions are chosen to match the GPU's warp/wavefront size.
//     See platform/windows/compute/workgroup.rs for detailed rationale.
//   - enable_motion_blur = false on older/weaker tiers to save ~15% frame time.
// =============================================================================

#![cfg(target_os = "windows")]

use crate::{
    platform::windows::gpu::vendor::VendorDetails,
    quality::preset::QualityPreset,
};

pub struct IntelPresets;

impl IntelPresets {
    pub fn preset(details: &VendorDetails) -> QualityPreset {
        // Dispatch to sub-preset based on architecture generation
        todo!()
    }

    pub fn safe_minimum() -> QualityPreset {
        QualityPreset {
            label:              "Intel Arc — Minimum",
            spp:                2,
            max_steps:          32,
            max_offline_spp:    64,
            workgroup_x:        8,
            workgroup_y:        8,
            enable_taa:         false,
            enable_bloom:       false,
            enable_motion_blur: false,
            target_fps:         30,
        }
    }
}