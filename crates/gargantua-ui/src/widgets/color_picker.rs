// ============================================================
// FILE: crates/gargantua-ui/src/widgets/color_picker.rs
// LINES: ~260
// CATEGORY: UI — Custom linear-space color picker widget
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Custom egui color picker that works in linear light space
//   (not gamma-corrected sRGB). Used for post-fx color grading,
//   bloom color tint, and background color selection.
//   Displays: hue wheel, saturation/value square, hex input,
//   and linear RGB float readout.
//
// CONTENTS (~260 lines):
//   pub struct ColorPicker {
//       pub color_linear: [f32; 3],  // current color in linear RGB
//       pub show_alpha:   bool,      // show alpha channel slider
//       wheel_size:       f32,       // hue wheel radius in pixels
//   }
//
//   impl ColorPicker {
//       pub fn new(initial_linear: [f32; 3]) -> Self
//
//       // Draw the color picker widget in the given ui.
//       // Returns true if color changed this frame.
//       pub fn draw(&mut self, ui: &mut egui::Ui, label: &str) -> bool
//         // Collapsible header with label + color swatch preview
//         // When open:
//         //   Hue ring: 360° arc with colored segments (gamma-converted for display)
//         //   SV square: saturation/value selector inside hue ring
//         //   Hex input field: #RRGGBB (sRGB, for copy/paste)
//         //   Linear float readout: "R: 0.214  G: 0.045  B: 0.002"
//         //   Sliders: R, G, B linear [0.0–1.0] (or [0.0–10.0] for HDR)
//
//       // Convert linear RGB → HSV for wheel display
//       fn linear_to_hsv(rgb: [f32; 3]) -> [f32; 3]
//
//       // Convert HSV → linear RGB
//       fn hsv_to_linear(hsv: [f32; 3]) -> [f32; 3]
//
//       // Convert linear → sRGB gamma for hex display
//       fn linear_to_srgb_hex(rgb: [f32; 3]) -> String
//         // "#" + 6 hex chars, e.g. "#FF8040"
//
//       // Parse hex string "#RRGGBB" → linear RGB
//       fn parse_hex(s: &str) -> Option<[f32; 3]>
//   }
//
// USES (imports from):
//   egui  → Ui, Color32, Painter, Rect, Response
//
// USED BY:
//   crates/gargantua-ui/src/menu/tabs/postfx_tab.rs
//     → bloom color tint picker, color grading pickers
//
// NOTE FOR AI:
//   ALL internal computation is in LINEAR light space.
//   When displaying on screen (egui Color32): apply sRGB gamma:
//     srgb = linear^(1/2.2) approximately, or exact piecewise function.
//   Hex field shows sRGB hex (industry standard for copy/paste to other apps).
//   Linear float sliders range [0.0–1.0] for SDR, [0.0–10.0] for HDR colors.
//   HDR range active when hdr_enabled=true in PostFxTab.
// ============================================================