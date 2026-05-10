// ============================================================
// FILE: crates/gargantua-ui/src/hud/animation.rs
// LINES: ~220
// CATEGORY: UI — HUD panel slide-in/slide-out animations
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Smooth spring-based animations for UI panel open/close,
//   tab transitions, and overlay fade-in/fade-out.
//   Uses a simple critically-damped spring for all transitions
//   (no external animation library dependency).
//
// CONTENTS (~220 lines):
//   pub struct HudAnimator {
//       pub menu_x:      SpringVal,  // menu panel x offset (0=open, -300=closed)
//       pub menu_alpha:  SpringVal,  // menu panel opacity (0.0–1.0)
//       pub overlay_alpha: SpringVal,// physics readout opacity
//       pub tab_alpha:   SpringVal,  // tab content cross-fade
//   }
//
//   // Critically-damped spring: value tracks target smoothly
//   pub struct SpringVal {
//       pub value:    f32,   // current animated value
//       pub target:   f32,   // desired end value
//       pub velocity: f32,   // current velocity (internal)
//       pub stiffness:f32,   // spring stiffness k (default 200.0)
//       pub damping:  f32,   // damping coefficient (default 2*sqrt(k) for critical)
//   }
//
//   impl SpringVal {
//       pub fn new(initial: f32, stiffness: f32) -> Self
//       pub fn set_target(&mut self, target: f32)
//
//       // Advance spring by dt seconds
//       pub fn tick(&mut self, dt: f32)
//         // a = -stiffness * (value - target) - damping * velocity
//         // velocity += a * dt
//         // value    += velocity * dt
//
//       pub fn is_settled(&self) -> bool
//         // abs(value - target) < 0.001 && abs(velocity) < 0.001
//   }
//
//   impl HudAnimator {
//       pub fn new() -> Self
//
//       // Advance all springs by dt
//       pub fn tick(&mut self, dt: f32)
//
//       // Open/close menu with animation
//       pub fn open_menu(&mut self)   // menu_x.set_target(0.0)
//       pub fn close_menu(&mut self)  // menu_x.set_target(-320.0)
//
//       // Fade overlay in/out
//       pub fn show_overlay(&mut self) // overlay_alpha.set_target(1.0)
//       pub fn hide_overlay(&mut self) // overlay_alpha.set_target(0.0)
//   }
//
// USES (imports from):
//   No internal crate imports. Pure math (spring physics).
//
// USED BY:
//   hud/mod.rs  → HudAnimator owned by Hud, ticked each frame
//   hud/layout.rs → reads menu_x.value for panel position offset
//   hud/visibility.rs → calls open_menu/close_menu on toggle
//
// NOTE FOR AI:
//   Spring stiffness=200, critical damping=2*sqrt(200)≈28.3.
//   dt comes from egui's input.predicted_dt (usually 1/60 s).
//   Do NOT use egui's built-in animation (ctx.animate_bool) —
//   the spring system gives more control over feel.
//   All spring values are f32 (GPU-ready, no f64 here).
// ============================================================