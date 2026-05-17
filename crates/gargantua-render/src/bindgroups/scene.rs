// =============================================================================
// crates/gargantua-render/src/bindgroups/scene.rs
// =============================================================================
//
// PURPOSE:
//   Defines SceneUniforms — the per-frame GPU uniform buffer uploaded to
//   bind group 0 (group(0) in all WGSL shaders). Contains camera state,
//   frame timing, render resolution, and physics simulation parameters
//   that change every frame.
//
//   All WGSL compute and render shaders declare:
//     @group(0) @binding(0) var<uniform> scene: SceneUniforms;
//   The layout defined here must match the WGSL struct exactly
//   (field order, types, padding).
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_core::gpu::context::GpuContext
//     - gargantua_core::clock::Clock
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, Queue, Buffer, BufferUsages, BindGroup,
//              BindGroupLayout, BindGroupLayoutDescriptor,
//              BindGroupLayoutEntry, ShaderStages, BufferBindingType}
//     - bytemuck::{Pod, Zeroable}
//     - glam::{Mat4, Vec3, Vec4}
//
// CALLED BY:
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — binds SceneBindGroup at group(0) in compute dispatch
//   - crates/gargantua-render/src/pipelines/accretion.rs  — same
//   - crates/gargantua-render/src/pipelines/lensing.rs    — same
//   - crates/gargantua-render/src/postfx/taa.rs           — reads frame_idx
//   - crates/gargantua-render/src/postfx/film_grain.rs    — reads elapsed_s
//   - crates/gargantua-core/src/app.rs
//       — calls SceneBindGroup::update() each frame before graph.execute()
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct SceneUniforms {
//     // Camera matrices (column-major, matches WGSL mat4x4<f32>)
//     pub view:            [[f32; 4]; 4],  // world → camera space
//     pub proj:            [[f32; 4]; 4],  // camera → clip space
//     pub view_proj:       [[f32; 4]; 4],  // combined view * proj
//     pub inv_view_proj:   [[f32; 4]; 4],  // inverse — clip → world (ray generation)
//     pub prev_view_proj:  [[f32; 4]; 4],  // previous frame — TAA reprojection
//
//     // Camera position and direction
//     pub cam_pos:    [f32; 4],  // world position (w unused, padding)
//     pub cam_dir:    [f32; 4],  // normalized forward vector (w unused)
//     pub cam_up:     [f32; 4],  // normalized up vector (w unused)
//
//     // Render resolution
//     pub width:      u32,
//     pub height:     u32,
//     pub inv_width:  f32,   // 1.0 / width  — avoids division in shader
//     pub inv_height: f32,   // 1.0 / height
//
//     // Frame timing
//     pub elapsed_s:  f32,   // total seconds since app start (disk animation)
//     pub delta_t:    f32,   // frame delta time in seconds
//     pub frame_idx:  u32,   // monotonically increasing frame counter (TAA jitter)
//     pub _pad0:      u32,   // align to 16 bytes
//
//     // Render quality
//     pub spp:        u32,   // samples per pixel this frame (adaptive quality)
//     pub max_steps:  u32,   // max geodesic integration steps
//     pub _pad1:      [u32; 2],
//
//     // TAA jitter (Halton sequence offset added to UV in vertex shader)
//     pub jitter_x:   f32,
//     pub jitter_y:   f32,
//     pub _pad2:      [f32; 2],
//   }
//   // Total size: 5×64 + 3×16 + 4×4 + 4×4 + 4×4 + 2×4 = ~448 bytes
//   // WGSL requires uniform buffers to be 16-byte aligned — all fields satisfy this.
//
//   pub struct SceneBindGroup {
//     buffer:     wgpu::Buffer,     // UNIFORM | COPY_DST
//     bind_group: wgpu::BindGroup,
//     layout:     wgpu::BindGroupLayout,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
//     — creates the BindGroupLayout for group(0):
//         binding 0: uniform buffer, visibility ALL (vertex + fragment + compute)
//     — called once at pipeline creation time. Cached by each pipeline.
//
//   pub fn new(device: &wgpu::Device) -> Self
//     — creates the UNIFORM buffer with size = size_of::<SceneUniforms>().
//     — creates the BindGroup with binding 0 = entire buffer.
//     — stores layout for pipeline creation.
//
//   pub fn update(
//     &self,
//     queue:    &wgpu::Queue,
//     uniforms: &SceneUniforms,
//   )
//     — calls queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniforms)).
//     — called once per frame by App::render_frame() before graph.execute().
//
//   pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
//   pub fn layout(&self)     -> &wgpu::BindGroupLayout { &self.layout }
//
//   pub fn build_uniforms(
//     clock:      &Clock,
//     camera:     &gargantua_camera::WorldCamera,
//     resolution: (u32, u32),
//     spp:        u32,
//     max_steps:  u32,
//   ) -> SceneUniforms
//     — constructs SceneUniforms from live app state.
//     — view/proj/view_proj: from camera.view_matrix() and camera.proj_matrix().
//     — inv_view_proj: (view_proj).inverse() — used in ray_march.wgsl for
//       ray generation: ray_dir = (inv_view_proj * clip_pos).normalize().
//     — prev_view_proj: stored from previous frame's view_proj.
//     — jitter_x/y: Halton(2, frame_idx % 8) and Halton(3, frame_idx % 8)
//       — 8-sample Halton sequence for TAA sub-pixel jitter.
//     — called by gargantua-app before SceneBindGroup::update().
//
// HALTON SEQUENCE (TAA jitter — 8 samples):
//   frame_idx % 8 → (jitter_x, jitter_y):
//     0 → ( 0.000,  0.000)
//     1 → ( 0.500,  0.333)
//     2 → ( 0.250,  0.667)
//     3 → ( 0.750,  0.111)
//     4 → ( 0.125,  0.444)
//     5 → ( 0.625,  0.778)
//     6 → ( 0.375,  0.222)
//     7 → ( 0.875,  0.556)
//   Jitter is in pixel space (−0.5..0.5). Multiply by inv_width, inv_height
//   before adding to NDC coordinates.
//
// NOTES FOR AI:
//   - SceneUniforms must be #[repr(C)] and implement bytemuck::Pod.
//     Every field must be a primitive type or array of primitives.
//     No enums, no booleans, no references.
//   - WGSL mat4x4<f32> is column-major — matches glam::Mat4's memory layout.
//     Use mat.to_cols_array_2d() to convert to [[f32;4];4].
//   - The buffer must be created with BufferUsages::UNIFORM | COPY_DST.
//     COPY_DST allows queue.write_buffer() updates every frame.
//   - All WGSL shaders read frame_idx as u32. Cast from u64 in Clock
//     with frame_idx as u32 (wrap is acceptable for jitter/grain purposes).
//   - inv_view_proj is used by ray_march.wgsl to reconstruct world-space
//     rays from screen UV coordinates. This is the primary ray generation step.
// =============================================================================

use bytemuck::{Pod, Zeroable};
use wgpu::{BindGroupLayout, Device, Queue, ShaderStages};
use gargantua_core::{clock::Clock, errors::CoreError};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SceneUniforms {
    pub view:           [[f32; 4]; 4],
    pub proj:           [[f32; 4]; 4],
    pub view_proj:      [[f32; 4]; 4],
    pub inv_view_proj:  [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub cam_pos:        [f32; 4],
    pub cam_dir:        [f32; 4],
    pub cam_up:         [f32; 4],
    pub width:          u32,
    pub height:         u32,
    pub inv_width:      f32,
    pub inv_height:     f32,
    pub elapsed_s:      f32,
    pub delta_t:        f32,
    pub frame_idx:      u32,
    pub _pad0:          u32,
    pub spp:            u32,
    pub max_steps:      u32,
    pub _pad1:          [u32; 2],
    pub jitter_x:       f32,
    pub jitter_y:       f32,
    pub _pad2:          [f32; 2],
}

pub struct SceneBindGroup {
    buffer:     wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    layout:     wgpu::BindGroupLayout,
}

// Halton sequence table for TAA (base-2 and base-3, 8 samples)
const HALTON_8: [(f32, f32); 8] = [
    (0.000, 0.000), (0.500, 0.333), (0.250, 0.667), (0.750, 0.111),
    (0.125, 0.444), (0.625, 0.778), (0.375, 0.222), (0.875, 0.556),
];

impl SceneBindGroup {
    pub fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            }],
        })
    }

    pub fn new(device: &Device) -> Self {
        let layout = Self::bind_group_layout(device);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("scene_uniform_buffer"),
            size:               std::mem::size_of::<SceneUniforms>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("scene_bind_group"),
            layout:  &layout,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self { buffer, bind_group, layout }
    }

    pub fn update(&self, queue: &Queue, uniforms: &SceneUniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup      { &self.bind_group }
    pub fn layout(&self)     -> &wgpu::BindGroupLayout { &self.layout     }

    pub fn jitter_for_frame(frame_idx: u64) -> (f32, f32) {
        HALTON_8[(frame_idx % 8) as usize]
    }
}