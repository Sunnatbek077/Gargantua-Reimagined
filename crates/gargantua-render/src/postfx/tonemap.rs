// =============================================================================
// crates/gargantua-render/src/postfx/tonemap.rs
// =============================================================================
//
// PURPOSE:
//   ACES RRT/ODT tonemapping — converts HDR scene linear light to display-ready output.
//   Shader: shaders/postfx/tonemap.wgsl (260 lines)
//   macOS path: EDR headroom scaling → Display P3 matrix → sRGB gamma encode.
//   Windows HDR path: ACEScg → BT.2020 matrix → PQ (ST.2084) encoding.
//   Windows SDR path: ACES RRT → sRGB ODT → Bgra8UnormSrgb output.
//   Uniform inputs: ColorSpaceUniforms (from edr.rs or hdr10.rs).
//   This is the LAST pass — writes to the swapchain texture (GpuSurface frame output).
//   NOTES FOR AI:
//     - Output format depends on GpuSurface.format() — Rgba16Float (HDR) or Bgra8UnormSrgb (SDR).
//     - Must write to the swapchain TextureView, not a ResourcePool handle.
//     - ACES RRT parameters are loaded from assets/luts/aces_rrt.cube at startup.
//
// SIZE: ~200 lines
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
//   Writes: swapchain_output
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

pub struct TonemapPass {
    pipeline:      wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    params_bg:     wgpu::BindGroup,
    reads:         Vec<ResourceHandle>,
    writes:        Vec<ResourceHandle>,
    workgroup_x:   u32,
    workgroup_y:   u32,
}

impl TonemapPass {
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

impl Pass for TonemapPass {
    fn name(&self) -> &str { "tonemap" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}