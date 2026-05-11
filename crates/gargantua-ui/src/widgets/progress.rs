// ============================================================
// FILE: crates/gargantua-ui/src/widgets/progress.rs
// LINES: ~160
// CATEGORY: UI — Custom styled progress bar widget
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Custom progress bar widget with styling consistent with the
//   Gargantua UI theme. Supports label overlay, animated shimmer
//   when indeterminate, and color customization.
//   Used by bake_progress.rs and render_progress.rs.
//
// CONTENTS (~160 lines):
//   pub struct ProgressBar {
//       pub value:       f32,          // 0.0–1.0 (use f32::NAN for indeterminate)
//       pub label:       Option<String>,// text overlaid on bar
//       pub color:       egui::Color32, // bar fill color
//       pub height:      f32,          // bar height in pixels [8–32]
//       pub show_pct:    bool,         // show "62%" text on right
//       shimmer_offset:  f32,          // internal: shimmer animation phase
//   }
//
//   impl ProgressBar {
//       pub fn new(value: f32) -> Self
//       pub fn label(mut self, text: impl Into<String>) -> Self  // builder pattern
//       pub fn color(mut self, c: egui::Color32) -> Self
//       pub fn height(mut self, h: f32) -> Self
//
//       // Draw progress bar and return Response
//       pub fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response
//         // If value.is_nan() → indeterminate (animated shimmer)
//         // Else → filled rect from left: fill_width = width * value
//         // Rounded corners (radius = height/2)
//         // Label centered on bar (white text, shadow)
//         // Percentage on right if show_pct=true
//         // Shimmer: bright streak sweeping left→right at 1.5s period
//
//       fn tick_shimmer(&mut self, dt: f32)
//         // shimmer_offset = (shimmer_offset + dt / 1.5) % 1.0
//   }
//
// USES (imports from):
//   egui  → Ui, Rect, Color32, Painter, Response
//
// USED BY:
//   crates/gargantua-ui/src/overlay/bake_progress.rs
//   crates/gargantua-ui/src/overlay/render_progress.rs
//   crates/gargantua-ui/src/overlay/stats_bar.rs  → mini FPS bar
//
// NOTE FOR AI:
//   Indeterminate mode (value=NAN): draw a sliding 30%-width bright block
//   that bounces left-to-right (not a full fill). Use shimmer_offset.
//   Default color: theme accent blue (#4D9DE0 in sRGB, converted to Color32).
//   Rounded corners: use egui Painter::rect_filled with Rounding::same(h/2).
//   dt from egui::Context::input(|i| i.predicted_dt).
// ============================================================