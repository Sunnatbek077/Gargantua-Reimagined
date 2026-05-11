// ============================================================
// FILE: crates/gargantua-ui/tests/preset.rs
// LINES: ~260
// CATEGORY: Integration test — Preset save/load/validate
// RUN: cargo test --package gargantua-ui --test preset
// ============================================================
//
// PURPOSE:
//   Validates the preset system end-to-end: schema serialization,
//   built-in preset correctness, user preset store CRUD operations,
//   validation logic, and backward compatibility of schema versioning.
//
// TESTED FUNCTIONS:
//   PresetSchema::validate()
//   PresetSchema::to_toml() / from_toml()
//   BuiltinPreset::to_schema()
//   BuiltinPreset::all()
//   UserPresetStore::save(), load_from_disk(), delete(), rename(), get()
//   UserPresetStore::conflicts_with_builtin()
//
// SETUP:
//   fn temp_dir() -> tempfile::TempDir
//     // Creates a temp directory for user preset store tests
//     // Cleaned up automatically when TempDir is dropped
//
// TEST CASES (~260 lines):
//
//   #[test]
//   fn test_preset_schema_roundtrip_toml()
//     // Create a PresetSchema with known values
//     // to_toml() → String → from_toml() → compare all fields
//     // assert_eq!(original.physics.spin, roundtripped.physics.spin)
//     // assert_eq!(original.name, roundtripped.name)
//
//   #[test]
//   fn test_preset_schema_validate_valid()
//     // PresetSchema::default_interstellar().validate() → Ok(())
//
//   #[test]
//   fn test_preset_schema_validate_invalid_spin()
//     // schema.physics.spin = 1.5 → validate() → Err containing "spin"
//
//   #[test]
//   fn test_preset_schema_validate_negative_mass()
//     // schema.physics.mass_solar = -1.0 → validate() → Err containing "mass"
//
//   #[test]
//   fn test_preset_schema_validate_fov_out_of_range()
//     // schema.camera.fov_deg = 200.0 → validate() → Err containing "fov"
//
//   #[test]
//   fn test_builtin_all_returns_all_variants()
//     // BuiltinPreset::all().len() == 6
//     // All variants present: InterstellarGargantua, M87Star, SgrAStar,
//     // Schwarzschild, ExtremalKerr, NeutronStarMerger
//
//   #[test]
//   fn test_builtin_interstellar_params()
//     // BuiltinPreset::InterstellarGargantua.to_schema():
//     //   spin ≈ 0.6, mass > 1e7, charge == 0.0
//     //   postfx.tonemap_mode == "aces"
//
//   #[test]
//   fn test_builtin_m87_params()
//     // BuiltinPreset::M87Star.to_schema():
//     //   mass ≈ 6.5e9, spin ≈ 0.9
//     //   accretion.jet_on == true
//     //   accretion.b_field > 1000.0
//
//   #[test]
//   fn test_builtin_schwarzschild_zero_spin()
//     // BuiltinPreset::Schwarzschild.to_schema().physics.spin == 0.0
//     // charge == 0.0
//
//   #[test]
//   fn test_builtin_extremal_spin_limit()
//     // BuiltinPreset::ExtremalKerr.to_schema().physics.spin ≈ 0.998
//     // spin must be < 1.0 (enforced by validate())
//
//   #[test]
//   fn test_builtin_all_validate()
//     // For every builtin preset: to_schema().validate() == Ok(())
//     // None of the built-in presets should fail validation
//
//   #[test]
//   fn test_user_store_save_and_load()
//     // let dir = temp_dir();
//     // let mut store = UserPresetStore::load_from_disk(dir.path()); // empty
//     // store.save(schema.clone()).unwrap();
//     // let store2 = UserPresetStore::load_from_disk(dir.path()); // reload
//     // assert_eq!(store2.get("MyPreset").unwrap().physics.spin, schema.physics.spin)
//
//   #[test]
//   fn test_user_store_delete()
//     // save → delete("MyPreset") → get("MyPreset") == None
//     // File should not exist in temp_dir after delete
//
//   #[test]
//   fn test_user_store_rename()
//     // save("Old") → rename("Old", "New") → get("New").is_some()
//     // get("Old") == None after rename
//
//   #[test]
//   fn test_user_store_names_sorted()
//     // Save presets: "Zebra", "Apple", "Mango"
//     // names() returns ["Apple", "Mango", "Zebra"] (sorted)
//
//   #[test]
//   fn test_user_store_conflicts_with_builtin()
//     // conflicts_with_builtin("M87*") == true
//     // conflicts_with_builtin("My Custom Preset") == false
//     // conflicts_with_builtin("Interstellar Gargantua") == true
//
//   #[test]
//   fn test_user_store_corrupt_file_skipped()
//     // Write a corrupt .toml file to temp_dir/presets/bad.toml
//     // load_from_disk() → does not panic, skips bad file
//     // store.presets.len() == 0 (only bad file, nothing valid)
//
//   #[test]
//   fn test_preset_filename_sanitization()
//     // Save preset with name "My Cool Preset 2025!"
//     // File should be: my_cool_preset_2025.toml (sanitized)
//     // assert!(temp_dir.path().join("presets/my_cool_preset_2025.toml").exists())
//
//   #[test]
//   fn test_preset_schema_version_field()
//     // PresetSchema::default_interstellar().version > 0
//     // Ensures version is set (used for future migration)
//
//   #[test]
//   fn test_export_import_roundtrip()
//     // export_to_path(&schema, &tmp_file) → import_from_path(&tmp_file)
//     // Imported schema fields match original
//
// USES (imports from):
//   gargantua_ui::presets::{PresetSchema, BuiltinPreset, UserPresetStore}
//   gargantua_ui::errors::UiError
//   tempfile::TempDir   // dev-dependency
//   std::fs
// ============================================================