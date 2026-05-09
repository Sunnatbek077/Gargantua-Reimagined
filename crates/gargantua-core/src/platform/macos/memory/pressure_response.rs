// =============================================================================
// crates/gargantua-core/src/platform/macos/memory/pressure_response.rs
// =============================================================================
//
// PURPOSE:
//   Defines the response strategy when the OS signals memory pressure.
//   Maps MemoryPressureLevel to concrete actions: which caches to evict,
//   how much to reduce render quality, and when to restore quality after
//   pressure subsides.
//
//   Works alongside memory_pressure.rs (which detects the pressure level)
//   and unified_allocator.rs (which performs the actual eviction).
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::platform::macos::gpu::memory_pressure::MemoryPressureLevel
//     - crate::frame::resource::ResourcePool
//     - crate::platform::macos::quality::ChipTier
//   External: none
//
// CALLED BY:
//   - crate::platform::macos::memory::unified_allocator::UnifiedAllocator
//       — calls respond_to_pressure() on pressure level change
//   - crates/gargantua-app/src/app.rs
//       — calls quality_override() each frame to check SPP reduction
//
// PUBLIC TYPES:
//
//   pub struct PressureResponse {
//     current_level:  MemoryPressureLevel,
//     quality_factor: f32,    // multiplier for SPP (1.0 = normal, 0.25 = minimal)
//     evict_luts:     bool,   // evict cached LUT textures from ResourcePool
//     evict_history:  bool,   // evict TAA history buffer (saves ~200MB at 4K)
//     disable_bloom:  bool,   // disable bloom post-fx (saves ~120MB at 4K)
//   }
//
//   pub struct QualityOverride {
//     pub spp_multiplier:   f32,     // multiply base SPP by this
//     pub max_steps:        u32,     // cap geodesic steps
//     pub disable_bloom:    bool,
//     pub disable_taa:      bool,    // disabling TAA frees history buffer
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new() -> Self
//     — creates a PressureResponse with Normal level defaults:
//         quality_factor = 1.0
//         evict_luts = false
//         evict_history = false
//         disable_bloom = false
//
//   pub fn respond_to_pressure(
//     &mut self,
//     level: MemoryPressureLevel,
//     pool:  &mut ResourcePool,
//   )
//     — called by UnifiedAllocator when pressure level changes.
//     — actions per level:
//
//       MemoryPressureLevel::Normal:
//         quality_factor = 1.0
//         evict_luts = false, evict_history = false, disable_bloom = false
//         — no action needed; restore quality on next frame
//
//       MemoryPressureLevel::Warning:
//         quality_factor = 0.5     — halve SPP (e.g. 16 → 8)
//         disable_bloom = true     — free ~120MB bloom pyramid buffers
//         pool.release_all_transient()  — free all reusable textures
//         evict_history = false    — keep TAA history (quality impact too high)
//
//       MemoryPressureLevel::Critical:
//         quality_factor = 0.25    — quarter SPP (e.g. 16 → 4)
//         disable_bloom = true
//         disable_taa = true       — free TAA history buffer (~200MB at 4K)
//         evict_luts = true        — evict LUT textures (~512MB)
//         pool.release_all_transient()
//         — LUTs will be re-baked when pressure returns to Normal.
//
//   pub fn quality_override(&self) -> QualityOverride
//     — returns the current quality reduction parameters.
//     — called by App::render_frame() to override base quality settings.
//     — if MemoryPressureLevel::Normal: returns identity override
//       (spp_multiplier=1.0, disable_bloom=false, disable_taa=false).
//
//   pub fn should_restore_quality(&self, level: MemoryPressureLevel) -> bool
//     — returns true if level == Normal AND quality_factor < 1.0.
//     — quality restoration is gradual: multiply quality_factor by 1.1 each
//       frame (not instant) to avoid re-triggering pressure on restoration.
//
// NOTES FOR AI:
//   - Quality reduction is immediate (applied next frame).
//     Quality restoration is gradual (10% per frame) to avoid pressure thrash.
//   - Evicting LUTs (geodesic_lut, blackbody_lut, doppler_lut) requires
//     re-baking them from gargantua-bake when pressure subsides.
//     Mark evicted LUTs with a dirty flag — gargantua-bake checks this on idle.
//   - The TAA history buffer at 4K RGBA16Float = 3840*2160*8 bytes = ~63MB.
//     At 8K it is ~252MB — significant savings when under pressure.
//   - Bloom pyramid at 4K: 6 levels, each half-res = ~120MB total RGBA16Float.
// =============================================================================

#![cfg(target_os = "macos")]

use crate::{
    frame::resource::ResourcePool,
    platform::macos::gpu::memory_pressure::MemoryPressureLevel,
};

pub struct QualityOverride {
    pub spp_multiplier: f32,
    pub max_steps:      u32,
    pub disable_bloom:  bool,
    pub disable_taa:    bool,
}

pub struct PressureResponse {
    current_level:  MemoryPressureLevel,
    quality_factor: f32,
    evict_luts:     bool,
    evict_history:  bool,
    disable_bloom:  bool,
    disable_taa:    bool,
}

impl PressureResponse {
    pub fn new() -> Self {
        Self {
            current_level:  MemoryPressureLevel::Normal,
            quality_factor: 1.0,
            evict_luts:     false,
            evict_history:  false,
            disable_bloom:  false,
            disable_taa:    false,
        }
    }

    pub fn respond_to_pressure(
        &mut self,
        level: MemoryPressureLevel,
        pool:  &mut ResourcePool,
    ) {
        todo!()
    }

    pub fn quality_override(&self) -> QualityOverride {
        QualityOverride {
            spp_multiplier: self.quality_factor,
            max_steps:      if self.quality_factor < 0.5 { 64 } else { u32::MAX },
            disable_bloom:  self.disable_bloom,
            disable_taa:    self.disable_taa,
        }
    }

    pub fn should_restore_quality(&self, level: MemoryPressureLevel) -> bool {
        level == MemoryPressureLevel::Normal && self.quality_factor < 1.0
    }
}