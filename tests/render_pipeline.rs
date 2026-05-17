// ============================================================
// FILE: tests/render_pipeline.rs
// LINES: ~260
// CATEGORY: Integration test — Full render pipeline smoke test
// RUN: cargo test --test render_pipeline
// ============================================================
//
// PURPOSE:
//   End-to-end smoke test for the entire render pipeline:
//   loads pre-baked assets, initializes the wgpu render pipeline,
//   renders a single frame headlessly, reads back the pixel buffer,
//   and validates the output against known reference values.
//   Catches regressions in shader code, pipeline config, or
//   physics-to-GPU parameter uploads.
//
// PREREQUISITE:
//   Baked assets must exist in assets/baked/ before running.
//   Run: cargo run --example bake_only -- --assets ./assets
//   (Or use the pre-baked assets committed to the repository)
//
// TEST CASES (~260 lines):
//
//   #[test]
//   fn test_render_pipeline_initializes()
//     // Initialize headless wgpu device + queue
//     // Load baked assets from assets/baked/ (skip if missing)
//     // Build RenderPipeline with default InterstellarGargantua preset
//     // assert!(pipeline is created without panic)
//
//   #[test]
//   fn test_single_frame_no_crash()
//     // Render 1 frame headlessly (spp=1, 256×144 resolution for speed)
//     // assert!(render_frame() does not panic or return Err)
//     // Basic smoke test: pipeline runs without wgpu errors
//
//   #[test]
//   fn test_output_not_all_black()
//     // Render 1 frame, read back pixel buffer
//     // Compute average luminance of all pixels
//     // assert!(avg_luminance > 0.001)
//     // (If all black: shader bug, wrong camera position, or asset failure)
//
//   #[test]
//   fn test_output_not_all_white()
//     // Render 1 frame, read back pixel buffer
//     // assert!(avg_luminance < 0.999)
//     // (If all white: tonemapping broken, exposure too high)
//
//   #[test]
//   fn test_horizon_pixels_are_black()
//     // Render with camera very close to the BH (r = 3.0 M)
//     // The event horizon shadow should occupy the screen center
//     // Sample center 10×10 pixel block
//     // assert!(center_luminance < 0.01)  // near-black center
//
//   #[test]
//   fn test_disk_pixels_are_warm()
//     // Render with camera at equatorial plane (theta = pi/2), r = 20M
//     // The accretion disk should produce warm orange-yellow pixels
//     // Sample horizontal band at screen center (disk location)
//     // assert!(disk_red_channel > disk_blue_channel)  // warm color
//
//   #[test]
//   fn test_schwarzschild_symmetric_lensing()
//     // Render with spin=0 (Schwarzschild)
//     // Lensing should be left-right symmetric (no frame dragging)
//     // Compare luminance of left half vs right half of frame
//     // assert!(abs(left_lum - right_lum) < 0.05)  // within 5%
//
//   #[test]
//   fn test_kerr_asymmetric_disk()
//     // Render with spin=0.9 (Kerr)
//     // Doppler beaming: approaching side (left) brighter than receding (right)
//     // Compare luminance of left disk vs right disk
//     // assert!(left_disk_lum > right_disk_lum * 1.5)  // left significantly brighter
//
//   #[test]
//   fn test_high_spin_smaller_shadow()
//     // Render spin=0.0 and spin=0.9 at same resolution
//     // Count black pixels (r < threshold) in center
//     // assert!(shadow_pixels_spin09 < shadow_pixels_spin00)
//     // High spin → smaller photon sphere → smaller shadow
//
//   #[test]
//   fn test_postfx_tonemap_no_overflow()
//     // Render with ACES tonemap, read back LDR buffer
//     // All pixel values should be in [0.0, 1.0] range
//     // assert!(max_pixel_value <= 1.0)
//
//   #[test]
//   fn test_bloom_increases_bright_pixel_count()
//     // Render with bloom_enabled=false, count pixels > 0.9
//     // Render with bloom_enabled=true, count pixels > 0.9
//     // assert!(bloom_bright_pixels > no_bloom_bright_pixels)
//     // (Bloom spreads bright pixels to neighbors)
//
//   #[test]
//   fn test_frame_deterministic()
//     // Render same frame twice with identical params
//     // Read back both pixel buffers
//     // assert!(buffer_1 == buffer_2)  // bitwise identical
//     // (Ensures no random/time-seeded non-determinism in shaders)
//
// SETUP HELPERS:
//   fn headless_device() -> Option<(wgpu::Device, wgpu::Queue)>
//     // Returns None if no GPU adapter available (CI skip)
//
//   fn load_assets_or_skip() -> Option<BakedAssets>
//     // Returns None if assets/baked/ files are missing
//     // Prints: "Skipping: run bake_only example first"
//
//   fn render_small_frame(
//       device: &wgpu::Device, queue: &wgpu::Queue,
//       assets: &BakedAssets, preset: BuiltinPreset,
//       width: u32, height: u32, spp: u32,
//   ) -> Vec<f32>
//     // Returns flat RGBA f32 pixel buffer (width * height * 4 floats)
//
//   fn avg_luminance(pixels: &[f32]) -> f32
//     // BT.709: dot(rgb, (0.2126, 0.7152, 0.0722)) averaged over all pixels
//
// USES (imports from):
//   gargantua_render::RenderPipeline
//   gargantua_bake::BakeParams
//   gargantua_ui::presets::BuiltinPreset
//   wgpu, pollster
//   std::f32::consts::PI
//
// NOTE FOR AI:
//   Resolution 256×144 keeps test runtime < 5s per test case on M1 Pro.
//   spp=1 for smoke tests, spp=4 for visual quality tests.
//   All tests use #[cfg(not(ci_no_gpu))] to skip gracefully on CPU-only CI.
//   test_frame_deterministic: critical regression test — any non-determinism
//   (rand, time-seeded, race condition) will be caught here.
//   Pixel buffer readback: wgpu texture → staging buffer → map_read → Vec<f32>.
// ============================================================