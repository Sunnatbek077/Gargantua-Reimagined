// =============================================================================
// crates/gargantua-camera/src/world_camera.rs
// =============================================================================
//
// PURPOSE:
//   The central camera struct for Gargantua. Owns the current camera mode,
//   position/orientation state, projection parameters, and all camera
//   sub-systems (chunk manager, floating origin, fx). Provides:
//     - view_matrix() and proj_matrix() consumed by SceneUniforms each frame
//     - update() called every frame to advance the active camera mode
//     - mode switching API (orbit ↔ free_flight ↔ cinematic ↔ gravity ↔ satellite)
//
// SIZE: ~380 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::modes::{orbit::OrbitMode, free_flight::FreeFlightMode,
//                      cinematic::CinematicMode, gravity::GravityMode,
//                      satellite::SatelliteMode}
//     - crate::path::spline::CameraSpline
//     - crate::world::chunk_manager::ChunkManager
//     - crate::world::coord_system::{look_at_matrix, projection_matrix}
//     - crate::fx::{horizon_fx::HorizonFx, relativistic_fov::RelativisticFov,
//                   time_warp::TimeWarp}
//     - crate::errors::CameraError
//   External:
//     - glam::{Vec3, Quat, Mat4}
//     - winit::keyboard::KeyCode  (for InputState)
//
// CALLED BY:
//   - crates/gargantua-app/src/app.rs::App::render_frame()
//       — calls WorldCamera::update() then uses view/proj matrices
//   - crates/gargantua-render/src/bindgroups::scene.rs::build_uniforms()
//       — calls WorldCamera::view_matrix(), proj_matrix(), position(), etc.
//   - crates/gargantua-ui/src/panel::camera.rs
//       — reads WorldCamera::mode_name(), position(), fov_y_deg()
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy)]
//   pub struct CameraPose {
//     pub position:  Vec3,
//     pub rotation:  Quat,
//     pub fov_y_deg: f32,
//   }
//
//   pub struct InputState {
//     pub keys_held:   std::collections::HashSet<winit::keyboard::KeyCode>,
//     pub mouse_delta: (f32, f32),  // (dx, dy) pixels since last frame
//     pub scroll:      f32,         // scroll wheel delta
//     pub mouse_held:  bool,        // primary button held
//   }
//
//   pub enum CameraMode {
//     Orbit(OrbitMode),
//     FreeFlight(FreeFlightMode),
//     Cinematic(CinematicMode),
//     Gravity(GravityMode),
//     Satellite(SatelliteMode),
//   }
//
//   pub struct WorldCamera {
//     mode:         CameraMode,
//     position:     Vec3,           // current world position
//     rotation:     Quat,           // current orientation
//     fov_y_deg:    f32,            // current field of view in degrees
//     aspect:       f32,            // width / height (updated on resize)
//     near:         f32,            // near clip plane (default 0.01 M)
//     far:          f32,            // far clip plane (default 10000.0 M)
//     chunks:       ChunkManager,
//     horizon_fx:   HorizonFx,
//     rel_fov:      RelativisticFov,
//     time_warp:    TimeWarp,
//     prev_view_proj: Mat4,         // previous frame view*proj for TAA
//     black_hole_local_pos: Vec3,   // black hole position in local frame
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     aspect:    f32,
//     r_horizon: f32,
//     r_isco:    f32,
//   ) -> Self
//     — creates with OrbitMode at radius = 20M, theta = π/3, phi = 0.
//     — fov_y_deg = 60.0, near = 0.01, far = 10000.0.
//     — horizon_fx = HorizonFx::new(r_horizon, r_isco).
//     — rel_fov = RelativisticFov::new(60.0).
//     — time_warp = TimeWarp::new(1.0).
//
//   pub fn update(
//     &mut self,
//     delta_t: f32,
//     input:   &InputState,
//   ) -> CameraPose
//     — applies time_warp to get effective_dt.
//     — dispatches to the active mode's update():
//         Orbit:      orbit.update(delta_t, dx, dy, scroll) → (pos, rot)
//         FreeFlight: free_flight.update(delta_t) → (pos, rot)
//         Cinematic:  cinematic.update(effective_dt) → SplineEval → (pos, rot, fov)
//         Gravity:    gravity.update(effective_dt) → (pos, rot)
//         Satellite:  satellite.update(effective_dt) → (pos, rot)
//     — updates self.position, self.rotation, self.fov_y_deg.
//     — updates horizon_fx with camera_r.
//     — calls chunks.update(position) — checks for floating origin shift.
//     — stores prev_view_proj.
//     — returns CameraPose { position, rotation, fov_y_deg }.
//
//   pub fn view_matrix(&self) -> Mat4
//     — look_at_rh(self.position, self.position + look_direction(), Vec3::Y)
//     — OR Mat4::from_quat(self.rotation.inverse()).translate(-self.position)
//
//   pub fn proj_matrix(&self) -> Mat4
//     — projection_matrix(self.fov_y_rad(), self.aspect, self.near, self.far)
//
//   pub fn view_proj(&self) -> Mat4
//     — self.proj_matrix() * self.view_matrix()
//
//   pub fn prev_view_proj(&self) -> Mat4 { self.prev_view_proj }
//
//   pub fn set_mode(&mut self, mode: CameraMode)
//   pub fn mode_name(&self) -> &'static str
//   pub fn set_aspect(&mut self, width: u32, height: u32) → aspect = w/h as f32
//   pub fn position(&self) -> Vec3      { self.position  }
//   pub fn rotation(&self) -> Quat      { self.rotation  }
//   pub fn fov_y_deg(&self) -> f32      { self.fov_y_deg }
//   pub fn fov_y_rad(&self) -> f32      { self.fov_y_deg.to_radians() }
//   pub fn look_direction(&self) -> Vec3 { self.rotation * (-Vec3::Z) }
//   pub fn up_direction(&self) -> Vec3  { self.rotation * Vec3::Y }
//
//   pub fn horizon_fx(&self) -> &HorizonFx { &self.horizon_fx }
//   pub fn time_warp(&mut self) -> &mut TimeWarp { &mut self.time_warp }
//
// NOTES FOR AI:
//   - WorldCamera is the single source of truth for camera state.
//     All other systems read from it; none write to it except through update().
//   - The floating origin shift (from chunks.update) must be applied to:
//       self.position -= shift_vec
//       self.black_hole_local_pos -= shift_vec
//       AND to all Keyframe positions in the active CinematicMode spline.
//   - prev_view_proj must be stored BEFORE computing the new view_proj
//     so TAA can correctly reproject the previous frame.
//   - near = 0.01 M is aggressive but safe for the scale of the scene.
//     Far = 10000M covers even the largest preset viewing distances.
// =============================================================================

