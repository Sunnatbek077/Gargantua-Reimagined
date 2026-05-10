// ============================================================
// FILE: crates/gargantua-physics/src/accretion/mhd.rs
// LINES: ~420
// CATEGORY: Physics — MHD turbulence + Blandford-Znajek jet model
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Magnetohydrodynamic disk model: adds magnetic turbulence on top
//   of Novikov-Thorne temperature profile, and models the relativistic
//   jet powered by Blandford-Znajek (BZ) spin energy extraction.
//   Drives disk surface brightness variation and jet visualization.
//
// CONTENTS (~420 lines):
//   pub struct MhdDisk {
//       pub nt_disk:  NovikovThorneDisk,
//       pub beta:     f64,   // plasma β = P_gas/P_mag, default 10
//       pub b_field:  f64,   // large-scale B field in Gauss
//       pub jet_on:   bool,  // enable BZ jet in render
//   }
//
//   impl MhdDisk {
//       pub fn new(nt: NovikovThorneDisk, beta: f64, b_field: f64) -> Self
//
//       pub fn temperature(&self, r: f64, seed: u64) -> f64
//         // T_mhd(r) = T_NT(r) * (1 + δ(r, seed))
//         // δ = turbulence_amplitude(r, seed)
//         // seed = frame_index → time-evolving turbulence
//
//       pub fn surface_brightness(&self, r: f64, phi: f64, seed: u64) -> f64
//         // T^4 weighted with azimuthal hot-spot variation
//
//       pub fn jet_power(&self) -> f64
//         // Blandford-Znajek formula:
//         // P_BZ = (κ / 4πc) * Φ_BH² * Ω_H²
//         // Ω_H  = a / (2 r_+)         [horizon angular velocity]
//         // Φ_BH = B * π * r_+²        [magnetic flux through horizon]
//         // κ ≈ 0.044 (split-monopole field geometry)
//
//       pub fn jet_lorentz_factor(&self, z: f64) -> f64
//         // Γ(z) = Γ_max * tanh(z / z_0)
//         // Γ_max ≈ 10 (M87-like), z_0 = 100M (acceleration zone)
//         // For visualization only — jet is not ray-traced
//
//       fn turbulence_amplitude(&self, r: f64, seed: u64) -> f64
//         // α-disk: δ ~ alpha / sqrt(beta),  alpha = 0.1 (Shakura-Sunyaev)
//         // Uses PCG hash (deterministic) — NOT rand::random()
//         // Must be reproducible: same (r, seed) → same δ
//   }
//
// USES (imports from):
//   crate::accretion::novikov_thorne → NovikovThorneDisk, temperature()
//   crate::accretion::isco           → r_isco, event_horizon
//   crate::metric::kerr              → event_horizon() for Ω_H
//   crate::errors                    → PhysicsResult
//
// USED BY:
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → surface_brightness(r, phi, frame_idx) per disk fragment
//     → jet_power(), jet_lorentz_factor() uploaded as shader uniforms
//   crates/gargantua-ui/src/menu/tabs/accretion_tab.rs
//     → sliders: beta [1–100], b_field [1–1e5 G], jet_on toggle
//     → displays jet_power() in Watts
//
// NOTE FOR AI:
//   BZ power: P_BZ = (κ/4πc) * Φ_BH² * Ω_H²
//   For M87*: P_BZ ≈ 10^42 erg/s → tune b_field ≈ 10^4 G near horizon.
//   turbulence_amplitude() uses PCG32 hash — NOT standard rand.
//   Reason: rand is non-deterministic across platforms; PCG hash gives
//   identical noise on CPU and GPU (GPU reimplements same hash).
//   jet_lorentz_factor() z is in geometrized units (same as r).
//   jet_on=false → jet_power() and jet_lorentz_factor() still computed
//   for physics readout; only the render pipeline skips jet draw calls.
// ============================================================