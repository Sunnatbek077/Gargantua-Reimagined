// ============================================================
// FILE: crates/gargantua-physics/src/accretion/novikov_thorne.rs
// LINES: ~340
// CATEGORY: Physics — Novikov-Thorne relativistic thin disk model
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Novikov-Thorne (1973) relativistic thin disk: computes local
//   disk temperature T(r) at each radius. Drives blackbody spectrum,
//   brightness, and color in the render. Standard GR accretion disk
//   model used throughout astrophysics for AGN and X-ray binaries.
//
// CONTENTS (~340 lines):
//   pub struct NovikovThorneDisk {
//       pub mass:           f64,  // M in G=c=1
//       pub spin:           f64,  // a dimensionless
//       pub accretion_rate: f64,  // Ṁ in Eddington units [0..1], default 0.1
//       pub r_isco:         f64,  // inner edge from isco::kerr_isco()
//       pub r_outer:        f64,  // outer edge, default 20M
//       pub efficiency:     f64,  // η = 1 − E_ISCO (GR binding energy fraction)
//   }
//
//   impl NovikovThorneDisk {
//       pub fn new(mass: f64, spin: f64, accretion_rate: f64) -> PhysicsResult<Self>
//         // calls isco::kerr_isco() to set r_isco
//         // sets efficiency η = 1 − circular_orbit_energy(r_isco)
//
//       pub fn temperature(&self, r: f64) -> f64
//         // Returns T(r) in Kelvin
//         // NT flux: F(r) = (3GMṀ / 8πr³) * C(r, a)
//         // Stefan-Boltzmann: T = (F / σ_SB)^(1/4)
//         // Returns 0.0 for r < r_isco (no disk inside ISCO)
//
//       pub fn luminosity(&self) -> f64
//         // L = η * Ṁ * c²  [Watts]
//         // Also: L = 4π ∫ F(r) r dr from r_isco to r_outer
//
//       pub fn peak_temperature(&self) -> f64
//         // max T(r) — typically at r ≈ 1.36 * r_ISCO
//
//       pub fn temperature_profile(&self, n: usize) -> Vec<(f64, f64)>
//         // n log-spaced radii in [r_isco, r_outer]
//         // Returns Vec<(r_geom, T_kelvin)> for GPU texture upload
//
//       fn nt_correction_factor(&self, r: f64) -> f64
//         // C(r,a) = [1 − (r_ISCO/r)^(1/2)] * g(r, a)
//         // g encodes angular momentum deficit at ISCO
//         // → 1 for r >> M (Newtonian limit)
//   }
//
// USES (imports from):
//   crate::accretion::isco  → kerr_isco(), circular_orbit_energy(), keplerian_freq()
//   crate::metric::kerr     → KerrNewman for g_tt at r
//   crate::units            → geom_to_meters (Ṁ SI conversion)
//   crate::errors           → PhysicsResult
//
// USED BY:
//   crate::accretion::spectrum  → temperature(r) → Planck → RGB
//   crate::accretion::mhd       → wraps this, adds turbulence
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → uploads temperature_profile() as 1D GPU texture (256 samples)
//   crates/gargantua-ui/src/menu/tabs/accretion_tab.rs
//     → displays peak_temperature(), luminosity()
//
// NOTE FOR AI:
//   NT flux: F(r) = (3GMṀ)/(8πr³) * C(r,a)
//   σ_SB = 5.670374419e-8 W/m²/K⁴ (Stefan-Boltzmann constant)
//   Eddington luminosity: L_Edd = 1.26e31 * (M/M_sun) Watts
//   accretion_rate=1.0 → Ṁ = L_Edd / (η c²) [Eddington-limited]
//   Default accretion_rate = 0.1 (10% Eddington — typical Seyfert AGN)
//   temperature_profile() uses log-spacing between r_isco and r_outer.
//   r_outer default = 20M. User can increase up to ~100M via UI slider.
// ============================================================