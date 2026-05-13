// =============================================================================
// FILE: crates/gargantua-app/src/state/url_serde.rs
// CRATE: gargantua-app
// LINES: ~220
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Serialises and deserialises SimState to/from a compact URL-safe string,
//   enabling "share this view" links. The URL encodes all physics parameters,
//   camera position, and render settings so anyone with the link can
//   reproduce the exact same view in their browser or desktop app.
//
// WHAT THIS FILE CONTAINS:
//   - `pub fn encode(state: &SimState) -> String`
//             Serialises SimState to JSON via serde_json.
//             Compresses the JSON bytes with zstd (level 3 — fast, ~3:1 ratio).
//             Base64url-encodes the compressed bytes (URL-safe, no padding).
//             Prepends: "https://gargantua.app/#v1=" + base64url_string.
//             Returns the full share URL as a String.
//   - `pub fn decode(url: &str) -> AppResult<SimState>`
//             Strips the URL prefix, returns AppError::StateDeserialize
//             if the prefix is missing or version tag is unknown.
//             Decodes base64url → compressed bytes.
//             Decompresses with zstd → JSON bytes.
//             Deserialises JSON → SimState via serde_json.
//             Calls state.validate() — rejects physically impossible values.
//             Returns AppError::StateDeserialize on any step failure.
//   - `pub fn encode_minimal(state: &SimState) -> String`
//             Only encodes: mass, spin, charge, camera_position.
//             Other fields fall back to SimState::default() on decode.
//             Produces a shorter URL (~100 chars) for social media sharing.
//   - Constants:
//       `const URL_PREFIX: &str = "https://gargantua.app/#v1="`
//       `const ZSTD_LEVEL: i32 = 3`
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs          → SimState (Serialize + Deserialize)
//   - serde_json (external)       → to_string(), from_str()
//   - zstd (external)             → encode_all(), decode_all()
//   - base64 (external, url-safe) → Engine::encode(), Engine::decode()
//   - state/event_bus.rs          → EventBus::emit(ShareUrlGenerated(url))
//   - errors.rs                   → AppResult, AppError::StateDeserialize
//
// INBOUND:
//   - ui/menu/share_button.rs      → calls encode() when user clicks "Share"
//   - systems/input.rs             → calls decode() on startup if URL hash present
//   - tests/url_serde.rs           → round-trip and edge case tests
//
// NOTES:
//   - URL version prefix "v1=" allows future schema migrations. If the schema
//     changes, bump to "v2=" and add a migration path in decode().
//   - zstd compression reduces a typical SimState JSON (~600 bytes) to ~180 bytes.
//   - On WASM, the URL hash is read via web_sys::window()?.location().hash().
//     On native, it is passed as a command-line argument or deep link URL.
// =============================================================================
