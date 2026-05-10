// ============================================================
// FILE: crates/gargantua-ui/src/overlay/stats_bar.rs
// LINES: ~200
// CATEGORY: UI — Performance stats bar (bottom of screen)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Bottom-edge stats bar showing real-time performance metrics:
//   FPS, frame time (ms), GPU render time, GPU memory usage,
//   ray march step count, and active shader variant.
//   Color-coded: green=good, yellow=warning, red=bad.
//
// CONTENTS (~200 lines):
//   pub struct StatsBar {
//       pub show_gpu_time:  bool,   // show GPU timer query result
//       pub show_gpu_mem:   bool,   // show VRAM usage
//       pub show_ray_stats: bool,   // show ray march step count
//       pub history_len:    usize,  // FPS graph history length [30–300]
//       fps_history:        VecDeque<f32>,  // rolling FPS history
//   }
//
//   impl StatsBar {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &mut self,
//           ctx: &egui::Context,
//           stats: &RenderStats,
//           layout: &HudLayout,
//           i18n: &I18n,
//       )
//         // egui::Area anchored to BottomLeft of screen
//         // Full-width dark bar (height ~24px)
//         //
//         // Left side:
//         //   "FPS: 60.2" (green if >55, yellow if >30, red if <30)
//         //   "Frame: 16.6ms"
//         //   "GPU: 12.3ms" if show_gpu_time
//         //   "VRAM: 1.2 / 8.0 GB" if show_gpu_mem
//         //
//         // Right side:
//         //   "Ray steps: 256"  if show_ray_stats
//         //   "Shader: kerr_rk4_adaptive" (active shader variant name)
//         //
//         // Mini FPS sparkline graph (last history_len frames)
//
//       fn fps_color(fps: f32) -> egui::Color32
//         // > 55 → green,  > 30 → yellow,  ≤ 30 → red
//   }
//
// USES (imports from):
//   gargantua_app::state::RenderStats
//   crate::hud::layout::HudLayout
//   crate::i18n::I18n
//   egui
//   std::collections::VecDeque
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs
//
// NOTE FOR AI:
//   RenderStats comes from gargantua-render's wgpu::QuerySet timer.
//   GPU time is from timestamp queries (requires wgpu TIMESTAMP_QUERY feature).
//   If GPU timer is unavailable (some drivers): show "--" instead of ms.
//   VRAM usage: wgpu::Device::poll() + custom memory tracker in render crate.
//   FPS sparkline: mini 80×20px line graph using egui Painter polyline.
// ============================================================