// =============================================================================
// FILE: crates/gargantua-app/src/state/undo.rs
// CRATE: gargantua-app
// LINES: ~200
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Unlimited undo/redo history for SimState changes. Every meaningful user
//   action (changing spin, mass, camera mode, quality preset) pushes a snapshot
//   of the entire SimState onto the undo stack. Ctrl+Z / Cmd+Z restores the
//   previous snapshot; Ctrl+Shift+Z / Cmd+Shift+Z redoes.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct UndoHistory`:
//       past:    VecDeque<SimState>    — undo stack; back = most recent
//       future:  Vec<SimState>         — redo stack; cleared on new push
//       max_len: usize                 — cap on history depth (default: 200)
//   - `impl UndoHistory`:
//       `pub fn new(max_len: usize) -> Self`
//             Initialises empty stacks with the given depth limit.
//       `pub fn push(&mut self, state: SimState)`
//             Pushes state onto self.past.
//             Clears self.future (new action invalidates redo history).
//             If past.len() > max_len: pops the oldest entry from the front.
//       `pub fn undo(&mut self, current: SimState) -> Option<SimState>`
//             If self.past is empty: returns None.
//             Else: pops the most recent past state.
//             Pushes current onto self.future.
//             Returns the popped past state (caller applies it to SimState).
//       `pub fn redo(&mut self, current: SimState) -> Option<SimState>`
//             If self.future is empty: returns None.
//             Else: pops the most recent future state.
//             Pushes current onto self.past.
//             Returns the popped future state.
//       `pub fn can_undo(&self) -> bool`   — returns !self.past.is_empty()
//       `pub fn can_redo(&self) -> bool`   — returns !self.future.is_empty()
//       `pub fn clear(&mut self)`
//             Clears both stacks (called on scene reset / new file).
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs  → SimState (Clone + data-only, cheap to snapshot)
//   - state/event_bus.rs  → EventBus::emit(AppEvent::UndoStateChanged)
//   - std::collections::VecDeque — undo stack storage
//
// INBOUND (who calls UndoHistory):
//   - systems/input.rs              → calls undo() / redo() on Ctrl+Z / Ctrl+Shift+Z
//   - ui/menu/tabs/physics_tab.rs   → calls push() when user commits a slider change
//   - ui/toolbar.rs                 → reads can_undo() / can_redo() for button state
//
// NOTES:
//   - SimState is ~400 bytes (all plain f64/f32/bool fields). With max_len = 200,
//     the undo history uses at most ~80 KB — negligible memory.
//   - push() is called at the END of a drag (mouse release), not on every
//     slider tick. This prevents 60 undo steps for one spin adjustment.
//     The UI's slider widget must call push() only on drag_released().
//   - There is no "undo tree" (branching history); this is a linear stack.
// =============================================================================


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
//             Base64url-encodes the compressed bytes (URL-safe alphabet, no padding).
//             Prepends the application URL prefix:
//               "https://gargantua.app/#v1=" + base64url_string
//             Returns the full share URL as a String.
//   - `pub fn decode(url: &str) -> AppResult<SimState>`
//             Strips the URL prefix to extract the base64url payload.
//             Returns AppError::StateDeserialize if the prefix is missing.
//             Decodes base64url → compressed bytes.
//             Decompresses with zstd → JSON bytes.
//             Deserialises JSON → SimState via serde_json.
//             Calls state.validate() — rejects physically impossible values.
//             Returns AppError::StateDeserialize with a message on any step failure.
//   - `pub fn encode_minimal(state: &SimState) -> String`
//             Variant that only includes the most important params (mass, spin,
//             charge, camera_position) to produce a shorter URL (~100 chars).
//             Other fields fall back to SimState::default() on decode.
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
// INBOUND (who calls url_serde):
//   - ui/menu/share_button.rs      → calls encode() when user clicks "Share"
//   - systems/input.rs             → calls decode() on startup if URL hash present
//   - tests/url_serde.rs           → round-trip and edge case tests
//
// NOTES:
//   - URL version prefix "v1=" allows future schema migrations. If the schema
//     changes, bump to "v2=" and add a migration path in decode().
//   - zstd compression reduces a typical SimState JSON (~600 bytes) to ~180 bytes,
//     keeping URLs short enough to share in chat messages or social media.
//   - WASM: zstd works because it has a wasm32 build. No platform-specific code.
//   - On WASM, the URL hash is read via web_sys::window()?.location().hash().
//     On native, it is passed as a command-line argument or deep link URL.
// =============================================================================
