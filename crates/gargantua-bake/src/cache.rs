// ============================================================
// FILE: crates/gargantua-bake/src/cache.rs
// LINES: ~220
// CATEGORY: Bake — Incremental bake cache (skip unchanged LUTs)
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Tracks which LUTs are up-to-date and which need re-baking.
//   Computes a hash of bake parameters; if the hash matches
//   the cached hash on disk, that LUT is skipped. Dramatically
//   speeds up re-bakes when only one parameter changed.
//
// CONTENTS (~220 lines):
//   pub struct BakeCache {
//       entries: HashMap<LutKind, CacheEntry>,
//       cache_file: std::path::PathBuf,  // assets/baked/.bake_cache.toml
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//   pub enum LutKind {
//       Geodesic,    // geodesic LUT (spin × impact param)
//       Blackbody,   // blackbody temperature → RGB LUT
//       Doppler,     // β × λ → shift factor LUT
//       BlueNoise,   // blue noise texture
//       CurlNoise,   // curl noise 3D texture
//       Starmap,     // starmap SH coefficient array
//   }
//
//   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//   struct CacheEntry {
//       param_hash: u64,         // xxHash of the BakeParams subset
//       baked_at:   String,      // ISO 8601 timestamp
//       file_path:  String,      // relative path to the baked asset
//       file_size:  u64,         // bytes (sanity check)
//   }
//
//   impl BakeCache {
//       pub fn load(assets_dir: &std::path::Path) -> Self
//         // Reads .bake_cache.toml, returns empty cache if missing
//
//       pub fn save(&self) -> BakeResult<()>
//         // Writes .bake_cache.toml to disk
//
//       // Returns true if this LUT is up-to-date (no re-bake needed)
//       pub fn is_fresh(&self, kind: LutKind, param_hash: u64) -> bool
//         // Checks: entry exists AND entry.param_hash == param_hash
//         //         AND file_path exists on disk AND file_size matches
//
//       // Mark a LUT as freshly baked
//       pub fn mark_baked(
//           &mut self, kind: LutKind,
//           param_hash: u64, file_path: &str, file_size: u64,
//       )
//
//       // Compute parameter hash for a given LutKind from BakeParams
//       pub fn param_hash(kind: LutKind, params: &BakeParams) -> u64
//         // Uses xxhash_rust::xxh64 on relevant params subset
//         // e.g. for Geodesic: hashes spin_steps, impact_steps, rk4_steps
//         //      for Blackbody: hashes blackbody_lut_size only
//
//       pub fn invalidate(&mut self, kind: LutKind)
//         // Force re-bake by removing the cache entry for this kind
//
//       pub fn invalidate_all(&mut self)
//   }
//
// USES (imports from):
//   crate::errors           → BakeResult
//   crate::scheduler::BakeParams
//   xxhash_rust::xxh64      → fast 64-bit hash
//   serde, toml             → cache file serialization
//   std::{collections::HashMap, path, fs}
//
// USED BY:
//   crate::scheduler::BakeScheduler::run()
//     → checks is_fresh() before each LUT bake step
//     → calls mark_baked() after each successful LUT write
//   crates/gargantua-bake/tests/cache.rs
//     → validates cache hit/miss logic
//
// NOTE FOR AI:
//   Cache file location: assets/baked/.bake_cache.toml
//   The leading dot makes it hidden on Unix (intentional).
//   file_size check: if the .exr file was manually deleted or truncated,
//   is_fresh() returns false even if hash matches.
//   xxhash_rust crate must be in Cargo.toml as a dependency.
// ============================================================