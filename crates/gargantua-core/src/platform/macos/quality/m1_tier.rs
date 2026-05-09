// =============================================================================
// crates/gargantua-core/src/platform/macos/quality/m1_tier.rs
// =============================================================================
//
// PURPOSE:
//   Defines the render quality preset for M1 Base chips.
//   Called by quality/mod.rs::from_chip_info() when the chip tier matches.
//   Returns a QualityPreset tuned to the M1 Base's GPU core count,
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
// PRESET VALUES (M1 Base):
//   Real-time SPP:      2  (targets 60fps at 1440p)
//   High quality SPP:   4  (targets 30fps at 1440p, or 60fps at 1080p)
//   Max steps:          64   (geodesic integration steps per ray)
//   Max offline SPP:    128 (offline render ceiling)
//   Workgroup:          (8, 8, 1)
//   TAA enabled:        false
//   Bloom enabled:      false
//   Motion blur:        false
//   Target FPS:         60
//
// VARIANT HANDLING:
//   The preset is further refined based on ChipInfo.gpu_cores:
//   Higher core counts within the same tier get proportionally higher SPP.
//   Example for M1 Base:
//     base gpu_cores → spp = 2
//     higher gpu_cores → spp scales linearly up to 4
//
// NOTES FOR AI:
//   - All SPP values target 60fps at 1440p on the base configuration.
//     At 4K, divide SPP by 4 to maintain the same frame time.
//   - target_fps = 60: used by the render loop to set the frame time budget.
//   - Do not set spp = 0 — minimum is 1 (1 sample always produces a valid image).
//   - enable_taa = false: TAA requires a history buffer. On low-memory
//     configs, TAA may be force-disabled by pressure_response.rs.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::platform::macos::{
    gpu::chip_detect::ChipInfo,
    quality::QualityPreset,
};

pub struct M1Tier;

impl M1Tier {
    pub fn preset(info: &ChipInfo) -> QualityPreset {
        // Scale SPP linearly with GPU core count within this tier
        let spp = Self::scaled_spp(info.gpu_cores);

        QualityPreset {
            label:              "M1 Base",
            spp,
            max_steps:          64,
            max_offline_spp:    128,
            workgroup_x:        8,
            workgroup_y:        8,
            enable_taa:         false,
            enable_bloom:       false,
            enable_motion_blur: false,
            target_fps:         60,
        }
    }

    fn scaled_spp(gpu_cores: u32) -> u32 {
        // Base SPP at minimum core count for this tier,
        // scaled up proportionally for higher core counts.
        let base_spp: u32 = 2;
        let max_spp:  u32 = 4;
        // Clamp to [base_spp, max_spp]
        (base_spp + gpu_cores / 4).min(max_spp).max(base_spp)
    }
}