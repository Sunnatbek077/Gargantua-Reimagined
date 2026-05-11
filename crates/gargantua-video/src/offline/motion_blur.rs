// =============================================================================
// FILE: crates/gargantua-video/src/offline/motion_blur.rs
// CRATE: gargantua-video
// LINES: ~260
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Sub-frame temporal jitter for offline motion blur. Renders the same frame
//   N times with the physics simulation advanced by small time offsets (0° to
//   180° shutter angle), then averages them in the accumulator.
//   Produces cinematic 180° shutter motion blur for offline renders.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct OfflineMotionBlur`:
//       shutter_angle:  f32     — degrees (0–360; default 180°)
//       subframe_count: u32     — number of temporal samples (default 16)
//   - `impl OfflineMotionBlur`:
//       `pub fn new(shutter_angle: f32, subframe_count: u32) -> Self`
//       `pub fn subframe_offsets(&self, fps: f32) -> Vec<f64>`
//             Returns a Vec of time offsets in seconds for each sub-frame.
//             For 180° shutter at 24 FPS: offsets span 0 to 1/(24×2) seconds.
//             Distributed uniformly across the shutter open interval.
//       `pub fn time_for_subframe(&self, frame: u64, subframe: u32, fps: f32) -> f64`
//             Returns the exact simulation time for sub-frame N within frame F.
//             Used by renderer.rs to set physics_sync's simulation time.
//
// OUTBOUND DEPENDENCIES:
//   - None (pure time arithmetic)
//
// INBOUND (who uses OfflineMotionBlur):
//   - video/offline/renderer.rs → calls subframe_offsets() to schedule
//                                   multiple render passes per output frame
//
// NOTES:
//   - 16 sub-frames per output frame is standard VFX pipeline quality.
//     For preview renders, 4 sub-frames are sufficient.
//   - The shutter_angle=0 case disables motion blur (single sub-frame per frame).
//   - This is a temporal accumulation approach; the tile-based GPU motion blur
//     (motion_blur.wgsl) is used for real-time mode instead.
// =============================================================================
