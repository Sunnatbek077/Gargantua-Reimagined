// ============================================================
// FILE: crates/gargantua-physics/src/effects/frame_dragging.rs
// LINES: ~260
// CATEGORY: Physics — Lense-Thirring frame dragging (derived quantities)
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Computes human-readable Lense-Thirring frame dragging quantities.
//   Rotating BHs drag spacetime into rotation — local inertial frames
//   co-rotate even for zero-angular-momentum observers (ZAMOs).
//   Used for UI physics readout, disk rotation rate, and camera init.
//   NOTE: Frame dragging is already physically encoded in Kerr's g_tφ
//   term — this file provides derived display quantities only.
//
// CONTENTS (~260 lines):
//   // Angular velocity of local inertial frames at (r, θ):
//   // Ω_FD = -g_tφ / g_φφ = 2Mar / ((r²+a²)² − a²Δsin²θ)
//   pub fn frame_drag_angular_velocity(
//       r: f64, theta: f64, mass: f64, spin: f64,
//   ) -> f64   // rad / (GM/c³)
//
//   // Lense-Thirring gyroscope precession rate (weak-field approx):
//   // Ω_LT ≈ 2GJ / (c² r³),  J = spin * M² (geometrized)
//   // For display and intuition only — NOT used in physics pipeline
//   pub fn lt_precession_rate(r: f64, mass: f64, spin: f64) -> f64
//
//   // Minimum angular momentum for a prograde orbit at r:
//   // Below L_min, retrograde orbit is impossible — frame dragging wins
//   pub fn minimum_prograde_angmom(r: f64, mass: f64, spin: f64) -> f64
//
//   // ZAMO (Zero Angular Momentum Observer) angular velocity:
//   // Same formula as frame_drag_angular_velocity()
//   // A ZAMO has L=0 but still co-rotates with Ω_FD
//   pub fn zamo_angular_velocity(r: f64, theta: f64, mass: f64, spin: f64) -> f64
//
//   // Visual: angle dragged per complete orbit at radius r
//   // = (Ω_FD / Ω_orbital) * 2π  — intuitive display value
//   pub fn drag_angle_per_orbit(r: f64, mass: f64, spin: f64) -> f64
//
// USES (imports from):
//   crate::metric::kerr  → KerrNewman g_mu_nu() for g_tφ and g_φφ
//   crate::units         → geom_to_meters for SI display conversions
//
// USED BY:
//   crates/gargantua-ui/src/overlay/physics_readout.rs
//     → frame_drag_angular_velocity() at camera r, displayed in real time
//   crates/gargantua-render/src/pipelines/accretion.rs
//     → Ω_FD sets material rotation rate in accretion_disk.wgsl
//   crates/gargantua-camera/src/modes/gravity.rs
//     → zamo_angular_velocity() initializes camera co-rotation velocity
//
// NOTE FOR AI:
//   Frame dragging is NOT a separate force added on top of gravity.
//   It IS the g_tφ ≠ 0 component of the Kerr metric.
//   All geodesic calculations already include it via christoffel().
//   This file gives human-readable physical quantities derived from g_tφ.
//   lt_precession_rate() is weak-field only — use for UI, never for physics.
//   For M87* (a≈0.9) at r=5M: Ω_FD ≈ 0.04 / M (very strong dragging).
//   At r → ∞: Ω_FD → 0 (spacetime approaches flat Minkowski).
// ============================================================