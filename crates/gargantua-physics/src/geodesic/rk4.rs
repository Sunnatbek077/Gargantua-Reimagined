// ============================================================
// FILE: crates/gargantua-physics/src/geodesic/rk4.rs
// LINES: ~320
// CATEGORY: Physics — RK4 geodesic integrator (CPU reference)
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   CPU-side 4th-order Runge-Kutta integrator for null geodesics
//   (photon paths) through any MetricTensor. Three roles:
//     1. Offline LUT baking — called by lut_baker.rs for every
//        (spin, impact_param) cell in the 2D geodesic LUT
//     2. Validation baseline — compared against GPU RK4 output
//     3. Real-time camera gravity mode — traces camera geodesic
//
// CONTENTS (~320 lines):
//   // State vector layout: [t, r, θ, φ, ṫ, ṙ, θ̇, φ̇]
//   // indices:              [0, 1, 2, 3, 4, 5, 6, 7]
//   // λ is the affine parameter (NOT coordinate time t)
//   pub type GeodesicState = [f64; 8];
//
//   pub struct Rk4Integrator<'a> {
//       metric:    &'a dyn MetricTensor,
//       step_size: f64,   // initial h, default 0.1 (geometrized units)
//       max_steps: u32,   // safety cap, default 10_000
//   }
//
//   impl<'a> Rk4Integrator<'a> {
//       pub fn new(metric: &'a dyn MetricTensor, h: f64) -> Self
//
//       pub fn trace(
//           &self,
//           r0: f64, theta0: f64, phi0: f64,
//           b: f64,   // impact parameter b = L/E (conserved)
//       ) -> PhysicsResult<Vec<GeodesicState>>
//         // Loop: call step() → termination::check() → push state
//         // Also calls disk_intersection() for accretion color sampling
//         // Returns Err(GeodesicDiverged) if max_steps reached
//
//       fn step(&self, state: &GeodesicState) -> GeodesicState
//         // k1 = h * rhs(s)
//         // k2 = h * rhs(s + k1/2)
//         // k3 = h * rhs(s + k2/2)
//         // k4 = h * rhs(s + k3)
//         // return s + (k1 + 2k2 + 2k3 + k4) / 6
//         // Calls geodesic_rhs() (→ christoffel()) 4 times per step
//
//       fn geodesic_rhs(&self, state: &GeodesicState) -> GeodesicState
//         // Returns derivative dS/dλ:
//         //   [0..3] = state[4..7]   (dx/dλ = velocity)
//         //   [4..7] = -Γ^k_μν ẋ^μ ẋ^ν  (geodesic equation acceleration)
//         // Calls self.metric.christoffel(r, theta) here
//
//       fn rk4_combine(s, k1, k2, k3, k4, h) -> GeodesicState
//         // Weighted sum: s + h/6 * (k1 + 2k2 + 2k3 + k4)
//   }
//
// USES (imports from):
//   crate::metric::mod.rs         → MetricTensor (christoffel)
//   crate::errors                 → PhysicsResult, GeodesicDiverged
//   crate::geodesic::termination  → TerminationCondition::check()
//                                   disk_intersection()
//
// USED BY:
//   crates/gargantua-bake/src/geodesic/lut_baker.rs
//     → trace() for every (spin, b) cell in 2D LUT
//   crate::geodesic::adaptive
//     → wraps step() with Cash-Karp error estimate
//   crates/gargantua-camera/src/modes/gravity.rs
//     → real-time camera geodesic tracing
//   tests/geodesic.rs
//     → photon orbit check: r stabilises at r_ph = 3M
//
// NOTE FOR AI:
//   GeodesicState indices:
//     [0]=t  [1]=r  [2]=θ  [3]=φ  [4]=ṫ  [5]=ṙ  [6]=θ̇  [7]=φ̇
//   geodesic_rhs() acceleration term:
//     accel[k] = -sum_{mu,nu} Γ^k_μν * state[4+mu] * state[4+nu]
//   This is the CPU reference implementation.
//   GPU counterpart: shaders/compute/geodesic_rk4.wgsl
//   Math must be IDENTICAL — any fix here → also fix in WGSL.
//   Near horizon (r < 1.1 * r_+): use step_size = 0.01 to avoid divergence.
//   Default step_size = 0.1 is safe for r > 3M.
// ============================================================