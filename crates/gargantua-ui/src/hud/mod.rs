// ============================================================
// FILE: crates/gargantua-ui/src/hud/mod.rs
// LINES: ~260
// CATEGORY: UI — Main HUD coordinator (root UI frame)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Root HUD struct that owns all UI sub-systems and drives the
//   main egui frame each tick. Coordinates menu panel, overlay
//   widgets, animation state, layout, and visibility toggling.
//   Called once per frame by gargantua-app.
//
// CONTENTS (~260 lines):
//   pub mod animation;
//   pub mod layout;
//   pub mod toggle_button;
//   pub mod visibility;
//
//   pub struct Hud {
//       pub menu:        crate::menu::MenuPanel,
//       pub overlay:     crate::overlay::OverlaySet,
//       pub anim:        animation::HudAnimator,
//       pub layout:      layout::HudLayout,
//       pub toggle:      toggle_button::ToggleButton,
//       pub visibility:  visibility::VisibilityState,
//       pub kb_nav:      crate::accessibility::KeyboardNav,
//   }
//
//   impl Hud {
//       pub fn new(ctx: &egui::Context) -> Self
//         // Initializes all sub-systems, loads fonts, sets egui style
//
//       // Main per-frame entry point — called by gargantua-app each tick
//       pub fn draw(
//           &mut self,
//           ctx: &egui::Context,
//           physics: &PhysicsState,       // read-only physics values for display
//           render_stats: &RenderStats,   // FPS, GPU time, etc.
//           app_state: &mut AppState,     // mutable: responds to UI events
//       )
//         // 1. kb_nav.process(ctx)
//         // 2. anim.tick(dt)
//         // 3. If visibility.menu_visible: draw menu panel (left side)
//         // 4. overlay.draw(ctx, physics, render_stats)
//         // 5. toggle.draw(ctx, &mut visibility)
//         // 6. kb_nav.draw_focus_ring(painter)
//
//       // Handle window resize — recalculates HudLayout
//       pub fn on_resize(&mut self, new_size: egui::Vec2)
//   }
//
// USES (imports from):
//   animation.rs       → HudAnimator
//   layout.rs          → HudLayout
//   toggle_button.rs   → ToggleButton
//   visibility.rs      → VisibilityState
//   crate::menu        → MenuPanel
//   crate::overlay     → OverlaySet
//   crate::accessibility → KeyboardNav
//   gargantua_app::state::{PhysicsState, RenderStats, AppState}
//
// USED BY:
//   crates/gargantua-app/src/app.rs
//     → hud.draw(ctx, &physics, &render_stats, &mut app_state) each frame
//
// NOTE FOR AI:
//   Hud::draw() is the single egui render call per frame.
//   All egui panels (menu, overlays, toggle) are drawn here in order.
//   Do NOT call egui::CentralPanel or TopBottomPanel outside this file.
//   PhysicsState is read-only here — mutations go through AppState events.
// ============================================================