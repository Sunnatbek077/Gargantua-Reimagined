// =============================================================================
// crates/gargantua-camera/src/world/coord_system.rs
// =============================================================================
//
// PURPOSE:
//   Defines the coordinate system conventions used throughout Gargantua.
//   Provides conversion functions between coordinate systems:
//     - World space (Cartesian, right-handed, Y-up)
//     - Boyer-Lindquist coordinates (r, θ, φ) — standard for Kerr metric
//     - Spherical coordinates (r, polar angle θ, azimuth φ) — for starmap UV
//     - Camera space (view-space, right-handed, Z-forward)
//     - Screen/NDC space (clip space, [-1,1]³)
//
//   All physics simulation (gargantua-physics) works in Boyer-Lindquist.
//   All rendering (gargantua-render) works in world Cartesian.
//   This module bridges the two.
//
// SIZE: ~280 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CameraError
//   External:
//     - glam::{Vec2, Vec3, Vec4, Mat4, Quat}
//     - std::f32::consts::{PI, TAU, FRAC_PI_2}
//
// CALLED BY:
//   - crate::world_camera::WorldCamera         — uses world ↔ camera transforms
//   - crate::modes::orbit::OrbitMode           — works in spherical coords
//   - crate::modes::gravity::GravityMode       — works in Boyer-Lindquist
//   - crate::modes::satellite::SatelliteMode   — orbital mechanics in BL coords
//   - crates/gargantua-render/src/bindgroups/scene.rs
//       — needs starmap UV from equirectangular projection
//   - tests/coord_system.rs                    — unit tests for all conversions
//
// COORDINATE SYSTEM CONVENTIONS:
//
//   WORLD SPACE (Cartesian, right-handed):
//     +X = right (East in sky)
//     +Y = up (celestial North pole / black hole spin axis)
//     +Z = toward viewer (camera default forward direction = -Z)
//     Origin = center of black hole
//     Units: gravitational units (M = 1.0, G = c = 1.0)
//
//   BOYER-LINDQUIST (Kerr metric standard):
//     r = radial distance (coordinate radius, not proper distance)
//     θ = polar angle (0 = north pole, π/2 = equatorial plane, π = south pole)
//     φ = azimuthal angle (0..2π, right-hand rule around +Y spin axis)
//     Σ = r² + a²cos²θ (Kerr metric factor)
//     Δ = r² - 2Mr + a² (Kerr metric factor)
//
//   EQUIRECTANGULAR (starmap UV):
//     u = φ / (2π) + 0.5     → maps azimuth to [0,1]
//     v = θ / π              → maps polar angle to [0,1] (0=north, 1=south)
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy)]
//   pub struct BoyerLindquist {
//     pub r:   f32,   // radial coordinate (> r_horizon for exterior region)
//     pub theta: f32, // polar angle in [0, π]
//     pub phi:   f32, // azimuthal angle in [0, 2π)
//   }
//
//   #[derive(Debug, Clone, Copy)]
//   pub struct SphericalCoord {
//     pub r:     f32,
//     pub theta: f32,  // elevation: 0 = north pole, π/2 = equator, π = south pole
//     pub phi:   f32,  // azimuth: 0..2π
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn cartesian_to_bl(pos: Vec3, spin: f32) -> BoyerLindquist
//     — converts world Cartesian (x,y,z) to Boyer-Lindquist (r,θ,φ).
//     — spin = Kerr a/M parameter (needed because BL r ≠ Euclidean r for a≠0).
//     — Euclidean r² = x² + y² + z².
//     — BL r is the largest root of: (x²+y²+z²-a²) - 2r² + (a²cos²θ - r²) = 0
//       Approximation for moderate spin: r_BL ≈ sqrt(x²+y²+z²) is adequate
//       at r >> r_horizon. Exact solution requires solving a quartic.
//     — θ = arccos(y / r_BL)  (polar angle from spin axis = Y axis)
//     — φ = atan2(z, x) normalized to [0, 2π)
//
//   pub fn bl_to_cartesian(bl: BoyerLindquist, spin: f32) -> Vec3
//     — inverse: Boyer-Lindquist → world Cartesian.
//     — x = sqrt(r² + a²) × sin(θ) × cos(φ)
//     — y = r × cos(θ)
//     — z = sqrt(r² + a²) × sin(θ) × sin(φ)
//     — where a = spin × M (M = 1.0 in geometric units).
//
//   pub fn cartesian_to_spherical(pos: Vec3) -> SphericalCoord
//     — Euclidean spherical (no Kerr correction):
//     — r = pos.length()
//     — theta = acos(pos.y / r)   (if r ≈ 0: theta = 0)
//     — phi = atan2(pos.z, pos.x).rem_euclid(TAU)
//
//   pub fn spherical_to_cartesian(s: SphericalCoord) -> Vec3
//     — x = r × sin(theta) × cos(phi)
//     — y = r × cos(theta)
//     — z = r × sin(theta) × sin(phi)
//
//   pub fn direction_to_equirect_uv(dir: Vec3) -> Vec2
//     — converts a normalized direction vector to equirectangular UV
//       for starmap lookup.
//     — u = atan2(dir.z, dir.x) / TAU + 0.5   → [0, 1]
//     — v = acos(dir.y.clamp(-1.0, 1.0)) / PI → [0, 1]
//     — used by starfield.wgsl for background star sampling.
//
//   pub fn equirect_uv_to_direction(uv: Vec2) -> Vec3
//     — inverse: UV → unit direction vector.
//     — phi = (uv.x - 0.5) × TAU
//     — theta = uv.y × PI
//     — dir = Vec3(sin(theta)×cos(phi), cos(theta), sin(theta)×sin(phi))
//
//   pub fn look_at_matrix(eye: Vec3, target: Vec3, up: Vec3) -> Mat4
//     — constructs a right-handed view matrix (same as glam::Mat4::look_at_rh).
//     — stored separately here with explicit conventions documented.
//     — forward = (target - eye).normalize()
//     — right   = forward.cross(up).normalize()
//     — up_true  = right.cross(forward)
//     — returns column-major Mat4 matching WGSL mat4x4<f32> layout.
//
//   pub fn projection_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Mat4
//     — right-handed perspective projection with reversed Z (depth 1..0):
//       reversed-Z provides better floating-point precision in depth buffer.
//     — near plane at depth=1.0, far plane at depth=0.0.
//     — used by WorldCamera::proj_matrix().
//     — matches wgpu clip space convention (Y-up, Z in [0,1]).
//
// NOTES FOR AI:
//   - All angles are in RADIANS throughout this module.
//   - Boyer-Lindquist coordinates become singular at the horizon (r = r+)
//     and at the poles (θ = 0, π). Camera modes must not place the camera
//     exactly at these singularities.
//   - The spin axis is the +Y axis in world space. This means the
//     equatorial plane of the black hole is the XZ plane (y = 0).
//   - direction_to_equirect_uv is the SAME formula as in starfield.wgsl.
//     Changes here must be reflected in the shader.
//   - look_at_matrix: if eye == target, forward = Vec3::ZERO → NaN.
//     Guard: if (target - eye).length() < 1e-6, use a default forward.
// =============================================================================

