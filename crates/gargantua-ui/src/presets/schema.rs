// ============================================================
// FILE: crates/gargantua-ui/src/presets/schema.rs
// LINES: ~220
// CATEGORY: UI — Preset data schema (serializable parameter bundle)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines PresetSchema — the complete set of all user-adjustable
//   parameters that can be saved as a named preset.
//   Used for both built-in presets (builtin.rs) and user presets (user.rs).
//   Serialized to TOML via serde.
//
// CONTENTS (~220 lines):
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct PresetSchema {
//       pub name:        String,
//       pub description: String,
//       pub version:     u32,         // schema version for migration
//       pub physics:     PhysicsParams,
//       pub accretion:   AccretionParams,
//       pub camera:      CameraParams,
//       pub postfx:      PostFxParams,
//   }
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct PhysicsParams {
//       pub mass_solar:    f64,   // black hole mass in solar masses
//       pub spin:          f64,   // dimensionless spin a ∈ (-0.998, +0.998)
//       pub charge:        f64,   // electric charge Q
//   }
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct AccretionParams {
//       pub accretion_rate: f64,  // Eddington fraction [0.01–1.0]
//       pub r_outer:        f64,  // outer disk radius in M
//       pub beta:           f64,  // plasma β
//       pub b_field:        f64,  // magnetic field in Gauss
//       pub jet_on:         bool,
//       pub disk_visible:   bool,
//   }
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct CameraParams {
//       pub r:       f64,    // initial camera r in M
//       pub theta:   f64,    // initial camera θ (radians)
//       pub phi:     f64,    // initial camera φ (radians)
//       pub fov_deg: f32,
//       pub mode:    String, // "orbit" | "free_flight" | "gravity"
//   }
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct PostFxParams {
//       pub bloom_enabled:   bool,
//       pub bloom_intensity: f32,
//       pub tonemap_mode:    String, // "aces" | "reinhard" | "filmic" | "none"
//       pub exposure:        f32,
//       pub chromatic_ab:    f32,
//       pub vignette:        f32,
//   }
//
//   impl PresetSchema {
//       pub fn default_interstellar() -> Self
//         // Returns the "Interstellar Gargantua" reference preset
//
//       pub fn validate(&self) -> Result<(), String>
//         // Checks: |spin| < 1, mass > 0, fov ∈ [10, 170], etc.
//         // Returns Err with description if any field is out of range
//
//       pub fn to_toml(&self) -> String
//         // toml::to_string_pretty(self).unwrap()
//
//       pub fn from_toml(s: &str) -> Result<Self, toml::de::Error>
//         // toml::from_str(s)
//   }
//
// USES (imports from):
//   serde (external)  → Serialize, Deserialize
//   toml  (external)  → serialization
//
// USED BY:
//   presets/builtin.rs  → BuiltinPreset returns PresetSchema values
//   presets/user.rs     → UserPresetStore stores Vec<PresetSchema>
//   menu/tabs/export_tab.rs → save/load PresetSchema
//   crates/gargantua-app/src/app.rs
//     → applies PresetSchema to AppState on preset load
//
// NOTE FOR AI:
//   version field: increment when adding new fields to PresetSchema.
//   from_toml() should handle missing optional fields gracefully
//   (use #[serde(default)] on new fields for backward compatibility).
//   Do NOT use #[serde(rename)] — field names are part of the
//   user-visible .toml preset file format.
// ============================================================