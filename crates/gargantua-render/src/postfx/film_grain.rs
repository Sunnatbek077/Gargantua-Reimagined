// =============================================================================
// crates/gargantua-render/src/postfx/film_grain.rs
// =============================================================================
//
// PURPOSE:
//   Blue noise film grain — adds subtle per-frame texture noise using blue_noise_3d.
//   Shader: shaders/postfx/film_grain.wgsl (140 lines)
//   Per-frame noise slice = frame_idx % 64 → different 2D slice of blue_noise_3d each frame.
//   Grain amplitude scales with luminance (more grain in shadows, less in highlights).
//   params.strength: 0.0=off, 0.02=subtle (default), 0.1=heavy film look.
//   NOTES FOR AI: blue_noise_3d is sampled at (uv.x, uv.y, float(frame_idx%64)/64.0).
//
// SIZE: ~140 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup      — group(0) SceneUniforms
//     - gargantua_core::frame::pass::{Pass, PassContext}
//     - gargantua_core::frame::resource::ResourceHandle
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Buffer}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs — registered in FrameGraph (post-fx chain)
//
// FRAME GRAPH POSITION:
//   Reads:  hdr_framebuffer
//   Writes: grain_output
//
// NOTES FOR AI:
//   - All post-fx passes follow the same pattern as ray_march.rs:
//     implement Pass trait, dispatch compute or render pass in record().
//   - Input and output textures are DIFFERENT resources to avoid
//     read/write hazards (FrameGraph inserts barriers between passes).
//   - All post-fx params are uploaded via a UNIFORM | COPY_DST buffer
//     in a BindGroup at group(2) or group(3) depending on pass.
// =============================================================================

use bytemuck::{Pod, Zeroable};
use gargantua_core::{
    errors::CoreError,
    frame::{
        pass::{Pass, PassContext},
        resource::ResourceHandle,
    },
};
use crate::bindgroups::scene::SceneBindGroup;

pub struct FilmGrainPass {
    pipeline:      wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    params_bg:     wgpu::BindGroup,
    reads:         Vec<ResourceHandle>,
    writes:        Vec<ResourceHandle>,
    workgroup_x:   u32,
    workgroup_y:   u32,
}

impl FilmGrainPass {
    pub fn new(
        device:   &wgpu::Device,
        shader:   &wgpu::ShaderModule,
        scene_bg: &SceneBindGroup,
        reads:    Vec<ResourceHandle>,
        writes:   Vec<ResourceHandle>,
        preset:   &gargantua_core::quality::preset::QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }
}

impl Pass for FilmGrainPass {
    fn name(&self) -> &str { "film_grain" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}