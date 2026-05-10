// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/render_tab.rs
// LINES: ~240
// CATEGORY: UI — Offline render settings tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Controls offline high-quality render settings: resolution,
//   samples per pixel, ray march steps, output format, and
//   triggers the render. Progress shown in overlay/render_progress.rs.
//
// CONTENTS (~240 lines):
//   pub struct RenderTab {
//       pub width:          u32,   // output width  [720–7680]
//       pub height:         u32,   // output height [480–4320]
//       pub spp:            u32,   // samples per pixel [1–256]
//       pub ray_steps:      u32,   // ray march steps [64–2048]
//       pub output_format:  OutputFormat,  // PNG / EXR / JPEG
//       pub output_path:    String,
//       pub rendering:      bool,
//       pub last_render_s:  Option<f32>,   // seconds for last render
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum OutputFormat { Png, Exr, Jpeg }
//
//   impl RenderTab {
//       pub fn new() -> Self
//       pub fn draw(&mut self, ui: &mut egui::Ui, app_state: &mut AppState,
//           search: &SearchBar, i18n: &I18n)
//         // Section: "Resolution"
//         //   Combo: Presets (1080p, 4K, 8K, Custom)
//         //   If Custom: width × height text inputs
//         //
//         // Section: "Quality"
//         //   Slider: SPP [1–256] + estimated render time display
//         //   Slider: Ray march steps [64–2048]
//         //
//         // Section: "Output"
//         //   Radio: PNG | EXR | JPEG
//         //   Text: output path + Browse button
//         //
//         // Button: "Render" → emit AppEvent::StartRender(RenderParams)
//         // If rendering: progress bar + "Cancel" button
//   }
//
// USES (imports from):
//   gargantua_render::RenderParams
//   gargantua_app::state::{AppState, AppEvent}
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//   rfd  → native file dialog
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.render
//
// NOTE FOR AI:
//   Estimated render time: spp * ray_steps * width * height * 2ns (rough).
//   EXR format preserves HDR float data — use for compositing pipelines.
//   PNG is 8-bit sRGB (tonemapped). JPEG is lossy 8-bit.
//   SPP > 64 triggers temporal accumulation mode in the render pipeline.
// ============================================================