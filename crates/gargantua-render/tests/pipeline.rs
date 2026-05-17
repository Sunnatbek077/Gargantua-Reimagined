// =============================================================================
// crates/gargantua-render/tests/pipeline.rs
// =============================================================================
//
// PURPOSE:
//   Integration tests for the gargantua-render pipeline system. Verifies that
//   all render and compute passes can be constructed, registered into the
//   FrameGraph, and executed on a headless GPU device without panicking or
//   producing GPU validation errors.
//
//   These tests do NOT check visual output (pixel-level correctness) — they
//   verify structural integrity: bind group layouts match shader declarations,
//   resource dependencies are correctly declared, pipeline creation succeeds,
//   and the FrameGraph executes without API errors.
//
//   If no GPU adapter is available (CI on ubuntu-latest without GPU), all
//   tests in this file are skipped gracefully via GpuContext::try_headless().
//
// SIZE: ~280 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_core::gpu::context::GpuContext
//     - gargantua_core::frame::frame_graph::FrameGraph
//     - gargantua_core::frame::resource::ResourcePool
//     - gargantua_core::quality::preset::QualityPreset
//     - gargantua_render::bindgroups::scene::{SceneBindGroup, SceneUniforms}
//     - gargantua_render::bindgroups::textures::{TexturesBindGroup, BakedTextures}
//     - gargantua_render::pipelines::ray_march::RayMarchPass
//     - gargantua_render::pipelines::geodesic_gpu::GeodesicGpuPass
//     - gargantua_render::pipelines::accretion::AccretionPass
//     - gargantua_render::pipelines::lensing::LensingPass
//     - gargantua_render::pipelines::starfield::StarfieldPass
//     - gargantua_render::postfx::{taa, bloom, chromatic, film_grain,
//                                    motion_blur, tonemap}
//     - gargantua_render::shader_reload::ShaderReloader
//   External:
//     - wgpu::{Device, Queue, TextureDescriptor, TextureUsages, TextureFormat,
//              Extent3d, TextureDimension}
//     - std::path::PathBuf
//
// TEST STRUCTURE:
//   Each test follows the same 4-step pattern:
//     1. SETUP:    acquire a headless GpuContext (skip if unavailable)
//     2. CREATE:   construct the pass / resource under test
//     3. EXECUTE:  run one frame through the FrameGraph
//     4. VERIFY:   assert no wgpu validation errors were emitted
//
// HEADLESS GPU SETUP:
//   GpuContext::try_headless() attempts to create a wgpu device with:
//     - Backend: Vulkan (Linux CI), Metal (macOS), DX12 (Windows)
//     - No surface (offscreen only)
//     - Minimal features (no f16, no timestamp queries)
//   Returns None if no adapter is found, in which case tests are skipped.
//   On GitHub Actions ubuntu-latest: uses software rasterizer (llvmpipe/lavapipe)
//   if a real GPU is not available — compute shaders run correctly but slowly.
//
// SKIPPING STRATEGY:
//   ```rust
//   let Some(ctx) = GpuContext::try_headless() else {
//       eprintln!("[skip] No GPU adapter — skipping render pipeline test");
//       return;
//   };
//   ```
//   This prevents CI failures on machines without GPU support.
//
// TEST CASES:
//
//   test scene_bind_group_creation
//     — creates SceneBindGroup on the headless device.
//     — calls update() with a zero-initialized SceneUniforms.
//     — asserts no GPU errors. Verifies the uniform buffer exists and has
//       correct size (std::mem::size_of::<SceneUniforms>() bytes).
//
//   test textures_bind_group_creation
//     — creates minimal placeholder textures for all 5 BakedTextures slots:
//         geodesic_lut:  1×1 Rgba32Float
//         blackbody_lut: 1×1 Rgba16Float (1D texture)
//         doppler_lut:   1×1 Rgba16Float
//         blue_noise_3d: 1×1×1 R8Unorm (3D)
//         starmap:       1×1 Rgba16Float
//     — creates TexturesBindGroup from these placeholders.
//     — asserts bind group layout matches the expected 7 entries
//       (5 textures + 2 samplers).
//     — verifies lut_sampler uses Linear filtering and noise_sampler uses Nearest.
//
//   test ray_march_pass_creation
//     — loads ray_march.wgsl via ShaderReloader.
//     — creates SceneBindGroup, TexturesBindGroup, and a 1×1 output texture.
//     — constructs RayMarchPass with a minimal QualityPreset (spp=1, steps=16).
//     — asserts Ok(pass) — pipeline compilation succeeded.
//     — verifies pass.name() == "ray_march".
//
//   test geodesic_gpu_pass_creation
//     — loads geodesic_rk4.wgsl.
//     — creates GeodesicGpuPass with a Schwarzschild GeodesicParams (spin=0).
//     — asserts pipeline creation succeeded.
//     — calls update_params() with spin=0.5 (Kerr params change) — no panic.
//
//   test accretion_pass_creation
//     — loads accretion_disk.wgsl.
//     — creates AccretionPass with AccretionParams for M87* preset.
//     — asserts pipeline creation succeeded.
//
//   test full_pipeline_frame_graph
//     — registers all 5 render passes into a FrameGraph in the correct order:
//         1. StarfieldPass
//         2. GeodesicGpuPass
//         3. RayMarchPass
//         4. AccretionPass
//         5. LensingPass
//     — declares resource dependencies (reads/writes).
//     — calls FrameGraph::execute() for one frame.
//     — asserts no wgpu validation errors.
//     — verifies topological sort completed (no CyclicDependency error).
//
//   test post_fx_chain
//     — registers all 7 post-fx passes in order:
//         TaaPass → BloomPass → ChromaticPass → FilmGrainPass →
//         MotionBlurPass → LensFlarePass → TonemapPass
//     — executes one frame through the complete post-fx chain.
//     — asserts each pass transitions correctly between input/output textures.
//     — verifies the final TonemapPass writes to the swapchain-format texture.
//
//   test shader_reload_loads_all_shaders
//     — creates ShaderReloader pointing at shaders/ directory.
//     — calls load_shader() for every WGSL file in shaders/.
//     — asserts all shaders compile without errors.
//     — verifies that a deliberately broken WGSL (test fixture) returns
//       Err(RenderError::ShaderCompilation { .. }).
//
//   test bind_group_layout_consistency
//     — creates the bind group layout for each pass independently.
//     — creates a dummy PipelineLayout with the same layouts.
//     — calls device.create_compute_pipeline() with each pass's layout.
//     — asserts no InvalidResource or BindTypeMismatch wgpu errors.
//     — This is the most important structural test: it catches group/binding
//       declaration mismatches between Rust and WGSL without running the shader.
//
//   test resource_pool_transient_lifecycle
//     — allocates a transient RGBA16Float texture via ResourcePool.
//     — registers it as RayMarchPass output (write resource).
//     — calls FrameGraph::reset() — verifies the texture is returned to pool.
//     — allocates again — verifies the same texture slot is reused (pool recycles).
//
//   test taa_history_persistent
//     — creates TaaPass with a 64×64 resolution.
//     — verifies the TAA history buffer is allocated as a PERSISTENT resource
//       (ResourceHandle that survives FrameGraph::reset()).
//     — calls notify_resize() with 128×128 — verifies old history is released
//       and new persistent resource is allocated at the new resolution.
//
// HELPER FUNCTIONS:
//
//   fn headless_ctx() -> Option<(GpuContext, ResourcePool, FrameGraph)>
//     — convenience: creates GpuContext::try_headless(), then wraps it with
//       ResourcePool and FrameGraph. Returns None if no GPU available.
//
//   fn minimal_baked_textures(device: &wgpu::Device, queue: &wgpu::Queue)
//       -> BakedTextures
//     — creates 1-pixel placeholder textures for all 5 BakedTextures fields.
//     — used by most tests that need a TexturesBindGroup without real baked data.
//
//   fn minimal_quality_preset() -> QualityPreset
//     — returns a QualityPreset with minimum settings for fast pipeline creation:
//         spp=1, max_steps=16, workgroup=(8,8), all post-fx disabled.
//     — avoids long pipeline compilation in CI.
//
//   fn shader_dir() -> std::path::PathBuf
//     — returns path to shaders/ relative to the workspace root.
//     — uses std::env::var("CARGO_MANIFEST_DIR") to find the project root.
//
// NOTES FOR AI:
//   - All GPU tests are integration tests (in tests/ dir, not #[test] in src/).
//     They are compiled as a separate binary and can use #[test] normally.
//   - wgpu validation errors are NOT automatically surfaced as Rust panics.
//     Use device.push_error_scope(wgpu::ErrorFilter::Validation) before
//     and device.pop_error_scope().block_on() after to catch them explicitly.
//   - The test for broken WGSL requires a test fixture file at:
//       PLANNED: crates/gargantua-render/tests/fixtures/broken.wgsl
//     containing a syntax error (e.g., `fn main() { invalid_syntax!!! }`).
//   - Shader loading in tests uses the real shaders/ directory (not embedded).
//     This means tests must be run from the workspace root or with the correct
//     CARGO_MANIFEST_DIR set. `cargo test` handles this automatically.
//   - ResourcePool in tests should be created with device.clone() (Arc<Device>).
//   - FrameGraph::execute() requires a valid CommandEncoder — create one with
//     device.create_command_encoder(&Default::default()) and submit after.
// =============================================================================
