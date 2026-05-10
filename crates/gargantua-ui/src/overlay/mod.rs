// ============================================================
// FILE: crates/gargantua-ui/src/overlay/mod.rs
// LINES: ~120
// CATEGORY: UI — Overlay set coordinator
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Owns and coordinates all overlay widgets that float on top of
//   the render view (not inside the menu panel). Checks visibility
//   flags before drawing each overlay. Drawn last so overlays appear
//   on top of everything else.
//
// CONTENTS (~120 lines):
//   pub mod bake_progress;
//   pub mod camera_readout;
//   pub mod crosshair;
//   pub mod physics_readout;
//   pub mod render_progress;
//   pub mod stats_bar;
//
//   pub struct OverlaySet {
//       pub bake_progress:   bake_progress::BakeProgress,
//       pub camera_readout:  camera_readout::CameraReadout,
//       pub crosshair:       crosshair::Crosshair,
//       pub physics_readout: physics_readout::PhysicsReadout,
//       pub render_progress: render_progress::RenderProgress,
//       pub stats_bar:       stats_bar::StatsBar,
//   }
//
//   impl OverlaySet {
//       pub fn new() -> Self
//
//       pub fn draw(
//           &mut self,
//           ctx: &egui::Context,
//           physics: &PhysicsState,
//           render_stats: &RenderStats,
//           visibility: &VisibilityState,
//           layout: &HudLayout,
//       )
//         // Draw each overlay only if visibility flag is true:
//         // if visibility.physics_readout → physics_readout.draw(...)
//         // if visibility.stats_bar       → stats_bar.draw(...)
//         // if visibility.crosshair       → crosshair.draw(...)
//         // if visibility.camera_readout  → camera_readout.draw(...)
//         // if visibility.bake_progress   → bake_progress.draw(...)
//         // if visibility.render_progress → render_progress.draw(...)
//   }
//
// USES (imports from):
//   All overlay sub-modules
//   crate::hud::visibility::VisibilityState
//   crate::hud::layout::HudLayout
//   gargantua_app::state::{PhysicsState, RenderStats}
//   egui
//
// USED BY:
//   crates/gargantua-ui/src/hud/mod.rs  → OverlaySet owned by Hud
//
// NOTE FOR AI:
//   Overlays use egui::Area (floating, no frame) — not egui::Panel.
//   Draw order matters: overlays drawn last appear on top of the render.
//   bake_progress and render_progress are auto-shown by AppState events,
//   not user-toggled — visibility flags for these are managed by app.rs.
// ============================================================