// ============================================================
// FILE: crates/gargantua-ui/src/menu/search.rs
// LINES: ~180
// CATEGORY: UI — Control search bar within the menu panel
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Search bar that filters visible controls within the active tab.
//   User types a query (e.g., "spin") and only matching sliders/
//   toggles are shown. Highlights matching text in control labels.
//   Shortcut: Ctrl+F / Cmd+F to focus the search bar.
//
// CONTENTS (~180 lines):
//   pub struct SearchBar {
//       pub query:       String,      // current search text
//       pub is_focused:  bool,
//       pub result_count:usize,       // number of matching controls
//   }
//
//   impl SearchBar {
//       pub fn new() -> Self
//
//       // Draw search input field, return current query string
//       pub fn draw(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> &str
//         // egui TextEdit with placeholder i18n.t("menu.search_placeholder")
//         // Clear button (✕) on the right when query is non-empty
//         // Ctrl+F / Cmd+F → focus this widget
//
//       // Check if a control label matches the current query
//       // Used by tabs to decide whether to show a control
//       pub fn matches(&self, label: &str) -> bool
//         // Case-insensitive substring match
//         // Empty query → always true (show all)
//
//       // Highlight matching substring in label text
//       // Returns egui LayoutJob with highlighted span
//       pub fn highlight(&self, label: &str) -> egui::text::LayoutJob
//
//       pub fn clear(&mut self)
//       pub fn is_active(&self) -> bool  // query.is_empty() == false
//   }
//
// USES (imports from):
//   egui              → Ui, TextEdit, text::LayoutJob
//   crate::i18n::I18n
//
// USED BY:
//   menu/mod.rs       → SearchBar owned by MenuPanel, drawn below tab bar
//   menu/tabs/*.rs    → each tab calls search.matches(label) per control
//
// NOTE FOR AI:
//   matches() is case-insensitive: "Spin", "spin", "SPIN" all match "spin".
//   Empty query (query.is_empty()) → matches() always returns true.
//   highlight() uses egui's LayoutJob with a yellow background span
//   on the matched portion of the label string.
//   Ctrl+F must check !ctx.wants_keyboard_input() to avoid
//   consuming the shortcut while user types in another field.
// ============================================================