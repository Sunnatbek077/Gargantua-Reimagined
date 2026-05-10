// =============================================================================
// crates/gargantua-render/src/lib.rs
// =============================================================================
//
// PURPOSE:
//   Crate root for gargantua-render. Declares all public modules and applies
//   crate-wide lint configuration. gargantua-render owns all GPU render and
//   compute pass implementations, post-processing, HDR output, and shader
//   hot-reload. It depends on gargantua-core and gargantua-physics.
//
// SIZE: ~80 lines
//
// CRATE-WIDE LINTS:
//   #![deny(clippy::all)]
//   #![deny(clippy::pedantic)]
//   #![allow(clippy::module_name_repetitions)]
//   #![allow(clippy::cast_precision_loss)]
//   #![allow(clippy::too_many_arguments)]
//     — pipeline constructors legitimately take many args (device, shader,
//       scene_bg, tex_bg, output, params, preset, ...)
//
// PUBLIC MODULES:
//
//   pub mod bindgroups
//     — bindgroups::scene::SceneBindGroup      — group(0) SceneUniforms
//     — bindgroups::textures::TexturesBindGroup — group(1) baked textures
//
//   pub mod pipelines
//     — pipelines::ray_march::RayMarchPass     — main photon ray marcher
//     — pipelines::geodesic_gpu::GeodesicGpuPass — GPU RK4 geodesic integrator
//     — pipelines::accretion::AccretionPass    — Novikov-Thorne disk emission
//     — pipelines::lensing::LensingPass        — gravitational lensing
//     — pipelines::starfield::StarfieldPass    — background star field
//
//   pub mod postfx
//     — postfx::taa::TaaPass                  — temporal anti-aliasing
//     — postfx::bloom::BloomPass              — dual Kawase bloom
//     — postfx::chromatic::ChromaticPass      — chromatic aberration
//     — postfx::film_grain::FilmGrainPass     — blue noise film grain
//     — postfx::motion_blur::MotionBlurPass   — tile-based motion blur
//     — postfx::lens_flare::LensFlarePass     — bokeh + diffraction flare
//     — postfx::tonemap::TonemapPass          — ACES RRT/ODT + HDR encode
//
//   pub mod errors
//     — RenderError: unified error type for this crate
//
//   pub mod hdr
//     — HdrOutput: cross-platform HDR coordinator (EDR/HDR10/SDR)
//
//   pub mod shader_reload
//     — ShaderReloader: dev-only WGSL hot-reload via filesystem watcher
//
// RE-EXPORTS (convenience for gargantua-app):
//   pub use errors::RenderError;
//   pub use hdr::HdrOutput;
//   pub use pipelines::ray_march::RayMarchPass;
//   pub use postfx::tonemap::TonemapPass;
//
// DEPENDENCY GRAPH (within gargantua-render):
//
//   lib.rs
//   ├── errors.rs          ← gargantua_core::CoreError
//   ├── hdr.rs             ← errors, gargantua_core::platform::*/hdr/*
//   ├── shader_reload.rs   ← errors
//   ├── bindgroups/
//   │   ├── scene.rs       ← gargantua_core::{clock, errors}
//   │   └── textures.rs    ← gargantua_core::errors
//   ├── pipelines/
//   │   ├── ray_march.rs   ← bindgroups, gargantua_core::frame::pass, errors
//   │   ├── geodesic_gpu.rs ← bindgroups::scene, gargantua_physics, errors
//   │   ├── accretion.rs   ← bindgroups, errors
//   │   ├── lensing.rs     ← bindgroups, errors
//   │   └── starfield.rs   ← bindgroups, errors
//   └── postfx/
//       ├── taa.rs         ← bindgroups::scene, errors
//       ├── bloom.rs       ← bindgroups::scene, errors
//       ├── chromatic.rs   ← bindgroups::scene, errors
//       ├── film_grain.rs  ← bindgroups::scene, errors
//       ├── motion_blur.rs ← bindgroups::scene, errors
//       ├── lens_flare.rs  ← bindgroups::scene, errors
//       └── tonemap.rs     ← bindgroups::scene, hdr, errors
//
// EXTERNAL CRATE DEPENDENCIES:
//   gargantua-core    — GpuContext, FrameGraph, Pass, ResourcePool, CoreError
//   gargantua-physics — GeodesicParams source (KerrParams, ISCO calculations)
//   wgpu              — all GPU types
//   bytemuck          — Pod/Zeroable for uniform structs
//   glam              — Vec2/3/4, Mat3/4 for color math
//   #[cfg(debug_assertions)] notify — filesystem watcher for hot-reload
//
// NOTES FOR AI:
//   - gargantua-render does not own the event loop or window — that is
//     gargantua-core::App and gargantua-app.
//   - All pipelines receive their ShaderModule from ShaderReloader, which
//     loads from disk in dev and from embedded bytes (include_str!) in release.
//   - The FrameGraph in gargantua-core controls execution order. Pipelines
//     in this crate only implement Pass::record() — they do not submit work.
// =============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_arguments)]

pub mod bindgroups {
    pub mod scene;
    pub mod textures;
}

pub mod pipelines {
    pub mod accretion;
    pub mod geodesic_gpu;
    pub mod lensing;
    pub mod ray_march;
    pub mod starfield;
}

pub mod postfx {
    pub mod bloom;
    pub mod chromatic;
    pub mod film_grain;
    pub mod lens_flare;
    pub mod motion_blur;
    pub mod taa;
    pub mod tonemap;
}

pub mod errors;
pub mod hdr;
pub mod shader_reload;

// Convenience re-exports
pub use errors::RenderError;
pub use hdr::HdrOutput;
pub use pipelines::ray_march::RayMarchPass;
pub use postfx::tonemap::TonemapPass;