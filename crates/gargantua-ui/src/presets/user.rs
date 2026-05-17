// ============================================================
// FILE: crates/gargantua-ui/src/presets/user.rs
// LINES: ~240
// CATEGORY: UI — User preset store (save/load from disk)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Manages user-created presets: save, load, delete, rename,
//   and list. Presets are stored as individual .toml files in the
//   app data directory. Loaded into memory at app startup.
//
// CONTENTS (~240 lines):
//   pub struct UserPresetStore {
//       presets:  Vec<PresetSchema>,   // all loaded user presets
//       data_dir: std::path::PathBuf,  // path to presets directory
//   }
//
//   impl UserPresetStore {
//       // Load all .toml files from data_dir/presets/
//       pub fn load_from_disk(data_dir: &std::path::Path) -> Self
//         // Reads all *.toml files, parses with PresetSchema::from_toml()
//         // Invalid/corrupt files are skipped (logged as warning)
//
//       // Save a preset to disk as {name}.toml
//       // Overwrites if preset with same name exists
//       pub fn save(&mut self, schema: PresetSchema) -> std::io::Result<()>
//         // Sanitizes filename: spaces→underscores, remove special chars
//         // Writes schema.to_toml() to data_dir/presets/{name}.toml
//         // Updates self.presets in memory
//
//       // Delete a preset by name
//       pub fn delete(&mut self, name: &str) -> std::io::Result<()>
//         // Removes from disk and from self.presets
//
//       // Rename a preset
//       pub fn rename(&mut self, old: &str, new: &str) -> std::io::Result<()>
//         // Renames file on disk, updates PresetSchema.name field
//
//       // Get all preset names for UI listing
//       pub fn names(&self) -> Vec<&str>
//         // Returns names sorted alphabetically
//
//       // Get a preset schema by name
//       pub fn get(&self, name: &str) -> Option<&PresetSchema>
//
//       // Check if name conflicts with a builtin preset name
//       pub fn conflicts_with_builtin(name: &str) -> bool
//         // Returns true if name matches any BuiltinPreset::name()
//
//       // Export a preset to an arbitrary file path
//       pub fn export_to_path(schema: &PresetSchema, path: &std::path::Path) -> std::io::Result<()>
//
//       // Import a preset from an arbitrary file path
//       pub fn import_from_path(&mut self, path: &std::path::Path) -> Result<(), String>
//   }
//
// USES (imports from):
//   crate::presets::schema::PresetSchema
//   crate::presets::builtin::BuiltinPreset  → conflicts_with_builtin
//   std::{fs, path, io}
//
// USED BY:
//   crates/gargantua-ui/src/menu/tabs/export_tab.rs
//     → calls save(), delete(), names(), get()
//   crates/gargantua-app/src/state/sim_state.rs
//     → UserPresetStore initialized at startup with app data dir
//
// NOTE FOR AI:
//   data_dir on macOS: ~/Library/Application Support/gargantua/presets/
//   data_dir on Windows: %APPDATA%\gargantua\presets\
//   Use dirs::data_dir() (external crate) to get the base path.
//   Filename sanitization: replace ' '→'_', remove chars not in [a-zA-Z0-9_-].
//   Max preset name length: 64 characters.
//   conflicts_with_builtin() prevents user from shadowing built-in names.
// ============================================================