// =============================================================================
// FILE: crates/gargantua-app/src/state/event_bus.rs
// CRATE: gargantua-app
// LINES: ~220
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Application-wide typed event bus. Allows any subsystem (UI, physics,
//   camera, video) to publish and subscribe to events without holding direct
//   references to each other, keeping the architecture loosely coupled.
//   Uses a multi-producer multi-consumer broadcast channel internally.
//
// WHAT THIS FILE CONTAINS:
//   - `#[derive(Debug, Clone)] pub enum AppEvent`:
//       SimStateChanged(SimState)
//             Emitted when black hole mass, spin, or charge changes.
//             Received by: render pipelines, physics overlay, camera system.
//       QualityPresetChanged(QualityPreset)
//             Emitted when user changes quality in the menu.
//             Received by: adaptive quality, frame graph, render pipelines.
//       CameraPathStarted { path_id: u64 }
//             Emitted when a stored camera path begins playing.
//             Received by: replay.rs, UI overlay.
//       CameraPathFinished { path_id: u64 }
//             Emitted when a camera path playback ends.
//       RenderStarted { total_frames: u64 }
//             Emitted by OfflineRenderer when export begins.
//             Received by: ui/overlay/render_progress.rs.
//       RenderProgress { frame: u64, total: u64, eta_secs: f64 }
//             Emitted per frame during offline render.
//       RenderFinished { output_path: PathBuf }
//             Emitted when export completes successfully.
//       RenderCancelled
//             Emitted when user cancels an export.
//       PluginError { name: String, message: String }
//             Emitted when a plugin's on_frame returns Err.
//             Received by: PLANNED: crates/gargantua-ui/src/menu/tabs/plugin_tab.rs.
//       UndoStateChanged { can_undo: bool, can_redo: bool }
//             Emitted after every undo/redo/push operation.
//             Received by: toolbar buttons (grey-out undo/redo).
//       ShareUrlGenerated(String)
//             Emitted when a share URL has been built from SimState.
//             Received by: UI clipboard copy handler.
//
//   - `pub struct EventBus`:
//       tx: tokio::sync::broadcast::Sender<AppEvent>
//   - `impl EventBus`:
//       `pub fn new(capacity: usize) -> Self`
//             Creates a broadcast channel with capacity slots (default: 256).
//       `pub fn emit(&self, event: AppEvent)`
//             Calls self.tx.send(event). Errors (no receivers) are silently
//             ignored — events are fire-and-forget.
//       `pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AppEvent>`
//             Returns a new Receiver. Subsystems call this once at startup
//             and then poll/await the receiver each frame.
//       `pub fn try_recv_all(rx: &mut Receiver<AppEvent>) -> Vec<AppEvent>`
//             Helper: drains all pending events from a receiver without blocking.
//             Used in the synchronous winit event loop (non-async context).
//
// OUTBOUND DEPENDENCIES:
//   - tokio (external, feature="sync") → broadcast channel
//   - state/sim_state.rs               → SimState (carried in events)
//   - gargantua_core::quality          → QualityPreset (carried in events)
//   - std::path::PathBuf               → RenderFinished::output_path
//
// INBOUND (who emits/subscribes):
//   - systems/physics_sync.rs   → emits SimStateChanged after integration
//   - systems/input.rs          → emits QualityPresetChanged on hotkey
//   - systems/replay.rs         → emits CameraPath* events
//   - crates/gargantua-video/src/offline/renderer.rs → emits RenderStarted/Progress/Finished
//   - plugin/registry.rs        → emits PluginError on failure
//   - state/undo.rs             → emits UndoStateChanged
//   - state/url_serde.rs        → emits ShareUrlGenerated
//   - ui subsystems             → subscribe to relevant events
//
// NOTES:
//   - Broadcast channels clone the event for every subscriber. AppEvent derives
//     Clone so all variants must contain only Clone types (no wgpu resources).
//   - On WASM, tokio::sync::broadcast works without the tokio runtime
//     because only the sync features are used (no async await needed here).
// =============================================================================
