// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/physics_tab.rs
// LINES: ~380
// CATEGORY: UI — Physics tab (black hole parameters)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   The Physics tab controls all black hole parameters:
//   mass, spin, charge, and displays derived values (ISCO,
//   photon sphere, event horizon, ergosphere, Penrose efficiency).
//   Changes here propagate to gargantua-physics and trigger
//   GPU uniform buffer updates.
//
// CONTENTS (~380 lines):
//   pub struct PhysicsTab {
//       // Slider state (local copies, synced to AppState on change)
//       pub mass_solar:    f64,    // black hole mass in solar masses
//       pub spin:          f64,    // dimensionless spin a ∈ (-1, 1)
//       pub charge:        f64,    // electric charge Q (usually 0)
//       pub accretion_rate:f64,    // Ṁ in Eddington units
//
//       // Display-only computed values
//       isco_props:        Option<IscoProperties>,
//       redshift_at_cam:   f64,    // gravitational z at camera position
//       time_dilation:     f64,    // τ/t at camera position
//   }
//
//   impl PhysicsTab {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &mut self,
//           ui: &mut egui::Ui,
//           app_state: &mut AppState,
//           physics: &PhysicsState,
//           search: &SearchBar,
//           i18n: &I18n,
//       )
//         // Section: "Black Hole Parameters"
//         //   Slider: Mass [1e6 – 1e10 M_sun, log scale]
//         //   Slider: Spin [-0.998 – +0.998]
//         //   Slider: Charge [0.0 – 0.5]
//         //
//         // Section: "Derived Properties" (read-only display)
//         //   r_ISCO (in M and km), r_photon_sphere, r_horizon
//         //   Binding energy η%, orbital frequency Hz
//         //   Ergosphere radius at equator
//         //   Penrose efficiency %
//         //
//         // Section: "Relativistic Effects" (at camera position)
//         //   Gravitational redshift z
//         //   Time dilation τ/t
//         //   Frame dragging Ω_FD
//         //
//         // Each slider calls search.matches(label) before drawing
//         // On change: emit AppEvent::PhysicsChanged { mass, spin, charge }
//
//       fn update_derived(&mut self, physics: &PhysicsState)
//         // Calls compute_isco_properties(mass, spin) each frame
//         // Updates isco_props, redshift_at_cam, time_dilation
//   }
//
// USES (imports from):
//   gargantua_physics::accretion::isco::IscoProperties
//   gargantua_physics::accretion::isco::compute_isco_properties
//   gargantua_physics::effects::redshift::*
//   gargantua_physics::effects::penrose::penrose_efficiency
//   gargantua_physics::effects::frame_dragging::frame_drag_angular_velocity
//   gargantua_app::state::{AppState, PhysicsState, AppEvent}
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.physics
//
// NOTE FOR AI:
//   Mass slider is LOG scale (1e6 to 1e10 M_sun):
//   display = 10^slider_value, slider range [6.0, 10.0].
//   Spin clamped to ±0.998 (not ±1.0 — extremal Kerr is numerically bad).
//   On ANY parameter change: emit AppEvent::PhysicsChanged.
//   gargantua-app then rebuilds KerrNewman and uploads KerrGpuParams to GPU.
//   update_derived() runs every frame (not only on change) because
//   redshift_at_cam changes as the camera moves even with fixed physics.
// ============================================================