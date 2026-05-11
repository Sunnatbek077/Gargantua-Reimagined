// ============================================================
// FILE: crates/gargantua-ui/src/widgets/keyframe_editor.rs
// LINES: ~380
// CATEGORY: UI — Keyframe animation editor widget
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Timeline-based keyframe editor for animating camera paths and
//   parameter changes over time. Used in the Render tab for creating
//   animated renders (frame sequences). Shows a timeline with
//   draggable keyframe diamonds and a playhead.
//
// CONTENTS (~380 lines):
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct Keyframe {
//       pub time:  f32,         // time in seconds
//       pub value: f32,         // parameter value at this keyframe
//       pub easing: EasingMode, // interpolation to next keyframe
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
//   pub enum EasingMode { Linear, EaseIn, EaseOut, EaseInOut, Step }
//
//   pub struct KeyframeEditor {
//       pub keyframes:    Vec<Keyframe>,
//       pub duration:     f32,       // total animation duration in seconds
//       pub playhead:     f32,       // current time position
//       pub selected:     Option<usize>,  // index of selected keyframe
//       pub zoom:         f32,       // timeline zoom level [0.1–10.0]
//       pub scroll:       f32,       // horizontal scroll offset
//   }
//
//   impl KeyframeEditor {
//       pub fn new(duration: f32) -> Self
//
//       // Draw timeline editor in given UI region
//       // Returns true if keyframes changed this frame
//       pub fn draw(&mut self, ui: &mut egui::Ui) -> bool
//         // Timeline ruler (seconds, tick marks)
//         // Keyframe diamonds: drag to move, right-click to delete/easing
//         // Playhead: drag red line, click to scrub
//         // Left panel: keyframe value list (editable)
//         // Toolbar: Add KF button, zoom slider, play/pause
//
//       // Evaluate interpolated value at given time
//       pub fn evaluate(&self, t: f32) -> f32
//         // Finds surrounding keyframes, applies easing function
//         // Returns keyframes[0].value if t < first keyframe
//         // Returns keyframes[last].value if t > last keyframe
//
//       // Add keyframe at current playhead time with given value
//       pub fn add_keyframe(&mut self, value: f32)
//         // Inserts sorted by time, replaces if same time exists
//
//       pub fn remove_keyframe(&mut self, index: usize)
//       pub fn sort_by_time(&mut self)
//
//       // Easing function: maps t ∈ [0,1] → eased t ∈ [0,1]
//       fn ease(t: f32, mode: EasingMode) -> f32
//         // Linear: t
//         // EaseIn: t² (quadratic)
//         // EaseOut: 1-(1-t)²
//         // EaseInOut: t<0.5 ? 2t² : 1-2(1-t)²
//         // Step: t < 1.0 ? 0.0 : 1.0
//   }
//
// USES (imports from):
//   egui         → Ui, Painter, Rect, Pos2, Color32, Response
//   serde        → Serialize, Deserialize
//
// USED BY:
//   crates/gargantua-ui/src/menu/tabs/render_tab.rs
//     → camera path animation editor for video export
//
// NOTE FOR AI:
//   evaluate() interpolates between consecutive keyframes:
//     t_local = (t - kf_i.time) / (kf_i+1.time - kf_i.time)
//     t_eased = ease(t_local, kf_i.easing)
//     value   = lerp(kf_i.value, kf_i+1.value, t_eased)
//   Keyframe diamonds are 8×8 px rotated squares (◆).
//   Right-click on diamond: context menu with "Delete" and "Easing" submenu.
//   Multiple KeyframeEditor instances exist (one per animated parameter).
// ============================================================