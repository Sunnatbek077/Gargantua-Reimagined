// =============================================================================
// FILE: crates/gargantua-app/src/systems/replay.rs
// CRATE: gargantua-app
// LINES: ~260
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Records and plays back sequences of SimState snapshots for the "replay"
//   feature. A replay is a time series of (timestamp, SimState) pairs that can
//   be saved to disk and played back deterministically — the same physics,
//   camera path, and render settings produce the same frames every time.
//   Replays are also used as the data source for offline video rendering.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct ReplayFrame`:
//       timestamp_s:  f64       — simulation time when snapshot was taken
//       state:        SimState  — full SimState snapshot
//   - `pub struct Replay`:
//       frames:       Vec<ReplayFrame>
//       fps:          f32       — original recording frame rate
//       duration_s:   f64       — total replay duration in seconds
//   - `impl Replay`:
//       `pub fn to_json(&self) -> String`
//       `pub fn from_json(s: &str) -> AppResult<Self>`
//       `pub fn frame_at(&self, t: f64) -> &ReplayFrame`
//             Binary searches frames for the closest timestamp to t.
//   - `pub struct ReplaySystem`:
//       recording:   Option<Replay>         — currently recording replay
//       playback:    Option<(Replay, f64)>  — active playback + playhead time
//       event_bus:   Arc<EventBus>
//   - `impl ReplaySystem`:
//       `pub fn start_recording(&mut self)`
//       `pub fn record_frame(&mut self, state: &SimState, timestamp: f64)`
//             Appends a ReplayFrame; called every frame while recording is active.
//       `pub fn stop_recording(&mut self) -> Option<Replay>`
//             Finalises replay, clears self.recording, returns completed Replay.
//       `pub fn start_playback(&mut self, replay: Replay)`
//             Sets playhead to 0.0, emits CameraPathStarted event.
//       `pub fn tick_playback(&mut self, dt: DeltaTime) -> Option<&SimState>`
//             Advances playhead by dt.sim.
//             Returns &SimState from the nearest replay frame.
//             Returns None when playback ends; emits CameraPathFinished.
//       `pub fn stop_playback(&mut self)`
//       `pub fn save_to_file(&self, replay: &Replay, path: &Path) -> AppResult<()>`
//             Writes compressed .replay.zst file (JSON + zstd).
//       `pub fn load_from_file(path: &Path) -> AppResult<Replay>`
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs   → SimState (Clone + Serialize + Deserialize)
//   - state/event_bus.rs   → EventBus (CameraPathStarted/Finished events)
//   - gargantua_core::clock → DeltaTime
//   - serde_json (external) → Serialize/Deserialize
//   - zstd (external)       → compress/decompress .replay.zst files
//   - errors.rs             → AppResult
//
// INBOUND:
//   - gargantua_core::app::App   → calls tick_playback() / record_frame() each frame
//   - crates/gargantua-video/src/offline/renderer.rs  → calls load_from_file() for SimState timeline
//   - crates/gargantua-ui/src/menu/tabs/camera_tab.rs → start/stop recording, load file, scrub playhead
//
// NOTES:
//   - A 60-second replay at 60 FPS = 3600 SimState snapshots ≈ 2.2 MB
//     uncompressed → ~400 KB as .replay.zst.
//   - The offline renderer uses tick_playback() to set SimState per output frame,
//     achieving perfect consistency between replay and final video.
// =============================================================================
