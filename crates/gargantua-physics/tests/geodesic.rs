// ============================================================
// FILE: crates/gargantua-physics/tests/geodesic.rs
// LINES: ~280
// CATEGORY: Integration test — RK4 geodesic integrator
// RUN: cargo test --package gargantua-physics --test geodesic
// ============================================================
//
// PURPOSE:
//   Validates the RK4 geodesic integrator and adaptive step control.
//   Tests: photon orbit stability, impact parameter conservation,
//   termination conditions, disk intersection detection, and
//   cross-validation between CPU RK4 and analytic Schwarzschild.
//
// TESTED FUNCTIONS (from crate::geodesic):
//   Rk4Integrator::new(metric, h)
//   Rk4Integrator::trace(r0, theta0, phi0, b)
//   AdaptiveIntegrator::new(metric)
//   AdaptiveIntegrator::trace(r0, theta0, phi0, b)
//   TerminationCondition::check(state)
//   disk_intersection(prev, curr, r_isco, r_outer)
//
// SETUP (shared across tests):
//   fn schwarzschild_1m() -> Schwarzschild { Schwarzschild::new(1.0).unwrap() }
//   fn kerr_m87() -> KerrNewman { KerrNewman::new(1.0, 0.9, 0.0).unwrap() }
//
// TEST CASES (~280 lines):
//
//   #[test]
//   fn test_photon_orbit_schwarzschild()
//     // Photon with b = 3√3 M orbits at r = 3M (photon sphere)
//     // Trace with step=0.05, max_steps=50_000 from r0=20M, theta0=π/2
//     // After ~1 orbit, r should oscillate near 3M
//     // Check: min(path[*].r) > 2.5M, max deviation from 3M < 0.5M
//
//   #[test]
//   fn test_photon_captured_below_photon_sphere()
//     // Photon with b < 3√3 M is captured → TerminationReason::ReachedHorizon
//     // b = 4.0 (below critical b_c = 3√3 ≈ 5.196)
//     // assert_eq!(termination_reason, TerminationReason::ReachedHorizon)
//
//   #[test]
//   fn test_photon_escapes_above_photon_sphere()
//     // Photon with b > 3√3 M escapes → TerminationReason::EscapedToInfinity
//     // b = 8.0
//     // assert_eq!(termination_reason, TerminationReason::EscapedToInfinity)
//
//   #[test]
//   fn test_conserved_impact_parameter()
//     // Impact parameter b = L/E must be conserved along geodesic
//     // Compute b from state[3,7] (φ̇) and state[4] (ṫ):
//     //   b = -g_φφ * φ̇ / (g_tt * ṫ)   [approximate for equatorial]
//     // Check relative variation < 1% across full trace
//
//   #[test]
//   fn test_null_geodesic_condition()
//     // Null condition: g_μν ẋ^μ ẋ^ν = 0 (photon has zero mass)
//     // Compute g_μν v^μ v^ν at each step of traced path
//     // assert: abs(null_condition) < 1e-6 throughout trace
//     // (tests that integrator preserves null constraint)
//
//   #[test]
//   fn test_kerr_photon_orbit_prograde()
//     // For Kerr a=0.9, prograde photon sphere at r_ph ≈ 1.97M
//     // Photon with appropriate b traces orbit near r_ph
//     // b_prograde = r_ph² / sqrt(r_ph² - 2Mr_ph + a²) (approx)
//     // Check r stays near r_ph (tighter orbit than Schwarzschild)
//
//   #[test]
//   fn test_disk_intersection_detected()
//     // Create two states crossing θ = π/2:
//     //   prev: theta = π/2 + 0.1, r = 8.0  (above equator)
//     //   curr: theta = π/2 - 0.1, r = 8.0  (below equator)
//     // disk_intersection(prev, curr, r_isco=3.0, r_outer=20.0)
//     // assert!(result.is_some())
//     // Returned λ should be ≈ midpoint between prev and curr λ
//
//   #[test]
//   fn test_disk_intersection_outside_isco_ignored()
//     // Same θ crossing but r = 1.5 < r_isco = 3.0
//     // assert!(disk_intersection(...).is_none())
//     // No disk material inside ISCO
//
//   #[test]
//   fn test_disk_intersection_outside_outer_ignored()
//     // Same θ crossing but r = 25.0 > r_outer = 20.0
//     // assert!(disk_intersection(...).is_none())
//
//   #[test]
//   fn test_adaptive_vs_fixed_step_agreement()
//     // Trace same geodesic with both Rk4Integrator(h=0.05) and
//     // AdaptiveIntegrator (tol=1e-8)
//     // Final state (r, θ, φ) should agree to within 1e-4
//     // (adaptive should be at least as accurate as fixed)
//
//   #[test]
//   fn test_geodesic_max_steps_returns_error()
//     // Create integrator with max_steps=10 (intentionally too low)
//     // Trace photon with b=5.196 (photon sphere — never terminates naturally)
//     // assert!(result.is_err())
//     // Error should be PhysicsError::GeodesicDiverged
//
//   #[test]
//   fn test_near_horizon_step_reduction()
//     // Trace photon toward horizon with step_size=0.1
//     // At r < 1.1 * r_+: verify step_size is internally reduced to 0.01
//     // (checks that integrator does not blow up near horizon)
//     // Proxy: path should not contain r < 0 or NaN values
//
// USES (imports from):
//   gargantua_physics::geodesic::{Rk4Integrator, AdaptiveIntegrator}
//   gargantua_physics::geodesic::termination::{TerminationCondition, TerminationReason, disk_intersection}
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::schwarzschild::Schwarzschild
//   gargantua_physics::errors::PhysicsError
//   approx::assert_relative_eq
//
// PERFORMANCE NOTE:
//   Some tests trace 50_000 steps. Run with --release for speed:
//   cargo test --package gargantua-physics --test geodesic --release
// ============================================================