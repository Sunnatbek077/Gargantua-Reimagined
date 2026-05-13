// ============================================================
// FILE: examples/export_sequence.rs
// LINES: ~240
// CATEGORY: Example — Headless render of an image sequence
// RUN: cargo run --example export_sequence --release -- --frames 24 --output ./frames/
// ============================================================
//
// PURPOSE:
//   Headless (no window) example that renders a sequence of frames
//   using gargantua-render. Iterates through a camera path defined
//   in a JSON file (from camera_path.rs output) and renders each
//   frame to a PNG or EXR file. Used for video production.
//
// CONTENTS (~240 lines):
//   fn main()
//     // 1. Parse CLI args:
//     //    --frames N (total frames, default 60)
//     //    --fps N (frame rate for camera path time, default 24)
//     //    --output <dir> (output directory for frame files)
//     //    --camera <json> (camera path JSON from camera_path.rs)
//     //    --format png|exr (output format, default png)
//     //    --spp N (samples per pixel, default 4)
//     //    --width N --height N (resolution, default 1920×1080)
//     //    --preset <name> (built-in preset to use, default "interstellar")
//     //
//     // 2. Load camera path from JSON (or use default orbit if no --camera)
//     //
//     // 3. Initialize wgpu headlessly (no surface)
//     //
//     // 4. Load BakeParams → load baked LUTs from assets/baked/
//     //    Panic with helpful message if LUTs are missing (run bake_only first)
//     //
//     // 5. Initialize RenderPipeline with the loaded assets
//     //
//     // 6. For frame in 0..n_frames:
//     //    t = frame as f32 / fps
//     //    cam = interpolate_camera_path(&path, t)
//     //    update_render_uniforms(cam, &physics_params)
//     //    render_frame(&device, &queue, &pipeline)
//     //    texture_to_file(&output_texture, &format!("{}/frame_{:04}.{}", out_dir, frame, ext))
//     //    println!("Rendered frame {}/{}", frame+1, n_frames)
//     //
//     // 7. Print total time and average fps
//
// USES (imports from):
//   gargantua_render::RenderPipeline
//   gargantua_bake::BakeParams
//   gargantua_ui::presets::BuiltinPreset
//   wgpu, pollster
//   serde_json  → load camera path JSON
//   image       → save PNG frames (external crate)
//   exr         → save EXR frames
//   std::{fs, env, path, time::Instant}
//
// USED BY:
//   Video production workflow (see docs/video_export.md)
//   CI visual regression tests (renders reference frames for comparison)
//
// NOTE FOR AI:
//   MUST run with --release (render is ~10× slower in debug).
//   Headless wgpu requires no winit surface — just Device + Queue.
//   Output texture must be BGRA8 (PNG) or RGBA32F (EXR).
//   Readback: wgpu::Buffer with MAP_READ usage, then copy texture → buffer.
//   LUT check: if assets/baked/geodesic_lut.exr is missing, print:
//     "Error: LUTs not found. Run: cargo run --example bake_only first."
//   Average frame time should be ~0.5–5s depending on spp and resolution.
// ============================================================