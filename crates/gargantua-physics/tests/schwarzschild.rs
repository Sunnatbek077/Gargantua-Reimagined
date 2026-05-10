// ============================================================
// FILE: crates/gargantua-physics/tests/schwarzschild.rs
// LINES: ~180
// CATEGORY: Integration test — Schwarzschild metric (analytic baseline)
// RUN: cargo test --package gargantua-physics --test schwarzschild
// ============================================================
//
// PURPOSE:
//   Validates the Schwarzschild metric against exact analytic values.
//   Since Schwarzschild is the simplest non-trivial metric, these tests
//   serve as a sanity baseline. Also verifies that KerrNewman(a=0)
//   is numerically identical to Schwarzschild at every point.
//
// TESTED FUNCTIONS (from crate::metric::schwarzschild):
//   Schwarzschild::new(mass)
//   Schwarzschild::from_solar_masses(m_sun)
//   MetricTensor::g_mu_nu(r, theta)
//   MetricTensor::g_mu_nu_inv(r, theta)
//   MetricTensor::christoffel(r, theta)
//   MetricTensor::isco_radius()
//   MetricTensor::photon_sphere()
//   MetricTensor::event_horizon()
//   MetricTensor::ergosphere(theta)
//
// TEST CASES (~180 lines):
//
//   #[test]
//   fn test_event_horizon_is_2m()
//     // Schwarzschild::new(1.0).event_horizon() == 2.0
//     // Schwarzschild::new(3.5).event_horizon() == 7.0
//     // assert_relative_eq! with epsilon=1e-12
//
//   #[test]
//   fn test_photon_sphere_is_3m()
//     // photon_sphere() == 3.0 * mass
//     // Check for mass=1.0 and mass=5.0
//
//   #[test]
//   fn test_isco_is_6m()
//     // isco_radius() == 6.0 * mass exactly
//     // Check for mass=1.0, 2.0, 10.0
//
//   #[test]
//   fn test_ergosphere_equals_horizon()
//     // ergosphere(any θ) == event_horizon() for Schwarzschild
//     // (no ergosphere without spin)
//     // Check at theta = 0, π/4, π/2, π
//
//   #[test]
//   fn test_metric_diagonal()
//     // All off-diagonal elements == 0.0 (Schwarzschild is diagonal)
//     // g_mu_nu: check g[0][1], g[0][2], g[0][3], g[1][2], g[1][3], g[2][3] == 0
//     // especially g[0][3] = g_tφ == 0.0 (no frame dragging)
//
//   #[test]
//   fn test_g_tt_at_horizon()
//     // g_tt = -(1 - 2M/r)  →  g_tt(2M) = 0
//     // g_mu_nu(2.0, PI/2)[0][0] ≈ 0.0
//     // assert_relative_eq!(result, 0.0, epsilon=1e-10)
//
//   #[test]
//   fn test_g_tt_at_large_r()
//     // g_tt → -1 as r → ∞ (flat spacetime limit)
//     // g_mu_nu(1000.0, PI/2)[0][0] ≈ -1.0
//     // assert_relative_eq!(result, -1.0, epsilon=0.001)
//
//   #[test]
//   fn test_g_rr_at_large_r()
//     // g_rr → +1 as r → ∞
//     // g_mu_nu(1000.0, PI/2)[1][1] ≈ 1.0
//
//   #[test]
//   fn test_g_theta_theta()
//     // g_θθ = r² — pure geometry, no mass dependence factor
//     // At r=5M: g_θθ = 25.0 (for M=1)
//     // assert_relative_eq!(g_mu_nu(5.0, PI/3)[2][2], 25.0, epsilon=1e-10)
//
//   #[test]
//   fn test_g_phi_phi_equator()
//     // g_φφ = r² sin²θ. At equator (theta=π/2): g_φφ = r²
//     // At r=5M, theta=π/2: g_φφ = 25.0
//     // assert_relative_eq!(result, 25.0, epsilon=1e-10)
//
//   #[test]
//   fn test_metric_inverse_is_correct()
//     // g_μν g^νλ = δ^λ_μ  (4×4 identity)
//     // Multiply g_mu_nu * g_mu_nu_inv at r=6M, theta=π/3
//     // All diagonal elements ≈ 1.0, off-diagonal ≈ 0.0 (epsilon=1e-10)
//
//   #[test]
//   fn test_christoffel_gamma_r_tt()
//     // Γ^r_tt = M(r-2M)/r³  (Schwarzschild exact)
//     // At r=6M, M=1: Γ^r_tt = 1*(6-2)/(6³) = 4/216 ≈ 0.01852
//     // assert_relative_eq!(christoffel[1][0][0], 0.01852, epsilon=1e-5)
//
//   #[test]
//   fn test_christoffel_gamma_t_tr()
//     // Γ^t_tr = M / (r(r-2M))
//     // At r=6M, M=1: Γ^t_tr = 1/(6*4) = 1/24 ≈ 0.04167
//     // assert_relative_eq!(christoffel[0][0][1], 0.04167, epsilon=1e-5)
//
//   #[test]
//   fn test_christoffel_symmetry_schwarzschild()
//     // Γ^λ_μν = Γ^λ_νμ for all indices
//     // At (r=4M, theta=π/4): check christoffel[l][m][n] == christoffel[l][n][m]
//
//   #[test]
//   fn test_schwarzschild_matches_kerr_a0()
//     // CROSS-CHECK: Schwarzschild.g_mu_nu == KerrNewman(a=0).g_mu_nu
//     // At multiple (r, theta) points: (3M,π/3), (5M,π/2), (10M,π/4)
//     // Each component must match within epsilon=1e-12
//     // This is the key regression test for kerr.rs a=0 limit
//
//   #[test]
//   fn test_from_solar_masses()
//     // Schwarzschild::from_solar_masses(1.0).event_horizon()
//     // == schwarzschild_radius of 1 solar mass in meters
//     // ≈ 2953 meters (r_s of Sun = 2GM_sun/c²)
//     // Checks unit conversion via units::solar_mass_to_geom
//
// USES (imports from):
//   gargantua_physics::metric::schwarzschild::Schwarzschild
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::mod::MetricTensor
//   approx::assert_relative_eq
//   std::f64::consts::PI
// ============================================================