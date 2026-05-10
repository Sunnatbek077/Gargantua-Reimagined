// ============================================================
// FILE: crates/gargantua-physics/src/metric/kerr.rs
// LINES: ~480
// CATEGORY: Physics — Kerr-Newman spacetime metric (full analytic)
// PLATFORM: cross-platform (Mac + Windows + WASM)
// ============================================================
//
// PURPOSE:
//   Full analytic Kerr-Newman metric in Boyer-Lindquist coordinates.
//   Central physics primitive — nearly all other modules depend on this.
//   Includes analytic Christoffel symbols (~200 lines) and
//   a GPU-compatible flat struct for wgpu uniform buffer upload.
//
// CONTENTS (~480 lines):
//   pub struct KerrNewman {
//       pub mass:   f64,   // M in G=c=1 units
//       pub spin:   f64,   // a = J/M dimensionless, |a| < 1
//       pub charge: f64,   // Q electric charge (usually 0 for astrophysical BH)
//   }
//
//   impl KerrNewman {
//       pub fn new(mass: f64, spin: f64, charge: f64) -> PhysicsResult<Self>
//         // validates: mass > 0, |spin| < 1, charge² ≤ mass² + spin²
//
//       pub fn from_solar_masses(m_sun: f64, spin: f64) -> PhysicsResult<Self>
//         // converts M_sun → G=c=1 via units::solar_mass_to_geom()
//
//       fn sigma(&self, r: f64, theta: f64) -> f64
//         // Σ = r² + a²cos²θ
//
//       fn delta(&self, r: f64) -> f64
//         // Δ = r² − 2Mr + a² + Q²
//
//       pub fn to_gpu_params(&self) -> KerrGpuParams
//   }
//
//   impl MetricTensor for KerrNewman {
//       fn g_mu_nu(r, theta) → [[f64;4];4]
//         // g_tt  = -(1 - 2Mr/Σ)
//         // g_rr  = Σ/Δ
//         // g_θθ  = Σ
//         // g_φφ  = (r²+a²+2Mra²sin²θ/Σ) sin²θ
//         // g_tφ  = g_φt = -2Mra sin²θ / Σ   ← frame dragging cross-term
//
//       fn g_mu_nu_inv(r, theta) → [[f64;4];4]
//         // Analytic contravariant inverse
//
//       fn christoffel(r, theta) → [[[f64;4];4];4]
//         // ANALYTIC override — ~200 lines of symbolic derivatives
//         // Non-zero components: Γ^t_tr, Γ^t_tθ, Γ^t_φr, Γ^t_φθ,
//         //   Γ^r_tt, Γ^r_rr, Γ^r_θθ, Γ^r_φφ, Γ^r_tφ,
//         //   Γ^θ_rθ, Γ^θ_φφ, Γ^θ_tφ,
//         //   Γ^φ_tr, Γ^φ_tθ, Γ^φ_rφ, Γ^φ_θφ
//
//       fn isco_radius()    → f64   // Bardeen (1972): r_ISCO = f(M, a)
//       fn photon_sphere()  → f64   // r_ph = f(M, a)
//       fn event_horizon()  → f64   // r_+ = M + sqrt(M²−a²−Q²)
//       fn ergosphere(θ)    → f64   // r_erg = M + sqrt(M²−a²cos²θ−Q²)
//   }
//
//   // GPU flat struct — must byte-match WGSL layout:
//   #[repr(C)]
//   pub struct KerrGpuParams {
//       pub mass:   f32,
//       pub spin:   f32,
//       pub charge: f32,
//       pub r_s:    f32,   // = 2M
//       pub r_plus: f32,   // event horizon
//       pub r_isco: f32,   // ISCO radius
//       pub r_ph:   f32,   // photon sphere
//       _pad:       f32,   // alignment to 32 bytes
//   }
//   impl From<&KerrNewman> for KerrGpuParams
//
// USES (imports from):
//   super::mod.rs    → MetricTensor trait
//   crate::units     → solar_mass_to_geom, schwarzschild_radius
//   crate::errors    → PhysicsError::InvalidSpin, NegativeMass
//
// USED BY:
//   geodesic/rk4.rs              → &KerrNewman passed to integrator
//   accretion/isco.rs            → isco_radius(), event_horizon()
//   accretion/novikov_thorne.rs  → temperature model
//   accretion/mhd.rs             → spin for BZ jet power, Ω_H = a/(2r_+)
//   effects/penrose.rs           → ergosphere()
//   effects/frame_dragging.rs    → g_tφ for Ω_FD
//   effects/redshift.rs          → g_tt, g_tφ, g_φφ
//   crates/gargantua-render/src/pipelines/geodesic_gpu.rs
//     → uploads KerrGpuParams to wgpu uniform buffer
//   crates/gargantua-render/src/bindgroups/physics.rs
//     → PhysicsParams bind group layout
//   crates/gargantua-ui/src/menu/tabs/physics_tab.rs
//     → reads mass, spin, charge from UI sliders
//
// NOTE FOR AI:
//   Boyer-Lindquist: (t, r, θ, φ), metric signature (−,+,+,+).
//   Σ = r² + a²cos²θ,   Δ = r² − 2Mr + a² + Q²
//   g_tφ ≠ 0 is the ONLY off-diagonal term — it IS frame dragging.
//   KerrGpuParams MUST stay #[repr(C)] and field-order must match
//   the WGSL struct in shaders/compute/geodesic_rk4.wgsl exactly:
//     struct KerrParams { mass: f32, spin: f32, charge: f32, r_s: f32, ... }
//   Analytic christoffel() is called 4× per RK4 step per photon per frame.
//   DO NOT replace it with numerical_christoffel() — 10× slowdown.
// ============================================================