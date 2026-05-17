// =============================================================================
// crates/gargantua-core/src/platform/macos/gpu/chip_detect.rs
// =============================================================================
//
// PURPOSE:
//   Detects the Apple Silicon chip model at runtime by querying the
//   sysctl hw.model key and parsing the result. Returns a ChipInfo struct
//   that other modules use to select optimal quality tiers, workgroup sizes,
//   and feature flags.
//
//   Also detects whether the machine is Apple Silicon or Intel so that
//   ANE (Neural Engine) and EDR code paths are gated correctly.
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::super::quality::ChipTier   — enum returned by detect()
//     - crate::errors::CoreError
//   External:
//     - libc::{sysctl, CTL_HW, HW_MODEL}
//     - std::ffi::CStr
//
// CALLED BY:
//   - crate::platform::macos::gpu::adapter_metal::create_metal_adapter()
//   - crate::platform::macos::compute::neural_engine::NeuralEngine::new()
//   - crate::platform::macos::compute::simd_group::simd_config_for_chip()
//   - crate::platform::macos::memory::unified_allocator::UnifiedAllocator::new()
//   - crates/gargantua-core/src/app.rs  — stores ChipInfo for lifetime of app
//
// PUBLIC TYPES:
//
//   pub struct ChipInfo {
//     pub tier:          ChipTier,      // M1/M2/M3/M4/M5 series enum
//     pub variant:       ChipVariant,   // Base/Pro/Max/Ultra
//     pub gpu_cores:     u32,           // actual GPU core count
//     pub cpu_cores:     u32,           // performance + efficiency cores total
//     pub neural_engine: bool,          // true for all Apple Silicon
//     pub is_apple_silicon: bool,       // false for Intel Macs
//     pub model_string:  String,        // raw hw.model e.g. "MacBookPro18,3"
//   }
//
//   pub enum ChipVariant {
//     Base,   // M1, M2, M3, M4, M5
//     Pro,    // M1 Pro, M2 Pro, ...
//     Max,    // M1 Max, M2 Max, ...
//     Ultra,  // M1 Ultra, M2 Ultra, ...
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn detect() -> ChipInfo
//     — reads hw.model via sysctl():
//         let mut mib = [libc::CTL_HW, libc::HW_MODEL];
//         sysctl(mib.as_mut_ptr(), 2, buf.as_mut_ptr(), &mut size, null_mut(), 0)
//     — parses the returned C string e.g. "MacBookPro18,3"
//     — maps model identifier to ChipTier + ChipVariant:
//
//         "MacBookPro18,3" | "MacBookPro18,4"  → M1 Pro
//         "MacBookPro18,1" | "MacBookPro18,2"  → M1 Max
//         "Mac13,2"                             → M1 Ultra (Mac Studio)
//         "MacBookAir10,1" | "MacBook10,1"     → M1 Base
//         "MacBookPro19,1" | "MacBookPro19,2"  → M2 Pro
//         "MacBookPro19,3" | "MacBookPro19,4"  → M2 Max
//         "Mac14,13" | "Mac14,14"              → M2 Ultra
//         "MacBookPro21,1" | "MacBookPro21,2"  → M3 Pro
//         "MacBookPro21,3" | "MacBookPro21,4"  → M3 Max
//         "MacBookPro22,1" | "MacBookPro22,2"  → M4 Pro
//         "MacBookPro22,3" | "MacBookPro22,4"  → M4 Max
//         ... (full mapping table in implementation)
//
//     — reads gpu_cores from hw.perflevel0.physicalcpu (approximation)
//       or falls back to a known value table per model identifier.
//     — sets is_apple_silicon = true for all M-series identifiers.
//     — for Intel Macs (e.g. "MacBookPro16,1"): returns ChipInfo with
//         is_apple_silicon = false, tier = ChipTier::Unknown.
//
//   pub fn gpu_core_count(model: &str) -> u32
//     — looks up the known GPU core count for a model identifier.
//     — fallback table:
//         M1 Base: 7 or 8 (BTO)
//         M1 Pro:  14 or 16
//         M1 Max:  24 or 32
//         M2 Base: 8 or 10
//         M2 Pro:  16 or 19
//         M2 Max:  30 or 38
//         M3 Base: 10
//         M3 Pro:  18
//         M3 Max:  30 or 40
//         M4 Base: 10
//         M4 Pro:  20
//         M4 Max:  32 or 40
//     — returns 8 as a safe default for unknown models.
//
//   pub fn is_apple_silicon() -> bool
//     — lightweight check: reads hw.model and checks for known AS identifiers.
//     — faster than full detect() — use when only the boolean is needed.
//
// NOTES FOR AI:
//   - sysctl is a POSIX API available on macOS. Use libc crate for bindings.
//   - The hw.model string format is "MacProductLine,Config" where Config
//     is a number that encodes GPU/CPU tier. The mapping is not officially
//     documented by Apple — maintain the table from public sources.
//   - detect() is called once at startup and the result is stored in App.
//     Do not call it every frame.
//   - For future M5 chips: add new model identifiers as they are announced.
//     The function should not panic on unknown identifiers — return a safe
//     default (ChipTier::Unknown, ChipVariant::Base, gpu_cores: 8).
// =============================================================================

#![cfg(target_os = "macos")]

use crate::{errors::CoreError, platform::macos::quality::ChipTier};

#[derive(Debug, Clone)]
pub enum ChipVariant {
    Base,
    Pro,
    Max,
    Ultra,
}

#[derive(Debug, Clone)]
pub struct ChipInfo {
    pub tier:             ChipTier,
    pub variant:          ChipVariant,
    pub gpu_cores:        u32,
    pub cpu_cores:        u32,
    pub neural_engine:    bool,
    pub is_apple_silicon: bool,
    pub model_string:     String,
}

pub fn detect() -> ChipInfo {
    todo!()
}

pub fn gpu_core_count(model: &str) -> u32 {
    todo!()
}

pub fn is_apple_silicon() -> bool {
    todo!()
}