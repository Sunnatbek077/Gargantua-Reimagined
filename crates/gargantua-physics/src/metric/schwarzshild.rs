// ============================================================
// FILE: crates/gargantua-physics/src/metric/schwarzshild.rs
// LINES: ~260
// CATEGORY: Physics — Schwarzschild metric (non-rotating BH)
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Schwarzschild metric — Kerr special case with a=0, Q=0.
//   Used for unit tests and analytic validation.
//   NOT used in the main render pipeline (KerrNewman is always used).
//   Provides clean analytic baseline for Christoffel comparison tests.
//
// CONTENTS (~260 lines):
//   pub struct Schwarzschild { pub mass: f64 }
//
//   impl Schwarzschild {
//       pub fn new(mass: f64) -> PhysicsResult<Self>
//       pub fn from_solar_masses(m_sun: f64) -> PhysicsResult<Self>
//   }
//
//   impl MetricTensor for Schwarzschild {
//       fn g_mu_nu(r, theta) → [[f64;4];4]
//         // Diagonal metric (no frame dragging at a=0):
//         // g_tt = -(1 - 2M/r),   g_rr = 1/(1 - 2M/r)
//         // g_θθ = r²,            g_φφ = r²sin²θ
//         // All off-diagonal = 0
//
//       fn g_mu_nu_inv(r, theta) → [[f64;4];4]
//         // Trivial analytic inverse of diagonal matrix
//
//       fn christoffel(r, theta) → [[[f64;4];4];4]
//         // Analytic Γ — simpler than Kerr (no g_tφ cross terms)
//         // Non-zero: Γ^t_tr, Γ^r_tt, Γ^r_rr, Γ^r_θθ, Γ^r_φφ,
//         //            Γ^θ_rθ, Γ^θ_φφ, Γ^φ_rφ, Γ^φ_θφ
//
//       fn isco_radius()   → f64   // Fixed constant: 6M
//       fn photon_sphere() → f64   // Fixed constant: 3M
//       fn event_horizon() → f64   // Fixed constant: 2M = r_Schwarzschild
//       fn ergosphere(θ)   → f64   // = event_horizon() (no ergosphere at a=0)
//   }
//
// USES (imports from):
//   super::mod.rs   → MetricTensor trait
//   crate::units    → solar_mass_to_geom, schwarzschild_radius
//   crate::errors   → PhysicsError::NegativeMass
//
// USED BY:
//   crates/gargantua-physics/tests/schwarzschild.rs
//     → g_tt(r_s)=0, r_ISCO=6M, r_ph=3M, r_horizon=2M
//   crates/gargantua-physics/tests/kerr_metric.rs
//     → cross-check: KerrNewman(spin=0).isco_radius() == 6M
//   crates/gargantua-physics/tests/geodesic.rs
//     → photon orbit at r=3M verified with Schwarzschild metric
//
// NOTE FOR AI:
//   On-disk filename is schwarzshild.rs (typo retained); type is Schwarzschild.
//   Schwarzschild = Kerr(a=0, Q=0). All ISCO/photon sphere values
//   are fixed constants of M with no spin dependence:
//     r_ISCO = 6M,  r_ph = 3M,  r_horizon = 2M
//   ergosphere() returns event_horizon() because at a=0 the static
//   limit and the event horizon coincide at r=2M.
//   Use this ONLY for tests — never in the render pipeline.
// ============================================================