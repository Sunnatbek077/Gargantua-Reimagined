// =============================================================================
// crates/gargantua-render/src/postfx/lens_flare.rs
// =============================================================================
//
// PURPOSE:
//   Lens flare — hexagonal bokeh and prism diffraction around bright sources.
//   No dedicated WGSL shader (uses fullscreen.wgsl + custom compute).
//   Detects bright pixel clusters above flare_threshold in the HDR framebuffer.
//   Renders: primary flare (octagonal iris), secondary flares (ghosts along optical axis),
//            chromatic dispersion (rainbow streak), dust diffraction (starburst pattern).
//   NOTES FOR AI:
//     - lens_flare is expensive (~3ms at 4K). Disable for real-time 120fps.
//     - Uses only a RENDER pass (not compute) — fullscreen triangle + alpha blend.
//     - flare_threshold: 2.0 (default) — only renders on pixels 2× above SDR white.
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
//   Writes: flare_output
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

pub struct LensFlarePass {
    pipeline:      wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    params_bg:     wgpu::BindGroup,
    reads:         Vec<ResourceHandle>,
    writes:        Vec<ResourceHandle>,
    workgroup_x:   u32,
    workgroup_y:   u32,
}

impl LensFlarePass {
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

impl Pass for LensFlarePass {
    fn name(&self) -> &str { "lens_flare" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}