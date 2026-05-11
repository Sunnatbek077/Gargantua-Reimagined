// ============================================================
// FILE: crates/gargantua-ui/src/shortcuts.rs
// LINES: ~180
// CATEGORY: UI — Global keyboard shortcut definitions and handler
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines all global keyboard shortcuts and processes them
//   each frame. Emits AppEvents when shortcuts are triggered.
//   Centralizes shortcut logic so it is not scattered across files.
//
// CONTENTS (~180 lines):
//   pub struct ShortcutHandler;
//
//   impl ShortcutHandler {
//       // Process all global shortcuts from egui input.
//       // Called once per frame BEFORE any UI drawing.
//       // Returns Vec<AppEvent> for each triggered shortcut.
//       pub fn process(ctx: &egui::Context) -> Vec<AppEvent>
//         // Only fires when !ctx.wants_keyboard_input() (not typing)
//         //
//         // Shortcuts:
//         //   H            → AppEvent::ToggleMenu
//         //   F1           → AppEvent::TogglePhysicsReadout
//         //   F2           → AppEvent::ToggleStatsBar
//         //   F3           → AppEvent::ToggleCrosshair
//         //   Ctrl/Cmd+F   → AppEvent::FocusSearch
//         //   Ctrl/Cmd+S   → AppEvent::SavePreset(current)
//         //   Ctrl/Cmd+Z   → AppEvent::Undo
//         //   Ctrl/Cmd+Y   → AppEvent::Redo
//         //   Ctrl/Cmd+R   → AppEvent::ResetCamera
//         //   F11 / Ctrl+Shift+F → AppEvent::ToggleFullscreen
//         //   Escape       → AppEvent::CancelActiveOperation
//         //   Space        → AppEvent::TogglePlayback (animation)
//         //   [ / ]        → AppEvent::PrevFrame / NextFrame
//   }
//
//   // All global shortcuts listed for display in a "Keyboard Shortcuts" dialog
//   pub struct ShortcutEntry {
//       pub keys:        &'static str,  // e.g. "Ctrl+S"
//       pub description: &'static str,  // e.g. "Save current preset"
//   }
//
//   pub fn all_shortcuts() -> &'static [ShortcutEntry]
//     // Returns full list for display in Help > Keyboard Shortcuts dialog
//
// USES (imports from):
//   egui         → Context, Key, Modifiers
//   gargantua_app::state::AppEvent
//
// USED BY:
//   crates/gargantua-ui/src/hud/mod.rs
//     → ShortcutHandler::process(ctx) called at start of Hud::draw()
//   crates/gargantua-app/src/app.rs
//     → processes returned Vec<AppEvent>
//
// NOTE FOR AI:
//   Mac: use Cmd (Modifiers::COMMAND), Windows/Linux: use Ctrl (Modifiers::CTRL).
//   Check platform: #[cfg(target_os = "macos")] Modifiers::COMMAND else Modifiers::CTRL.
//   All shortcuts MUST check !ctx.wants_keyboard_input() first.
//   Ctrl+Z/Ctrl+Y undo/redo: gargantua-app maintains an UndoStack.
//   Escape: cancels bake if baking, cancels render if rendering, else clears focus.
// ============================================================