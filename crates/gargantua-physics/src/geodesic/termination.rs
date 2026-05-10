// ============================================================
// FILE: crates/gargantua-physics/src/geodesic/termination.rs
// LINES: ~160
// CATEGORY: Physics — Geodesic termination and disk intersection
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Determines when to stop tracing a photon geodesic.
//   Called every step inside rk4.rs and adaptive.rs.
//   Two physical outcomes: photon absorbed by BH (horizon),
//   or escaped to infinity (starfield). Also detects equatorial
//   disk crossings for accretion color sampling.
//
// CONTENTS (~160 lines):
//   #[derive(Debug, Clone, Copy, PartialEq)]
//   pub enum TerminationReason {
//       ReachedHorizon,      // r <= r_horizon * 1.05 → photon absorbed
//       EscapedToInfinity,   // r >= r_max           → hits starfield
//       MaxStepsReached,     // safety fallback
//   }
//
//   pub struct TerminationCondition {
//       pub r_horizon: f64,  // event_horizon() * 1.05 (numerical buffer)
//       pub r_max:     f64,  // 1000.0 * mass (far-field boundary)
//   }
//
//   impl TerminationCondition {
//       pub fn from_metric(metric: &dyn MetricTensor, mass: f64) -> Self
//
//       pub fn check(&self, state: &GeodesicState) -> Option<TerminationReason>
//         // Reads state[1] = r
//         // r <= r_horizon → Some(ReachedHorizon)
//         // r >= r_max     → Some(EscapedToInfinity)
//         // else           → None (continue tracing)
//   }
//
//   // Detects photon crossing equatorial plane (θ = π/2)
//   // between two consecutive integration steps.
//   // Uses linear interpolation of θ between prev and curr.
//   // Returns Some(lambda) = affine parameter at crossing,
//   // or None if no crossing or outside disk radii.
//   pub fn disk_intersection(
//       prev: &GeodesicState,
//       curr: &GeodesicState,
//       r_isco: f64,
//       r_outer: f64,
//   ) -> Option<f64>
//
// USES (imports from):
//   crate::metric::mod.rs  → MetricTensor (event_horizon)
//   crate::geodesic::rk4   → GeodesicState type alias
//
// USED BY:
//   crate::geodesic::rk4::Rk4Integrator::trace()      → check() every step
//   crate::geodesic::adaptive::AdaptiveIntegrator::trace() → same
//   crates/gargantua-render/src/pipelines/ray_march.rs
//     → disk_intersection() to sample accretion disk color at crossing
//
// NOTE FOR AI:
//   r_horizon threshold = 1.05 * r_+ (NOT exactly r_+).
//   5% buffer avoids numerical blowup at Boyer-Lindquist horizon singularity.
//   disk_intersection(): crossing detected when sign(prev[2] - π/2)
//   differs from sign(curr[2] - π/2). Then linear interpolate λ.
//   At the crossing: check r_isco < r_at_crossing < r_outer.
//   If outside disk annulus → return None (no disk material there).
// ============================================================