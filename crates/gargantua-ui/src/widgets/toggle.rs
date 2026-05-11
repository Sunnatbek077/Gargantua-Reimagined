// ============================================================
// FILE: crates/gargantua-ui/src/widgets/toggle.rs
// LINES: ~160
// CATEGORY: UI — Custom animated toggle switch widget
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   iOS-style animated toggle switch. Replaces egui's default checkbox
//   with a smooth sliding toggle for boolean parameters.
//   Used throughout the tabs for on/off settings (jet_on, disk_visible,
//   bloom_enabled, show_aberration, etc.).
//
// CONTENTS (~160 lines):
//   pub struct Toggle<'a> {
//       label:  &'a str,
//       value:  &'a mut bool,
//       tooltip:Option<&'a str>,
//   }
//
//   impl<'a> Toggle<'a> {
//       pub fn new(label: &'a str, value: &'a mut bool) -> Self
//       pub fn tooltip(mut self, t: &'a str) -> Self
//
//       // Draw toggle and return true if state changed this frame
//       pub fn draw(self, ui: &mut egui::Ui, search: &SearchBar) -> bool
//         // If search.matches(label) == false → skip
//         // Layout: [Toggle switch 40×22px] [Label]
//         //
//         // Toggle switch drawing:
//         //   Background: rounded rect 40×22px
//         //     OFF: dark gray (#374151)
//         //     ON:  accent blue (#4D9DE0)
//         //   Circle knob: 18×18px, white
//         //     OFF: left position (x = track_left + 2)
//         //     ON:  right position (x = track_right - 20)
//         //   Animation: knob_x animates with spring (stiffness=300)
//         //     SpringVal stored as egui::Memory::data (keyed by widget Id)
//         //
//         // Click anywhere on toggle → toggle *value, return true
//   }
//
// USES (imports from):
//   egui  → Ui, Painter, Rect, Color32, Response, Id, Memory
//   crate::menu::search::SearchBar
//
// USED BY:
//   menu/tabs/accretion_tab.rs  → jet_on, disk_visible, inner_glow
//   menu/tabs/camera_tab.rs     → show_aberration, show_time_warp
//   menu/tabs/postfx_tab.rs     → bloom_enabled, hdr_enabled
//   menu/tabs/export_tab.rs     → include_ui screenshot option
//
// NOTE FOR AI:
//   Animation state (knob_x SpringVal) stored in egui::Memory::data
//   keyed by egui::Id::new(label) to persist across frames without
//   requiring ownership in the caller struct.
//   Toggle width=40px, height=22px, knob diameter=18px.
//   Background color transition is also animated (linear interpolation
//   between dark gray and accent blue based on knob_x.value position).
// ============================================================