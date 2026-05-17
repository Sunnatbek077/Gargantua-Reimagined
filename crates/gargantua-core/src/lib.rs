// ============================================================
// FILE: crates/gargantua-core/src/lib.rs
// LINES: ~45
// CATEGORY: Core — crate entry point
// PLATFORM: Mac + Windows + WASM
// ============================================================
//
// PURPOSE:
//   Public interface of the gargantua-core crate.
//   Declares engine foundation modules: GPU context, frame graph,
//   resource pool, clock, adaptive quality, platform HAL, and App.
//   Contains zero per-frame logic — only module wiring and re-exports.
//
// CONTENTS (~45 lines):
//   pub mod app;        // App struct, render_frame(), winit integration
//   pub mod clock;      // DeltaTime, simulation vs wall clock
//   pub mod errors;     // CoreError (thiserror)
//   pub mod frame;      // FrameGraph, Pass trait, ResourcePool, barriers
//   pub mod gpu;        // GpuContext, GpuSurface, GpuProfiler, limits
//   pub mod logging;    // tracing subscriber setup (native + WASM)
//   pub mod platform;   // macOS / Windows HAL (cfg-gated subtrees)
//   pub mod quality;    // QualityPreset, AdaptiveQuality, detector
//
// USES (imports from):
//   All sub-modules above (via `pub mod` only).
//
// USED BY:
//   crates/gargantua-core/src/app.rs
//     → re-exports and composes subsystems declared here
//   crates/gargantua-render/src/lib.rs
//     → GpuContext, FrameGraph, Pass, ResourcePool
//   crates/gargantua-bake/src/scheduler.rs
//     → GpuContext for offline LUT compute
//   crates/gargantua-video/src/offline/renderer.rs
//     → GpuContext, ResourcePool for export path
//   PLANNED: crates/gargantua-app/src/main.rs
//     → binary entry; constructs gargantua_core::app::App
//
// NOTE FOR AI:
//   This file is module declarations + re-exports only.
//   The runnable application loop lives in app.rs (this crate), not in
//   gargantua-app. gargantua-app orchestrates SimState, plugins, and UI;
//   gargantua-core owns GPU, frame graph, and platform HAL.
//   Register every new top-level module here with `pub mod`.
// ============================================================
