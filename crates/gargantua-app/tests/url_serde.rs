// ============================================================
// FILE: crates/gargantua-app/tests/url_serde.rs
// LINES: ~200
// CATEGORY: Integration test — URL-based preset serialization
// RUN: cargo test --package gargantua-app --test url_serde
// ============================================================
//
// PURPOSE:
//   Validates the URL serialization/deserialization of AppState
//   parameters. Allows users to share their black hole configuration
//   as a URL (e.g. https://gargantua.app/?spin=0.9&mass=6.5e9&r=20).
//   Tests: encode AppState → URL query string, decode URL → AppState,
//   invalid URL handling, and round-trip fidelity.
//
// TESTED FUNCTIONS (from crates/gargantua-app/src/url_state.rs):
//   UrlState::from_app_state(state: &AppState) -> String
//     // Encodes AppState fields as URL query parameters
//     // e.g. "?spin=0.9&mass=6.5e9&cam_r=20.0&fov=60&tonemap=aces"
//
//   UrlState::to_app_state(url: &str) -> Result<AppState, UrlError>
//     // Parses URL query string → AppState
//     // Unknown keys are ignored (forward compatibility)
//     // Invalid values → UrlError::InvalidParam { key, value }
//
//   UrlState::merge_into(url: &str, base: &mut AppState) -> Result<(), UrlError>
//     // Applies only the params present in the URL to base state
//     // Leaves unspecified fields at their current values
//
// KEY URL PARAMETER NAMES:
//   spin      → physics.spin         (f64, -0.998..+0.998)
//   mass      → physics.mass_solar   (f64, 1e6..1e10)
//   charge    → physics.charge       (f64, 0..0.5)
//   mdot      → accretion.accretion_rate (f64, 0.01..1.0)
//   r_outer   → accretion.r_outer    (f64)
//   cam_r     → camera.r             (f64)
//   cam_theta → camera.theta         (f64, radians)
//   cam_phi   → camera.phi           (f64, radians)
//   fov       → camera.fov_deg       (f32)
//   tonemap   → postfx.tonemap_mode  (string: aces/reinhard/filmic/none)
//   exposure  → postfx.exposure      (f32)
//   bloom     → postfx.bloom_enabled (bool: 1/0)
//   preset    → loads a BuiltinPreset by name (overrides other params)
//
// TEST CASES (~200 lines):
//
//   #[test]
//   fn test_encode_default_state()
//     // from_app_state(&AppState::default()) → URL string
//     // assert!(url.contains("spin=0"))
//     // assert!(url.contains("tonemap=aces"))
//     // assert!(url.starts_with("?"))
//
//   #[test]
//   fn test_roundtrip_all_params()
//     // Build AppState with specific values (spin=0.9, mass=6.5e9, etc.)
//     // url = from_app_state(&state)
//     // recovered = to_app_state(&url).unwrap()
//     // assert_relative_eq!(recovered.physics.spin, 0.9, epsilon=1e-6)
//     // assert_relative_eq!(recovered.physics.mass_solar, 6.5e9, epsilon=1e3)
//     // assert_eq!(recovered.postfx.tonemap_mode, "aces")
//
//   #[test]
//   fn test_roundtrip_camera_params()
//     // cam_r=15.5, cam_theta=1.047, cam_phi=3.14, fov=55.0
//     // Roundtrip: encode → decode → compare all 4 camera fields
//     // Precision: relative epsilon 1e-5
//
//   #[test]
//   fn test_partial_url_merges_into_base()
//     // base = AppState with spin=0.5
//     // url  = "?spin=0.9&fov=50"  (only 2 params specified)
//     // merge_into(&url, &mut base)
//     // assert!(base.physics.spin == 0.9)   // updated
//     // assert!(base.camera.fov_deg == 50.0) // updated
//     // assert!(base.physics.mass_solar == original_mass) // unchanged
//
//   #[test]
//   fn test_unknown_params_ignored()
//     // url = "?spin=0.5&unknown_key=xyz&future_param=42"
//     // to_app_state(&url) → Ok(state)  (no error for unknown keys)
//     // state.physics.spin == 0.5  (known param applied)
//
//   #[test]
//   fn test_invalid_spin_returns_error()
//     // url = "?spin=2.0"  (|spin| > 1 is invalid)
//     // to_app_state(&url) → Err(UrlError::InvalidParam { key: "spin", .. })
//
//   #[test]
//   fn test_invalid_tonemap_returns_error()
//     // url = "?tonemap=invalid_mode"
//     // to_app_state(&url) → Err(UrlError::InvalidParam { key: "tonemap", .. })
//
//   #[test]
//   fn test_invalid_float_returns_error()
//     // url = "?spin=notanumber"
//     // to_app_state(&url) → Err(UrlError::InvalidParam)
//
//   #[test]
//   fn test_preset_param_loads_builtin()
//     // url = "?preset=M87*"
//     // state = to_app_state(&url).unwrap()
//     // assert_relative_eq!(state.physics.spin, 0.9, epsilon=0.01)
//     // assert_relative_eq!(state.physics.mass_solar, 6.5e9, epsilon=1e6)
//
//   #[test]
//   fn test_preset_with_override()
//     // url = "?preset=M87*&spin=0.5"
//     // preset is loaded first, then spin=0.5 overrides the preset spin
//     // state.physics.spin == 0.5  (override wins)
//     // state.physics.mass_solar ≈ 6.5e9  (from preset, not overridden)
//
//   #[test]
//   fn test_url_encoding_special_chars()
//     // Preset names with spaces: "Interstellar Gargantua" → "Interstellar%20Gargantua"
//     // Decode: "%20" → " " correctly
//
//   #[test]
//   fn test_empty_url_returns_defaults()
//     // to_app_state("") → Ok(AppState::default())
//     // to_app_state("?") → Ok(AppState::default())
//
// USES (imports from):
//   gargantua_app::url_state::{UrlState, UrlError}
//   gargantua_app::state::AppState
//   gargantua_ui::presets::BuiltinPreset
//   approx::assert_relative_eq
//
// NOTE FOR AI:
//   URL encoding uses percent-encoding for special chars (spaces, +, &, etc.).
//   Use the `percent-encoding` crate (external) in url_state.rs implementation.
//   f64 precision in URLs: format with 6 significant digits to keep URLs short.
//   The URL feature is primarily for the WASM/browser build (web/index.html)
//   where users can share links. Native app also supports it via CLI: --url "?..."
//   This test has NO wgpu dependency — pure serialization logic.
// ============================================================