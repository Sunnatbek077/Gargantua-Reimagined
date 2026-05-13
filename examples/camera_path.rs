// ============================================================
// FILE: examples/camera_path.rs
// LINES: ~200
// CATEGORY: Example — Define and export a camera animation path
// RUN: cargo run --example camera_path -- --output ./camera_path.json
// ============================================================
//
// PURPOSE:
//   Demonstrates how to programmatically define a camera animation
//   path around a black hole using keyframes, then export it to JSON
//   for use in the video export pipeline. Shows the camera animation
//   API without needing the full GUI.
//
// CONTENTS (~200 lines):
//   fn main()
//     // 1. Parse CLI args: --output <path>, --duration <seconds>,
//     //    --spin <a>, --mass <M_sun>
//     //
//     // 2. Define keyframes manually:
//     //    let keyframes = vec![
//     //        CameraKeyframe { time: 0.0,  r: 25.0, theta: PI/2.0, phi: 0.0,    fov: 60.0 },
//     //        CameraKeyframe { time: 5.0,  r: 10.0, theta: PI/3.0, phi: PI/2.0, fov: 50.0 },
//     //        CameraKeyframe { time: 10.0, r: 5.0,  theta: PI/2.0, phi: PI,     fov: 40.0 },
//     //        CameraKeyframe { time: 15.0, r: 15.0, theta: PI/4.0, phi: 3*PI/2, fov: 55.0 },
//     //        CameraKeyframe { time: 20.0, r: 25.0, theta: PI/2.0, phi: 2*PI,   fov: 60.0 },
//     //    ];
//     //
//     // 3. Interpolate and print the path at 24 fps:
//     //    for frame in 0..total_frames {
//     //        let t = frame as f32 / 24.0;
//     //        let cam = interpolate_keyframes(&keyframes, t);
//     //        println!("Frame {:04}: r={:.2}M θ={:.1}° φ={:.1}°",
//     //            frame, cam.r, cam.theta.to_degrees(), cam.phi.to_degrees());
//     //    }
//     //
//     // 4. Export to JSON:
//     //    let json = serde_json::to_string_pretty(&keyframes)?;
//     //    fs::write(&output_path, json)?;
//     //    println!("Saved {} keyframes to {}", keyframes.len(), output_path);
//
//   // Camera keyframe struct
//   #[derive(serde::Serialize, serde::Deserialize)]
//   struct CameraKeyframe {
//       time:  f32,    // seconds
//       r:     f64,    // Boyer-Lindquist r in M
//       theta: f64,    // polar angle (radians)
//       phi:   f64,    // azimuthal angle (radians)
//       fov:   f32,    // field of view (degrees)
//   }
//
//   // Hermite spline interpolation between keyframes
//   fn interpolate_keyframes(kfs: &[CameraKeyframe], t: f32) -> CameraKeyframe
//     // Finds bounding keyframes, applies cubic Hermite spline
//     // Handles phi wraparound (0 → 2π circular interpolation)
//
// USES (imports from):
//   serde, serde_json (external)  → JSON serialization
//   std::{fs, env, f64::consts::PI}
//
// USED BY:
//   Video export pipeline documentation (README_render.md)
//   Users who want to script camera paths programmatically
//
// NOTE FOR AI:
//   phi interpolation: use shortest-arc interpolation to avoid
//   spinning 360° when going from 350° to 10° (should go -20°, not +340°).
//   Hermite spline: C1 continuous (position and tangent match at keyframes).
//   Tangents estimated from finite differences of neighboring keyframes.
//   Output JSON is loaded by RenderTab's video export feature.
// ============================================================