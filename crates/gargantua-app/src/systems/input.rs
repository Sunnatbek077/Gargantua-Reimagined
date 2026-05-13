// =============================================================================
// FILE: crates/gargantua-app/src/systems/input.rs
// CRATE: gargantua-app
// LINES: ~280
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Processes raw keyboard, mouse, and touch input from winit and translates
//   it into high-level application actions: camera movement, undo/redo,
//   quality preset hotkeys, menu toggle, screenshot, and parameter nudge.
//   Acts as the bridge between raw OS events and the rest of the app state.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct InputSystem`:
//       camera_controller: CameraController   — delegates camera movement input
//       undo_history:      Arc<Mutex<UndoHistory>>
//       event_bus:         Arc<EventBus>
//       key_state:         HashSet<VirtualKeyCode>   — currently held keys
//       mouse_delta:       (f32, f32)                — accumulated mouse delta
//       mouse_buttons:     HashSet<MouseButton>
//   - `impl InputSystem`:
//       `pub fn new(undo: Arc<Mutex<UndoHistory>>, bus: Arc<EventBus>) -> Self`
//       `pub fn handle_event(&mut self, event: &WindowEvent,
//                             sim: &mut SimState) -> bool`
//             Returns true if the event was consumed (should not propagate to UI).
//             Dispatches based on WindowEvent variant:
//               KeyboardInput { key: Z, modifiers: Ctrl/Cmd } → undo()
//               KeyboardInput { key: Z, modifiers: Ctrl+Shift/Cmd+Shift } → redo()
//               KeyboardInput { key: F1 }    → toggle main menu
//               KeyboardInput { key: F12 }   → emit screenshot request
//               KeyboardInput { key: 1..6 }  → set quality preset 1–6
//               KeyboardInput { key: Space } → toggle time_scale 0 ↔ 1
//               MouseMotion(delta)           → forward to camera_controller
//               MouseWheel(delta)            → camera zoom / time_scale nudge
//               Touch events                 → normalised to mouse equivalents
//       `pub fn tick(&mut self, sim: &mut SimState, dt: DeltaTime)`
//             Called each frame for held-key continuous actions:
//               WASD / arrow keys → camera_controller.move_camera(dt)
//               +/- keys         → nudge sim.time_scale by ±0.1
//       `fn undo(&mut self, sim: &mut SimState)`
//             Locks undo_history, calls undo(sim.clone()), applies returned state.
//             Emits UndoStateChanged via event_bus.
//       `fn redo(&mut self, sim: &mut SimState)`
//             Same pattern as undo() but in reverse direction.
//
// OUTBOUND DEPENDENCIES:
//   - state/undo.rs           → UndoHistory::undo(), redo()
//   - state/sim_state.rs      → SimState (modified on undo/redo/hotkeys)
//   - state/event_bus.rs      → EventBus::emit(UndoStateChanged, etc.)
//   - gargantua_camera::controller → CameraController::move_camera()
//   - gargantua_core::clock   → DeltaTime
//   - winit (external)        → WindowEvent, VirtualKeyCode, MouseButton
//
// INBOUND:
//   - gargantua_core::app::App → calls input_system.handle_event() from the
//                                 winit event loop closure
//   - plugin/registry.rs       → calls input_system.tick() each frame after
//                                 plugin ticks
//
// NOTES:
//   - On WASM, winit maps browser KeyboardEvent to VirtualKeyCode; the same
//     code runs unchanged. Touch events are mapped to synthetic mouse events
//     by winit automatically.
//   - Modifier key detection (Ctrl vs Cmd) uses winit's ModifiersState:
//       native Mac → Cmd (Super), native Win → Ctrl, WASM → Ctrl always.
// =============================================================================


