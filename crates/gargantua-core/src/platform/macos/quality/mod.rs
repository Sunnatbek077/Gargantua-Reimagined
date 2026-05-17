// FILE: crates/gargantua-core/src/platform/macos/quality/mod.rs
#![cfg(target_os = "macos")]

pub use crate::quality::preset::QualityPreset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipTier {
    M1,
    M2,
    M3,
    M4,
    M5,
    Unknown,
}

pub mod m1_max_tier;
pub mod m1_pro_tier;
pub mod m1_tier;
pub mod m2_series;
pub mod m3_series;
pub mod m4_series;
pub mod m5_series;

use super::gpu::chip_detect::ChipInfo;

pub fn from_chip_info(info: &ChipInfo) -> QualityPreset {
    match info.tier {
        ChipTier::M1 => m1_tier::M1Tier::preset(info),
        ChipTier::M2 => m2_series::M2Tier::preset(info),
        ChipTier::M3 => m3_series::M3Tier::preset(info),
        ChipTier::M4 => m4_series::M4Tier::preset(info),
        ChipTier::M5 => m5_series::M5Tier::preset(info),
        ChipTier::Unknown => crate::quality::preset::QualityPreset {
            label:              "Unknown Mac",
            spp:                8,
            max_steps:          128,
            max_offline_spp:    512,
            workgroup_x:        8,
            workgroup_y:        8,
            enable_taa:         false,
            enable_bloom:       false,
            enable_motion_blur: false,
            target_fps:         60,
        },
    }
}
