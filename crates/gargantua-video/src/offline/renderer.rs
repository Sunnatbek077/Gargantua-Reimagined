// =============================================================================
// FILE: crates/gargantua-video/src/offline/renderer.rs
// CRATE: gargantua-video
// LINES: ~320
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Orchestrates the complete offline video render pipeline: physics simulation
//   stepping, GPU ray march dispatch, sub-frame accumulation, denoising,
//   colour transform, and encoding. This is the entry point for "Export Video"
//   in the UI.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct OfflineRenderer`:
//       config:       OfflineConfig
//       encoder:      Box<dyn Encoder>
//       accumulator:  FrameAccumulator
//       denoiser:     Denoiser
//       motion_blur:  OfflineMotionBlur
//       color_xform:  ColorTransform
//       progress_tx:  std::sync::mpsc::Sender<RenderProgress>
//   - `pub struct RenderProgress`:
//       frame:        u64
//       total_frames: u64
//       elapsed_secs: f64
//       eta_secs:     f64
//   - `impl OfflineRenderer`:
//       `pub fn new(config: OfflineConfig, progress_tx: Sender<RenderProgress>)
//                  -> Result<Self, VideoError>`
//             Initialises all pipeline stages from config:
//               encoder  = encode::best_encoder(&config)
//               denoiser = Denoiser::best_available(ctx)
//               motion_blur = OfflineMotionBlur::new(config.shutter_angle, config.subframes)
//               color_xform = ColorTransform::new(config.output_space, config.lut_path)
//       `pub fn render_frame(&mut self, frame_idx: u64,
//                             sim: &mut SimState, ctx: &GpuContext) -> Result<(), VideoError>`
//             Main per-frame logic (deterministic):
//               1. For each sub-frame N in 0..config.subframes:
//                  a. Set sim.physics_time = motion_blur.time_for_subframe(frame_idx, N, fps)
//                  b. Dispatch GPU ray march (via existing render pipeline)
//                  c. Read back rendered texture to CPU
//                  d. accumulator.add_frame(pixels)
//               2. averaged = accumulator.average()
//               3. If denoising enabled: denoiser.denoise(averaged)
//               4. color_xform.apply_frame(&mut averaged)
//               5. encoder.encode_frame(as_yuv(&averaged), pts)
//               6. progress_tx.send(progress)
//               7. accumulator.reset()
//       `pub fn finish(&mut self) -> Result<(), VideoError>`
//             encoder.flush(), writes final bytes, closes output file.
//       `pub fn cancel(&mut self)`
//             Sets a cancellation flag; render_frame returns early on next call.
//
// OUTBOUND DEPENDENCIES:
//   - config.rs                     → OfflineConfig
//   - offline/accumulator.rs        → FrameAccumulator
//   - offline/motion_blur.rs        → OfflineMotionBlur
//   - denoise/mod.rs                → Denoiser
//   - encode/mod.rs                 → Encoder trait, best_encoder()
//   - color/transform.rs            → ColorTransform
//   - app/state/sim_state.rs        → SimState (physics time control)
//   - gpu/context.rs                → GpuContext (GPU dispatch)
//   - errors.rs                     → VideoError
//
// INBOUND (who calls OfflineRenderer):
//   - gargantua-app (via event_bus) → starts renderer in a background thread
//   - ui/overlay/render_progress.rs → receives RenderProgress via the channel
//
// NOTES:
//   - render_frame() is deterministic: the same frame_idx + SimState always
//     produces identical output. This is critical for pause/resume support.
//   - The renderer runs on a dedicated std::thread (not tokio async) to avoid
//     blocking the winit event loop.
// =============================================================================
