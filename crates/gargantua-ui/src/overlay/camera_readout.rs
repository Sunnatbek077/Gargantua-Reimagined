// ============================================================
// FILE: crates/gargantua-ui/src/overlay/camera_readout.rs
// LINES: ~180
// CATEGORY: UI — Camera position/velocity readout overlay
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Small overlay showing camera Boyer-Lindquist coordinates (r, θ, φ),
//   camera velocity (β = v/c), and gravitational time dilation.
//   Positioned below the physics_readout overlay. Used primarily
//   in free-flight and gravity camera modes.
//
// CONTENTS (~180 lines):
//   pub struct CameraReadout {
//       pub show_velocity: bool,  // show β row
//       pub show_coords:   bool,  // show r, θ, φ rows
//   }
//
//   impl CameraReadout {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &self,
//           ctx: &egui::Context,
//           physics: &PhysicsState,
//           layout: &HudLayout,
//           i18n: &I18n,
//       )
//         // egui::Area positioned below physics_readout overlay
//         // Row: "r = {r:.3} M  ({r_km:.0} km)"
//         // Row: "θ = {theta_deg:.2}°   φ = {phi_deg:.2}°"
//         // Row: "β = {beta:.4} c  (v = {v_km_s:.0} km/s)" if show_velocity
//         // Row: "τ/t = {time_dilation:.6}" (proper time rate)
//   }
//
// USES (imports from):
//   gargantua_app::state::PhysicsState
//   gargantua_physics::units::geom_to_meters
//   crate::hud::layout::HudLayout
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/overlay/mod.rs
//
// NOTE FOR AI:
//   r_km = geom_to_meters(r, mass_kg) / 1000.0  (convert to kilometers).
//   v_km_s = beta * c_si / 1000.0.
//   For β < 0.001: show v in m/s instead (more intuitive near-static).
//   This overlay is only useful in FreeFlight/Gravity camera modes —
//   in Orbit mode, r is fixed and the overlay is less informative.
// ============================================================