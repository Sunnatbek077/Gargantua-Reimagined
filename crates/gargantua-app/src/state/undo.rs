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
//   - crates/gargantua-ui/src/hud/mod.rs                 → reads can_undo() / can_redo() for button state
//
// NOTES:
//   - SimState is ~400 bytes (all plain f64/f32/bool fields). With max_len = 200,
//     the undo history uses at most ~80 KB — negligible memory.
//   - push() is called at the END of a drag (mouse release), not on every
//     slider tick. This prevents 60 undo steps for one spin adjustment.
//     The UI's slider widget must call push() only on drag_released().
//   - There is no "undo tree" (branching history); this is a linear stack.
// =============================================================================
