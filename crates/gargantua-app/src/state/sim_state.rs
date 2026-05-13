// =============================================================================
// FILE: crates/gargantua-app/src/state/sim_state.rs
// CRATE: gargantua-app
// LINES: ~260
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   The canonical, serialisable state of the entire physics simulation.
//   SimState is the single source of truth for black hole parameters,
//   accretion disk settings, camera position, and playback time.
//   It is serialised to JSON for URL sharing and undo/redo snapshots.
//
// WHAT THIS FILE CONTAINS:
//   - `#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]`
//     `pub struct SimState`:
//
//       --- Black hole physics ---
//       mass:              f64   — geometric mass M (solar masses in UI)
//       spin:              f64   — dimensionless spin a/M ∈ [0, 0.9999]
//       charge:            f64   — dimensionless charge Q/M ∈ [0, 1)
//
//       --- Accretion disk ---
//       accretion_rate:    f64   — Eddington fraction [0, 1]
//       disk_inner_radius: f64   — overrides ISCO if > 0.0 (units of M)
//       disk_outer_radius: f64   — outer edge in units of M (default: 50 M)
//       disk_inclination:  f64   — tilt angle in degrees [0, 90]
//       beta_mag:          f64   — MHD plasma beta parameter
//       jet_enabled:       bool  — whether the relativistic jet is rendered
//
//       --- Render settings ---
//       quality_level:     QualityLevel
//       spp:               u32
//       render_scale:      f32
//
//       --- Camera ---
//       camera_position:   [f64; 3]   — Boyer-Lindquist (r, θ, φ)
//       camera_direction:  [f64; 3]   — unit vector in BL coordinates
//       camera_fov_deg:    f32
//       camera_mode:       CameraMode — Free, Satellite, Plunge, Gravity, Path
//
//       --- Playback ---
//       simulation_time:   f64   — accumulated physics time in geometric units
//       time_scale:        f64   — 1.0 = real-time, 0.0 = paused
//
//   - `impl SimState`:
//       `pub fn default() -> Self`
//             Returns sensible defaults:
//               mass = 10 solar masses, spin = 0.9, charge = 0.0
//               accretion_rate = 0.1, quality = High
//               camera at r = 50M, equatorial plane, looking toward BH
//       `pub fn validate(&self) -> AppResult<()>`
//             Checks all fields are within valid physical bounds.
//             Returns AppError::Physics if spin >= 1.0, mass <= 0, etc.
//       `pub fn to_kerr_metric(&self) -> KerrMetric`
//             Converts mass + spin + charge to a KerrMetric struct for physics.
//       `pub fn to_gpu_uniforms(&self) -> PhysicsUniforms`
//             Packs relevant fields into a bytemuck::Pod struct for GPU upload.
//
// OUTBOUND DEPENDENCIES:
//   - gargantua_physics::metric::kerr → KerrMetric
//   - gargantua_core::quality         → QualityLevel
//   - gargantua_camera::modes         → CameraMode enum
//   - render/bindgroups/physics.rs    → PhysicsUniforms (GPU struct)
//   - serde (external)                → Serialize, Deserialize
//   - bytemuck (external)             → Pod for GPU upload
//   - errors.rs                       → AppResult
//
// INBOUND:
//   - systems/physics_sync.rs         → reads SimState each frame
//   - state/undo.rs                   → snapshots SimState for undo history
//   - state/url_serde.rs              → serialises SimState to/from URL
//   - ui/menu/tabs/physics_tab.rs     → reads/writes mass, spin, charge
//   - ui/menu/tabs/accretion_tab.rs   → reads/writes disk params
//   - plugin/scripting.rs             → Lua API writes spin/mass
//   - systems/replay.rs               → restores SimState from replay
//
// NOTES:
//   - SimState contains only plain data (no GPU resources, no Arc, no Mutex).
//     This makes it trivially Clone and Serialize without special handling.
//   - The Arc<RwLock<SimState>> wrapper lives in PluginContext and App;
//     this file only defines the data struct itself.
// =============================================================================
