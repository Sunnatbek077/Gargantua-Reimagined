// ============================================================
// FILE: crates/gargantua-bake/src/lib.rs
// LINES: ~55
// CATEGORY: Bake — Crate entry point
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Public interface of the gargantua-bake crate.
//   Re-exports all sub-modules and the top-level BakeParams struct.
//   The bake crate runs offline to produce pre-computed LUT assets
//   (geodesic LUT, blackbody LUT, Doppler LUT, noise textures,
//   starmap SH coefficients) that the render crate loads at startup.
//
// CONTENTS (~55 lines):
//   #![deny(unsafe_code)]
//
//   pub mod cache;
//   pub mod errors;
//   pub mod geodesic;
//   pub mod irradiance;
//   pub mod noise;
//   pub mod scheduler;
//   pub mod spectrum;
//
//   pub use errors::{BakeError, BakeResult};
//   pub use scheduler::{BakeParams, BakeScheduler};
//
// USES (imports from):
//   All sub-modules above
//
// USED BY:
//   crates/gargantua-app/src/lib.rs
//     → BakeScheduler::run(params) triggered by AppEvent::StartBake
//   crates/gargantua-ui/src/menu/tabs/bake_tab.rs
//     → BakeParams struct for UI controls
//
// NOTE FOR AI:
//   The bake crate is separate from the render crate intentionally:
//   baking is slow offline work (minutes), rendering is real-time.
//   Baked assets are saved to assets/baked/ as .exr and .bin files.
//   The render crate loads these assets at startup — it never runs
//   the bake pipeline itself.
// ============================================================