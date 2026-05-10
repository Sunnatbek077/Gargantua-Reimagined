// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/camera_tab.rs
// LINES: ~300
// CATEGORY: UI — Camera controls tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Controls camera position, FOV, camera mode (orbit/free-flight/
//   gravity), camera speed, and relativistic effects (aberration,
//   β parameter). Also shows current camera position and velocity
//   in Boyer-Lindquist coordinates.
//
// CONTENTS (~300 lines):
//   pub struct CameraTab {
//       pub fov_deg:       f32,    // field of view [30°–120°]
//       pub cam_speed:     f32,    // movement speed multiplier [0.1–10.0]
//       pub cam_beta:      f32,    // camera velocity β = v/c [0.0–0.99]
//       pub mode:          CameraMode,
//       pub show_aberration: bool, // relativistic aberration effect on/off
//       pub show_time_warp:  bool, // gravitational time warp on/off
//
//       // Display-only (read from PhysicsState.camera)
//       cam_r:    f64,   // camera r in geometrized units
//       cam_theta:f64,   // camera θ
//       cam_phi:  f64,   // camera φ
//       cam_z:    f64,   // gravitational redshift z at camera
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum CameraMode {
//       Orbit,      // orbit around BH at fixed r
//       FreeFlight, // WASD free movement, no gravity
//       Gravity,    // real geodesic motion (CPU RK4 camera path)
//   }
//
//   impl CameraTab {
//       pub fn new() -> Self
//
//       pub fn draw(&mut self, ui: &mut egui::Ui, app_state: &mut AppState,
//           physics: &PhysicsState, search: &SearchBar, i18n: &I18n)
//         // Section: "Camera Mode"
//         //   Radio: Orbit | Free Flight | Gravity
//         //
//         // Section: "Camera Settings"
//         //   Slider: FOV [30–120°]
//         //   Slider: Speed [0.1–10×]
//         //   Slider: β (velocity, free-flight only) [0.0–0.99]
//         //   Toggle: Show relativistic aberration
//         //   Toggle: Show gravitational time warp
//         //
//         // Section: "Camera Position" (read-only)
//         //   r = {cam_r} M,  θ = {cam_theta}°,  φ = {cam_phi}°
//         //   Redshift z = {cam_z:.4}
//         //
//         // Button: "Reset Camera" → emit AppEvent::CameraReset
//   }
//
// USES (imports from):
//   gargantua_app::state::{AppState, PhysicsState, AppEvent, CameraMode}
//   gargantua_physics::effects::redshift::gravitational_redshift
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.camera
//
// NOTE FOR AI:
//   β slider (cam_beta) is only active in FreeFlight mode.
//   In Orbit and Gravity modes, β is computed from physics (not user-set).
//   CameraMode::Gravity triggers Rk4Integrator in gargantua-camera crate.
//   FOV change: emit AppEvent::CameraFovChanged(fov_deg).
//   Reset camera: set cam_r=20M, cam_theta=π/2, cam_phi=0.
// ============================================================