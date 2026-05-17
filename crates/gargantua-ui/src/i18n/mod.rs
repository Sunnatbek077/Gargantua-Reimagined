// ============================================================
// FILE: crates/gargantua-ui/src/i18n/mod.rs
// LINES: ~180
// CATEGORY: UI — Internationalization (i18n) system
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Loads and manages translation strings for the UI.
//   Supports three languages: English (en), Russian (ru), Uzbek (uz).
//   Translations are stored in TOML files (en.toml, ru.toml, uz.toml).
//   All UI text must go through this system — no hardcoded strings.
//
// CONTENTS (~180 lines):
//   #[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
//   pub enum Language { En, Ru, Uz }
//
//   pub struct I18n {
//       pub current: Language,
//       strings:     HashMap<String, String>,  // key → translated string
//   }
//
//   impl I18n {
//       // Load translation for given language from embedded TOML
//       // TOML files are embedded at compile time via include_str!()
//       pub fn new(lang: Language) -> Self
//
//       // Switch to a different language, reloads strings map
//       pub fn set_language(&mut self, lang: Language)
//
//       // Get a translation string by key
//       // Returns the key itself if missing (never panics)
//       pub fn t(&self, key: &str) -> &str
//
//       // Get translation with runtime substitution:
//       // t_fmt("physics.mass_label", &[("{value}", "6.5e9")])
//       pub fn t_fmt(&self, key: &str, subs: &[(&str, &str)]) -> String
//
//       // List of all available languages for UI language picker
//       pub fn available_languages() -> &'static [Language]
//   }
//
//   // Embed all TOML files at compile time:
//   const EN_TOML: &str = include_str!("en.toml");
//   const RU_TOML: &str = include_str!("ru.toml");
//   const UZ_TOML: &str = include_str!("uz.toml");
//
// USES (imports from):
//   toml (external crate)  → parse TOML into HashMap
//   std::collections::HashMap
//
// USED BY:
//   Every UI file in this crate that displays text:
//     menu/tabs/*.rs, overlay/*.rs, hud/mod.rs
//   crates/gargantua-app/src/state/sim_state.rs
//     → saves/loads Language enum to disk
//
// NOTE FOR AI:
//   Translation key convention: "module.widget_name"
//   Examples: "physics.spin_label", "camera.fov_label", "menu.bake_button"
//   t() never panics — returns the key string if translation missing.
//   When adding new UI text: add the key to ALL THREE .toml files.
//   en.toml is the canonical source; ru.toml and uz.toml are translations.
// ============================================================