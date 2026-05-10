// ============================================================
// FILE: crates/gargantua-ui/src/overlay/crosshair.rs
// LINES: ~120
// CATEGORY: UI — Center crosshair overlay
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Draws a small crosshair at the center of the screen for aiming
//   reference. Can be toggled on/off. Style: thin cross (+) with
//   a gap in the center. Color is white with slight transparency.
//
// CONTENTS (~120 lines):
//   pub struct Crosshair {
//       pub size:    f32,   // arm length in pixels [10–40], default 20
//       pub gap:     f32,   // center gap [2–8], default 4
//       pub color:   egui::Color32,  // default: white at 80% alpha
//       pub thickness: f32, // line thickness [1–3], default 1.5
//   }
//
//   impl Crosshair {
//       pub fn new() -> Self
//
//       pub fn draw(&self, ctx: &egui::Context, layout: &HudLayout)
//         // Use egui::Area::new("crosshair") over the screen center
//         // Draw 4 lines (up, down, left, right) with gap in center:
//         //   Top:    (center_x, center_y - gap) to (center_x, center_y - size)
//         //   Bottom: (center_x, center_y + gap) to (center_x, center_y + size)
//         //   Left:   (center_x - gap, center_y) to (center_x - size, center_y)
//         //   Right:  (center_x + gap, center_y) to (center_x + size, center_y)
//   }
//
// USES (imports from):
//   crate::hud::layout::HudLayout  → crosshair_center position
//   egui                           → Area, Painter, Color32, Pos2, Stroke
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs
//
// NOTE FOR AI:
//   Uses egui::Painter lines directly — no textures or SVG.
//   Area must be non-interactive (sense = egui::Sense::hover())
//   so clicks pass through to the render view beneath it.
//   Default color: Color32::from_rgba_premultiplied(255, 255, 255, 200).
// ============================================================