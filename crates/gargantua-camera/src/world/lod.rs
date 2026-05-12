// =============================================================================
// crates/gargantua-camera/src/world/lod.rs
// =============================================================================
//
// PURPOSE:
//   Defines the LodLevel enum and per-level render quality parameters.
//   LOD (Level of Detail) controls the density of ray march samples in the
//   accretion disk, the starmap mip level, and the number of geodesic
//   integration steps for distant scene regions.
//
//   Gargantua does not render polygonal geometry, so LOD here refers to
//   volumetric sample density and shader computation depth — not mesh LOD.
//
// SIZE: ~140 lines
//
// DEPENDENCIES:
//   External: none
//
// CALLED BY:
//   - crate::world::chunk_manager::ChunkManager::lod_for_position()
//   - crates/gargantua-render/src/pipelines::accretion.rs
//       — uses LodLevel to scale AccretionParams.r_outer and sample density
//   - crates/gargantua-render/src/pipelines::starfield.rs
//       — uses LodLevel to select starmap mip level
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//   pub enum LodLevel {
//     Minimal = 0,   // max distance (8+ chunks) — coarse
//     Low     = 1,
//     Medium  = 2,
//     High    = 3,
//     Ultra   = 4,   // camera chunk — full quality
//   }
//
//   pub struct LodParams {
//     pub disk_step_scale:   f32,   // multiplier for disk radial sample count
//     pub max_geodesic_steps: u32,  // max RK4 steps for this LOD region
//     pub starmap_mip_bias:  f32,   // mip level bias for starmap sampling
//     pub disk_r_outer_scale: f32,  // scale factor for accretion disk outer radius
//   }
//
// PUBLIC FUNCTIONS:
//
//   impl LodLevel:
//
//     pub fn params(self) -> LodParams
//       — returns per-level parameters:
//
//         Ultra   (camera chunk):
//           disk_step_scale = 1.0
//           max_geodesic_steps = 512  (from QualityPreset)
//           starmap_mip_bias = 0.0    (highest mip = sharpest)
//           disk_r_outer_scale = 1.0  (full outer radius)
//
//         High    (1 chunk away):
//           disk_step_scale = 0.75
//           max_geodesic_steps = 384
//           starmap_mip_bias = 0.5
//           disk_r_outer_scale = 0.9
//
//         Medium  (2-3 chunks):
//           disk_step_scale = 0.5
//           max_geodesic_steps = 256
//           starmap_mip_bias = 1.0
//           disk_r_outer_scale = 0.75
//
//         Low     (4-7 chunks):
//           disk_step_scale = 0.25
//           max_geodesic_steps = 128
//           starmap_mip_bias = 2.0
//           disk_r_outer_scale = 0.5
//
//         Minimal (8+ chunks):
//           disk_step_scale = 0.1
//           max_geodesic_steps = 64
//           starmap_mip_bias = 3.0
//           disk_r_outer_scale = 0.25
//
//     pub fn is_full_quality(self) -> bool
//       — returns true if self == Ultra.
//
//     pub fn mip_level(self) -> u32
//       — returns starmap_mip_bias as u32 (rounded).
//       — used by starfield.rs for textureSampleLevel() mip argument.
//
// NOTES FOR AI:
//   - LodLevel is derived from ChunkManager::lod_for_position() based on
//     the camera chunk distance. It is NOT derived from GPU timing like
//     AdaptiveQuality — it is purely spatial.
//   - disk_step_scale multiplies the base sample count from AccretionParams.
//     Ultra = 1.0 means full quality; Minimal = 0.1 means 10% of samples.
//   - In practice, LOD only kicks in for free_flight at large distances.
//     At typical viewing distances (5-50M), only Ultra or High are used.
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LodLevel {
    Minimal = 0,
    Low     = 1,
    Medium  = 2,
    High    = 3,
    Ultra   = 4,
}

pub struct LodParams {
    pub disk_step_scale:    f32,
    pub max_geodesic_steps: u32,
    pub starmap_mip_bias:   f32,
    pub disk_r_outer_scale: f32,
}

impl LodLevel {
    pub fn params(self) -> LodParams {
        match self {
            LodLevel::Ultra   => LodParams { disk_step_scale: 1.00, max_geodesic_steps: 512, starmap_mip_bias: 0.0, disk_r_outer_scale: 1.00 },
            LodLevel::High    => LodParams { disk_step_scale: 0.75, max_geodesic_steps: 384, starmap_mip_bias: 0.5, disk_r_outer_scale: 0.90 },
            LodLevel::Medium  => LodParams { disk_step_scale: 0.50, max_geodesic_steps: 256, starmap_mip_bias: 1.0, disk_r_outer_scale: 0.75 },
            LodLevel::Low     => LodParams { disk_step_scale: 0.25, max_geodesic_steps: 128, starmap_mip_bias: 2.0, disk_r_outer_scale: 0.50 },
            LodLevel::Minimal => LodParams { disk_step_scale: 0.10, max_geodesic_steps: 64,  starmap_mip_bias: 3.0, disk_r_outer_scale: 0.25 },
        }
    }

    pub fn is_full_quality(self) -> bool { self == LodLevel::Ultra }

    pub fn mip_level(self) -> u32 { self.params().starmap_mip_bias as u32 }
}