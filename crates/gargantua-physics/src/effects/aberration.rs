// ============================================================
// FILE: crates/gargantua-physics/src/effects/aberration.rs
// LINES: ~240
// CATEGORY: Physics — Special relativistic stellar aberration
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   SR stellar aberration — at relativistic speeds, stars crowd toward
//   the direction of motion (headlight / lighthouse effect). Used by
//   free-flight camera mode and the starfield shader.
//   NOTE: GR light bending near the BH is handled separately by the
//   geodesic tracer. This file is purely special-relativistic.
//
// CONTENTS (~240 lines):
//   // Exact SR aberration formula:
//   // cos θ_obs = (cos θ_src − β) / (1 − β cos θ_src)
//   pub fn aberration_angle(theta_src: f64, beta: f64) -> f64
//
//   // Apply aberration to unit direction vector:
//   // velocity_dir: normalized camera velocity (rest frame)
//   // star_dir:     normalized direction to star (rest frame)
//   // Returns: observed star direction at speed β
//   pub fn aberrate_direction(
//       star_dir: [f64; 3],
//       velocity_dir: [f64; 3],
//       beta: f64,
//   ) -> [f64; 3]
//
//   // Solid angle compression: dΩ_obs / dΩ_src = 1 / D²
//   // Stars bunch into forward cone → appear brighter approaching
//   pub fn aberration_solid_angle_factor(theta_src: f64, beta: f64) -> f64
//
//   // FOV compression at relativistic speed:
//   // At β=0.99: 180° forward hemisphere shrinks to ~8° half-angle
//   pub fn relativistic_fov_compression(fov_deg: f64, beta: f64) -> f64
//
//   // Build 3×3 f32 aberration matrix for GPU upload:
//   // Rotates star directions from rest frame to aberrated frame at β
//   pub fn aberration_matrix(velocity_dir: [f64; 3], beta: f64) -> [[f32; 3]; 3]
//
// USES (imports from):
//   crate::units  → c_si (for β = v / c_si)
//
// USED BY:
//   crates/gargantua-camera/src/fx/relativistic_fov.rs
//     → relativistic_fov_compression() when user sets camera β via UI
//   crates/gargantua-render/src/pipelines/starfield.rs
//     → uploads aberration_matrix() as uniform to starfield.wgsl
//   shaders/render/starfield.wgsl
//     → applies 3×3 matrix per-fragment for star direction shift
//
// NOTE FOR AI:
//   SR ONLY — GR aberration is already encoded in null geodesics.
//   Do NOT add any Christoffel or metric calls here.
//   β validation: clamp β to [0.0, 0.999] before calling any function.
//   At β=0.0: no aberration, aberration_matrix returns identity.
//   At β=0.5:  forward hemisphere 90° → ~60°
//   At β=0.99: forward hemisphere 90° → ~8°
//   aberration_matrix() result is uploaded once per frame if camera moves.
// ============================================================