// ============================================================
// FILE: crates/gargantua-physics/src/geodesic/adaptive.rs
// LINES: ~280
// CATEGORY: Physics — Cash-Karp RK4(5) adaptive step-size control
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Wraps Rk4Integrator with Cash-Karp embedded RK4(5) pair for
//   automatic step-size control. Shrinks step near the event horizon
//   (extreme curvature) and grows it far away (saves compute).
//   Used ONLY for offline LUT baking — GPU uses fixed-step RK4.
//
// CONTENTS (~280 lines):
//   pub struct AdaptiveIntegrator<'a> {
//       inner: Rk4Integrator<'a>,
//       tol:   f64,   // relative error tolerance, default 1e-6
//       h_min: f64,   // minimum step size, default 1e-4
//       h_max: f64,   // maximum step size, default 1.0
//   }
//
//   impl<'a> AdaptiveIntegrator<'a> {
//       pub fn new(metric: &'a dyn MetricTensor) -> Self
//       pub fn with_tolerance(mut self, tol: f64) -> Self
//
//       pub fn trace(&self, r0: f64, theta0: f64, phi0: f64, b: f64)
//           -> PhysicsResult<Vec<GeodesicState>>
//         // Loop: cash_karp_step() → compare err vs tol
//         //   if err > tol: reject step, shrink h, retry
//         //   if err <= tol: accept state, update h for next step
//
//       fn cash_karp_step(
//           &self, state: &GeodesicState, h: f64,
//       ) -> (GeodesicState, f64, f64)
//         // Returns: (new_state, error_estimate, h_suggested_next)
//         // 6 derivative evaluations: k1..k6
//         // RK4 solution: y4 = s + sum(CK_B[i]  * k[i])
//         // RK5 solution: y5 = s + sum(CK_BS[i] * k[i])
//         // error = ||y5 - y4||_inf
//
//       fn update_stepsize(h: f64, err: f64, tol: f64) -> f64
//         // h_new = 0.9 * h * (tol / err)^0.2
//         // clamped to [h_min, h_max]
//   }
//
//   // Cash-Karp Butcher tableau (Numerical Recipes 3rd ed., Table 17.2):
//   const CK_A:  [[f64; 5]; 5] = [ ... ];  // stage coefficients a_ij
//   const CK_B:  [f64; 6]      = [ ... ];  // 4th-order weights b_i
//   const CK_BS: [f64; 6]      = [ ... ];  // 5th-order weights b*_i
//
// USES (imports from):
//   crate::geodesic::rk4        → Rk4Integrator, GeodesicState
//   crate::errors               → PhysicsResult, GeodesicDiverged
//   crate::geodesic::termination → TerminationCondition
//
// USED BY:
//   crates/gargantua-bake/src/geodesic/lut_baker.rs
//     → AdaptiveIntegrator for higher-accuracy offline LUT baking
//
// NOTE FOR AI:
//   Cash-Karp: 6 function evaluations per step (vs RK4's 4).
//   Step REJECTION: err > tol → do NOT advance state, retry with h_new.
//   Step ACCEPTANCE: err <= tol → advance state, set h = h_new for next.
//   Safety factor 0.9 prevents step from being immediately rejected again.
//   h_min = 1e-4 prevents infinite shrinking near coordinate singularity.
//   GPU path (geodesic_rk4.wgsl) uses FIXED step — no adaptive on GPU.
//   Adaptive is only justified offline where accuracy > speed.
// ============================================================