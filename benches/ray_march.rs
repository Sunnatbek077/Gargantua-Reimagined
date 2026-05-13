// =============================================================================
// FILE: benches/ray_march.rs
// CRATE: gargantua-render (benchmark)
// LINES: ~140
// PLATFORM: Mac + Windows (requires physical GPU; skipped on CI without GPU)
// =============================================================================
//
// PURPOSE:
//   GPU-side benchmarks for the ray marching compute pass using wgpu timestamp
//   queries. Measures actual GPU execution time (not CPU dispatch time) for the
//   ray_march.wgsl compute shader at different quality settings and resolutions.
//   Results drive decisions about SPP limits per quality tier and render_scale
//   defaults in the quality preset tables.
//
// WHAT THIS FILE CONTAINS:
//
//   --- IMPORTS ---
//   use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
//   use gargantua_core::gpu::context::GpuContext;
//   use gargantua_core::gpu::profiler::GpuProfiler;
//   use gargantua_render::pipelines::ray_march::RayMarchPipeline;
//   use gargantua_core::quality::preset::QualityPreset;
//   use gargantua_physics::metric::kerr::KerrMetric;
//   use pollster::block_on;
//
//   --- SETUP HELPER ---
//   struct GpuBenchContext {
//       ctx:         GpuContext,
//       pipeline:    RayMarchPipeline,
//       profiler:    GpuProfiler,
//       output_tex:  wgpu::Texture,
//   }
//
//   fn setup_gpu_bench(width: u32, height: u32, preset: &QualityPreset)
//                      -> Option<GpuBenchContext>
//         Calls block_on(GpuContext::new_headless()) — creates a wgpu device
//         without a window surface (offscreen rendering for benchmarks).
//         Returns None if no GPU is available (CI without GPU; benchmark is
//         silently skipped via #[cfg] or a runtime None check).
//         Creates RayMarchPipeline and GpuProfiler with 1 pass slot.
//         Creates a STORAGE | COPY_SRC output texture at width × height.
//
//   fn gpu_time_ms(ctx: &mut GpuBenchContext) -> f32
//         Records one full ray march dispatch into a CommandEncoder:
//           profiler.begin_pass(encoder, 0)
//           pipeline.record(encoder, &resources)
//           profiler.end_pass(encoder, 0)
//           profiler.resolve(encoder)
//         Submits encoder to ctx.queue().
//         Calls ctx.device().poll(wgpu::Maintain::Wait) to wait for GPU.
//         Calls profiler.read_results() and returns results[0] (ms).
//
//   --- BENCHMARKS ---
//
//   fn bench_ray_march_1080p_spp1(c: &mut Criterion)
//         Benchmarks the ray march pass at 1920×1080, SPP=1 (single sample).
//         This is the minimum quality (Potato preset) baseline.
//         Measures GPU ms per frame via gpu_time_ms().
//         Expected: < 4 ms on M1, < 2 ms on M4 Max, < 3 ms on RTX 4090.
//
//   fn bench_ray_march_1080p_spp_sweep(c: &mut Criterion)
//         Sweeps SPP values [1, 2, 4, 8, 16, 32] at 1920×1080 as BenchmarkIds.
//         Plots GPU time vs SPP — should be linear (each sample = same cost).
//         Any super-linear growth indicates a memory bandwidth bottleneck
//         (cache thrashing at high SPP due to accumulation buffer size).
//
//   fn bench_ray_march_4k_high_preset(c: &mut Criterion)
//         Benchmarks at 3840×2160 (4K) with QualityPreset::high()
//         (SPP=8, ray_steps=128).
//         Target: < 16.6 ms (60 FPS budget) on a high-end GPU.
//         If this exceeds 16.6 ms on the reference hardware, reduce the
//         default SPP for the High preset in quality/preset.rs.
//
//   fn bench_ray_march_8k_ultra_plus(c: &mut Criterion)
//         Benchmarks at 7680×4320 (8K) with QualityPreset::ultra_plus()
//         (SPP=128, ray_steps=512).
//         This is the offline render quality ceiling.
//         Expected: 200–800 ms per frame (not real-time; offline only).
//         Measures the lower bound for offline frame time at max quality.
//
//   fn bench_ray_march_spin_variation(c: &mut Criterion)
//         Benchmarks ray march at 1920×1080, SPP=4 for three spin values:
//         a = 0.0 (Schwarzschild), a = 0.5, a = 0.998 (near-extremal Kerr).
//         Higher spin → more complex geodesic deflection → more ray steps used.
//         Validates that near-extremal Kerr does not cause runaway step counts
//         that blow the 16.6 ms frame budget.
//
//   fn bench_ray_march_workgroup_size(c: &mut Criterion)
//         Compares three workgroup configurations at 1920×1080, SPP=4:
//           (8, 8, 1) = 64 threads  — Apple Silicon default
//           (16, 8, 1) = 128 threads
//           (8, 4, 1)  = 32 threads — NVIDIA warp-aligned
//         Uses BenchmarkId labels "64t", "128t", "32t".
//         Identifies the optimal workgroup size for the current GPU.
//         Results feed into platform/macos/compute/threadgroup.rs and
//         platform/windows/compute/workgroup.rs optimal_*_config() functions.
//
//   criterion_group!(
//       benches,
//       bench_ray_march_1080p_spp1,
//       bench_ray_march_1080p_spp_sweep,
//       bench_ray_march_4k_high_preset,
//       bench_ray_march_8k_ultra_plus,
//       bench_ray_march_spin_variation,
//       bench_ray_march_workgroup_size,
//   );
//   criterion_main!(benches);
//
// OUTBOUND DEPENDENCIES:
//   - criterion (dev-dependency)              → benchmarking framework
//   - pollster (dev-dependency)               → block_on() for async GPU init
//   - gargantua_core::gpu::context            → GpuContext::new_headless()
//   - gargantua_core::gpu::profiler           → GpuProfiler (timestamp queries)
//   - gargantua_render::pipelines::ray_march  → RayMarchPipeline
//   - gargantua_core::quality::preset         → QualityPreset
//   - gargantua_physics::metric::kerr         → KerrMetric (for spin variation bench)
//   - wgpu (external)                         → Texture, CommandEncoder, Maintain
//
// HOW TO RUN:
//   cargo bench -p gargantua-render --bench ray_march
//   cargo bench -p gargantua-render --bench ray_march -- bench_ray_march_4k
//       (run only the 4K benchmark)
//   Results saved to: target/criterion/ray_march/report/index.html
//
// NOTES:
//   - Timestamp queries are NOT available in WASM (WebGPU restriction).
//     These benchmarks only run on native Mac and Windows builds.
//   - GpuContext::new_headless() uses wgpu::RequestAdapterOptions with
//     force_fallback_adapter = false; if no real GPU is found, setup_gpu_bench()
//     returns None and the benchmark prints "GPU not available, skipping."
//   - CPU dispatch overhead (Rust → wgpu → Metal/DX12) is NOT measured here;
//     only the GPU execution time between timestamp begin/end is reported.
//   - Run benchmarks with the machine plugged in (not on battery) and with
//     other GPU workloads closed. macOS Power Nap and Windows Power Throttling
//     can add > 50% variance to GPU benchmark results.
//   - Compare results between platforms in docs/benchmarks.md.
//     Reference hardware: Apple M4 Max (40 GPU cores), NVIDIA RTX 5090.
// =============================================================================