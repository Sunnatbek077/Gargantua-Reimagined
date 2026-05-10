// ============================================================
// FILE: crates/gargantua-ui/src/overlay/render_progress.rs
// LINES: ~180
// CATEGORY: UI — Offline render progress overlay
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Progress overlay shown during offline high-quality render.
//   Similar to bake_progress.rs but for render operations.
//   Shows tile progress (if tiled render), SPP progress, elapsed
//   time, ETA, and estimated file size.
//
// CONTENTS (~180 lines):
//   pub struct RenderProgress {
//       pub progress:     f32,         // 0.0–1.0
//       pub current_spp:  u32,         // samples completed
//       pub total_spp:    u32,
//       pub tile:         Option<(u32, u32, u32, u32)>, // x, y, w, h
//       pub elapsed_s:    f32,
//       pub eta_s:        Option<f32>,
//       pub output_size_mb: Option<f32>,
//       pub is_rendering: bool,
//   }
//
//   impl RenderProgress {
//       pub fn new() -> Self
//       pub fn update(&mut self, progress: f32, spp: u32, elapsed: f32)
//
//       pub fn draw(&self, ctx: &egui::Context, app_state: &mut AppState, i18n: &I18n)
//         // Title: "Rendering..."
//         // Progress bar: [████░░░░] 43% (spp: 43/100)
//         // Tile indicator: "Tile 12/64" if tiled
//         // Elapsed + ETA
//         // Estimated file size if known
//         // Button: "Cancel" → AppEvent::CancelRender
//   }
//
// USES (imports from):
//   gargantua_app::state::{AppState, AppEvent}
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs
//   crates/gargantua-app/src/app.rs → update() on RenderProgressEvent
//
// NOTE FOR AI:
//   Tiled rendering: large renders split into tiles for GPU memory.
//   tile = Some((x, y, w, h)) when a tile is active.
//   output_size_mb is estimated: width * height * 3 * 4 bytes for EXR.
//   Same ETA formula as bake_progress.rs — share the function if possible.
// ============================================================