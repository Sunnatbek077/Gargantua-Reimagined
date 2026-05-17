// ============================================================
// FILE: crates/gargantua-ui/src/widgets/codec_selector.rs
// LINES: ~120 (planned)
// CATEGORY: UI — export codec picker widget
// PLATFORM: Mac + Windows + WASM
// ============================================================
//
// PURPOSE:
//   egui widget for choosing a video export codec (H.264, HEVC, AV1,
//   ProRes, EXR sequence, etc.). Lists only codecs that passed
//   gargantua_video::encode availability probing at startup.
//   Shows hardware vs software badge per entry.
//
// CONTENTS (planned):
//   pub struct CodecSelector { available: Vec<CodecEntry>, selected: usize }
//   pub fn show(&mut self, ui: &mut egui::Ui) -> Option<CodecId>
//   fn probe_codecs() -> Vec<CodecEntry>  — calls encode/mod.rs registry
//
// USES (imports from):
//   crate::widgets::tooltip.rs
//   crate::theme.rs
//   gargantua_video::encode — CodecId, probe_available()
//
// USED BY:
//   crates/gargantua-ui/src/menu/tabs/export_tab.rs
//     → export settings panel
//
// NOTE FOR AI:
//   Implementation not yet present — this file documents the intended API.
//   Do not confuse with gargantua-core platform/windows/video/* HAL probes;
//   those detect hardware; this widget only displays user-facing choices.
// ============================================================
