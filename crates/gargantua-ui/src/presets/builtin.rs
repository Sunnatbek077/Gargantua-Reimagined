// ============================================================
// FILE: crates/gargantua-ui/src/presets/builtin.rs
// LINES: ~280
// CATEGORY: UI — Built-in reference presets
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Defines built-in factory presets that ship with the application.
//   These are read-only (cannot be overwritten by user).
//   Provides scientifically inspired configurations for well-known
//   black holes and cinematic references.
//
// CONTENTS (~280 lines):
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum BuiltinPreset {
//       InterstellarGargantua,  // Film-accurate Gargantua (spin≈0.6, Thorne params)
//       M87Star,                // Real M87* parameters (spin≈0.9, mass=6.5e9 M_sun)
//       SgrAStar,               // SgrA* (spin≈0.6, mass=4.1e6 M_sun)
//       Schwarzschild,          // Non-rotating BH (spin=0, charge=0)
//       ExtremalKerr,           // Near-extremal spin (a=0.998)
//       NeutronStarMerger,      // Post-merger exotic params
//   }
//
//   impl BuiltinPreset {
//       pub fn all() -> &'static [BuiltinPreset]
//         // Returns slice of all 6 variants for UI listing
//
//       pub fn name(&self) -> &'static str
//         // "Interstellar Gargantua", "M87*", "SgrA*", etc.
//
//       pub fn description(&self) -> &'static str
//         // Short 1-2 sentence description of each preset
//
//       pub fn to_schema(&self) -> PresetSchema
//         // Returns the full PresetSchema for this built-in
//
//       pub fn is_readonly(&self) -> bool  // always true for BuiltinPreset
//   }
//
//   // PRESET DATA:
//   //
//   // InterstellarGargantua:
//   //   mass=1e8 M_sun, spin=0.6, charge=0
//   //   accretion_rate=0.1, r_outer=15M, jet_on=false
//   //   camera r=20M, theta=π/2 (equatorial plane view)
//   //   postfx: ACES tonemap, bloom_intensity=1.2
//   //
//   // M87Star:
//   //   mass=6.5e9 M_sun, spin=0.9, charge=0
//   //   accretion_rate=0.05, r_outer=20M, jet_on=true, b_field=1e4
//   //   camera r=25M, theta=π/3 (slightly above equator, like EHT view)
//   //
//   // SgrAStar:
//   //   mass=4.1e6 M_sun, spin=0.6, charge=0
//   //   accretion_rate=0.01 (low — SgrA* is dim), r_outer=12M
//   //
//   // Schwarzschild:
//   //   mass=1e8 M_sun, spin=0.0, charge=0
//   //   Shows symmetric lens flare (no frame dragging asymmetry)
//   //
//   // ExtremalKerr:
//   //   spin=0.998 (near-extremal), camera close in (r=3M)
//   //   ISCO at r≈1.23M, extreme disk asymmetry
//   //
//   // NeutronStarMerger:
//   //   spin=0.7, charge=0.3 (Kerr-Newman), exotic disk params
//
// USES (imports from):
//   crate::presets::schema::PresetSchema
//   crate::presets::schema::{PhysicsParams, AccretionParams, CameraParams, PostFxParams}
//
// USED BY:
//   crates/gargantua-ui/src/menu/tabs/export_tab.rs
//     → displays builtin preset list as read-only reference section
//   crates/gargantua-core/src/app.rs
//     → loads a BuiltinPreset on startup if no user settings found
//
// NOTE FOR AI:
//   BuiltinPreset::all() is used to populate the preset dropdown.
//   Built-in presets are NEVER written to disk — they are hardcoded.
//   InterstellarGargantua is the default startup preset.
//   M87* parameters reference: EHT Collaboration 2019 (ApJL 875, L1).
//   SgrA* parameters reference: EHT Collaboration 2022 (ApJL 930, L12).
// ============================================================