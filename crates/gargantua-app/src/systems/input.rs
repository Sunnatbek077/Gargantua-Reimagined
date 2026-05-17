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
