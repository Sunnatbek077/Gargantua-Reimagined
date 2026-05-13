// ============================================================
// FILE: examples/photon_orbit.rs
// LINES: ~180
// CATEGORY: Example — Visualize photon orbit around a Kerr black hole
// RUN: cargo run --example photon_orbit
// ============================================================
//
// PURPOSE:
//   CPU-only example that traces photon orbits at the photon sphere
//   radius (r_ph) for various spin values and prints orbit parameters.
//   Also traces a family of geodesics near r_ph to show the unstable
//   orbit separatrix. Outputs ASCII art spiral or CSV data.
//
// CONTENTS (~180 lines):
//   fn main()
//     // 1. For spin in [-0.9, -0.5, 0.0, 0.5, 0.9]:
//     //    let bh = KerrNewman::new(1.0, spin, 0.0).unwrap();
//     //    let r_ph = bh.photon_sphere();
//     //    let b_crit = critical_impact_param(&bh, r_ph);
//     //
//     //    println!("spin={:.1}: r_ph={:.4}M, b_crit={:.4}M", spin, r_ph, b_crit);
//     //
//     // 2. Trace 3 geodesics per spin:
//     //    b = b_crit * 0.99  → should be CAPTURED
//     //    b = b_crit         → should orbit at r_ph (unstable)
//     //    b = b_crit * 1.01  → should ESCAPE
//     //
//     //    for (label, b_factor) in [("captured",0.99), ("orbit",1.00), ("escaped",1.01)] {
//     //        let path = integrator.trace(50.0, PI/2.0, 0.0, b_crit * b_factor)?;
//     //        let final_r = path.last().unwrap()[1];
//     //        let result = if final_r < r_ph * 1.1 { "CAPTURED" } else { "ESCAPED" };
//     //        println!("  b={:.4} ({}) → {} steps, r_final={:.4}, {}", ...);
//     //    }
//     //
//     // 3. Print Lense-Thirring frame dragging at r_ph:
//     //    let omega = frame_drag_angular_velocity(r_ph, PI/2.0, 1.0, spin);
//     //    println!("  Ω_FD at r_ph = {:.6} rad/M", omega);
//     //
//     // 4. Optional --csv flag: output all path coordinates as CSV
//     //    for (r, theta, phi) in path: println!("{},{},{}", r, theta, phi);
//
//   // Compute critical impact parameter for given metric and r_ph
//   fn critical_impact_param(bh: &KerrNewman, r_ph: f64) -> f64
//     // b_crit = r_ph² / sqrt(r_ph² - 2M*r_ph + a²)  (equatorial, approx)
//     // More accurate: numerical solve for b where dr/dλ → 0 at r_ph
//
// USES (imports from):
//   gargantua_physics::metric::kerr::KerrNewman
//   gargantua_physics::metric::mod::MetricTensor
//   gargantua_physics::geodesic::rk4::Rk4Integrator
//   gargantua_physics::effects::frame_dragging::frame_drag_angular_velocity
//   std::f64::consts::PI
//
// USED BY:
//   Physics documentation (docs/photon_sphere.md)
//   Understanding the geodesic LUT bake parameters (b_crit is key)
//
// NOTE FOR AI:
//   r_ph is UNSTABLE: photons there orbit forever in theory but
//   numerical errors cause them to eventually escape or be captured.
//   The b=b_crit trace may take many more steps than b=b_crit*0.99.
//   Use max_steps=100_000 for the orbit trace (not the default 10_000).
//   --csv mode: useful for feeding into Python/matplotlib for visualization.
//   Expected output for spin=0: r_ph=3.000M, b_crit≈5.196M (= 3√3 M).
// ============================================================