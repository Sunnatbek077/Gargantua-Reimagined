// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/bake_tab.rs
// LINES: ~260
// CATEGORY: UI — LUT baking tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Controls the offline LUT (Look-Up Table) baking pipeline.
//   Users configure geodesic LUT resolution, spin range,
//   blackbody LUT size, and Doppler LUT size, then trigger
//   the bake. Progress is shown in overlay/bake_progress.rs.
//
// CONTENTS (~260 lines):
//   pub struct BakeTab {
//       pub geo_lut_spin_steps:   u32,  // spin axis resolution [8–256]
//       pub geo_lut_impact_steps: u32,  // impact parameter axis [64–4096]
//       pub geo_lut_rk4_steps:    u32,  // max RK4 steps per geodesic [1000–50000]
//       pub blackbody_lut_size:   u32,  // temperature LUT points [256–4096]
//       pub doppler_lut_n_beta:   u32,  // β axis size [64–512]
//       pub doppler_lut_n_lambda: u32,  // wavelength axis size [64–512]
//       pub output_dir:           String, // output path for .exr files
//       pub baking:               bool,   // true while bake is running
//       pub last_bake_duration:   Option<std::time::Duration>,
//   }
//
//   impl BakeTab {
//       pub fn new() -> Self
//
//       pub fn draw(&mut self, ui: &mut egui::Ui, app_state: &mut AppState,
//           search: &SearchBar, i18n: &I18n)
//         // Section: "Geodesic LUT"
//         //   Slider: Spin steps, Impact steps, RK4 steps
//         //   Estimated size display: "~{MB} MB on disk"
//         //
//         // Section: "Spectrum LUTs"
//         //   Slider: Blackbody LUT size, Doppler β steps, Doppler λ steps
//         //
//         // Section: "Output"
//         //   Text field: output directory path
//         //   Button: "Browse..." (opens native file dialog)
//         //
//         // Button: "Bake All LUTs" (disabled while baking=true)
//         //   → emit AppEvent::StartBake(BakeParams)
//         //
//         // If baking: show spinner + "Baking... (cancel)" button
//         // If last_bake_duration is Some: "Last bake: {secs}s"
//   }
//
// USES (imports from):
//   gargantua_bake::BakeParams
//   gargantua_app::state::{AppState, AppEvent}
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//   rfd (external crate)  → native file dialog for "Browse..."
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.bake
//
// NOTE FOR AI:
//   Estimated LUT size formula:
//     geo_lut = spin_steps * impact_steps * 3 * 4 bytes (f32 RGB)
//     blackbody_lut = size * 3 * 4 bytes
//     doppler_lut = n_beta * n_lambda * 4 bytes
//   While baking: disable the "Bake" button, show spinner animation.
//   baking flag is set to false when AppEvent::BakeComplete arrives.
//   rfd::FileDialog is async on some platforms — handle with channels.
// ============================================================