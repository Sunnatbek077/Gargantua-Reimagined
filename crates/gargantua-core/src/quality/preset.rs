// =============================================================================
// crates/gargantua-core/src/quality/preset.rs
// =============================================================================
//
// PURPOSE:
//   Defines the cross-platform QualityPreset struct used by all platform
//   quality modules (macos/quality/*.rs, windows/quality/*.rs) and the
//   adaptive quality system. This is the single authoritative definition —
//   platform modules return this struct; platform code does not define it.
//
// SIZE: ~80 lines
//
// DEPENDENCIES: none
//
// CALLED BY:
//   - crate::platform::macos::quality::*      — all tier modules return this
//   - crate::platform::windows::quality::*    — all vendor presets return this
//   - crate::quality::detector::QualityDetector
//   - crate::quality::adaptive::AdaptiveQuality
//   - crates/gargantua-render/src/pipelines::*  — reads spp, max_steps
//   - crates/gargantua-ui/src/panel::settings   — displays label, target_fps
//
// PUBLIC TYPES:
//
//   pub struct QualityPreset {
//     pub label:              &'static str,
//       — human-readable name shown in UI. Examples:
//           "M1 Pro — High", "RTX 4090 — Ultra", "Minimum (Safe)"
//
//     pub spp:                u32,
//       — samples per pixel for real-time rendering.
//         Higher = better quality but slower. Adapted by adaptive.rs.
//
//     pub max_steps:          u32,
//       — maximum geodesic integration steps per ray.
//         Higher = more accurate photon paths near the event horizon.
//         Also adapted by adaptive.rs.
//
//     pub max_offline_spp:    u32,
//       — maximum SPP for offline (non-real-time) rendering.
//         Used by gargantua-video when rendering at full quality.
//         Not adapted — user can set up to this limit.
//
//     pub workgroup_x:        u32,
//     pub workgroup_y:        u32,
//       — compute shader workgroup dimensions. Set by platform quality module.
//         Injected into WGSL via PipelineCompilationOptions overrides.
//
//     pub enable_taa:         bool,
//       — temporal anti-aliasing. Requires TAA history buffer (~63MB at 4K).
//         Disabled on low-memory configs by pressure_response.rs.
//
//     pub enable_bloom:       bool,
//       — bloom post-fx (Kawase dual filter). Uses ~120MB pyramid at 4K.
//         First to be disabled under memory/performance pressure.
//
//     pub enable_motion_blur: bool,
//       — tile-based motion blur. Uses velocity buffer (~32MB at 4K).
//         Disabled on lower tiers to save frame time.
//
//     pub target_fps:         u32,
//       — target frame rate for real-time rendering (60 or 120).
//         Sets AdaptiveQuality::target_frame_ms = 1000.0 / target_fps.
//   }
//
//   impl QualityPreset:
//     pub fn frame_budget_ms(&self) -> f32
//       — returns 1000.0 / self.target_fps as f32.
//
//     pub fn total_memory_mb(&self, width: u32, height: u32) -> f32
//       — estimates total GPU memory this preset requires at the given resolution:
//           framebuffer (HDR): width * height * 8 bytes → MB
//           TAA history:       same if enable_taa
//           bloom pyramid:     ~0.33 * framebuffer if enable_bloom
//           motion blur:       ~0.125 * framebuffer if enable_motion_blur
//           geodesic LUT:      ~512MB (fixed, baked)
//           blue noise:        ~256MB (fixed, baked)
//       — used by UI to show memory estimate before starting render.
// =============================================================================

pub struct QualityPreset {
    pub label:              &'static str,
    pub spp:                u32,
    pub max_steps:          u32,
    pub max_offline_spp:    u32,
    pub workgroup_x:        u32,
    pub workgroup_y:        u32,
    pub enable_taa:         bool,
    pub enable_bloom:       bool,
    pub enable_motion_blur: bool,
    pub target_fps:         u32,
}

impl QualityPreset {
    pub fn frame_budget_ms(&self) -> f32 {
        1000.0 / self.target_fps as f32
    }

    pub fn total_memory_mb(&self, width: u32, height: u32) -> f32 {
        let px           = (width * height) as f32;
        let fb_mb        = px * 8.0 / (1024.0 * 1024.0); // RGBA16Float
        let taa_mb       = if self.enable_taa         { fb_mb        } else { 0.0 };
        let bloom_mb     = if self.enable_bloom        { fb_mb * 0.33 } else { 0.0 };
        let mb_mb        = if self.enable_motion_blur  { fb_mb * 0.125} else { 0.0 };
        let lut_mb       = 512.0;  // geodesic LUT (fixed)
        let noise_mb     = 256.0;  // blue noise   (fixed)
        fb_mb + taa_mb + bloom_mb + mb_mb + lut_mb + noise_mb
    }
}