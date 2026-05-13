// ============================================================
// FILE: examples/bake_only.rs
// LINES: ~120
// CATEGORY: Example — Run the bake pipeline without opening a window
// RUN: cargo run --example bake_only -- --assets ./assets
// ============================================================
//
// PURPOSE:
//   Standalone CLI example that runs the entire LUT bake pipeline
//   headlessly (no GUI, no wgpu window). Useful for CI/CD pipelines,
//   server-side pre-baking, or scripted asset generation.
//   Prints bake progress to stdout and exits when done.
//
// CONTENTS (~120 lines):
//   fn main()
//     // 1. Parse CLI args: --assets <dir>, --force, --spin-steps N,
//     //    --impact-steps N, --blackbody-size N
//     //
//     // 2. Build BakeParams from CLI args
//     //    (uses BakeParams defaults for unspecified fields)
//     //
//     // 3. Create wgpu Device + Queue headlessly:
//     //    pollster::block_on(async {
//     //        wgpu::Instance::new(...).request_adapter(...).request_device(...)
//     //    })
//     //
//     // 4. Create mpsc channel for progress events
//     //
//     // 5. Spawn background thread: BakeScheduler::new(params, cancel).run(tx)
//     //
//     // 6. Main thread: receive BakeProgressEvent in a loop, print:
//     //    "[62%] Baking geodesic LUT (spin 5/8)..."
//     //    Use 
 to overwrite the same line (progress bar in terminal)
//     //
//     // 7. On completion: print "Bake complete in 45.2s" and exit 0
//     //    On BakeError: print error and exit 1
//
// USES (imports from):
//   gargantua_bake::{BakeScheduler, BakeParams}
//   gargantua_bake::scheduler::BakeProgressEvent
//   wgpu (external)
//   pollster (external)   → block_on for async wgpu init
//   std::{sync, thread, env}
//
// USED BY:
//   CI pipeline (GitHub Actions) — bakes assets before running tests
//   Developers who want to pre-bake without opening the full app
//
// NOTE FOR AI:
//   This is a binary example, not a library. Entry point is fn main().
//   Cargo.toml must have: [[example]] name = "bake_only" path = "examples/bake_only.rs"
//   Headless wgpu: use PowerPreference::HighPerformance, no surface.
//   pollster crate is a minimal async executor for blocking on wgpu futures.
//   Progress printing: use print!(...) + std::io::stdout().flush() for 
 trick.
// ============================================================