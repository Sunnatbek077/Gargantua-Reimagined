// ============================================================
// FILE: crates/gargantua-ui/src/widgets/slider_labeled.rs
// LINES: ~220
// CATEGORY: UI — Labeled slider with value display and reset button
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Wrapper around egui::Slider with a fixed-width label on the left,
//   value display on the right, and an optional reset-to-default button.
//   Used for every parameter slider in the Physics, Accretion, Camera,
//   and PostFX tabs. Integrates with the search system.
//
// CONTENTS (~220 lines):
//   pub struct SliderLabeled<'a> {
//       label:        &'a str,
//       value:        &'a mut f64,
//       range:        std::ops::RangeInclusive<f64>,
//       default:      Option<f64>,      // value for reset button
//       unit:         Option<&'a str>,  // e.g. "M☉", "°", "G", "M"
//       log_scale:    bool,
//       format:       &'a str,          // format string e.g. "{:.3}"
//       tooltip:      Option<&'a str>,  // hover tooltip text
//   }
//
//   impl<'a> SliderLabeled<'a> {
//       pub fn new(label: &'a str, value: &'a mut f64,
//           range: std::ops::RangeInclusive<f64>) -> Self
//
//       // Builder methods:
//       pub fn default_val(mut self, v: f64) -> Self
//       pub fn unit(mut self, u: &'a str) -> Self
//       pub fn log(mut self) -> Self         // enable logarithmic scale
//       pub fn format(mut self, f: &'a str) -> Self
//       pub fn tooltip(mut self, t: &'a str) -> Self
//
//       // Draw the slider row, returns true if value changed
//       pub fn draw(self, ui: &mut egui::Ui, search: &SearchBar) -> bool
//         // If search.matches(label) == false → skip (do not draw)
//         // Layout: [Label 120px] [Slider ----] [Value] [↺ reset]
//         // Label: left-aligned, fixed width 120px, search highlight if matching
//         // Slider: egui::Slider with logarithmic if log_scale=true
//         // Value: right-aligned, formatted with format string + unit
//         // Reset button (↺): only shown if default is Some and value ≠ default
//         //   Click reset → *value = default.unwrap()
//         // Tooltip: shown on hover over label or slider
//   }
//
// USES (imports from):
//   egui             → Ui, Slider, Response, Label
//   crate::menu::search::SearchBar
//
// USED BY:
//   Every tab that has sliders:
//   menu/tabs/physics_tab.rs, accretion_tab.rs, camera_tab.rs,
//   postfx_tab.rs, bake_tab.rs, render_tab.rs
//
// NOTE FOR AI:
//   Label column: fixed 120px width (use ui.add_sized or horizontal layout).
//   log_scale: passes egui::Slider::logarithmic(true) to the egui slider.
//   format string convention: "{:.3}" = 3 decimal places, "{:.2e}" = scientific.
//   Reset button: small ↺ icon button, 16×16px, appears to the right of value.
//   Only draw reset if *value != default (within f64::EPSILON).
//   search.matches(label) must be called BEFORE any ui.add() calls,
//   otherwise egui allocates layout space even for hidden widgets.
// ============================================================