// ============================================================
// FILE: crates/gargantua-ui/src/menu/tabs/export_tab.rs
// LINES: ~200
// CATEGORY: UI — Export/share settings tab
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Export options for sharing renders: screenshot (current viewport),
//   parameter preset save/load, and video export settings (frame
//   sequence for camera animation path renders).
//
// CONTENTS (~200 lines):
//   pub struct ExportTab {
//       pub screenshot_path:  String,
//       pub screenshot_format: ScreenshotFormat,  // PNG / EXR
//       pub include_ui:       bool,   // include HUD in screenshot
//
//       pub preset_name:      String,
//       pub preset_list:      Vec<String>,  // loaded from disk
//
//       pub video_fps:        u32,   // [24–120]
//       pub video_frames:     u32,   // total frames for animation
//       pub video_path:       String,
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum ScreenshotFormat { Png, Exr }
//
//   impl ExportTab {
//       pub fn new() -> Self
//       pub fn draw(&mut self, ui: &mut egui::Ui, app_state: &mut AppState,
//           search: &SearchBar, i18n: &I18n)
//         // Section: "Screenshot"
//         //   Radio: PNG | EXR
//         //   Toggle: Include HUD
//         //   Button: "Take Screenshot" → AppEvent::Screenshot(path, format)
//         //
//         // Section: "Presets"
//         //   List of saved presets (name → load button)
//         //   Text: preset name input + "Save Preset" button
//         //   → AppEvent::SavePreset(name) / AppEvent::LoadPreset(name)
//         //
//         // Section: "Video Export"
//         //   Slider: FPS, Total frames
//         //   Text: output path
//         //   Button: "Export Frame Sequence"
//   }
//
// USES (imports from):
//   gargantua_app::state::{AppState, AppEvent}
//   crate::menu::search::SearchBar
//   crate::i18n::I18n
//   egui
//   rfd  → file dialog
//
// USED BY:
//   crates/gargantua-ui/src/menu/mod.rs  → TabSet.export
//
// NOTE FOR AI:
//   Screenshots use wgpu texture readback — triggered via AppEvent,
//   not directly from this file. The UI only emits the event.
//   Preset files are stored in the app's data directory as .toml.
//   Video export produces a numbered frame sequence (frame_0001.png etc.)
//   — not a video file directly. Users run ffmpeg on the sequence.
// ============================================================