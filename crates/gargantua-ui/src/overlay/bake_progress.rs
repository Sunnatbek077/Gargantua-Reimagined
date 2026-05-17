// ============================================================
// FILE: crates/gargantua-ui/src/overlay/bake_progress.rs
// LINES: ~200
// CATEGORY: UI — LUT bake progress overlay
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Progress overlay shown during LUT baking. Displays overall
//   progress (%), current step (e.g., "Baking geodesic LUT: 3/8"),
//   elapsed time, estimated remaining time, and a cancel button.
//   Auto-appears when baking starts; auto-hides when complete.
//
// CONTENTS (~200 lines):
//   pub struct BakeProgress {
//       pub progress:     f32,          // 0.0–1.0 overall
//       pub current_step: String,       // "Baking geodesic LUT (spin 3/8)"
//       pub elapsed_s:    f32,          // seconds elapsed
//       pub eta_s:        Option<f32>,  // estimated remaining seconds
//       pub is_baking:    bool,
//   }
//
//   impl BakeProgress {
//       pub fn new() -> Self
//
//       pub fn update(&mut self, progress: f32, step: &str, elapsed: f32)
//         // Called by app.rs when BakeProgressEvent received
//         // Computes eta_s = elapsed * (1.0 - progress) / progress
//
//       pub fn draw(
//           &self,
//           ctx: &egui::Context,
//           app_state: &mut AppState,
//           i18n: &I18n,
//       )
//         // Centered modal-like Area (not blocking input)
//         // Title: "Baking LUTs..."
//         // Progress bar: [████████░░░░░░░] 62%
//         // Label: current_step
//         // Label: "Elapsed: 14s  |  ETA: ~8s"
//         // Button: "Cancel" → emit AppEvent::CancelBake
//   }
//
// USES (imports from):
//   gargantua_app::state::{AppState, AppEvent}
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs
//   crates/gargantua-core/src/app.rs
//     → calls bake_progress.update() on BakeProgressEvent
//
// NOTE FOR AI:
//   BakeProgress is positioned at screen center (not anchored to corner).
//   It is NOT a modal — clicks still pass through to UI behind it.
//   eta_s formula: (elapsed / progress) * (1.0 - progress).
//   Avoid division by zero: only compute eta if progress > 0.01.
//   is_baking=false → this overlay is not drawn (visibility.bake_progress=false).
// ============================================================