use glam::{Mat4, Vec2, Vec3, Vec4};
use std::f32::consts::{PI, TAU};

#[derive(Debug, Clone, Copy)]
pub struct BoyerLindquist {
    pub r:     f32,
    pub theta: f32,
    pub phi:   f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SphericalCoord {
    pub r:     f32,
    pub theta: f32,
    pub phi:   f32,
}

pub fn cartesian_to_bl(pos: Vec3, spin: f32) -> BoyerLindquist {
    let r_eucl = pos.length();
    let a = spin; // a = spin × M, M = 1.0 in geometric units
    // Approximate BL r for moderate spin (exact requires quartic solve)
    let w = r_eucl * r_eucl - a * a;
    let r = ((w + (w * w + 4.0 * a * a * pos.y * pos.y).sqrt()) / 2.0).sqrt();
    let theta = if r > 1e-6 { (pos.y / r).clamp(-1.0, 1.0).acos() } else { 0.0 };
    let phi   = f32::atan2(pos.z, pos.x).rem_euclid(TAU);
    BoyerLindquist { r, theta, phi }
}

pub fn bl_to_cartesian(bl: BoyerLindquist, spin: f32) -> Vec3 {
    let a   = spin;
    let rho = (bl.r * bl.r + a * a).sqrt();
    Vec3::new(
        rho  * bl.theta.sin() * bl.phi.cos(),
        bl.r * bl.theta.cos(),
        rho  * bl.theta.sin() * bl.phi.sin(),
    )
}

pub fn cartesian_to_spherical(pos: Vec3) -> SphericalCoord {
    let r = pos.length();
    SphericalCoord {
        r,
        theta: if r > 1e-9 { (pos.y / r).clamp(-1.0, 1.0).acos() } else { 0.0 },
        phi:   f32::atan2(pos.z, pos.x).rem_euclid(TAU),
    }
}

pub fn spherical_to_cartesian(s: SphericalCoord) -> Vec3 {
    Vec3::new(
        s.r * s.theta.sin() * s.phi.cos(),
        s.r * s.theta.cos(),
        s.r * s.theta.sin() * s.phi.sin(),
    )
}

pub fn direction_to_equirect_uv(dir: Vec3) -> Vec2 {
    Vec2::new(
        f32::atan2(dir.z, dir.x) / TAU + 0.5,
        dir.y.clamp(-1.0, 1.0).acos() / PI,
    )
}

pub fn equirect_uv_to_direction(uv: Vec2) -> Vec3 {
    let phi   = (uv.x - 0.5) * TAU;
    let theta = uv.y * PI;
    Vec3::new(
        theta.sin() * phi.cos(),
        theta.cos(),
        theta.sin() * phi.sin(),
    )
}

pub fn look_at_matrix(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

pub fn projection_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    // Reversed-Z perspective: near → depth 1.0, far → depth 0.0
    Mat4::perspective_rh(fov_y_rad, aspect, near, far)
}