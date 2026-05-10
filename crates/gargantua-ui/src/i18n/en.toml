// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/accretion_tab.rs
// LINES: ~340
// CATEGORY: UI — Accretion disk parameters tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Controls all accretion disk parameters: Novikov-Thorne model
//   settings, MHD turbulence, disk geometry, and jet toggle.
//   Displays computed peak temperature, luminosity, and a live
//   color swatch showing the disk's peak blackbody color.
//
// CONTENTS (~340 lines):
//   pub struct AccretionTab {
//       pub accretion_rate: f64,   // Ṁ in Eddington units [0.01–1.0]
//       pub r_outer:        f64,   // outer disk radius in M [5M–100M]
//       pub beta:           f64,   // plasma β for MHD [1–100]
//       pub b_field:        f64,   // magnetic field in Gauss [1–1e5]
//       pub jet_on:         bool,  // enable BZ jet visualization
//       pub disk_visible:   bool,  // show/hide accretion disk entirely
//       pub inner_glow:     bool,  // inner disk glow effect toggle
//
//       // Display-only computed values
//       peak_temp_k:    f64,     // peak disk temperature in Kelvin
//       luminosity_w:   f64,     // total disk luminosity in Watts
//       peak_color:     [f32;3], // linear sRGB of peak temperature
//   }
//
//   impl AccretionTab {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &mut self, ui: &mut egui::Ui,
//           app_state: &mut AppState,
//           physics: &PhysicsState,
//           search: &SearchBar, i18n: &I18n,
//       )
//         // Section: "Disk Model"
//         //   Slider: Accretion rate [0.01–1.0 Eddington, log scale]
//         //   Slider: Outer radius [5M–100M]
//         //   Toggle: Show disk
//         //
//         // Section: "MHD & Magnetic Field"
//         //   Slider: Plasma β [1–100, log scale]
//         //   Slider: B field [1–1e5 G, log scale]
//         //   Toggle: Enable jet
//         //
//         // Section: "Disk Properties" (read-only)
//         //   Peak temperature + color swatch
//         //   Luminosity (in solar luminosities)
//         //   Jet power (in Watts, if jet_on)
//         //   ISCO inner radius (synced from PhysicsTab)
//
//       fn update_derived(&mut self, physics: &PhysicsState)
//         // Calls NovikovThorneDisk::peak_temperature()
//         // Calls spectrum::blackbody_to_xyz() + xyz_to_srgb() for color swatch
//         // Calls MhdDisk::jet_power() if jet_on
//   }
//
// USES (imports from):
//   gargantua_physics::accretion::novikov_thorne::NovikovThorneDisk
//   gargantua_physics::accretion::mhd::MhdDisk
//   gargantua_physics::accretion::spectrum::{blackbody_to_xyz, xyz_to_srgb}
//   gargantua_app::state::{AppState, PhysicsState, AppEvent}
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.accretion
//
// NOTE FOR AI:
//   Accretion rate slider is LOG: displayed as "X% Eddington",
//   internal value = 10^slider_value (range: log10(0.01)=-2 to log10(1)=0).
//   Color swatch: 40×20px egui rect filled with peak_color (linear RGB).
//   Luminosity display: convert Watts → solar luminosities (L_sun=3.828e26 W).
//   On any change: emit AppEvent::AccretionChanged { ... }.
// ============================================================