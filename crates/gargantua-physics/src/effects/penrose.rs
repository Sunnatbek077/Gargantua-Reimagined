// ============================================================
// FILE: crates/gargantua-physics/src/effects/penrose.rs
// LINES: ~320
// CATEGORY: Physics — Penrose process + ergosphere geometry
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Models the Penrose process — energy extraction from the ergosphere
//   of a rotating BH. Provides ergosphere geometry for the render,
//   physics readout values (efficiency, extractable energy), and
//   BZ jet power estimate for UI display alongside mhd.rs.
//
// CONTENTS (~320 lines):
//   // Ergosphere outer radius at polar angle θ:
//   // r_ergo(θ) = M + sqrt(M² − a²cos²θ)
//   // At equator θ=π/2: r_ergo = 2M (same as Schwarzschild horizon)
//   // At poles   θ=0,π: r_ergo = r_+ (merges with event horizon)
//   pub fn ergosphere_radius(theta: f64, mass: f64, spin: f64) -> f64
//
//   // Is point (r, θ) inside ergosphere? r_+ < r < r_ergo(θ)
//   pub fn in_ergosphere(r: f64, theta: f64, mass: f64, spin: f64) -> bool
//
//   // Maximum energy extractable via Penrose process:
//   // E_max/Mc² = 1 − 1/sqrt(2) ≈ 0.293  (extremal a=1 limit)
//   pub fn max_extractable_energy(mass: f64, spin: f64) -> f64
//
//   // Penrose efficiency η = (E_out − E_in) / E_in
//   // Can exceed 100% because the infalling particle carries negative energy
//   pub fn penrose_efficiency(spin: f64) -> f64
//
//   // Frame dragging angular velocity at (r, θ):
//   // Ω_FD = -g_tφ / g_φφ = 2Mar / ((r²+a²)² − a²Δsin²θ)
//   pub fn frame_drag_omega(r: f64, theta: f64, mass: f64, spin: f64) -> f64
//
//   // BZ jet power estimate: P_BZ ∝ a² B² M²
//   // Must match mhd.rs::jet_power() numerically — keep in sync
//   pub fn bz_power_estimate(mass: f64, spin: f64, b_gauss: f64) -> f64
//
//   // Ergosphere cross-section mesh for 3D visualization:
//   // Returns Vec<[f32;3]> Cartesian (x,y,z) at n_theta θ-steps
//   // x = r_ergo(θ)*sin(θ),  z = r_ergo(θ)*cos(θ),  y = 0
//   pub fn ergosphere_surface(mass: f64, spin: f64, n_theta: usize) -> Vec<[f32; 3]>
//
// USES (imports from):
//   crate::metric::kerr  → event_horizon(), g_mu_nu() (g_tφ, g_φφ)
//   crate::units         → geom_to_meters (energy in Joules for display)
//
// USED BY:
//   crates/gargantua-ui/src/overlay/physics_readout.rs
//     → penrose_efficiency(), max_extractable_energy() in overlay panel
//   crates/gargantua-ui/src/menu/tabs/physics_tab.rs
//     → ergosphere radius display, visualization toggle
//   crates/gargantua-render/src/pipelines/ray_march.rs
//     → in_ergosphere() to apply special glowing ergosphere shading
//
// NOTE FOR AI:
//   No ergosphere for a=0 — ergosphere_radius(θ) = event_horizon() at a=0.
//   in_ergosphere() returns false for all a=0 input.
//   Validation: a=0.998, max efficiency ≈ 20.7% | a=0.9, ≈ 19.0%
//   bz_power_estimate() must stay consistent with mhd.rs::jet_power().
//   Both use: P_BZ = (κ/4πc) * (B * π * r_+²)² * (a/(2r_+))²
//   ergosphere_surface() returns Cartesian, NOT Boyer-Lindquist.
// ============================================================