// ============================================================
// FILE: crates/gargantua-bake/tests/cache.rs
// LINES: ~220
// CATEGORY: Integration test — BakeCache hit/miss logic
// RUN: cargo test --package gargantua-bake --test cache
// ============================================================
//
// PURPOSE:
//   Validates BakeCache: correct cache hit/miss detection,
//   hash computation, file existence checks, invalidation,
//   and persistence across load/save cycles.
//
// TESTED FUNCTIONS (from crate::cache):
//   BakeCache::load()
//   BakeCache::save()
//   BakeCache::is_fresh()
//   BakeCache::mark_baked()
//   BakeCache::param_hash()
//   BakeCache::invalidate()
//   BakeCache::invalidate_all()
//
// SETUP:
//   fn default_params() -> BakeParams
//     // Returns BakeParams with standard default values
//
//   fn temp_assets_dir() -> tempfile::TempDir
//     // Creates temp directory simulating assets/baked/
//
// TEST CASES (~220 lines):
//
//   #[test]
//   fn test_empty_cache_always_miss()
//     // Fresh cache: is_fresh(any_kind, any_hash) == false
//     // No entries → everything needs baking
//
//   #[test]
//   fn test_cache_hit_after_mark_baked()
//     // Write a dummy file to temp dir
//     // cache.mark_baked(Geodesic, hash, path, size)
//     // cache.is_fresh(Geodesic, hash) == true
//
//   #[test]
//   fn test_cache_miss_wrong_hash()
//     // mark_baked with hash=100
//     // is_fresh(Geodesic, 999) == false  (different hash)
//
//   #[test]
//   fn test_cache_miss_file_deleted()
//     // mark_baked → delete the actual file on disk
//     // is_fresh() == false (file no longer exists)
//
//   #[test]
//   fn test_cache_miss_file_wrong_size()
//     // mark_baked with size=1000
//     // Write file with only 500 bytes
//     // is_fresh() == false (size mismatch)
//
//   #[test]
//   fn test_cache_persist_round_trip()
//     // mark_baked for Geodesic and Blackbody
//     // cache.save() → load new cache from same dir
//     // new_cache.is_fresh(Geodesic, same_hash) == true
//     // new_cache.is_fresh(Blackbody, same_hash) == true
//
//   #[test]
//   fn test_invalidate_single()
//     // mark_baked Geodesic and Blackbody
//     // cache.invalidate(Geodesic)
//     // is_fresh(Geodesic) == false
//     // is_fresh(Blackbody) == true  (untouched)
//
//   #[test]
//   fn test_invalidate_all()
//     // mark_baked all 6 LutKind variants
//     // cache.invalidate_all()
//     // All is_fresh() == false
//
//   #[test]
//   fn test_param_hash_geodesic_changes_with_spin_steps()
//     // params1.geo_spin_steps = 64
//     // params2.geo_spin_steps = 128
//     // param_hash(Geodesic, &params1) != param_hash(Geodesic, &params2)
//
//   #[test]
//   fn test_param_hash_geodesic_ignores_blackbody_params()
//     // params1 and params2 differ only in blackbody_lut_size
//     // param_hash(Geodesic, &params1) == param_hash(Geodesic, &params2)
//     // Geodesic hash should NOT depend on blackbody params
//
//   #[test]
//   fn test_param_hash_stable()
//     // Same params → same hash across multiple calls
//     // (deterministic hash, not random)
//
//   #[test]
//   fn test_corrupt_cache_file_handled()
//     // Write garbage bytes to .bake_cache.toml
//     // BakeCache::load() should not panic
//     // Returns empty cache (treats corrupt as missing)
//
// USES (imports from):
//   gargantua_bake::cache::{BakeCache, LutKind}
//   gargantua_bake::scheduler::BakeParams
//   tempfile::TempDir
//   std::fs
// ============================================================