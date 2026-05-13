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
//             Received by: ui/menu/tabs/plugin_tab.rs.
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
//   - video/offline/renderer.rs → emits RenderStarted/Progress/Finished
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


// =============================================================================
// FILE: crates/gargantua-app/src/state/sim_state.rs
// CRATE: gargantua-app
// LINES: ~260
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   The canonical, serialisable state of the entire physics simulation.
//   SimState is the single source of truth for black hole parameters,
//   accretion disk settings, camera position, and playback time.
//   It is serialised to JSON for URL sharing and undo/redo snapshots.
//
// WHAT THIS FILE CONTAINS:
//   - `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`
//     `pub struct SimState`:
//
//       --- Black hole physics ---
//       mass:         f64       — geometric mass M (solar masses in UI, geometric internally)
//       spin:         f64       — dimensionless spin a/M ∈ [0, 0.9999]
//       charge:       f64       — dimensionless charge Q/M ∈ [0, 1) (usually 0)
//
//       --- Accretion disk ---
//       accretion_rate:   f64   — Eddington fraction [0, 1]
//       disk_inner_radius: f64  — manually overrides ISCO if > 0.0 (in units of M)
//       disk_outer_radius: f64  — outer edge in units of M (default: 50 M)
//       disk_inclination:  f64  — tilt angle in degrees [0, 90]
//       beta_mag:          f64  — MHD plasma beta parameter
//       jet_enabled:       bool — whether the relativistic jet is rendered
//
//       --- Render settings ---
//       quality_level:     QualityLevel
//       spp:               u32
//       render_scale:      f32
//
//       --- Camera ---
//       camera_position:   [f64; 3]  — Boyer-Lindquist (r, θ, φ)
//       camera_direction:  [f64; 3]  — unit vector in BL coordinates
//       camera_fov_deg:    f32
//       camera_mode:       CameraMode  — Free, Satellite, Plunge, Gravity, Path
//
//       --- Playback ---
//       simulation_time:   f64       — accumulated physics time in geometric units
//       time_scale:        f64       — 1.0 = real-time, 0.0 = paused
//
//   - `impl SimState`:
//       `pub fn default() -> Self`
//             Returns sensible defaults:
//               mass = 10 solar masses, spin = 0.9, charge = 0.0
//               accretion_rate = 0.1, quality = High
//               camera at r = 50M, equatorial plane, looking toward BH
//       `pub fn validate(&self) -> AppResult<()>`
//             Checks all fields are within valid physical bounds.
//             Returns AppError::Physics if spin >= 1.0, mass <= 0, etc.
//       `pub fn to_kerr_metric(&self) -> KerrMetric`
//             Converts mass + spin + charge to a KerrMetric struct for physics.
//       `pub fn to_gpu_uniforms(&self) -> PhysicsUniforms`
//             Packs the relevant fields into a bytemuck::Pod struct for
//             upload to the GPU physics uniform buffer.
//
// OUTBOUND DEPENDENCIES:
//   - gargantua_physics::metric::kerr → KerrMetric
//   - gargantua_core::quality         → QualityLevel, QualityPreset
//   - gargantua_camera::modes         → CameraMode enum
//   - render/bindgroups/physics.rs    → PhysicsUniforms (GPU struct)
//   - serde (external)                → Serialize, Deserialize
//   - bytemuck (external)             → Pod for GPU upload
//   - errors.rs                       → AppResult
//
// INBOUND (who reads/writes SimState):
//   - systems/physics_sync.rs  → reads SimState each frame for integration
//   - state/undo.rs            → snapshots SimState for undo history
//   - state/url_serde.rs       → serialises SimState to/from URL
//   - ui/menu/tabs/physics_tab.rs → reads/writes mass, spin, charge
//   - ui/menu/tabs/accretion_tab.rs → reads/writes disk params
//   - plugin/scripting.rs      → Lua API writes spin/mass via SimState
//   - systems/replay.rs        → restores SimState from a replay recording
//
// NOTES:
//   - SimState contains only plain data (no GPU resources, no Arc, no Mutex).
//     This makes it trivially Clone and Serialize without special handling.
//   - The Arc<RwLock<SimState>> wrapper lives in PluginContext and App;
//     this file only defines the data struct itself.
// =============================================================================
