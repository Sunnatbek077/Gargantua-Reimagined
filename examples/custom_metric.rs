// ============================================================
// FILE: examples/custom_metric.rs
// LINES: ~220
// CATEGORY: Example — Implement a custom MetricTensor
// RUN: cargo run --example custom_metric
// ============================================================
//
// PURPOSE:
//   Shows how to implement the MetricTensor trait for a custom
//   spacetime metric (Reissner-Nordström charged BH as example).
//   Demonstrates the extension point of the physics crate.
//   Useful for researchers who want to test non-standard metrics.
//
// CONTENTS (~220 lines):
//   // Reissner-Nordström metric: charged, non-rotating BH
//   // ds² = -f(r)dt² + dr²/f(r) + r²dΩ²
//   // f(r) = 1 - 2M/r + Q²/r²
//   struct ReissnerNordstrom {
//       mass:   f64,
//       charge: f64,
//   }
//
//   impl ReissnerNordstrom {
//       fn f(&self, r: f64) -> f64
//         // f(r) = 1 - 2M/r + Q²/r²
//
//       fn outer_horizon(&self) -> f64
//         // r_+ = M + sqrt(M² - Q²)
//
//       fn inner_horizon(&self) -> f64
//         // r_- = M - sqrt(M² - Q²)
//   }
//
//   impl MetricTensor for ReissnerNordstrom {
//       fn g_mu_nu(&self, r: f64, theta: f64) -> [[f64;4];4]
//         // g_tt = -f(r),   g_rr = 1/f(r),   g_θθ = r²,   g_φφ = r²sin²θ
//         // Off-diagonal = 0 (no rotation → no frame dragging)
//
//       fn g_mu_nu_inv(&self, r: f64, theta: f64) -> [[f64;4];4]
//         // Trivial inverse of diagonal metric
//
//       fn christoffel(&self, r: f64, theta: f64) -> [[[f64;4];4];4]
//         // Analytic Christoffel for RN metric (similar to Schwarzschild)
//         // Additional Q² terms in Γ^r_tt and Γ^t_tr
//
//       fn isco_radius(&self) -> f64
//         // Numerical: find r where d²V_eff/dr² = 0
//         // V_eff(r) = -M/r + Q²/(2r²) + L²/(2r²) - ML²/r³ + Q²L²/(2r⁴)
//
//       fn photon_sphere(&self) -> f64
//         // Numerical: find r where dV_null/dr = 0
//
//       fn event_horizon(&self) -> f64  { self.outer_horizon() }
//       fn ergosphere(&self, _: f64) -> f64 { self.outer_horizon() }
//   }
//
//   fn main()
//     // Compare RN vs Schwarzschild at same mass:
//     // let rn = ReissnerNordstrom { mass: 1.0, charge: 0.5 };
//     // let sw = Schwarzschild::new(1.0).unwrap();
//     //
//     // println!("RN r_+ = {:.4} M (vs Schwarzschild 2M)", rn.event_horizon());
//     // println!("RN ISCO = {:.4} M", rn.isco_radius());
//     //
//     // Trace a geodesic with RN metric:
//     // let integrator = Rk4Integrator::new(&rn, 0.05);
//     // let path = integrator.trace(20.0, PI/2.0, 0.0, 5.0).unwrap();
//     // println!("RN geodesic: {} steps, final r={:.4}", path.len(), path.last().unwrap()[1]);
//
// USES (imports from):
//   gargantua_physics::metric::mod::MetricTensor
//   gargantua_physics::metric::schwarzschild::Schwarzschild
//   gargantua_physics::geodesic::rk4::Rk4Integrator
//   std::f64::consts::PI
//
// USED BY:
//   Researchers extending the physics crate with custom metrics
//   Documentation (docs/extending_metrics.md)
//
// NOTE FOR AI:
//   MetricTensor is object-safe — dyn MetricTensor works with Rk4Integrator.
//   This example has NO dependency on wgpu or egui.
//   The custom metric is NOT uploaded to the GPU — it is CPU-only.
//   For GPU rendering with a custom metric, the WGSL shader would also
//   need to be modified — that is beyond the scope of this example.
//   ISCO numerical search: bisection or Newton's method on V_eff derivative.
// ============================================================