// ============================================================
// FILE: crates/gargantua-ui/src/lib.rs
// LINES: ~60
// CATEGORY: UI — Crate root and public interface
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Public interface of the gargantua-ui crate.
//   Declares all sub-modules and re-exports the top-level types
//   that external crates need. Contains crate-level attributes.
//
// CONTENTS (~60 lines):
//   #![deny(unsafe_code)]
//   #![warn(missing_docs)]
//
//   pub mod accessibility;
//   pub mod hud;
//   pub mod i18n;
//   pub mod menu;
//   pub mod overlay;
//   pub mod presets;
//   pub mod widgets;
//   pub mod errors;
//   pub mod shortcuts;
//   pub mod theme;
//
//   // Top-level re-exports for convenience:
//   pub use hud::Hud;
//   pub use i18n::{I18n, Language};
//   pub use theme::Theme;
//   pub use presets::{PresetSchema, UserPresetStore};
//
// USES (imports from):
//   All sub-modules declared above
//
// USED BY:
//   crates/gargantua-app/src/app.rs
//     → use gargantua_ui::{Hud, I18n, Theme}
//
// NOTE FOR AI:
//   This file is declarations + re-exports only. No logic here.
//   New sub-modules must be registered here with `pub mod`.
//   crate root attribute #![deny(unsafe_code)] applies to
//   the entire gargantua-ui crate — never add `unsafe` blocks here.
// ============================================================