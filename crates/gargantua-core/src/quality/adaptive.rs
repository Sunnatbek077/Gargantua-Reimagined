// =============================================================================
// crates/gargantua-core/src/quality/adaptive.rs
// =============================================================================
//
// PURPOSE:
//   Implements adaptive quality control — dynamically adjusts SPP, step count,
//   and post-fx toggles at runtime to maintain the target frame time (1/fps).
//   Measures actual GPU frame time via GpuProfiler and scales quality up or
//   down to stay within budget, without requiring user intervention.
//
//   This is the "cruise control" of the render pipeline: the user sets a
//   target FPS (60 or 120) and the adaptive system finds the highest quality
//   that fits within the frame budget on the current hardware.
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::gpu::profiler::GpuProfiler      — reads actual GPU frame time
//     - crate::quality::preset::QualityPreset  — base preset to adapt from
//     - crate::quality::detector::QualityDetector — provides initial preset
//   External:
//     - std::collections::VecDeque             — rolling frame time history
//
// CALLED BY:
//   - crates/gargantua-app/src/app.rs::App::render_frame()
//       — calls AdaptiveQuality::update() each frame
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — reads AdaptiveQuality::current_spp()
//   - crates/gargantua-ui/src/overlay/stats_bar.rs
//       — displays current adaptive quality state
//
// PUBLIC TYPES:
//
//   pub struct AdaptiveQuality {
//     target_frame_ms: f32,          // 1000.0 / target_fps
//     current_spp:     u32,          // actively used SPP this frame
//     current_steps:   u32,          // actively used max_steps this frame
//     history:         VecDeque<f32>, // last N frame times in ms (N=8)
//     preset:          QualityPreset, // base preset (from detector)
//     locked:          bool,          // true = user locked quality, no adaptation
//   }
//
//   pub struct AdaptiveState {
//     pub spp:            u32,
//     pub max_steps:      u32,
//     pub enable_bloom:   bool,
//     pub enable_taa:     bool,
//     pub enable_mb:      bool,
//     pub frame_ms:       f32,   // last measured frame time
//     pub target_ms:      f32,
//     pub is_over_budget: bool,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(preset: QualityPreset) -> Self
//     — initializes with preset.spp and preset.max_steps.
//     — target_frame_ms = 1000.0 / preset.target_fps as f32.
//     — history is empty until enough frames have been measured.
//
//   pub fn update(&mut self, profiler: &GpuProfiler)
//     — reads profiler.total_gpu_ms() and pushes to history.
//     — computes smoothed_ms = average of last 8 frames.
//     — SCALE DOWN (over budget): smoothed_ms > target_frame_ms * 1.05
//         Step 1: reduce current_spp by 25% (floor to minimum 1).
//         Step 2: if spp already at min, disable motion blur.
//         Step 3: if still over, disable bloom.
//         Step 4: if still over, reduce current_steps by 25%.
//     — SCALE UP (under budget): smoothed_ms < target_frame_ms * 0.85
//         Step 1: increase current_steps by 10% (cap at preset.max_steps).
//         Step 2: if steps at max, increase current_spp by 10% (cap at preset.spp).
//         Step 3: if spp at max, re-enable bloom if preset enables it.
//         Step 4: if bloom enabled, re-enable motion blur if preset enables it.
//     — Scale up is intentionally slower than scale down (hysteresis):
//       scale down triggers at 105% budget, scale up at 85% budget.
//       This prevents oscillation around the budget threshold.
//     — If locked: skip all adaptation, always use preset values.
//
//   pub fn current_state(&self) -> AdaptiveState
//     — returns current quality parameters for use by render passes.
//
//   pub fn lock(&mut self, locked: bool)
//     — enables/disables adaptation. Called by user "Lock Quality" toggle.
//
//   pub fn reset_to_preset(&mut self)
//     — resets current_spp and current_steps to preset values.
//     — called when a new preset is loaded or resolution changes.
//
//   pub fn set_target_fps(&mut self, fps: u32)
//     — updates target_frame_ms = 1000.0 / fps as f32.
//     — clears history to avoid stale data affecting new target.
//
// NOTES FOR AI:
//   - The history VecDeque is capped at 8 frames. Use history.pop_front()
//     when len() > 8 after push_back().
//   - Minimum SPP = 1 (always produces a valid render, just very noisy).
//     Minimum steps = 16 (below this, geodesic integration is inaccurate).
//   - Scale down is immediate (applied next frame). Scale up is gradual
//     (10% per frame) to prevent thrashing. This asymmetry is intentional.
//   - Adaptation is disabled in the offline render pipeline (gargantua-video):
//     offline renders use preset.max_offline_spp unconditionally.
// =============================================================================

use std::collections::VecDeque;

use crate::{gpu::profiler::GpuProfiler, quality::preset::QualityPreset};

pub struct AdaptiveState {
    pub spp:            u32,
    pub max_steps:      u32,
    pub enable_bloom:   bool,
    pub enable_taa:     bool,
    pub enable_mb:      bool,
    pub frame_ms:       f32,
    pub target_ms:      f32,
    pub is_over_budget: bool,
}

pub struct AdaptiveQuality {
    target_frame_ms: f32,
    current_spp:     u32,
    current_steps:   u32,
    history:         VecDeque<f32>,
    preset:          QualityPreset,
    locked:          bool,
}

impl AdaptiveQuality {
    pub fn new(preset: QualityPreset) -> Self {
        let target_frame_ms = 1000.0 / preset.target_fps as f32;
        let spp   = preset.spp;
        let steps = preset.max_steps;
        Self {
            target_frame_ms,
            current_spp:   spp,
            current_steps: steps,
            history:       VecDeque::with_capacity(8),
            preset,
            locked: false,
        }
    }

    pub fn update(&mut self, profiler: &GpuProfiler) {
        if self.locked { return; }
        todo!()
    }

    pub fn current_state(&self) -> AdaptiveState {
        let last_ms = self.history.back().copied().unwrap_or(0.0);
        AdaptiveState {
            spp:            self.current_spp,
            max_steps:      self.current_steps,
            enable_bloom:   self.preset.enable_bloom,
            enable_taa:     self.preset.enable_taa,
            enable_mb:      self.preset.enable_motion_blur,
            frame_ms:       last_ms,
            target_ms:      self.target_frame_ms,
            is_over_budget: last_ms > self.target_frame_ms * 1.05,
        }
    }

    pub fn lock(&mut self, locked: bool)         { self.locked = locked; }
    pub fn current_spp(&self)    -> u32          { self.current_spp    }
    pub fn current_steps(&self)  -> u32          { self.current_steps  }

    pub fn reset_to_preset(&mut self) {
        self.current_spp   = self.preset.spp;
        self.current_steps = self.preset.max_steps;
        self.history.clear();
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_frame_ms = 1000.0 / fps as f32;
        self.history.clear();
    }
}