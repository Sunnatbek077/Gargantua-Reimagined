// =============================================================================
// crates/gargantua-camera/tests/coord_system.rs
// =============================================================================
//
// PURPOSE:
//   Integration tests for all coordinate system conversion functions in
//   world/coord_system.rs. Tests round-trip accuracy (Cartesian → BL → Cartesian,
//   spherical → Cartesian → spherical, direction → UV → direction) and
//   edge cases (poles, equatorial plane, origin proximity).
//
//   These tests run without GPU — pure math verification. They validate
//   that the Rust coord_system.rs matches the WGSL shader logic in
//   starfield.wgsl (direction_to_equirect_uv formula).
//
// SIZE: ~120 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_camera::world::coord_system::*
//   External:
//     - glam::{Vec2, Vec3}
//     - std::f32::consts::PI
//
// TEST CASES:
//
//   test cartesian_to_spherical_roundtrip
//     — for multiple positions (above pole, equatorial, diagonal):
//       cartesian → spherical → cartesian, assert distance < 1e-4.
//
//   test bl_to_cartesian_schwarzschild
//     — spin = 0.0: BL coordinates reduce to spherical.
//       For (r=10, θ=π/2, φ=0): expect Cartesian ≈ (10, 0, 0).
//       For (r=10, θ=0, φ=0):   expect Cartesian ≈ (0, 10, 0) (north pole).
//
//   test bl_roundtrip_low_spin
//     — for spin = 0.1: cartesian_to_bl then bl_to_cartesian.
//       Assert round-trip error < 0.5% of r (BL approximation accuracy).
//
//   test equirect_uv_roundtrip
//     — for 8 directions (±X, ±Y, ±Z, diagonals):
//       direction_to_equirect_uv then equirect_uv_to_direction.
//       Assert angle between input and output direction < 0.01 radians.
//
//   test equirect_uv_specific
//     — (1,0,0): u ≈ 0.5, v ≈ 0.5 (equatorial, 0° longitude)
//     — (0,1,0): u ≈ 0.5, v ≈ 0.0 (north pole)
//     — (0,-1,0): u ≈ 0.5, v ≈ 1.0 (south pole)
//     — (-1,0,0): u ≈ 0.0 (or 1.0), v ≈ 0.5 (180° longitude)
//
//   test look_at_matrix_z_forward
//     — eye=(0,0,5), target=(0,0,0), up=(0,1,0).
//       Forward direction in view space should be (0,0,-1) (right-handed).
//
//   test projection_matrix_clip_range
//     — near=0.01, far=10000: verify that a point at z=-near maps to
//       NDC depth near 1.0, and z=-far maps to near 0.0 (reversed-Z).
//
//   test polar_edge_cases
//     — cartesian_to_spherical((0,1,0)): theta ≈ 0 (north pole), r = 1.
//     — cartesian_to_spherical((0,0,0)): r = 0, theta = 0 (guard against NaN).
//     — spherical_to_cartesian({r=0, theta=0, phi=0}): result = (0,0,0).
//
// EXPECTED ACCURACY:
//   - Cartesian ↔ Spherical: exact (no approximation), error < 1e-6.
//   - Cartesian ↔ BL (spin=0): exact, error < 1e-6.
//   - Cartesian ↔ BL (spin=0.1): approximate, error < 0.5% of r.
//   - UV roundtrip: error < 0.01 radians angular.
//
// NOTES FOR AI:
//   - Run with: cargo test --package gargantua-camera
//   - No GPU, no wgpu, no winit required — pure math.
//   - Use assert_relative_eq! from the `approx` crate (add to dev-dependencies)
//     or manual: assert!((a - b).abs() < epsilon, "got {a}, expected {b}").
// =============================================================================

use gargantua_camera::world::coord_system::*;
use glam::{Vec2, Vec3};
use std::f32::consts::PI;

fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

fn vec3_approx_eq(a: Vec3, b: Vec3, eps: f32) -> bool {
    (a - b).length() < eps
}

