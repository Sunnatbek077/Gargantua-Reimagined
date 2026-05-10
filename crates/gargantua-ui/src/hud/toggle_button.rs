// ============================================================
// FILE: crates/gargantua-ui/src/hud/toggle_button.rs
// LINES: ~160
// CATEGORY: UI — Hamburger menu toggle button
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   The small hamburger (☰) button in the top-left corner that
//   shows/hides the menu panel. Also handles keyboard shortcut
//   (H key or Tab to focus). Animates between ☰ and ✕ icons.
//
// CONTENTS (~160 lines):
//   pub struct ToggleButton {
//       pub is_open: bool,         // current open/closed state
//       pub hover_anim: SpringVal, // hover highlight scale (1.0..1.1)
//       pub icon_anim:  SpringVal, // icon rotation for ☰↔✕ morph (0.0..1.0)
//   }
//
//   impl ToggleButton {
//       pub fn new() -> Self
//
//       // Draw the button and handle click
//       // Returns true if state changed this frame
//       pub fn draw(
//           &mut self,
//           ctx: &egui::Context,
//           layout: &HudLayout,
//           visibility: &mut VisibilityState,
//           anim: &mut HudAnimator,
//       ) -> bool
//         // Draws 32×32 button at layout.toggle_btn
//         // Click or H key → toggle is_open
//         // Calls anim.open_menu() or anim.close_menu() on toggle
//         // Shows tooltip "Show/Hide Menu (H)" on hover
//
//       // Draw animated ☰/✕ icon using egui Painter lines
//       fn draw_icon(&self, painter: &egui::Painter, rect: egui::Rect)
//         // icon_anim.value: 0.0 = ☰ (3 horizontal lines)
//         //                  1.0 = ✕ (two diagonal lines)
//         // Interpolates between the two states
//   }
//
// USES (imports from):
//   hud/animation.rs   → HudAnimator, SpringVal
//   hud/layout.rs      → HudLayout
//   hud/visibility.rs  → VisibilityState
//   egui               → Context, Painter, Rect, Key
//
// USED BY:
//   hud/mod.rs  → toggle.draw() called every frame in Hud::draw()
//
// NOTE FOR AI:
//   Keyboard shortcut: H key (not Ctrl+H — simple H when not typing).
//   Must check egui::Context::wants_keyboard_input() = false before
//   consuming H keypress (don't intercept when user types in text field).
//   Icon animation: icon_anim.set_target(1.0) on open, 0.0 on close.
//   Button size: 32×32 logical pixels, always visible (not hidden by menu).
// ============================================================