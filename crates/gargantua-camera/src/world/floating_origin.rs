// =============================================================================
// crates/gargantua-camera/src/world/floating_origin.rs
// =============================================================================
//
// PURPOSE:
//   Implements floating origin technique to maintain f32 precision when the
//   camera moves far from the world origin. In geometric units (M=1), the
//   black hole is at (0,0,0). f32 has ~7 decimal digits of precision; at
//   distances > 1000M, sub-meter precision requires doubles.
//
//   Solution: when the camera drifts more than SHIFT_THRESHOLD (default 500M)
//   from the world origin, shift the entire world so the camera is back near
//   (0,0,0). The cumulative offset is tracked so world positions can be
//   reconstructed if needed.
//
//   In practice, Gargantua's default camera distances range from 5M to 100M
//   from the black hole, so floating origin is rarely triggered. It exists
//   for free_flight mode where the camera can travel arbitrarily far.
//
// SIZE: ~140 lines
//
// DEPENDENCIES:
//   External:
//     - glam::Vec3
//
// CALLED BY:
//   - crate::world::chunk_manager::ChunkManager::update()
//   - crate::world_camera::WorldCamera::update()
//
// PUBLIC TYPES:
//
//   pub struct FloatingOrigin {
//     current_offset:   Vec3,    // cumulative world-space offset applied so far
//     shift_threshold:  f32,     // distance from origin that triggers a shift (default 500.0)
//     shift_count:      u32,     // number of shifts performed (for debugging)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new() -> Self
//     — creates with offset = (0,0,0), threshold = 500.0.
//
//   pub fn should_shift(&self, camera_pos: Vec3) -> bool
//     — returns true if camera_pos.length() > self.shift_threshold.
//
//   pub fn apply_shift(&mut self, camera_pos: Vec3) -> Vec3
//     — shift_amount = -camera_pos (move world so camera is at origin).
//     — self.current_offset += shift_amount.
//     — self.shift_count += 1.
//     — returns shift_amount so the caller can translate all world positions.
//     — After this call, camera_pos + shift_amount ≈ (0,0,0).
//     — NOTE: the black hole position must also be translated:
//         black_hole_pos += shift_amount
//         This is tracked in WorldCamera::black_hole_local_pos.
//
//   pub fn current_offset(&self) -> Vec3  { self.current_offset }
//   pub fn shift_count(&self)    -> u32   { self.shift_count    }
//
//   pub fn world_to_local(&self, world_pos: Vec3) -> Vec3
//     — converts an absolute world position to the current local frame:
//         local = world_pos + self.current_offset
//     — used to transform stored positions when an origin shift occurs.
//
//   pub fn local_to_world(&self, local_pos: Vec3) -> Vec3
//     — inverse: local → world:
//         world = local_pos - self.current_offset
//
// NOTES FOR AI:
//   - SHIFT_THRESHOLD of 500M is conservative. At 500M from a 1M black hole,
//     f32 still has ~4 digits of precision after the decimal — adequate.
//   - After apply_shift, ALL positions in the scene must be updated:
//       camera position, target position, black hole position, and any
//       stored path keyframe positions (gargantua-camera/src/path/keyframe.rs).
//   - The shift is applied in WORLD SPACE before any view matrix computation.
//     The view matrix is always computed in the shifted (local) frame.
//   - This is transparent to gargantua-render: the GPU always receives
//     positions relative to the current local origin.
// =============================================================================

use glam::Vec3;

pub struct FloatingOrigin {
    current_offset:  Vec3,
    shift_threshold: f32,
    shift_count:     u32,
}

impl FloatingOrigin {
    pub fn new() -> Self {
        Self {
            current_offset:  Vec3::ZERO,
            shift_threshold: 500.0,
            shift_count:     0,
        }
    }

    pub fn should_shift(&self, camera_pos: Vec3) -> bool {
        camera_pos.length() > self.shift_threshold
    }

    pub fn apply_shift(&mut self, camera_pos: Vec3) -> Vec3 {
        let shift = -camera_pos;
        self.current_offset += shift;
        self.shift_count    += 1;
        shift
    }

    pub fn current_offset(&self) -> Vec3  { self.current_offset  }
    pub fn shift_count(&self)    -> u32   { self.shift_count     }
    pub fn threshold(&self)      -> f32   { self.shift_threshold }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.shift_threshold = threshold.max(50.0); // minimum 50M threshold
    }

    pub fn world_to_local(&self, world_pos: Vec3) -> Vec3 {
        world_pos + self.current_offset
    }

    pub fn local_to_world(&self, local_pos: Vec3) -> Vec3 {
        local_pos - self.current_offset
    }
}

impl Default for FloatingOrigin {
    fn default() -> Self { Self::new() }
}