// ============================================================
// FILE: crates/gargantua-physics/src/metric/mod.rs
// LINES: ~180
// CATEGORY: Physics — MetricTensor trait definition
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Defines the core MetricTensor trait that all spacetime
//   metrics must implement. Kerr and Schwarzschild are the
//   two concrete implementations in sub-modules.
//
// CONTENTS (~180 lines):
//   pub mod kerr;
//   pub mod schwarzschild;
//
//   pub trait MetricTensor: Send + Sync {
//       fn g_mu_nu(&self, r: f64, theta: f64) -> [[f64; 4]; 4];
//         // Covariant metric components g_μν at position (r, θ)
//
//       fn g_mu_nu_inv(&self, r: f64, theta: f64) -> [[f64; 4]; 4];
//         // Inverse metric g^μν — used in geodesic equation
//
//       fn christoffel(&self, r: f64, theta: f64) -> [[[f64; 4]; 4]; 4];
//         // Christoffel symbols Γ^λ_μν
//         // Default impl: 5-point finite-difference stencil
//         // Kerr overrides with analytic formula for performance
//
//       fn isco_radius(&self) -> f64;
//       fn photon_sphere(&self) -> f64;
//       fn event_horizon(&self) -> f64;
//       fn ergosphere(&self, theta: f64) -> f64;
//         // For Schwarzschild: returns event_horizon() (no ergosphere)
//   }
//
//   // Default Christoffel via 5-point stencil:
//   // Γ^λ_μν = (1/2) g^λσ (∂_μ g_νσ + ∂_ν g_μσ - ∂_σ g_μν)
//   fn numerical_christoffel<M: MetricTensor>(m: &M, r: f64, theta: f64) -> [[[f64;4];4];4]
//
// USES (imports from):
//   kerr.rs           — KerrNewman struct
//   schwarzschild.rs  — Schwarzschild struct
//
// USED BY:
//   geodesic/rk4.rs              → christoffel() each RK4 step
//   accretion/isco.rs            → isco_radius(), event_horizon()
//   effects/penrose.rs           → ergosphere()
//   effects/redshift.rs          → g_mu_nu() for g_tt, g_tφ, g_φφ
//   crates/gargantua-render/src/pipelines/geodesic_gpu.rs
//   crates/gargantua-ui/src/menu/tabs/physics_tab.rs
//
// NOTE FOR AI:
//   MetricTensor is object-safe — dyn MetricTensor compiles fine.
//   Always use f64 (not f32) in all CPU physics code.
//   GPU uses f32 via KerrGpuParams (defined in kerr.rs).
//   Provide analytic christoffel() override for any perf-critical metric.
//   numerical_christoffel() is fallback only — too slow per-frame.
// ============================================================