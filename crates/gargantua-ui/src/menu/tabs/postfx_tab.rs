// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/postfx_tab.rs
// LINES: ~280
// CATEGORY: UI — Post-processing effects tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Controls all post-processing effects: bloom, tonemapping,
//   chromatic aberration, vignette, film grain, color grading,
//   and HDR/EDR settings for Mac Display P3. Changes are applied
//   in shaders/postfx/ pipeline.
//
// CONTENTS (~280 lines):
//   pub struct PostFxTab {
//       pub bloom_enabled:   bool,
//       pub bloom_threshold: f32,   // [0.0–2.0]
//       pub bloom_radius:    f32,   // [0.5–8.0]
//       pub bloom_intensity: f32,   // [0.0–3.0]
//
//       pub tonemap_mode:    TonemapMode,  // ACES / Reinhard / Filmic / None
//       pub exposure:        f32,   // EV exposure offset [-3.0–+3.0]
//       pub contrast:        f32,   // [0.5–2.0]
//
//       pub chromatic_ab:    f32,   // chromatic aberration strength [0.0–1.0]
//       pub vignette:        f32,   // vignette intensity [0.0–1.0]
//       pub film_grain:      f32,   // grain strength [0.0–0.5]
//
//       pub hdr_enabled:     bool,  // Mac EDR/Display P3 HDR
//       pub max_nits:        f32,   // peak brightness [100–1000 nits]
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum TonemapMode { Aces, Reinhard, Filmic, None }
//
//   impl PostFxTab {
//       pub fn new() -> Self
//       pub fn draw(&mut self, ui: &mut egui::Ui, app_state: &mut AppState,
//           search: &SearchBar, i18n: &I18n)
//         // Section: "Bloom"
//         //   Toggle + sliders: threshold, radius, intensity
//         // Section: "Tonemapping"
//         //   Radio: ACES | Reinhard | Filmic | None
//         //   Slider: Exposure, Contrast
//         // Section: "Lens Effects"
//         //   Sliders: Chromatic aberration, Vignette, Film grain
//         // Section: "HDR (Mac only)" — shown only on macOS
//         //   Toggle: Enable HDR
//         //   Slider: Peak brightness nits
//         // On change: emit AppEvent::PostFxChanged(PostFxParams)
//   }
//
// USES (imports from):
//   gargantua_app::state::{AppState, AppEvent}
//   gargantua_render::postfx::PostFxParams
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.postfx
//
// NOTE FOR AI:
//   HDR section is only shown on macOS (#[cfg(target_os = "macos")]).
//   Mac EDR uses Display P3 color space — set in wgpu surface config.
//   TonemapMode::None = linear passthrough (useful for HDR preview).
//   bloom_threshold in linear light units (not EV) — values above 1.0
//   are physically HDR (brighter than white).
// ============================================================