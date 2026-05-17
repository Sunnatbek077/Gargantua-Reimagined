// ============================================================
// FILE: crates/gargantua-ui/src/accessibility/keyboard_nav.rs
// LINES: ~240
// CATEGORY: UI — Keyboard navigation for all UI panels
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Full keyboard navigation system for the egui-based UI.
//   Allows users to navigate menus, tabs, sliders, and toggles
//   without a mouse. Tab/Shift-Tab cycle focus, arrow keys adjust
//   sliders, Enter/Space activate buttons.
//
// CONTENTS (~240 lines):
//   pub struct KeyboardNav {
//       pub enabled:       bool,        // global on/off toggle
//       pub focused_id:    Option<egui::Id>, // currently focused widget
//       pub focus_ring:    bool,        // draw visible focus ring
//   }
//
//   impl KeyboardNav {
//       pub fn new() -> Self
//
//       // Process keyboard events from egui input state.
//       // Call once per frame before drawing UI.
//       pub fn process(&mut self, ctx: &egui::Context)
//         // Tab        → advance focus to next focusable widget
//         // Shift+Tab  → move focus to previous focusable widget
//         // Arrow keys → adjust focused slider by ±1 step
//         // Enter/Space→ activate focused button or toggle
//         // Escape     → clear focus
//
//       // Register a widget as focusable (call during UI build)
//       pub fn register(&mut self, id: egui::Id, rect: egui::Rect)
//
//       // Draw focus ring around focused widget (call after UI draw)
//       pub fn draw_focus_ring(&self, painter: &egui::Painter)
//         // Draws a 2px blue outline rect around focused_id's rect
//
//       // Returns true if the given widget id is currently focused
//       pub fn is_focused(&self, id: egui::Id) -> bool
//
//       // Advance focus forward or backward through the focus ring list
//       fn advance_focus(&mut self, forward: bool)
//
//       // Build ordered focus list from registered widgets (by y then x)
//       fn sorted_focus_list(&self) -> Vec<egui::Id>
//   }
//
// USES (imports from):
//   egui (external crate)  → Context, Id, Rect, Painter, Key
//
// USED BY:
//   crates/gargantua-ui/src/hud/mod.rs
//     → calls process() each frame, draw_focus_ring() after all UI
//   crates/gargantua-app/src/systems/input.rs
//     → checks is_focused() to suppress camera input when UI is focused
//
// NOTE FOR AI:
//   egui has its own focus system (egui::Memory::focus) but it is
//   limited for complex navigation. KeyboardNav supplements it.
//   focus_ring=true by default — disable only in presentation mode.
//   Tab order is spatial (top-to-bottom, left-to-right), not DOM order.
//   On Mac: Cmd+Tab is system-level (app switch) — do NOT intercept it.
// ============================================================