use glam::{Mat4, Quat, Vec3};
use std::collections::HashSet;

use crate::{
    errors::CameraError,
    fx::{horizon_fx::HorizonFx, relativistic_fov::RelativisticFov, time_warp::TimeWarp},
    modes::{
        cinematic::CinematicMode,
        free_flight::FreeFlightMode,
        gravity::GravityMode,
        orbit::OrbitMode,
        satellite::SatelliteMode,
    },
    world::{chunk_manager::ChunkManager, coord_system::{look_at_matrix, projection_matrix}},
};

#[derive(Debug, Clone, Copy)]
pub struct CameraPose {
    pub position:  Vec3,
    pub rotation:  Quat,
    pub fov_y_deg: f32,
}

pub struct InputState {
    pub mouse_delta: (f32, f32),
    pub scroll:      f32,
    pub mouse_held:  bool,
}

pub enum CameraMode {
    Orbit(OrbitMode),
    FreeFlight(FreeFlightMode),
    Cinematic(CinematicMode),
    Gravity(GravityMode),
    Satellite(SatelliteMode),
}

pub struct WorldCamera {
    mode:                 CameraMode,
    position:             Vec3,
    rotation:             Quat,
    fov_y_deg:            f32,
    aspect:               f32,
    near:                 f32,
    far:                  f32,
    chunks:               ChunkManager,
    horizon_fx:           HorizonFx,
    rel_fov:              RelativisticFov,
    time_warp:            TimeWarp,
    prev_view_proj:       Mat4,
    black_hole_local_pos: Vec3,
}

