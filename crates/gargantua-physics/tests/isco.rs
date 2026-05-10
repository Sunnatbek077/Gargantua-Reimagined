// ============================================================
// FILE: crates/gargantua-physics/tests/isco.rs
// LINES: ~200
// CATEGORY: Integration test — ISCO and circular orbit computations
// RUN: cargo test --package gargantua-physics --test isco
// ============================================================
//
// PURPOSE:
//   Validates kerr_isco(), compute_isco_properties(), keplerian_freq(),
//   circular_orbit_energy(), circular_orbit_angmom() against known
//   astrophysical reference values (Bardeen 1972, published BH papers).
//
// TESTED FUNCTIONS (from crate::accretion::isco):
//   kerr_isco(mass, spin)             -> PhysicsResult<f64>
//   compute_isco_properties(mass, spin) -> PhysicsResult<IscoProperties>
//   keplerian_freq(r, mass, spin)     -> f64
//   circular_orbit_energy(r, mass, spin) -> f64
//   circular_orbit_angmom(r, mass, spin) -> f64
//
// TEST CASES (~200 lines):
//
//   #[test]
//   fn test_isco_schwarzschild()
//     // Schwarzschild (a=0): r_ISCO = 6M exactly
//     // kerr_isco(1.0, 0.0) == 6.0
//     // assert_relative_eq!(result, 6.0, epsilon=1e-10)
//
//   #[test]
//   fn test_isco_prograde_spin_shrinks_radius()
//     // Prograde spin (a>0) → r_ISCO decreases
//     // kerr_isco(1.0, 0.5) < kerr_isco(1.0, 0.0) < kerr_isco(1.0, -0.5)
//     // Three-way ordering assert
//
//   #[test]
//   fn test_isco_m87_reference()
//     // M87* black hole: spin a ≈ 0.9
//     // Published value: r_ISCO ≈ 2.321 M (prograde)
//     // kerr_isco(1.0, 0.9) ≈ 2.321
//     // assert_relative_eq!(result, 2.321, epsilon=0.001)
//
//   #[test]
//   fn test_isco_sgra_reference()
//     // SgrA* black hole: spin a ≈ 0.6
//     // Published value: r_ISCO ≈ 3.829 M (prograde)
//     // kerr_isco(1.0, 0.6) ≈ 3.829
//     // assert_relative_eq!(result, 3.829, epsilon=0.001)
//
//   #[test]
//   fn test_isco_extremal_limits()
//     // a → +1: r_ISCO → 1.0 M (prograde extremal limit)
//     // a → -1: r_ISCO → 9.0 M (retrograde extremal limit)
//     // Use a=0.999 and a=-0.999 as proxies
//     // assert_relative_eq!(kerr_isco(1.0, 0.999), 1.07, epsilon=0.1)
//     // assert_relative_eq!(kerr_isco(1.0, -0.999), 8.99, epsilon=0.05)
//
//   #[test]
//   fn test_isco_above_event_horizon()
//     // r_ISCO must always be > r_+ (event horizon)
//     // Check for a in [-0.999, +0.999] sampled at 20 points
//     // r_isco > r_plus for all tested spins
//
//   #[test]
//   fn test_isco_invalid_extremal_spin_errors()
//     // kerr_isco(1.0, 1.0)  → Err(PhysicsError::IscoFailed)
//     // kerr_isco(1.0, -1.0) → Err(PhysicsError::IscoFailed)
//     // kerr_isco(1.0, 1.5)  → Err(PhysicsError::IscoFailed) (|a|>1 invalid)
//
//   #[test]
//   fn test_keplerian_freq_schwarzschild()
//     // At r=6M, spin=0: Ω_K = M^(1/2) / r^(3/2) = 1/(6^(3/2)) for M=1
//     // = 1/14.697 ≈ 0.06804 (geometrized units)
//     // assert_relative_eq!(keplerian_freq(6.0, 1.0, 0.0), 0.06804, epsilon=1e-4)
//
//   #[test]
//   fn test_keplerian_freq_increases_with_spin()
//     // Prograde spin: ISCO is closer → Ω_K(r_ISCO) increases with spin
//     // Ω_K at r_ISCO(a=0.9) > Ω_K at r_ISCO(a=0.0)
//
//   #[test]
//   fn test_circular_energy_approaches_1_at_infinity()
//     // E → 1 as r → ∞ (particle at rest at infinity has E=1)
//     // circular_orbit_energy(1000.0, 1.0, 0.0) ≈ 1.0
//     // assert_relative_eq!(result, 1.0, epsilon=0.001)
//
//   #[test]
//   fn test_binding_energy_schwarzschild()
//     // Specific binding energy at ISCO = 1 - E_ISCO
//     // For Schwarzschild: E_ISCO = sqrt(8/9) ≈ 0.9428 → η ≈ 5.72%
//     // compute_isco_properties(1.0, 0.0).binding_energy ≈ 0.0572
//     // assert_relative_eq!(result.binding_energy, 0.0572, epsilon=0.0005)
//
//   #[test]
//   fn test_binding_energy_increases_with_prograde_spin()
//     // Prograde spin → more efficient accretion → higher binding energy
//     // a=0.9: η ≈ 17.4%, a=0.998: η ≈ 32%
//     // Check monotonic increase: η(0.0) < η(0.5) < η(0.9)
//
//   #[test]
//   fn test_isco_mass_scaling()
//     // ISCO radius scales linearly with mass (G=c=1):
//     // kerr_isco(2.0, 0.5) = 2.0 * kerr_isco(1.0, 0.5)
//     // assert_relative_eq!(result, 2.0 * r_isco_m1, epsilon=1e-10)
//
// USES (imports from):
//   gargantua_physics::accretion::isco::*
//   gargantua_physics::errors::PhysicsError
//   approx::assert_relative_eq
// ============================================================