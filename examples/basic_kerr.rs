// ============================================================
// FILE: examples/basic_kerr.rs
// LINES: ~160
// CATEGORY: Example — Minimal Kerr metric usage demonstration
// RUN: cargo run --example basic_kerr
// ============================================================
//
// PURPOSE:
//   Minimal example showing how to use the gargantua-physics crate:
//   creates a Kerr black hole, prints metric components, ISCO,
//   photon sphere, and traces a single geodesic to show the API.
//   Intended as a "getting started" reference for new contributors.
//
// CONTENTS (~160 lines):
//   fn main()
//     // 1. Create a Kerr black hole (M87* parameters):
//     //    let bh = KerrNewman::from_solar_masses(6.5e9, 0.9).unwrap();
//     //    println!("M = {:.2e} M☉, a = {}", 6.5e9, 0.9);
//     //    println!("Event horizon: r_+ = {:.4} M", bh.event_horizon());
//     //    println!("ISCO:          r_I  = {:.4} M", bh.isco_radius());
//     //    println!("Photon sphere: r_ph = {:.4} M", bh.photon_sphere());
//     //    println!("Ergosphere (θ=π/2): r_erg = {:.4} M", bh.ergosphere(PI/2.0));
//     //
//     // 2. Print metric components at r=10M, θ=π/2:
//     //    let g = bh.g_mu_nu(10.0, PI/2.0);
//     //    println!("g_tt = {:.6}", g[0][0]);
//     //    println!("g_tφ = {:.6} (frame dragging)", g[0][3]);
//     //    println!("g_rr = {:.6}", g[1][1]);
//     //
//     // 3. Trace a single geodesic with b = 6.0 (captured by BH):
//     //    let integrator = Rk4Integrator::new(&bh, 0.1);
//     //    let path = integrator.trace(20.0, PI/2.0, 0.0, 6.0).unwrap();
//     //    println!("Geodesic steps: {}", path.len());
//     //    println!("Final r: {:.4} M", path.last().unwrap()[1]);
//     //    let term = if path.last().unwrap()[1] < bh.event_horizon() * 1.1
//     //        { "CAPTURED" } else { "ESCAPED" };
//     //    println!("Result: {}", term);
//     //
//     // 4. Print ISCO properties:
//     //    let isco = compute_isco_properties(1.0, 0.9).unwrap();
//     //    println!("Binding energy η = {:.2}%", isco.binding_energy * 100.0);
//     //    println!("Orbital frequency = {:.4e} Hz", isco.orbital_freq);
//
// USES (imports from):
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::mod::MetricTensor
//   gargantua_physics::geodesic::rk4::Rk4Integrator
//   gargantua_physics::accretion::isco::compute_isco_properties
//   std::f64::consts::PI
//
// USED BY:
//   New contributors — first file to read when learning the physics API
//   Documentation (linked in README.md)
//
// NOTE FOR AI:
//   This example must compile and run with zero external dependencies
//   beyond gargantua-physics. No wgpu, no egui, no window needed.
//   Keep output human-readable — this is a demo/tutorial example.
//   All printed values should match known M87* reference values:
//     r_+ ≈ 1.436M, r_ISCO ≈ 2.321M, r_ph ≈ 1.968M (for a=0.9)
// ============================================================