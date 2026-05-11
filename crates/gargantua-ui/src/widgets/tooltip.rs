// ============================================================
// FILE: crates/gargantua-ui/src/widgets/tooltip.rs
// LINES: ~120
// CATEGORY: UI — Custom tooltip widget with physics formula support
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Enhanced tooltip widget that supports multi-line text and
//   inline physics formula display (e.g. "Ω_K = M^(1/2) / r^(3/2)").
//   Used via .tooltip("text") builder on SliderLabeled and Toggle.
//   Appears 300ms after hover with a smooth fade-in.
//
// CONTENTS (~120 lines):
//   // Show a styled tooltip for the last widget response
//   // Call after ui.add(widget) to attach tooltip to it
//   pub fn show_tooltip(
//       ui: &egui::Ui,
//       response: &egui::Response,
//       text: &str,
//   )
//     // Only shows if response.hovered() for >300ms
//     // Draws dark rounded popup with:
//     //   White text (main description line)
//     //   Gray secondary lines (formula, range info, etc.)
//     // Auto-positions: prefers below widget, falls back to above
//     // Fade-in: alpha ramps 0→255 over 150ms
//
//   // Multi-line tooltip with title + body
//   pub fn show_tooltip_titled(
//       ui: &egui::Ui,
//       response: &egui::Response,
//       title: &str,
//       body: &str,
//   )
//     // Title in bold white, body in gray
//     // Used for complex parameters (e.g. ISCO radius tooltip with formula)
//
//   // Internal: get or create hover duration tracker
//   fn hover_duration(ui: &egui::Ui, id: egui::Id) -> f32
//     // Stored in egui::Memory::data as f32 seconds
//     // Incremented by predicted_dt each frame while hovered
//     // Reset to 0 when not hovered
//
// USES (imports from):
//   egui  → Ui, Response, Id, Color32, Painter, Memory
//
// USED BY:
//   crates/gargantua-ui/src/widgets::slider_labeled.rs
//     → show_tooltip(ui, &response, self.tooltip) if tooltip.is_some()
//   crates/gargantua-ui/src/widgets::toggle.rs
//     → same pattern
//
// NOTE FOR AI:
//   300ms hover delay prevents tooltips from flashing on quick mouse passes.
//   hover_duration() uses egui::Id::new(("tooltip_hover", widget_id))
//   as the memory key to avoid collisions between different widgets.
//   Tooltip max width: 280px. Text wraps automatically.
//   Physics formulas in tooltip text use Unicode math chars (Ω, π, √, etc.)
//   — no LaTeX rendering, just Unicode strings.
// ============================================================