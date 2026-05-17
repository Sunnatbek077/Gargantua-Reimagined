// ============================================================
// FILE: crates/gargantua-ui/src/hud/layout.rs
// LINES: ~180
// CATEGORY: UI — HUD panel layout and sizing
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Computes pixel-precise layout rects for all HUD panels and
//   overlays based on window size and DPI scale. Recalculated on
//   resize. Single source of truth for all UI positioning.
//
// CONTENTS (~180 lines):
//   pub struct HudLayout {
//       pub screen:          egui::Rect,     // full window rect
//       pub menu_panel:      egui::Rect,     // left side panel (320px wide)
//       pub overlay_tl:      egui::Pos2,     // physics readout top-left
//       pub overlay_br:      egui::Pos2,     // stats bar bottom-right
//       pub toggle_btn:      egui::Rect,     // hamburger toggle button rect
//       pub crosshair_center:egui::Pos2,     // screen center for crosshair
//       pub dpi_scale:       f32,            // hidpi scale factor
//   }
//
//   impl HudLayout {
//       pub fn new(screen_size: egui::Vec2, dpi: f32) -> Self
//         // menu_panel: x=0, y=0, w=320*dpi, h=screen_size.y
//         // overlay_tl: top-right corner at (screen_w - 280*dpi, 10*dpi)
//         // toggle_btn: top-left at (8*dpi, 8*dpi), size=32×32*dpi
//         // crosshair: screen center
//
//       pub fn recalculate(&mut self, new_size: egui::Vec2, dpi: f32)
//         // Called on window resize — recomputes all rects
//
//       pub fn menu_panel_animated(&self, x_offset: f32) -> egui::Rect
//         // Returns menu_panel shifted by x_offset (for slide animation)
//         // x_offset from HudAnimator::menu_x.value
//
//       pub fn is_point_in_menu(&self, pos: egui::Pos2, x_offset: f32) -> bool
//         // Used by systems/input.rs to check if click is inside menu panel
//   }
//
// USES (imports from):
//   egui (external)  → Rect, Vec2, Pos2
//
// USED BY:
//   hud/mod.rs       → HudLayout owned by Hud, passed to panels
//   hud/animation.rs → menu_panel_animated() uses menu_x spring value
//   crates/gargantua-app/src/systems/input.rs
//     → is_point_in_menu() to suppress camera drag when clicking menu
//
// NOTE FOR AI:
//   All sizes are in logical pixels (not physical). DPI scaling is
//   applied internally so callers use device-independent values.
//   Menu panel width: 320 logical pixels (fixed, not resizable).
//   On 4K displays (dpi=2.0): physical menu width = 640px.
//   Recalculate on every winit WindowResized event, not every frame.
// ============================================================