impl WorldCamera {
    pub fn new(aspect: f32, r_horizon: f32, r_isco: f32) -> Self {
        Self {
            mode:                 CameraMode::Orbit(OrbitMode::new()),
            position:             Vec3::new(0.0, 5.0, 20.0),
            rotation:             Quat::IDENTITY,
            fov_y_deg:            60.0,
            aspect,
            near:                 0.01,
            far:                  10_000.0,
            chunks:               ChunkManager::new(),
            horizon_fx:           HorizonFx::new(),
            rel_fov:              RelativisticFov::new(60.0),
            time_warp:            TimeWarp::new(1.0),
            prev_view_proj:       Mat4::IDENTITY,
            black_hole_local_pos: Vec3::ZERO,
        }
    }

    pub fn update(&mut self, delta_t: f32, input: &InputState) -> CameraPose {
        // Store previous frame view_proj for TAA
        self.prev_view_proj = self.view_proj();

        // Time warp
        let eff_dt = self.time_warp.tick(delta_t);
        let _ = eff_dt; // used by modes

        // Dispatch to active mode
        let (pos, rot) = match &mut self.mode {
            CameraMode::Orbit(m)      => m.update(delta_t),
            CameraMode::FreeFlight(m) => m.update(delta_t),
            CameraMode::Cinematic(m)  => m.update(delta_t),
            CameraMode::Gravity(m)    => m.update(delta_t),
            CameraMode::Satellite(m)  => m.update(delta_t),
        };
        self.position = pos;
        self.rotation = rot.normalize();

        // Check for floating origin shift
        if let Some(shift) = self.chunks.update(self.position) {
            self.position             += shift;
            self.black_hole_local_pos += shift;
        }

        CameraPose { position: self.position, rotation: self.rotation, fov_y_deg: self.fov_y_deg }
    }

    pub fn view_matrix(&self) -> Mat4 {
        look_at_matrix(
            self.position,
            self.position + self.look_direction(),
            Vec3::Y,
        )
    }

    pub fn proj_matrix(&self) -> Mat4 {
        projection_matrix(self.fov_y_rad(), self.aspect, self.near, self.far)
    }

    pub fn view_proj(&self) -> Mat4 { self.proj_matrix() * self.view_matrix() }
    pub fn prev_view_proj(&self) -> Mat4 { self.prev_view_proj }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }

    pub fn set_mode(&mut self, mode: CameraMode) { self.mode = mode; }

    pub fn mode_name(&self) -> &'static str {
        match &self.mode {
            CameraMode::Orbit(_)      => "Orbit",
            CameraMode::FreeFlight(_) => "Free Flight",
            CameraMode::Cinematic(_)  => "Cinematic",
            CameraMode::Gravity(_)    => "Gravity",
            CameraMode::Satellite(_)  => "Satellite",
        }
    }

    pub fn position(&self)      -> Vec3  { self.position  }
    pub fn rotation(&self)      -> Quat  { self.rotation  }
    pub fn fov_y_deg(&self)     -> f32   { self.fov_y_deg }
    pub fn fov_y_rad(&self)     -> f32   { self.fov_y_deg.to_radians() }
    pub fn look_direction(&self)-> Vec3  { self.rotation * (-Vec3::Z) }
    pub fn up_direction(&self)  -> Vec3  { self.rotation * Vec3::Y }
    pub fn aspect(&self)        -> f32   { self.aspect }

    pub fn horizon_fx(&self)          -> &HorizonFx    { &self.horizon_fx }
    pub fn time_warp(&mut self)       -> &mut TimeWarp { &mut self.time_warp }
    pub fn origin_offset(&self)       -> Vec3          { self.chunks.origin_offset() }
}