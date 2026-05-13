// =============================================================================
// FILE: benches/geodesic_rk4.rs
// CRATE: gargantua-physics (benchmark)
// LINES: ~120
// PLATFORM: Mac + Windows (native only; Criterion does not run on WASM)
// =============================================================================
//
// PURPOSE:
//   Criterion benchmarks for the CPU geodesic RK4 integrator. Measures the
//   performance of a single RK4 step, a full geodesic path integration, and
//   the adaptive step-size variant under different black hole spin parameters.
//   Results are used to track regressions and to justify GPU offloading
//   decisions (the GPU baking pipeline must be faster than this CPU baseline).
//
// WHAT THIS FILE CONTAINS:
//
//   --- IMPORTS ---
//   use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
//   use gargantua_physics::metric::kerr::KerrMetric;
//   use gargantua_physics::geodesic::rk4::{Rk4Integrator, PhotonState};
//   use gargantua_physics::geodesic::adaptive::AdaptiveRk4;
//   use gargantua_physics::geodesic::termination::StandardTermination;
//
//   --- HELPER ---
//   fn make_kerr(spin: f64) -> KerrMetric
//         Returns KerrMetric::new(1.0, spin, 0.0).unwrap().
//         Mass = 1.0 geometric units (normalised).
//
//   fn equatorial_initial_state(r: f64) -> PhotonState
//         Returns a PhotonState at (t=0, r, θ=π/2, φ=0) with null tangent
//         vector k^μ set for a photon on a bound equatorial orbit at radius r.
//         k^t and k^φ are computed from the Kerr effective potential.
//
//   --- BENCHMARKS ---
//
//   fn bench_single_rk4_step(c: &mut Criterion)
//         Benchmarks one call to Rk4Integrator::step() for a photon near the
//         photon sphere (r = 3.0 M) in a maximally spinning Kerr metric (a = 0.998).
//         Setup (outside benchmark loop):
//           integrator = Rk4Integrator::new(make_kerr(0.998), 0.1, 512)
//           state = equatorial_initial_state(3.0)
//         Benchmark loop:
//           black_box(integrator.step(black_box(&state)))
//         Expected throughput: ~5–15 million steps/second on Apple M-series.
//         Used to check that Christoffel symbol evaluation is not the bottleneck.
//
//   fn bench_full_geodesic_integration(c: &mut Criterion)
//         Benchmarks a complete geodesic path from r = 20 M until the photon
//         either escapes to r > 200 M or crosses the event horizon.
//         Tests three spin values as BenchmarkId parameters: 0.0, 0.5, 0.998.
//         Setup:
//           integrator = Rk4Integrator::new(make_kerr(spin), 0.2, 1024)
//           terminator = StandardTermination { r_max: 200.0, r_horizon: metric.event_horizon_radius() }
//         Benchmark loop:
//           black_box(integrator.integrate(black_box(state.clone()), &terminator))
//         BenchmarkId label: format!("spin={}", spin)
//         Expected: a = 0.998 takes ~20% more steps than a = 0.0 (more complex metric).
//
//   fn bench_adaptive_rk4_vs_fixed(c: &mut Criterion)
//         Compares AdaptiveRk4 against fixed-step Rk4Integrator on the same
//         near-horizon geodesic (r_start = 1.5 M, a = 0.9).
//         Groups both variants in a single Criterion BenchmarkGroup named
//         "geodesic_near_horizon" to produce a side-by-side comparison chart.
//         Fixed:    step = 0.05, max_steps = 4096
//         Adaptive: error_target = 1e-6, h_min = 1e-4, h_max = 0.5
//         Expected: Adaptive is ~1.4× slower per path but takes 30% fewer total
//         steps because it enlarges the step away from the horizon.
//
//   fn bench_christoffel_evaluation(c: &mut Criterion)
//         Microbenchmark of MetricTensor::christoffel() alone, isolating the
//         cost of the 40-term analytic Christoffel symbol computation.
//         Setup: metric = make_kerr(0.9), r = 3.0, theta = std::f64::consts::FRAC_PI_2
//         Benchmark loop:
//           black_box(metric.christoffel(black_box(r), black_box(theta)))
//         Expected: ~80–200 ns on M-series. If > 500 ns, the analytic expressions
//         should be refactored (precompute intermediate Σ, Δ terms once).
//
//   fn bench_geodesic_batch_100(c: &mut Criterion)
//         Benchmarks 100 independent geodesic integrations in sequence (no
//         parallelism) to simulate a single LUT row bake on one CPU thread.
//         Photons start at evenly spaced impact parameters b ∈ [2.6M, 20M].
//         Measures total wall time; reported as "time per 100 geodesics".
//         Used to estimate how long a full LUT bake (512 × 512 paths) would
//         take single-threaded. bake/scheduler.rs uses rayon for parallelism;
//         this bench gives the per-thread baseline.
//
//   criterion_group!(
//       benches,
//       bench_single_rk4_step,
//       bench_full_geodesic_integration,
//       bench_adaptive_rk4_vs_fixed,
//       bench_christoffel_evaluation,
//       bench_geodesic_batch_100
//   );
//   criterion_main!(benches);
//
// OUTBOUND DEPENDENCIES:
//   - criterion (dev-dependency)                     → benchmarking framework
//   - gargantua_physics::metric::kerr                → KerrMetric
//   - gargantua_physics::geodesic::rk4               → Rk4Integrator, PhotonState
//   - gargantua_physics::geodesic::adaptive          → AdaptiveRk4
//   - gargantua_physics::geodesic::termination       → StandardTermination
//
// HOW TO RUN:
//   cargo bench -p gargantua-physics --bench geodesic_rk4
//   cargo bench -p gargantua-physics --bench geodesic_rk4 -- bench_christoffel
//       (run only the christoffel microbenchmark)
//   Results are saved to target/criterion/geodesic_rk4/report/index.html
//
// NOTES:
//   - black_box() prevents the compiler from optimising away the benchmark
//     body. All inputs and outputs must be wrapped in black_box().
//   - Criterion runs each benchmark for at least 5 seconds by default and
//     reports mean ± standard deviation + outlier analysis.
//   - On Apple M-series, enable the "perf" feature for more accurate timing:
//       cargo bench --features perf
//     This links against the macOS Instruments sampler for sub-nanosecond
//     timer resolution (vs the default 1 ns std::time::Instant).
//   - These benchmarks do NOT measure GPU geodesic performance; that is in
//     benches/ray_march.rs using wgpu::QuerySet timestamp queries.
// =============================================================================