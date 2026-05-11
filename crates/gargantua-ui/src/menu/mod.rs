// ============================================================
// FILE: crates/gargantua-ui/src/menu/mod.rs
// LINES: ~220
// CATEGORY: UI — Left side menu panel with tab bar
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   The main left-side control panel. Contains a tab bar at the top
//   (Physics, Accretion, Camera, PostFX, Bake, Render, Export) and
//   renders the active tab's content below. Scrollable if content
//   overflows. Width: 320 logical pixels.
//
// CONTENTS (~220 lines):
//   pub mod tabs;
//
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum ActiveTab {
//       Physics, Accretion, Camera, PostFx, Bake, Render, Export,
//   }
//
//   pub struct MenuPanel {
//       pub active_tab:    ActiveTab,
//       pub scroll_offset: f32,         // current vertical scroll position
//       pub search:        search::SearchBar,
//       tabs: TabSet,                   // holds all 7 tab structs
//   }
//
//   struct TabSet {
//       physics:   tabs::physics_tab::PhysicsTab,
//       accretion: tabs::accretion_tab::AccretionTab,
//       camera:    tabs::camera_tab::CameraTab,
//       postfx:    tabs::postfx_tab::PostFxTab,
//       bake:      tabs::bake_tab::BakeTab,
//       render:    tabs::render_tab::RenderTab,
//       export:    tabs::export_tab::ExportTab,
//   }
//
//   impl MenuPanel {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &mut self,
//           ui: &mut egui::Ui,
//           rect: egui::Rect,       // animated rect from HudLayout
//           app_state: &mut AppState,
//           physics: &PhysicsState,
//           i18n: &I18n,
//       )
//         // 1. Draw tab bar (icons + labels, 7 tabs)
//         // 2. Draw search bar below tab bar
//         // 3. Draw active tab content in scrollable area
//         // 4. Tab switch triggers scroll reset + animation
//
//       fn draw_tab_bar(&mut self, ui: &mut egui::Ui, i18n: &I18n)
//         // Horizontal strip of 7 icon+label tab buttons
//         // Highlighted tab = active_tab (bottom border accent color)
//
//       fn draw_active_tab(&mut self, ui: &mut egui::Ui, ...)
//         // match self.active_tab { Physics → tabs.physics.draw(), ... }
//   }
//
// USES (imports from):
//   tabs::*                     → all 7 tab structs
//   crate::menu::search         → SearchBar
//   crate::i18n::I18n
//   gargantua_app::state::{AppState, PhysicsState}
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/hud/mod.rs  → MenuPanel owned by Hud
//
// NOTE FOR AI:
//   Tab order in the UI (left to right): Physics, Accretion, Camera,
//   PostFX, Bake, Render, Export.
//   Tab icons are from the Lucide icon set (embedded as SVG paths).
//   search.rs filters visible controls within the active tab.
//   Scroll position resets to 0 when switching tabs.
// ============================================================