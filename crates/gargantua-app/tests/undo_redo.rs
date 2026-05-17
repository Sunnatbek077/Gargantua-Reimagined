// ============================================================
// FILE: crates/gargantua-app/tests/undo_redo.rs
// LINES: ~240
// CATEGORY: Integration test — Undo/Redo stack for AppState changes
// RUN: cargo test --package gargantua-app --test undo_redo
// ============================================================
//
// PURPOSE:
//   Validates the UndoStack system that tracks AppState mutations.
//   Every user action that changes physics, accretion, camera, or
//   postfx parameters is pushed onto the undo stack so the user
//   can Ctrl+Z / Ctrl+Y to step back and forward.
//   Tests: push, undo, redo, stack limit, branching (redo invalidation).
//
// TESTED FUNCTIONS (from crates/gargantua-app/src/undo.rs):
//   UndoStack::new(max_depth)
//   UndoStack::push(snapshot: AppSnapshot)
//   UndoStack::undo() -> Option<AppSnapshot>
//   UndoStack::redo() -> Option<AppSnapshot>
//   UndoStack::can_undo() -> bool
//   UndoStack::can_redo() -> bool
//   UndoStack::clear()
//   UndoStack::depth() -> usize
//
// KEY TYPES:
//   AppSnapshot — a lightweight clone of the mutable parts of AppState:
//     { physics: PhysicsParams, accretion: AccretionParams,
//       camera: CameraParams, postfx: PostFxParams }
//   UndoStack — VecDeque<AppSnapshot> with max capacity (default 64)
//
// TEST CASES (~240 lines):
//
//   #[test]
//   fn test_new_stack_is_empty()
//     // UndoStack::new(64)
//     // can_undo() == false
//     // can_redo() == false
//     // depth() == 0
//
//   #[test]
//   fn test_push_enables_undo()
//     // stack.push(snapshot_a)
//     // can_undo() == true
//     // can_redo() == false  (no redo yet)
//     // depth() == 1
//
//   #[test]
//   fn test_undo_returns_previous_snapshot()
//     // stack.push(snapshot_a)
//     // stack.push(snapshot_b)
//     // result = stack.undo()
//     // assert_eq!(result.unwrap().physics.spin, snapshot_a.physics.spin)
//     // (undo returns the state BEFORE the last change)
//
//   #[test]
//   fn test_undo_empty_stack_returns_none()
//     // UndoStack::new(64) (empty)
//     // stack.undo() == None
//
//   #[test]
//   fn test_redo_after_undo()
//     // stack.push(snapshot_a), stack.push(snapshot_b)
//     // stack.undo()            // back to snapshot_a
//     // result = stack.redo()   // forward to snapshot_b
//     // assert_eq!(result.unwrap().physics.spin, snapshot_b.physics.spin)
//
//   #[test]
//   fn test_redo_without_undo_returns_none()
//     // stack.push(snapshot_a)
//     // stack.redo() == None  (nothing to redo)
//
//   #[test]
//   fn test_push_after_undo_invalidates_redo()
//     // stack.push(a), stack.push(b)
//     // stack.undo()          // cursor at a
//     // stack.push(c)         // new branch: redo history (b) is discarded
//     // can_redo() == false   // b is gone
//     // stack.undo() → Some(a)
//
//   #[test]
//   fn test_stack_respects_max_depth()
//     // UndoStack::new(3)  (max 3 entries)
//     // push a, b, c, d  (4 pushes — oldest should be evicted)
//     // depth() == 3
//     // undo 3 times: get c, b, a  (d was current, a was oldest kept)
//     // 4th undo → None  (a was evicted)
//
//   #[test]
//   fn test_clear_empties_stack()
//     // push a, b, c
//     // stack.clear()
//     // can_undo() == false
//     // can_redo() == false
//     // depth() == 0
//
//   #[test]
//   fn test_multiple_undo_redo_cycles()
//     // push a, b, c
//     // undo → b, undo → a, undo → None
//     // redo → b, redo → c, redo → None
//     // Verifies cursor stays in bounds at both ends
//
//   #[test]
//   fn test_snapshot_physics_fields_preserved()
//     // Create snapshot with specific values:
//     //   physics.spin = 0.9, physics.mass_solar = 6.5e9
//     //   accretion.accretion_rate = 0.1
//     // push → undo → check all fields match exactly
//
//   #[test]
//   fn test_snapshot_camera_fields_preserved()
//     // snapshot.camera.r = 15.0, fov_deg = 55.0
//     // push → undo → check r and fov_deg match
//
//   #[test]
//   fn test_undo_redo_keyboard_shortcut_integration()
//     // Simulates Ctrl+Z / Ctrl+Y via ShortcutHandler events:
//     // apply AppEvent::Undo → stack.undo() called
//     // apply AppEvent::Redo → stack.redo() called
//     // Verifies event routing from shortcuts.rs to UndoStack
//
// SETUP HELPERS:
//   fn make_snapshot(spin: f64, r: f64) -> AppSnapshot
//     // Returns AppSnapshot with given spin and camera.r,
//     // all other fields set to defaults
//
// USES (imports from):
//   gargantua_app::undo::{UndoStack, AppSnapshot}
//   gargantua_app::state::AppEvent
//   gargantua_ui::presets::schema::{PhysicsParams, CameraParams,
//                                   AccretionParams, PostFxParams}
//
// NOTE FOR AI:
//   UndoStack uses a VecDeque<AppSnapshot> with a cursor index.
//   push() truncates redo history (everything after cursor) before pushing.
//   max_depth eviction: pop_front() when len() > max_depth after push.
//   AppSnapshot must be Clone + PartialEq for test assertions.
//   This test has NO wgpu dependency — pure CPU/state logic.
// ============================================================