// ============================================================
// FILE: crates/gargantua-physics/src/effects/redshift.rs
// LINES: ~220
// CATEGORY: Physics — Gravitational redshift and time dilation
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Gravitational redshift of photons climbing out of the BH potential
//   well. Near the horizon, photons approach infinite redshift — light
//   becomes infinitely red and infinitely dimmed. Also computes time
//   dilation used by the camera time-warp visual effect.
//
// CONTENTS (~220 lines):
//   // Full Kerr gravitational redshift factor (1+z) at (r, θ):
//   // 1 + z = 1 / sqrt(−g_tt − 2 g_tφ Ω − g_φφ Ω²)
//   // where Ω = keplerian_freq(r) = circular orbit angular velocity
//   pub fn gravitational_redshift(
//       r: f64, theta: f64, metric: &dyn MetricTensor,
//   ) -> f64   // returns (1+z) ≥ 1, diverges at horizon
//
//   // Fast Schwarzschild approximation (UI display only, a=0):
//   // 1 + z = (1 − 2M/r)^(−1/2)
//   pub fn schwarzschild_redshift(r: f64, mass: f64) -> f64
//
//   // Combined gravitational + Doppler total redshift:
//   // (1 + z_total) = (1 + z_grav) * (1 + z_doppler)
//   pub fn total_redshift(
//       r: f64, theta: f64,
//       beta: f64, cos_angle: f64,
//       metric: &dyn MetricTensor,
//   ) -> f64
//
//   // Redshift-corrected observed temperature:
//   // T_obs = T_emit / (1 + z_total)
//   pub fn observed_temperature(t_emit: f64, z_total: f64) -> f64
//
//   // Gravitational time dilation τ/t (proper/coordinate time):
//   // dτ/dt = 1/(1+z) = sqrt(−g_tt − 2 g_tφ Ω − g_φφ Ω²)
//   // Used by camera time_warp.rs for visual slowdown near horizon
//   pub fn time_dilation(r: f64, mass: f64, spin: f64) -> f64
//
// USES (imports from):
//   crate::metric::mod.rs   → MetricTensor: g_mu_nu() (g_tt, g_tφ, g_φφ)
//   crate::metric::kerr     → KerrNewman for full Kerr formula
//   crate::effects::doppler → doppler_factor() for total_redshift()
//
// USED BY:
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → total_redshift() per disk fragment, modifies T_obs
//   crates/gargantua-camera/src/fx/time_warp.rs
//     → time_dilation() for camera animation slow-down near horizon
//   crates/gargantua-ui/src/overlay/physics_readout.rs
//     → displays (1+z) at camera position in real time
//
// NOTE FOR AI:
//   Validation values for Schwarzschild:
//     r = 6M:  z ≈ 0.225  (22% redshifted)
//     r = 3M:  z ≈ 0.732  (73% redshifted)
//     r = 2.5M:z ≈ 1.000  (wavelength doubled)
//     r → 2M:  z → ∞
//   For Kerr (a > 0): ALWAYS use gravitational_redshift() with g_tφ term.
//   schwarzschild_redshift() is for UI display approximation only (a=0 assumed).
//   The g_tφ frame-dragging term is critical for correct Kerr redshift.
//   time_dilation() returns ∈ (0, 1]: 1.0 = flat spacetime, 0 = horizon.
// ============================================================