// ============================================================
// FILE: crates/gargantua-ui/src/errors.rs
// LINES: ~80
// CATEGORY: UI — UI crate error types
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines UiError — the single error enum for the gargantua-ui
//   crate. Covers preset I/O errors, theme loading failures,
//   and i18n missing translation key warnings.
//
// CONTENTS (~80 lines):
//   #[derive(Debug, thiserror::Error)]
//   pub enum UiError {
//       #[error("preset not found: {0}")]
//       PresetNotFound(String),
//
//       #[error("preset parse error in '{file}': {source}")]
//       PresetParseError { file: String, source: toml::de::Error },
//
//       #[error("preset I/O error: {0}")]
//       PresetIo(#[from] std::io::Error),
//
//       #[error("theme file not found: {0}")]
//       ThemeNotFound(String),
//
//       #[error("i18n key missing: '{key}' in language '{lang}'")]
//       MissingTranslation { key: String, lang: String },
//   }
//
//   pub type UiResult<T> = Result<T, UiError>;
//
// USES (imports from):
//   thiserror (external)  → #[derive(Error)]
//   toml      (external)  → toml::de::Error
//   std::io               → std::io::Error
//
// USED BY:
//   presets/user.rs   → returns UiError::PresetIo, PresetParseError
//   theme.rs          → returns UiError::ThemeNotFound
//   i18n/mod.rs       → logs UiError::MissingTranslation (non-fatal)
//
// NOTE FOR AI:
//   MissingTranslation is non-fatal — i18n::t() logs a warning
//   and returns the key string itself (never panics).
//   PresetIo wraps std::io::Error via #[from] for ? operator.
//   UiResult<T> = Result<T, UiError> is the crate-wide fallible return type.
// ============================================================