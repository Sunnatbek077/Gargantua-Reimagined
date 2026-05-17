// ============================================================
// FILE: crates/gargantua-ui/src/presets/mod.rs
// LINES: ~60
// CATEGORY: UI — Presets module entry point
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Public interface for the presets sub-module.
//   Re-exports PresetSchema, BuiltinPreset, and UserPresetStore
//   so other crates import presets from one place.
//
// CONTENTS (~60 lines):
//   pub mod builtin;
//   pub mod schema;
//   pub mod user;
//
//   pub use schema::PresetSchema;
//   pub use builtin::BuiltinPreset;
//   pub use user::UserPresetStore;
//
// USES (imports from):
//   builtin.rs  → BuiltinPreset enum + data
//   schema.rs   → PresetSchema struct (serializable parameter bundle)
//   user.rs     → UserPresetStore (load/save from disk)
//
// USED BY:
//   crates/gargantua-ui/src/lib.rs          → pub mod presets
//   crates/gargantua-ui/src/menu/tabs/export_tab.rs
//     → preset list display, save/load buttons
//   crates/gargantua-app/src/state/sim_state.rs
//     → UserPresetStore serialized to app data directory
//
// NOTE FOR AI:
//   This file is pure re-exports — no logic here.
//   Preset = a named bundle of all adjustable parameters
//   (physics, accretion, camera, postfx) saved as a TOML file.
// ============================================================