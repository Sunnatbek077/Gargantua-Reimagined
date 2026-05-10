// ============================================================
// FILE: crates/gargantua-ui/src/overlay/physics_readout.rs
// LINES: ~280
// CATEGORY: UI — Real-time physics readout overlay (top-right)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Semi-transparent overlay in the top-right corner showing live
//   physics values: black hole parameters, camera position, redshift,
//   frame dragging, ergosphere status, and Penrose efficiency.
//   Updates every frame from PhysicsState.
//
// CONTENTS (~280 lines):
//   pub struct PhysicsReadout {
//       pub compact_mode: bool,  // compact (1 column) vs full (2 columns)
//       pub bg_alpha:     f32,   // background opacity [0.0–1.0], default 0.6
//   }
//
//   impl PhysicsReadout {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &self,
//           ctx: &egui::Context,
//           physics: &PhysicsState,
//           layout: &HudLayout,
//           i18n: &I18n,
//       )
//         // egui::Area anchored to layout.overlay_tl (top-right corner)
//         // Semi-transparent dark background frame
//         //
//         // Row: "M = {mass:.2e} M☉  |  a = {spin:.4}  |  Q = {charge:.3}"
//         // Row: "r_ISCO = {r_isco:.2} M  |  r_+ = {r_horizon:.2} M"
//         // Row: "r = {cam_r:.2} M  |  θ = {cam_theta:.1}°  |  φ = {cam_phi:.1}°"
//         // Row: "z = {redshift:.4}  |  τ/t = {time_dilation:.4}"
//         // Row: "Ω_FD = {omega_fd:.4} rad/M  (frame dragging)"
//         // Row: if in_ergosphere: "⚠ IN ERGOSPHERE" (yellow warning)
//         // Row: "Penrose η = {penrose_eff:.1}%"
//         //
//         // Compact mode: only mass, spin, redshift, cam_r (single column)
//
//       fn format_mass(mass_solar: f64) -> String
//         // "1.23e9 M☉" or "6.50 × 10⁹ M☉" depending on magnitude
//   }
//
// USES (imports from):
//   gargantua_app::state::PhysicsState
//   gargantua_physics::effects::penrose::in_ergosphere
//   crate::hud::layout::HudLayout
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs  → PhysicsReadout in OverlaySet
//
// NOTE FOR AI:
//   Uses egui::Area with anchor = TopRight, not a Panel.
//   bg_alpha=0.6 by default: dark semi-transparent (#111827 at 60% opacity).
//   Values come from PhysicsState (computed by gargantua-app each frame).
//   in_ergosphere warning: show in yellow/orange, blink at 1Hz if true.
//   compact_mode toggled via context menu (right-click on the overlay).
// ============================================================