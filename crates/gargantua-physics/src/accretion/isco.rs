// ============================================================
// FILE: crates/gargantua-physics/src/accretion/isco.rs
// LINES: ~300
// CATEGORY: Physics — Innermost Stable Circular Orbit (ISCO)
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Computes ISCO radius — the inner edge of the accretion disk.
//   Below ISCO, no stable circular orbits exist; gas plunges into
//   the BH. Prograde spin shrinks ISCO toward horizon (higher efficiency);
//   retrograde spin expands it. Bardeen (1972) exact analytic formula.
//
// CONTENTS (~300 lines):
//   pub fn kerr_isco(mass: f64, spin: f64) -> PhysicsResult<f64>
//     // Bardeen (1972) exact formula:
//     //   z1 = 1 + (1−a²)^(1/3) * [(1+a)^(1/3) + (1−a)^(1/3)]
//     //   z2 = sqrt(3a² + z1²)
//     //   prograde:  r_ISCO = M*(3 + z2 − sqrt((3−z1)(3+z1+2z2)))
//     //   retrograde:r_ISCO = M*(3 + z2 + sqrt((3−z1)(3+z1+2z2)))
//     //   sign(spin) selects prograde vs retrograde branch
//
//   pub struct IscoProperties {
//       pub r_isco:         f64,  // ISCO radius in G=c=1 units
//       pub r_isco_meters:  f64,  // in physical meters
//       pub r_isco_rs:      f64,  // in units of Schwarzschild radius (r/2M)
//       pub orbital_freq:   f64,  // Keplerian frequency at ISCO in Hz
//       pub binding_energy: f64,  // specific binding energy (fraction of mc²)
//       pub spin_param:     f64,  // input spin a
//   }
//
//   pub fn compute_isco_properties(mass: f64, spin: f64) -> PhysicsResult<IscoProperties>
//
//   pub fn keplerian_freq(r: f64, mass: f64, spin: f64) -> f64
//     // Ω_K = M^(1/2) / (r^(3/2) + a * M^(1/2))
//
//   pub fn circular_orbit_energy(r: f64, mass: f64, spin: f64) -> f64
//     // E = (r − 2M ± a√(M/r)) / sqrt(r² − 3Mr ± 2a√(Mr))
//
//   pub fn circular_orbit_angmom(r: f64, mass: f64, spin: f64) -> f64
//     // L = M^(1/2) * (r² ∓ 2a M^(1/2) r^(1/2) + a²) / (r^(3/4) * D)
//
// USES (imports from):
//   crate::metric::kerr  → event_horizon() for sanity check (r_ISCO > r_+)
//   crate::units         → geom_to_meters, solar_mass_to_geom
//   crate::errors        → PhysicsResult, PhysicsError::IscoFailed
//
// USED BY:
//   crate::accretion::novikov_thorne → r_isco as inner disk boundary
//   crate::accretion::mhd            → r_isco for jet anchor
//   crate::accretion::spectrum       → inner edge for T(r) integration
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → r_isco uploaded to accretion_disk.wgsl as disk inner radius
//   crates/gargantua-ui/src/menu/tabs/physics_tab.rs
//     → displays IscoProperties: r_ISCO, binding_energy, orbital_freq
//   tests/isco.rs
//     → validates against M87* (a=0.9 → r≈2.32M) and SgrA* (a=0.6 → r≈3.83M)
//
// NOTE FOR AI:
//   Reference validation values:
//     Schwarzschild (a=0.0):      r_ISCO = 6.000 M
//     SgrA* prograde  (a=0.6):   r_ISCO ≈ 3.829 M
//     M87* prograde   (a=0.9):   r_ISCO ≈ 2.321 M
//     Extremal prograde (a→1):   r_ISCO → 1.000 M
//     Extremal retrograde (a→-1):r_ISCO → 9.000 M
//   Returns PhysicsError::IscoFailed for |spin| >= 1.0 (extremal Kerr
//   is numerically degenerate — avoid exact a=1 in production).
// ============================================================