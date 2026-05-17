// ============================================================
// FILE: tests/bake_cache.rs
// LINES: ~220
// CATEGORY: Integration test — Cross-crate bake cache + pipeline smoke test
// RUN: cargo test --test bake_cache
// ============================================================
//
// PURPOSE:
//   Top-level integration test that verifies the bake cache works
//   correctly end-to-end across crate boundaries (gargantua-bake +
//   gargantua-render). Tests the full bake → cache → render load
//   cycle without opening a window.
//   Complements the unit tests in crates/gargantua-bake/tests/cache.rs
//   which test the cache module in isolation.
//
// TESTED SCENARIO:
//   1. Run a minimal bake (blackbody LUT + Doppler LUT only)
//   2. Verify assets are written to a temp directory
//   3. Verify cache is marked fresh
//   4. Run a second bake — verify the cache skips re-baking (is_fresh = true)
//   5. Corrupt one asset — verify cache detects staleness (is_fresh = false)
//   6. Run bake again — verify only the stale asset is re-baked
//
// SETUP:
//   fn headless_device() -> (wgpu::Device, wgpu::Queue)
//     // pollster::block_on(async { ... })
//     // wgpu::Instance::new(Default::default())
//     //   .request_adapter(&wgpu::RequestAdapterOptions {
//     //       power_preference: wgpu::PowerPreference::HighPerformance,
//     //       compatible_surface: None, ..Default::default()
//     //   })
//     //   .await.unwrap()
//     //   .request_device(&wgpu::DeviceDescriptor::default(), None)
//     //   .await.unwrap()
//
//   fn minimal_bake_params(dir: &std::path::Path) -> BakeParams
//     // Only blackbody + Doppler:
//     // blackbody_lut_size = 64, doppler_n_beta = 32, doppler_n_lambda = 32
//     // geo_spin_steps = 0 (skip geodesic bake — too slow for CI)
//     // blue_noise_size = 0, curl_noise_size = 0, starmap_sh_order = 0
//
// TEST CASES (~220 lines):
//
//   #[test]
//   fn test_bake_creates_assets()
//     // Run minimal bake → Ok(())
//     // Check: blackbody_lut.exr exists in temp dir
//     // Check: doppler_lut.exr exists in temp dir
//     // Check: .bake_cache.toml exists in temp dir
//
//   #[test]
//   fn test_second_bake_uses_cache()
//     // First bake → success
//     // Record modification times of .exr files
//     // Second bake with same params → success
//     // Check: .exr file mtimes unchanged (files not re-written)
//     // (Cache hit = files not touched)
//
//   #[test]
//   fn test_cache_invalidated_after_param_change()
//     // First bake with blackbody_lut_size=64
//     // Second bake with blackbody_lut_size=128
//     // Cache should detect hash mismatch → re-bake blackbody
//     // blackbody_lut.exr file size should increase (128 > 64 entries)
//
//   #[test]
//   fn test_corrupted_asset_triggers_rebake()
//     // First bake → success
//     // Corrupt blackbody_lut.exr: write 10 bytes of garbage
//     // Second bake → detects file_size mismatch → re-bakes blackbody
//     // blackbody_lut.exr should be valid EXR again after second bake
//
//   #[test]
//   fn test_force_rebake_ignores_cache()
//     // First bake → success
//     // Second bake with force_rebake=true
//     // Both .exr files should be re-written (newer mtime)
//
//   #[test]
//   fn test_bake_cancel_midway()
//     // Set cancel_flag=true immediately before running scheduler
//     // scheduler.run() → Err(BakeError::Cancelled)
//     // No partial .exr files should be written to disk
//     // (Cancel before any bake step starts)
//
//   #[test]
//   fn test_progress_events_received()
//     // Run minimal bake with mpsc channel
//     // Collect all BakeProgressEvent in a Vec
//     // assert!(!events.is_empty())
//     // assert!(events.last().unwrap().overall >= 0.99)
//     // Events: overall progress is monotonically non-decreasing
//
// USES (imports from):
//   gargantua_bake::{BakeScheduler, BakeParams, BakeError}
//   gargantua_bake::cache::{BakeCache, LutKind}
//   gargantua_bake::scheduler::BakeProgressEvent
//   wgpu, pollster
//   tempfile::TempDir
//   std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, fs}
//
// NOTE FOR AI:
//   This test lives in tests/ (not inside a crate) — it is a
//   workspace-level integration test, compiled as a separate binary.
//   Requires wgpu — will skip gracefully on CI with no GPU:
//     if adapter is None: println!("No GPU, skipping"); return;
//   bake step weights in scheduler: Geodesic=0.50 — skip it (size=0)
//   to keep CI runtime < 30s.
//   File mtime comparison: use std::fs::metadata().modified().unwrap().
// ============================================================