// ============================================================
// FILE: examples/headless_render.rs
// LINES: ~200
// CATEGORY: Example — Single-frame headless render to file
// RUN: cargo run --example headless_render --release -- --output render.exr
// ============================================================
//
// PURPOSE:
//   Simplest possible headless render: renders a single frame with
//   default parameters and saves it to a file. No GUI, no animation.
//   Useful as a smoke test to verify the render pipeline works.
//   Simpler than export_sequence.rs — good starting point.
//
// CONTENTS (~200 lines):
//   fn main()
//     // 1. Parse CLI args: --output <path> (default: render.exr)
//     //    --width N (default 1920), --height N (default 1080)
//     //    --spp N (default 1, for fast smoke test)
//     //    --preset <name> (default "interstellar")
//     //
//     // 2. Print system info:
//     //    "Initializing wgpu..."
//     //    "GPU: {adapter_name}" (e.g. "Apple M1 Pro")
//     //    "Resolution: {width}×{height}, SPP: {spp}"
//     //
//     // 3. Initialize wgpu headlessly
//     //
//     // 4. Check for baked assets, load them
//     //    (print helpful error if missing)
//     //
//     // 5. Load preset, build physics params
//     //
//     // 6. Render single frame:
//     //    let t0 = Instant::now();
//     //    render_frame(&device, &queue, &pipeline, &output_texture)
//     //    println!("Rendered in {:.2}s", t0.elapsed().as_secs_f32());
//     //
//     // 7. Read back texture and save:
//     //    if output.ends_with(".exr") → write EXR (linear HDR)
//     //    if output.ends_with(".png") → tonemap then write PNG
//     //    println!("Saved to {}", output_path);
//
// USES (imports from):
//   gargantua_render::RenderPipeline
//   gargantua_bake::BakeParams
//   gargantua_ui::presets::BuiltinPreset
//   wgpu, pollster
//   image  → PNG output
//   exr    → EXR output
//   std::{env, time::Instant}
//
// USED BY:
//   CI smoke tests — run after every merge to main to verify no crashes
//   New contributors — first render example to try
//   README.md — linked as "Quick Start" headless render command
//
// NOTE FOR AI:
//   SPP=1 produces noisy output but is fast (smoke test purpose).
//   For a clean reference render: --spp 64 (takes ~30s on M1 Pro).
//   EXR output is preferred over PNG: preserves HDR values for later
//   tonemapping and compositing in external tools (DaVinci Resolve etc.).
//   png output: apply ACES tonemap internally before writing 8-bit sRGB.
//   Print GPU adapter name so CI logs show which GPU was used.
// ============================================================