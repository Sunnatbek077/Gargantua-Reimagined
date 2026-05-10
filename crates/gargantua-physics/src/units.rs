// ============================================================
// FILE: crates/gargantua-physics/src/units.rs
// LINES: ~80
// CATEGORY: Physics — unit system constants and converters
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Defines G = c = 1 geometrized unit system used throughout
//   the entire physics crate. Provides Schwarzschild radius
//   computation and mass conversion helpers.
//
// CONTENTS (~80 lines):
//   // Physical constants in SI (for conversion only)
//   pub const G_SI: f64 = 6.674e-11;      // m³ kg⁻¹ s⁻²
//   pub const C_SI: f64 = 2.998e8;        // m/s
//   pub const M_SUN_KG: f64 = 1.989e30;   // kg
//
//   // Schwarzschild radius: r_s = 2GM/c²  (in meters)
//   pub fn schwarzschild_radius(mass_kg: f64) -> f64
//
//   // Convert solar masses → geometrized mass M (G=c=1 units)
//   pub fn solar_mass_to_geom(m_sun: f64) -> f64
//
//   // Convert geometrized radius r → physical meters
//   pub fn geom_to_meters(r_geom: f64, mass_kg: f64) -> f64
//
//   // Dimensionless spin: a* = J c / (G M²), clamped to (-1, 1)
//   pub fn dimensionless_spin(j: f64, mass_kg: f64) -> f64
//
// USES (imports from):
//   No internal imports. Pure math constants + free functions.
//
// USED BY:
//   metric/kerr.rs           → uses solar_mass_to_geom, dimensionless_spin
//   metric/schwarzschild.rs  → uses schwarzschild_radius
//   accretion/isco.rs        → uses geom_to_meters for display
//   crates/gargantua-camera/src/world/coord_system.rs
//
// NOTE FOR AI:
//   All physics code uses G=c=1. SI constants here are ONLY
//   for user-facing unit conversion (UI display).
//   Never pass raw kg or meters into metric/geodesic functions —
//   always convert with these helpers first.
// ============================================================