// =============================================================================
// crates/gargantua-camera/src/world/chunk_manager.rs
// =============================================================================
//
// PURPOSE:
//   Manages spatial chunking of the scene around the camera for Level of
//   Detail (LOD) and floating-point precision management. When the camera
//   is far from the black hole (r >> 100M), the scene is divided into
//   chunks; nearby chunks get higher LOD, distant ones get lower LOD.
//
//   Also coordinates with floating_origin.rs: when the camera moves far from
//   the world origin, the chunk manager triggers an origin shift to keep
//   the camera near (0,0,0) in camera-relative space.
//
//   In Gargantua's current scope: the scene is a single black hole with no
//   extended geometry, so chunk management primarily handles the LOD of the
//   accretion disk sampling density and the starmap mip level selection.
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::world::floating_origin::FloatingOrigin
//     - crate::world::lod::LodLevel
//     - crate::errors::CameraError
//   External:
//     - glam::Vec3
//
// CALLED BY:
//   - crate::world_camera::WorldCamera::update()
//       — calls ChunkManager::update(camera_pos) each frame
//   - crates/gargantua-render/src/pipelines/accretion.rs
//       — queries lod_for_position() to set AccretionParams.r_outer
//
// PUBLIC TYPES:
//
//   pub struct ChunkManager {
//     origin:       FloatingOrigin,   // tracks world origin offset
//     camera_chunk: ChunkCoord,       // which chunk the camera is in
//     lod_table:    Vec<LodLevel>,    // LOD levels indexed by distance band
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//   pub struct ChunkCoord {
//     pub x: i32,
//     pub y: i32,
//     pub z: i32,
//   }
//
//   impl ChunkCoord:
//     pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self
//       — divides world position by chunk_size and floors to integer.
//     pub fn to_world_center(self, chunk_size: f32) -> Vec3
//       — returns center of this chunk in world space.
//     pub fn distance_sq_chunks(self, other: ChunkCoord) -> i32
//       — Chebyshev distance squared between two chunks.
//
// PUBLIC FUNCTIONS:
//
//   pub fn new() -> Self
//     — creates with origin at (0,0,0), chunk_size = 10.0 (in M units).
//     — LOD table: 5 levels (chunk distances 0,1,2,4,8+ from camera chunk).
//
//   pub fn update(&mut self, camera_pos: Vec3) -> Option<Vec3>
//     — updates camera_chunk from camera_pos.
//     — if camera moved to a new chunk: recomputes LOD assignments.
//     — checks if origin should shift (origin.should_shift(camera_pos)):
//         If true: returns Some(shift_vector) so App can re-center scene.
//         If false: returns None.
//
//   pub fn lod_for_position(&self, world_pos: Vec3) -> LodLevel
//     — determines LOD level for a given world position.
//     — used by accretion.rs to decide disk sampling resolution.
//
//   pub fn origin_offset(&self) -> Vec3
//     — returns the cumulative origin offset applied so far.
//     — used by WorldCamera to translate render positions.
// =============================================================================

use glam::Vec3;
use crate::world::{floating_origin::FloatingOrigin, lod::LodLevel};
use crate::errors::CameraError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (pos.x / chunk_size).floor() as i32,
            y: (pos.y / chunk_size).floor() as i32,
            z: (pos.z / chunk_size).floor() as i32,
        }
    }

    pub fn to_world_center(self, chunk_size: f32) -> Vec3 {
        Vec3::new(
            (self.x as f32 + 0.5) * chunk_size,
            (self.y as f32 + 0.5) * chunk_size,
            (self.z as f32 + 0.5) * chunk_size,
        )
    }

    pub fn distance_sq_chunks(self, other: ChunkCoord) -> i32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        dx.max(dy).max(dz) // Chebyshev distance
    }
}

pub struct ChunkManager {
    origin:       FloatingOrigin,
    camera_chunk: ChunkCoord,
    chunk_size:   f32,
    lod_table:    Vec<LodLevel>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            origin:       FloatingOrigin::new(),
            camera_chunk: ChunkCoord { x: 0, y: 0, z: 0 },
            chunk_size:   10.0,
            lod_table:    vec![
                LodLevel::Ultra,  // chunk distance 0 — camera chunk
                LodLevel::High,   // chunk distance 1
                LodLevel::Medium, // chunk distance 2
                LodLevel::Low,    // chunk distance 4
                LodLevel::Minimal,// chunk distance 8+
            ],
        }
    }

    pub fn update(&mut self, camera_pos: Vec3) -> Option<Vec3> {
        let new_chunk = ChunkCoord::from_world_pos(camera_pos, self.chunk_size);
        self.camera_chunk = new_chunk;
        if self.origin.should_shift(camera_pos) {
            let shift = self.origin.apply_shift(camera_pos);
            Some(shift)
        } else {
            None
        }
    }

    pub fn lod_for_position(&self, world_pos: Vec3) -> LodLevel {
        let pos_chunk = ChunkCoord::from_world_pos(world_pos, self.chunk_size);
        let dist      = self.camera_chunk.distance_sq_chunks(pos_chunk) as usize;
        let idx = match dist {
            0     => 0,
            1     => 1,
            2..=3 => 2,
            4..=7 => 3,
            _     => 4,
        };
        self.lod_table[idx.min(self.lod_table.len() - 1)]
    }

    pub fn origin_offset(&self) -> Vec3 {
        self.origin.current_offset()
    }
}

impl Default for ChunkManager {
    fn default() -> Self { Self::new() }
}