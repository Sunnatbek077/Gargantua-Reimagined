// ============================================================
// FILE: crates/gargantua-bake/src/scheduler.rs
// LINES: ~280
// CATEGORY: Bake — Main bake orchestrator and progress reporter
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   BakeScheduler owns the entire bake pipeline. It runs all LUT
//   bake steps in order, checks the cache to skip fresh LUTs,
//   reports progress via a channel, and handles cancellation.
//   Runs on a background thread (spawned by gargantua-app).
//
// CONTENTS (~280 lines):
//   // All user-configurable bake parameters (from BakeTab UI)
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   pub struct BakeParams {
//       pub assets_dir:         std::path::PathBuf,
//       pub geo_spin_steps:     u32,    // geodesic LUT spin axis [8–256]
//       pub geo_impact_steps:   u32,    // geodesic LUT impact param axis [64–4096]
//       pub geo_rk4_steps:      u32,    // max RK4 steps per geodesic [1000–50000]
//       pub blackbody_lut_size: u32,    // temperature LUT points [256–4096]
//       pub doppler_n_beta:     u32,    // Doppler LUT β axis [64–512]
//       pub doppler_n_lambda:   u32,    // Doppler LUT λ axis [64–512]
//       pub blue_noise_size:    u32,    // blue noise texture resolution [64–1024]
//       pub curl_noise_size:    u32,    // curl noise 3D texture resolution [32–256]
//       pub starmap_sh_order:   u32,    // spherical harmonics order [3–9]
//       pub force_rebake:       bool,   // ignore cache, bake everything
//   }
//
//   // Progress event sent through mpsc channel to the UI
//   #[derive(Debug, Clone)]
//   pub struct BakeProgressEvent {
//       pub step:     LutKind,      // which LUT is being baked
//       pub progress: f32,          // 0.0–1.0 within this step
//       pub overall:  f32,          // 0.0–1.0 across all steps
//       pub message:  String,       // e.g. "Baking geodesic LUT (spin 3/8)"
//   }
//
//   pub struct BakeScheduler {
//       params:   BakeParams,
//       cache:    BakeCache,
//       cancel_rx:std::sync::Arc<std::sync::atomic::AtomicBool>,
//   }
//
//   impl BakeScheduler {
//       pub fn new(
//           params: BakeParams,
//           cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
//       ) -> Self
//
//       // Run all bake steps. Sends progress via tx channel.
//       // Returns Ok(()) when done, Err(BakeError::Cancelled) if cancelled.
//       pub fn run(
//           mut self,
//           tx: std::sync::mpsc::Sender<BakeProgressEvent>,
//       ) -> BakeResult<()>
//         // Step 1: Geodesic LUT   → geodesic::let_baker::bake(...)
//         // Step 2: Blackbody LUT  → spectrum::blackbody::bake(...)
//         // Step 3: Doppler LUT    → spectrum::doppler_lut::bake(...)
//         // Step 4: Blue noise     → noise::blue_noise::bake(...)
//         // Step 5: Curl noise     → noise::curl_noise::bake(...)
//         // Step 6: Starmap SH     → irradiance::starmap::bake(...)
//         // Each step: check cancel_flag, check cache, bake, update cache
//
//       // Check if cancelled — call at start of each inner loop iteration
//       fn check_cancel(&self) -> BakeResult<()>
//         // if cancel_flag.load(Ordering::Relaxed) → Err(BakeError::Cancelled)
//
//       fn send_progress(
//           tx: &std::sync::mpsc::Sender<BakeProgressEvent>,
//           step: LutKind, progress: f32, overall: f32, msg: &str,
//       )
//   }
//
// USES (imports from):
//   crate::cache::{BakeCache, LutKind}
//   crate::errors::{BakeResult, BakeError}
//   crate::geodesic::let_baker
//   crate::spectrum::{blackbody, doppler_lut}
//   crate::noise::{blue_noise, curl_noise}
//   crate::irradiance::starmap
//   std::sync::{mpsc, Arc, atomic::AtomicBool}
//   serde
//
// USED BY:
//   crates/gargantua-app/src/lib.rs
//     → spawns std::thread::spawn(|| scheduler.run(tx))
//     → reads BakeProgressEvent from rx and sends to UI
//
// NOTE FOR AI:
//   BakeScheduler::run() runs on a background thread — NOT the main thread.
//   cancel_flag is an Arc<AtomicBool> shared between UI and scheduler.
//   UI sets it to true when user clicks "Cancel".
//   Progress channel: std::sync::mpsc::channel() (bounded would be ideal
//   but unbounded is fine since progress events are sent at most ~60/sec).
//   Step weights for overall progress (sum = 1.0):
//     Geodesic=0.50, Blackbody=0.05, Doppler=0.10,
//     BlueNoise=0.10, CurlNoise=0.15, Starmap=0.10
// ============================================================