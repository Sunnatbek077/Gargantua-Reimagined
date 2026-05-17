// ============================================================
// FILE: crates/gargantua-ui/src/hud/visibility.rs
// LINES: ~140
// CATEGORY: UI — Visibility state for all HUD panels
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Single struct holding show/hide flags for every HUD element.
//   Acts as the source of truth for what is currently visible.
//   Persisted to settings so visibility survives app restarts.
//
// CONTENTS (~140 lines):
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct VisibilityState {
//       pub menu_open:        bool,  // main left panel visible
//       pub physics_readout:  bool,  // top-right physics overlay
//       pub stats_bar:        bool,  // bottom stats bar (FPS, GPU ms)
//       pub crosshair:        bool,  // center crosshair
//       pub camera_readout:   bool,  // camera position/velocity overlay
//       pub bake_progress:    bool,  // LUT bake progress bar (auto-shown)
//       pub render_progress:  bool,  // render export progress (auto-shown)
//   }
//
//   impl VisibilityState {
//       pub fn default() -> Self
//         // All true except bake_progress=false, render_progress=false
//
//       pub fn toggle_menu(&mut self) { self.menu_open = !self.menu_open }
//       pub fn toggle_physics(&mut self) { self.physics_readout = !self.physics_readout }
//       pub fn toggle_stats(&mut self) { self.stats_bar = !self.stats_bar }
//
//       // Hide all overlays (presentation/screenshot mode)
//       pub fn hide_all_overlays(&mut self)
//
//       // Restore overlays to their last saved state
//       pub fn restore_overlays(&mut self, saved: &VisibilityState)
//   }
//
// USES (imports from):
//   serde (external) → Serialize, Deserialize (for settings persistence)
//
// USED BY:
//   hud/mod.rs         → VisibilityState owned by Hud
//   hud/toggle_button.rs → reads/writes menu_open
//   overlay/mod.rs     → checks each flag before drawing overlays
//   crates/gargantua-app/src/state/sim_state.rs
//     → serializes/deserializes VisibilityState to disk
//
// NOTE FOR AI:
//   bake_progress and render_progress are auto-managed:
//   they are set to true by their respective pipelines when active,
//   and set to false automatically when the operation completes.
//   Do NOT let users toggle them manually — they are status indicators.
//   VisibilityState is #[derive(serde)] — field names are part of
//   the settings file schema; do NOT rename fields without migration.
// ============================================================