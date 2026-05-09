// =============================================================================
// crates/gargantua-core/src/platform/macos/quality/m3_series.rs
// =============================================================================
//
// PURPOSE:
//   Defines the render quality preset for M3 Series chips.
//   Called by quality/mod.rs::from_chip_info() when the chip tier matches.
//   Returns a QualityPreset tuned to the M3 Series's GPU core count,
//   memory bandwidth, and thermal envelope.
//
// SIZE: ~80 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::{ChipTier, QualityPreset}
//     - crate::platform::macos::gpu::chip_detect::ChipInfo
//     - crate::platform::macos::gpu::gpu_cores::profile_from_chip
//   External: none
//
// CALLED BY:
//   - crate::platform::macos::quality::from_chip_info()
//
// PRESET VALUES (M3 Series):
//   Real-time SPP:      16  (targets 60fps at 1440p)
//   High quality SPP:   24  (targets 30fps at 1440p, or 60fps at 1080p)
//   Max steps:          256   (geodesic integration steps per ray)
//   Max offline SPP:    1024 (offline render ceiling)
//   Workgroup:          (16, 8, 1)
//   TAA enabled:        true
//   Bloom enabled:      true
//   Motion blur:        true
//   Target FPS:         60
//
// VARIANT HANDLING:
//   The preset is further refined based on ChipInfo.gpu_cores:
//   Higher core counts within the same tier get proportionally higher SPP.
//   Example for M3 Series:
//     base gpu_cores → spp = 16
//     higher gpu_cores → spp scales linearly up to 24
//
// NOTES FOR AI:
//   - All SPP values target 60fps at 1440p on the base configuration.
//     At 4K, divide SPP by 4 to maintain the same frame time.
//   - target_fps = 60: used by the render loop to set the frame time budget.
//   - Do not set spp = 0 — minimum is 1 (1 sample always produces a valid image).
//   - enable_taa = true: TAA requires a history buffer. On low-memory
//     configs, TAA may be force-disabled by pressure_response.rs.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::platform::macos::{
    gpu::chip_detect::ChipInfo,
    quality::QualityPreset,
};

pub struct M3Tier;

impl M3Tier {
    pub fn preset(info: &ChipInfo) -> QualityPreset {
        // Scale SPP linearly with GPU core count within this tier
        let spp = Self::scaled_spp(info.gpu_cores);

        QualityPreset {
            label:              "M3 Series",
            spp,
            max_steps:          256,
            max_offline_spp:    1024,
            workgroup_x:        16,
            workgroup_y:        8,
            enable_taa:         true,
            enable_bloom:       true,
            enable_motion_blur: true,
            target_fps:         60,
        }
    }

    fn scaled_spp(gpu_cores: u32) -> u32 {
        // Base SPP at minimum core count for this tier,
        // scaled up proportionally for higher core counts.
        let base_spp: u32 = 16;
        let max_spp:  u32 = 24;
        // Clamp to [base_spp, max_spp]
        (base_spp + gpu_cores / 4).min(max_spp).max(base_spp)
    }
}