// ============================================================
// FILE: examples/benchmark_geodesic.rs
// LINES: ~180
// CATEGORY: Example — CPU geodesic integrator performance benchmark
// RUN: cargo run --example benchmark_geodesic --release
// ============================================================
//
// PURPOSE:
//   Benchmarks the CPU geodesic integrators (fixed-step RK4 and
//   adaptive Cash-Karp) across a range of impact parameters and
//   spin values. Reports: traces/second, average steps per trace,
//   and comparison between fixed vs adaptive.
//   Used to track performance regressions in the integrators.
//
// CONTENTS (~180 lines):
//   fn main()
//     // 1. Setup: 5 spin values × 20 impact params = 100 geodesics
//     //    spins  = [-0.9, -0.5, 0.0, 0.5, 0.9]
//     //    b values: log-spaced from b_crit*1.01 to b_crit*5.0
//     //
//     // 2. Benchmark fixed RK4 (step=0.1):
//     //    let t0 = Instant::now();
//     //    for (spin, b) in &cases {
//     //        let bh = KerrNewman::new(1.0, *spin, 0.0).unwrap();
//     //        let integrator = Rk4Integrator::new(&bh, 0.1);
//     //        let path = integrator.trace(50.0, PI/2.0, 0.0, *b)?;
//     //        total_steps += path.len();
//     //    }
//     //    let rk4_ns = t0.elapsed().as_nanos() / 100;
//     //    println!("Fixed RK4:    {:>8} ns/trace, avg {:>5} steps", rk4_ns, avg_steps);
//     //
//     // 3. Benchmark adaptive (tol=1e-6):
//     //    Same loop with AdaptiveIntegrator
//     //    println!("Adaptive CK:  {:>8} ns/trace, avg {:>5} steps", ...);
//     //
//     // 4. Report speedup or slowdown ratio
//     //    println!("Adaptive/Fixed ratio: {:.2}x", adaptive_ns / rk4_ns);
//     //
//     // 5. Warm-up: run 10 traces before timing to avoid cold-start effects
//
// USES (imports from):
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::mod::MetricTensor
//   gargantua_physics::geodesic::rk4::Rk4Integrator
//   gargantua_physics::geodesic::adaptive::AdaptiveIntegrator
//   std::{time::Instant, f64::consts::PI}
//
// USED BY:
//   CI performance tracking — run on each PR to detect regressions
//   Developers tuning RK4 step size vs accuracy tradeoff
//
// NOTE FOR AI:
//   MUST be run with --release flag. Debug builds are 10–100× slower.
//   Warm-up 10 traces before timing (JIT/cache effects).
//   b_crit = 3√3 M ≈ 5.196 for Schwarzschild. For Kerr: computed per spin.
//   Expected perf (M1 Pro, release): ~5000–20000 ns/trace (fixed RK4).
//   Do NOT use std::hint::black_box — the compiler won't elide geodesic
//   computation since it has side effects (Vec allocation).
// ============================================================