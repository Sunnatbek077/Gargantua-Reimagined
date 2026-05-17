// =============================================================================
// crates/gargantua-core/src/quality/detector.rs
// =============================================================================
//
// PURPOSE:
//   Cross-platform quality detector. Examines the GPU adapter at startup and
//   dispatches to the correct platform-specific quality module to return the
//   optimal QualityPreset for this machine.
//
//   This is the single entry point that App::new() calls — it abstracts away
//   all platform-specific detection logic (macOS chip tier, Windows vendor).
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::context::GpuContext
//     - crate::quality::preset::QualityPreset
//     #[cfg(target_os = "macos")]
//     - crate::platform::macos::gpu::chip_detect::{detect, ChipInfo}
//     - crate::platform::macos::quality::{from_chip_info, safe_minimum}
//     #[cfg(target_os = "windows")]
//     - crate::platform::windows::gpu::vendor::{detect_vendor, VendorDetails}
//     - crate::platform::windows::quality::{nvidia_presets, amd_presets, intel_presets}
//   External: none
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs::App::new()
//       — QualityDetector::detect(&ctx) called once at startup
//   - crates/gargantua-ui/src/menu/tabs/render_tab.rs
//       — displays detected preset name
//
// PUBLIC TYPES:
//
//   pub struct QualityDetector {
//     preset:       QualityPreset,
//     platform_info: PlatformInfo,
//   }
//
//   pub enum PlatformInfo {
//     MacOs  { chip_info: ChipInfo     },  // only on macOS build
//     Windows{ vendor:    VendorDetails },  // only on Windows build
//     Wasm,                                 // WASM always minimum preset
//     Unknown,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn detect(ctx: &GpuContext) -> Self
//     — dispatches based on compile target:
//
//       #[cfg(target_os = "macos")]
//         chip_info = crate::platform::macos::gpu::chip_detect::detect()
//         preset    = crate::platform::macos::quality::from_chip_info(&chip_info)
//         platform_info = PlatformInfo::MacOs { chip_info }
//
//       #[cfg(target_os = "windows")]
//         vendor = crate::platform::windows::gpu::vendor::detect_vendor(&ctx.adapter)
//         preset = match vendor.vendor {
//           GpuVendor::Nvidia => nvidia_presets::NvidiaPresets::preset(&vendor)
//           GpuVendor::Amd   => amd_presets::AmdPresets::preset(&vendor)
//           GpuVendor::Intel => intel_presets::IntelPresets::preset(&vendor)
//           _                => safe_minimum()
//         }
//         platform_info = PlatformInfo::Windows { vendor }
//
//       #[cfg(target_arch = "wasm32")]
//         preset = wasm_preset()   — always minimum (browser GPU variability)
//         platform_info = PlatformInfo::Wasm
//
//       _ (unknown platform)
//         preset = safe_minimum()
//         platform_info = PlatformInfo::Unknown
//
//     — logs the detected preset via tracing::info!.
//     — returns QualityDetector { preset, platform_info }.
//
//   pub fn preset(&self) -> &QualityPreset
//     — returns reference to the detected preset.
//     — consumed by AdaptiveQuality::new() at startup.
//
//   pub fn platform_info(&self) -> &PlatformInfo
//     — returns platform-specific detection info for UI display.
//
// PRIVATE FUNCTIONS:
//
//   fn safe_minimum() -> QualityPreset
//     — same as macos::quality::safe_minimum():
//         spp=2, steps=32, workgroup=(8,8), all post-fx disabled.
//
//   fn wasm_preset() -> QualityPreset
//     — slightly higher than safe_minimum for modern browser GPUs:
//         spp=4, steps=64, workgroup=(8,8), taa=false, bloom=false.
//
// NOTES FOR AI:
//   - QualityDetector is constructed once and its preset is cloned into
//     AdaptiveQuality. After that, QualityDetector is kept only for UI display.
//   - On WASM: browser GPUs vary widely (mobile → desktop). The wasm_preset
//     is conservative (SPP=4) to work on integrated graphics. The user can
//     increase quality manually via the UI settings panel.
//   - ChipInfo is only available on macOS. VendorDetails is only available
//     on Windows. Use cfg gates to keep the enum variants platform-specific.
// =============================================================================

use crate::{gpu::context::GpuContext, quality::preset::QualityPreset};

pub enum PlatformInfo {
    #[cfg(target_os = "macos")]
    MacOs { chip_info: crate::platform::macos::gpu::chip_detect::ChipInfo },
    #[cfg(target_os = "windows")]
    Windows { vendor: crate::platform::windows::gpu::vendor::VendorDetails },
    Wasm,
    Unknown,
}

pub struct QualityDetector {
    preset:        QualityPreset,
    platform_info: PlatformInfo,
}

impl QualityDetector {
    pub fn detect(ctx: &GpuContext) -> Self {
        todo!()
    }

    pub fn preset(&self) -> &QualityPreset {
        &self.preset
    }

    pub fn platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }
}

fn safe_minimum() -> QualityPreset {
    QualityPreset {
        label:              "Minimum (Safe)",
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

fn wasm_preset() -> QualityPreset {
    QualityPreset {
        label:              "Browser (Conservative)",
        spp:                4,
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