#[test]
fn test_cartesian_to_spherical_roundtrip() {
    let positions = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(3.0, 4.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-2.0, 3.0, -1.0),
        Vec3::new(10.0, 0.001, 0.0), // near equatorial
    ];
    for pos in &positions {
        let sph = cartesian_to_spherical(*pos);
        let back = spherical_to_cartesian(sph);
        assert!(
            vec3_approx_eq(*pos, back, 1e-4),
            "Roundtrip failed for {pos}: got {back}"
        );
    }
}

#[test]
fn test_bl_to_cartesian_schwarzschild() {
    let spin = 0.0_f32;
    // Equatorial plane, 0° longitude → +X
    let bl_eq = BoyerLindquist { r: 10.0, theta: PI / 2.0, phi: 0.0 };
    let cart_eq = bl_to_cartesian(bl_eq, spin);
    assert!(approx_eq(cart_eq.x, 10.0, 1e-4), "x={}", cart_eq.x);
    assert!(approx_eq(cart_eq.y, 0.0,  1e-4), "y={}", cart_eq.y);
    assert!(approx_eq(cart_eq.z, 0.0,  1e-4), "z={}", cart_eq.z);

    // North pole
    let bl_np = BoyerLindquist { r: 10.0, theta: 0.0, phi: 0.0 };
    let cart_np = bl_to_cartesian(bl_np, spin);
    assert!(approx_eq(cart_np.y, 10.0, 1e-4), "y={}", cart_np.y);
}

#[test]
fn test_equirect_uv_roundtrip() {
    let directions = [
        Vec3::new(1.0, 0.0, 0.0).normalize(),
        Vec3::new(-1.0, 0.0, 0.0).normalize(),
        Vec3::new(0.0, 1.0, 0.0).normalize(),
        Vec3::new(0.0, -1.0, 0.0).normalize(),
        Vec3::new(0.0, 0.0, 1.0).normalize(),
        Vec3::new(0.0, 0.0, -1.0).normalize(),
        Vec3::new(1.0, 1.0, 1.0).normalize(),
        Vec3::new(-1.0, 0.5, 0.3).normalize(),
    ];
    for dir in &directions {
        let uv   = direction_to_equirect_uv(*dir);
        let back = equirect_uv_to_direction(uv).normalize();
        let dot  = dir.dot(back).clamp(-1.0, 1.0);
        let angle = dot.acos();
        assert!(
            angle < 0.01,
            "UV roundtrip angle error = {angle:.4} rad for dir={dir}"
        );
    }
}

#[test]
fn test_equirect_uv_specific() {
    // +X direction: azimuth = 0°, elevation = equatorial → u≈0.5, v≈0.5
    let uv_px = direction_to_equirect_uv(Vec3::X);
    assert!(approx_eq(uv_px.x, 0.5, 0.01), "u={}", uv_px.x);
    assert!(approx_eq(uv_px.y, 0.5, 0.01), "v={}", uv_px.y);

    // +Y = north pole → v≈0.0
    let uv_py = direction_to_equirect_uv(Vec3::Y);
    assert!(approx_eq(uv_py.y, 0.0, 0.01), "v(north pole)={}", uv_py.y);

    // -Y = south pole → v≈1.0
    let uv_ny = direction_to_equirect_uv(-Vec3::Y);
    assert!(approx_eq(uv_ny.y, 1.0, 0.01), "v(south pole)={}", uv_ny.y);
}

#[test]
fn test_polar_edge_cases() {
    // North pole
    let sph_np = cartesian_to_spherical(Vec3::new(0.0, 1.0, 0.0));
    assert!(approx_eq(sph_np.r, 1.0, 1e-6));
    assert!(approx_eq(sph_np.theta, 0.0, 1e-4));

    // Origin — must not produce NaN
    let sph_origin = cartesian_to_spherical(Vec3::ZERO);
    assert!(sph_origin.r.is_finite(), "r is NaN at origin");
    assert!(sph_origin.theta.is_finite(), "theta is NaN at origin");

    // Zero-radius spherical → zero Cartesian
    let cart = spherical_to_cartesian(SphericalCoord { r: 0.0, theta: 0.0, phi: 0.0 });
    assert!(vec3_approx_eq(cart, Vec3::ZERO, 1e-9));
}