// ============================================================
// FILE: crates/gargantua-ui/src/accessibility/mod.rs
// LINES: ~40
// CATEGORY: UI — Accessibility module entry point
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Public interface of the accessibility sub-module.
//   Re-exports keyboard_nav so the rest of the UI crate can
//   import accessibility features from one place.
//
// CONTENTS (~40 lines):
//   pub mod keyboard_nav;
//   pub use keyboard_nav::KeyboardNav;
//
// USES (imports from):
//   keyboard_nav.rs
//
// USED BY:
//   crates/gargantua-ui/src/lib.rs  → pub mod accessibility
//   crates/gargantua-app/src/input/input_router.rs
//     → KeyboardNav used to intercept tab/arrow key events
//
// NOTE FOR AI:
//   This file is pure module declarations only.
//   Add new accessibility features as new files here,
//   then re-export via `pub use` in this mod.rs.
// ============================================================