// =============================================================================
// FILE: crates/gargantua-app/src/systems/physics_sync.rs
// CRATE: gargantua-app
// LINES: ~240
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Each frame, reads the current SimState, steps the physics simulation
//   forward by DeltaTime.sim, and uploads the resulting parameters to the
//   GPU uniform buffer. Serves as the integration point between the CPU
//   physics layer (gargantua-physics) and the GPU render layer.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct PhysicsSync`:
//       uniform_buffer:  wgpu::Buffer   — GPU-side PhysicsUniforms buffer
//       metric:          KerrMetric     — cached from last SimState update
//       isco:            IscoBounds     — cached ISCO for disk inner boundary
//       mhd_disk:        MhdDisk        — accretion disk model instance
//       upload_heap:     Option<UploadHeap>  — Windows-only write-combined upload
//   - `impl PhysicsSync`:
//       `pub fn new(ctx: &GpuContext, sim: &SimState) -> AppResult<Self>`
//             Builds KerrMetric from sim.to_kerr_metric().
//             Computes IscoBounds via accretion::isco::compute_isco().
//             Creates wgpu::Buffer (UNIFORM | COPY_DST) for PhysicsUniforms.
//             On Windows: creates UploadHeap for write-combined CPU→GPU transfer.
//       `pub fn sync(&mut self, sim: &SimState, dt: DeltaTime,
//                    queue: &wgpu::Queue)`
//             Called every frame from App::tick():
//             1. If sim.mass or sim.spin changed since last frame:
//                  Rebuilds self.metric = KerrMetric::new(sim.mass, sim.spin, sim.charge)
//                  Recomputes self.isco = compute_isco(&self.metric)
//                  Recomputes self.mhd_disk from new metric + accretion_rate
//             2. Advances sim.simulation_time += dt.sim (write via Arc<RwLock>)
//             3. Builds PhysicsUniforms from sim.to_gpu_uniforms() + ISCO + MHD params
//             4. Uploads PhysicsUniforms to self.uniform_buffer:
//                  Mac:     queue.write_buffer() (unified memory — zero copy)
//                  Windows: upload_heap.write() + copy_to(uniform_buffer)
//       `pub fn uniform_buffer(&self) -> &wgpu::Buffer`
//             Returns a reference to the GPU uniform buffer; used by render
//             bind groups to bind the physics data.
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs                    → SimState, to_gpu_uniforms()
//   - gargantua_physics::metric::kerr       → KerrMetric
//   - gargantua_physics::accretion::isco    → compute_isco(), IscoBounds
//   - gargantua_physics::accretion::mhd     → MhdDisk
//   - render/bindgroups/physics.rs          → PhysicsUniforms (GPU struct layout)
//   - platform/windows/memory/upload_heap.rs→ UploadHeap (Windows only)
//   - gargantua_core::gpu::context          → GpuContext
//   - wgpu (external)                       → Buffer, Queue
//   - errors.rs                             → AppResult
//
// INBOUND:
//   - gargantua_core::app::App → calls physics_sync.sync() once per frame
//                                  in App::tick() before frame_graph.execute()
//
// NOTES:
//   - Rebuilding KerrMetric and ISCO only happens when mass/spin/charge changes
//     (typically on user slider input), NOT every frame. The hot path (no change)
//     skips the rebuild and only updates simulation_time and MHD turbulence seed.
//   - The MHD turbulence seed is incremented each frame to animate disk flicker;
//     this is the only per-frame physics calculation on the CPU hot path.
// =============================================================================


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
//       state:        SimState  — full SimState snapshot (mass, spin, camera, etc.)
//   - `pub struct Replay`:
//       frames:       Vec<ReplayFrame>
//       fps:          f32       — original recording frame rate
//       duration_s:   f64       — total replay duration in seconds
//   - `impl Replay`:
//       `pub fn to_json(&self) -> String`   — serde_json serialisation
//       `pub fn from_json(s: &str) -> AppResult<Self>`
//       `pub fn frame_at(&self, t: f64) -> &ReplayFrame`
//             Binary searches frames for the ReplayFrame with the closest
//             timestamp to t. Used for scrubbing.
//   - `pub struct ReplaySystem`:
//       recording:   Option<Replay>        — currently recording replay
//       playback:    Option<(Replay, f64)> — active playback + playhead time
//       event_bus:   Arc<EventBus>
//   - `impl ReplaySystem`:
//       `pub fn start_recording(&mut self)`
//             Creates a new empty Replay, sets self.recording = Some(replay).
//       `pub fn record_frame(&mut self, state: &SimState, timestamp: f64)`
//             Appends a ReplayFrame to self.recording if recording is active.
//             Called every frame from App::tick() while recording.
//       `pub fn stop_recording(&mut self) -> Option<Replay>`
//             Finalises the replay (sets duration_s), clears self.recording.
//             Returns the completed Replay to the caller for saving.
//       `pub fn start_playback(&mut self, replay: Replay)`
//             Sets self.playback = Some((replay, 0.0)).
//             Emits CameraPathStarted via event_bus.
//       `pub fn tick_playback(&mut self, dt: DeltaTime) -> Option<&SimState>`
//             Advances playhead by dt.sim.
//             Returns &SimState from the nearest replay frame for the caller to apply.
//             Returns None when playback ends; emits CameraPathFinished.
//       `pub fn stop_playback(&mut self)`
//             Clears self.playback, emits CameraPathFinished.
//       `pub fn save_to_file(&self, replay: &Replay, path: &Path) -> AppResult<()>`
//             Writes replay.to_json() to path (compressed .replay.zst file).
//       `pub fn load_from_file(path: &Path) -> AppResult<Replay>`
//             Reads and decompresses the .replay.zst file, calls Replay::from_json().
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs  → SimState (Clone + Serialize + Deserialize)
//   - state/event_bus.rs  → EventBus (CameraPathStarted/Finished events)
//   - gargantua_core::clock → DeltaTime
//   - serde_json (external) → Serialize/Deserialize
//   - zstd (external)       → compress/decompress .replay.zst files
//   - errors.rs             → AppResult
//
// INBOUND:
//   - gargantua_core::app::App   → calls tick_playback() each frame if active
//                                    calls record_frame() each frame if recording
//   - video/offline/renderer.rs  → calls load_from_file() to get SimState timeline
//   - ui/menu/tabs/replay_tab.rs → start/stop recording, load replay file,
//                                    scrub playhead position
//
// NOTES:
//   - Replay files (.replay.zst) are compressed JSON. A 60-second replay at 60 FPS
//     = 3600 SimState snapshots ≈ 2.2 MB uncompressed → ~400 KB compressed.
//   - The offline renderer uses tick_playback() to set SimState for each output frame,
//     achieving perfect temporal consistency between replay and final video.
// =============================================================================
