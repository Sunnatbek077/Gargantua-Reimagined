// ============================================================
// FILE: crates/gargantua-physics/tests/kerr_metric.rs
// LINES: ~260
// CATEGORY: Integration test — Kerr-Newman metric tensor
// RUN: cargo test --package gargantua-physics --test kerr_metric
// ============================================================
//
// PURPOSE:
//   Validates KerrNewman metric components, Christoffel symbols,
//   horizon/ISCO/photon-sphere formulas, and GPU param conversion.
//   Cross-checks analytic Christoffel against numerical finite-diff
//   and verifies the a=0 limit matches Schwarzschild exactly.
//
// TESTED FUNCTIONS (from crate::metric::kerr):
//   KerrNewman::new(mass, spin, charge)
//   KerrNewman::from_solar_masses(m_sun, spin)
//   MetricTensor::g_mu_nu(r, theta)
//   MetricTensor::g_mu_nu_inv(r, theta)
//   MetricTensor::christoffel(r, theta)
//   MetricTensor::isco_radius()
//   MetricTensor::photon_sphere()
//   MetricTensor::event_horizon()
//   MetricTensor::ergosphere(theta)
//   KerrNewman::to_gpu_params()
//
// TEST CASES (~260 lines):
//
//   #[test]
//   fn test_invalid_spin_rejected()
//     // KerrNewman::new(1.0, 1.0, 0.0) → Err(InvalidSpin)
//     // KerrNewman::new(1.0, 1.5, 0.0) → Err(InvalidSpin)
//     // KerrNewman::new(1.0, -1.0, 0.0) → Err(InvalidSpin)
//
//   #[test]
//   fn test_negative_mass_rejected()
//     // KerrNewman::new(-1.0, 0.0, 0.0) → Err(NegativeMass)
//     // KerrNewman::new(0.0, 0.0, 0.0)  → Err(NegativeMass)
//
//   #[test]
//   fn test_metric_signature()
//     // At (r=10M, theta=π/2): g_tt < 0, g_rr > 0, g_θθ > 0, g_φφ > 0
//     // g_tφ ≠ 0 for spin > 0 (frame dragging)
//     // g_tφ = 0 for spin = 0 (Schwarzschild limit)
//
//   #[test]
//   fn test_metric_inverse_identity()
//     // g_μν * g^νλ = δ^λ_μ  (identity matrix, 4×4)
//     // At (r=5M, theta=π/3): multiply g_mu_nu and g_mu_nu_inv
//     // Check each element of product vs identity within epsilon=1e-10
//
//   #[test]
//   fn test_kerr_a0_equals_schwarzschild()
//     // KerrNewman(a=0).g_mu_nu(r, θ) == Schwarzschild.g_mu_nu(r, θ)
//     // Check all 16 components at (r=5M, theta=π/4)
//     // assert_relative_eq! each component with epsilon=1e-12
//
//   #[test]
//   fn test_horizon_schwarzschild_limit()
//     // KerrNewman(a=0).event_horizon() == 2.0 * mass
//     // Also check g_tt → 0 as r → event_horizon (metric degeneracy)
//
//   #[test]
//   fn test_horizon_kerr_formula()
//     // r_+ = M + sqrt(M² - a²)
//     // a=0.5, M=1: r_+ = 1 + sqrt(1 - 0.25) = 1 + sqrt(0.75) ≈ 1.866
//     // assert_relative_eq!(event_horizon(), 1.8660, epsilon=1e-4)
//
//   #[test]
//   fn test_ergosphere_at_equator()
//     // At theta=π/2: r_ergo = M + sqrt(M² - 0) = 2M (same as Schwarzschild r_s)
//     // KerrNewman(a=0.9).ergosphere(PI/2) ≈ 2.0 * mass
//     // assert_relative_eq!(result, 2.0, epsilon=1e-10)
//
//   #[test]
//   fn test_ergosphere_at_poles()
//     // At theta=0: r_ergo = M + sqrt(M² - a²) = r_+ (event horizon)
//     // ergosphere(0.0) ≈ event_horizon()
//     // assert_relative_eq!(ergosphere(0.0), event_horizon(), epsilon=1e-10)
//
//   #[test]
//   fn test_ergosphere_larger_than_horizon()
//     // For 0 < theta < π/2: r_ergo(θ) > r_+ for spin > 0
//     // Check at theta = π/4, π/3, π/2 with a=0.5
//     // All ergosphere values should be > event_horizon()
//
//   #[test]
//   fn test_photon_sphere_schwarzschild()
//     // KerrNewman(a=0).photon_sphere() == 3.0 * mass
//     // assert_relative_eq!(result, 3.0, epsilon=1e-10)
//
//   #[test]
//   fn test_ordering_horizon_ph_isco()
//     // Physical ordering must hold: r_+ < r_ph < r_ISCO
//     // For a in [0.0, 0.9] sampled at 5 points:
//     // event_horizon() < photon_sphere() < isco_radius()
//
//   #[test]
//   fn test_analytic_vs_numerical_christoffel()
//     // Compare analytic christoffel() vs numerical_christoffel() (5-pt stencil)
//     // At (r=5M, theta=π/3), spin=0.5
//     // For each non-zero component Γ^λ_μν:
//     //   assert_relative_eq!(analytic[l][m][n], numerical[l][m][n], epsilon=1e-6)
//     // Tests that the 200-line analytic impl is correct
//
//   #[test]
//   fn test_christoffel_symmetry()
//     // Christoffel symbols are symmetric in lower indices: Γ^λ_μν = Γ^λ_νμ
//     // For all λ, μ, ν: christoffel[l][m][n] == christoffel[l][n][m]
//     // Check at (r=4M, theta=π/2), spin=0.7
//
//   #[test]
//   fn test_gpu_params_field_values()
//     // to_gpu_params() correctly converts f64 → f32 and computes derived fields
//     // mass=1.0, spin=0.5, charge=0.0:
//     //   gpu.r_s    ≈ 2.0  (= 2M)
//     //   gpu.r_plus ≈ 1.866 (= M + sqrt(M²-a²))
//     //   gpu.r_isco ≈ 4.233 (from kerr_isco)
//     //   gpu.r_ph   ≈ 2.495 (photon sphere)
//
// USES (imports from):
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::schwarzschild::Schwarzschild
//   gargantua_physics::metric::mod::MetricTensor
//   gargantua_physics::errors::PhysicsError
//   approx::assert_relative_eq
//   std::f64::consts::PI
// ============================================================