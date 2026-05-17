// =============================================================================
// crates/gargantua-render/src/pipelines/lensing.rs
// =============================================================================
//
// PURPOSE:
//   Applies gravitational lensing distortion to background starlight and
//   renders the Einstein ring — the bright ring of light that forms when
//   background stars are lensed by the black hole's gravity.
//
//   Runs shaders/render/lensing.wgsl (380 lines). Reads the geodesic
//   deflection from geodesic_gpu.rs output (or geodesic_lut texture) to
//   determine the apparent position of each background star in the image.
//
//   The lensing pass runs after ray_march but before post-fx. It composites
//   lensed starlight onto the framebuffer (additive blending in the shader).
//
// SIZE: ~300 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup           — group(0)
//     - crate::bindgroups::textures::TexturesBindGroup     — group(1) starmap
//     - gargantua_core::frame::pass::{Pass, PassContext, ComputePass}
//     - gargantua_core::frame::resource::ResourceHandle
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Buffer}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs — registered after accretion in FrameGraph
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct LensingParams {
//     pub mass:           f32,   // black hole mass M
//     pub spin:           f32,   // spin a/M
//     pub einstein_ring_r: f32,  // Einstein ring apparent radius (computed from mass, distance)
//     pub observer_dist:  f32,   // observer distance from black hole in M
//     pub disk_scale:     f32,   // angular scale of disk in scene space
//     pub lensing_strength: f32, // 0.0=no lensing, 1.0=full GR lensing (for debug/art)
//     pub ghost_threshold: f32,  // min brightness for secondary image (ghost ring) rendering
//     pub _pad:           f32,
//   }
//
//   pub struct LensingPass {
//     pipeline:       wgpu::ComputePipeline,
//     params_buffer:  wgpu::Buffer,
//     params_bg:      wgpu::BindGroup,
//     geodesic_handle: ResourceHandle,  // input: geodesic deflection texture
//     output_handle:  ResourceHandle,   // output: adds to framebuffer
//     reads:          Vec<ResourceHandle>,
//     writes:         Vec<ResourceHandle>,
//     workgroup_x:    u32,
//     workgroup_y:    u32,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:          &wgpu::Device,
//     shader:          &wgpu::ShaderModule, // lensing.wgsl
//     scene_bg:        &SceneBindGroup,
//     tex_bg:          &TexturesBindGroup,
//     geodesic_output: ResourceHandle,
//     framebuffer:     ResourceHandle,
//     params:          LensingParams,
//     preset:          &QualityPreset,
//   ) -> Result<Self, CoreError>
//     — pipeline layout:
//         group(0): SceneUniforms
//         group(1): baked textures (starmap at binding 4)
//         group(2): LensingParams uniform
//         group(3): geodesic texture (read) + framebuffer (read_write)
//
//   pub fn update_params(&self, queue: &wgpu::Queue, params: &LensingParams)
//
//   impl Pass: name = "lensing", dispatches lensing.wgsl
//
// LENSING ALGORITHM (lensing.wgsl):
//   For each pixel (screen UV → world ray direction):
//     1. Look up geodesic deflection angle α from geodesic_lut or geodesic output.
//     2. Compute lensed ray direction: rotate original ray by α around the
//        optical axis (black hole → observer direction).
//     3. Sample starmap at lensed direction → background star color.
//     4. Apply relativistic beaming: stars in the direction of motion appear
//        brighter and bluer (aberration), dimmer and redder opposing.
//     5. Add to framebuffer (accumulate starlight onto disk emission).
//
//   Secondary images (ghost ring):
//     Rays that orbit the black hole once before escaping produce a secondary
//     Einstein ring — a dimmer, inverted copy of the primary image.
//     lensing.wgsl handles secondary images for deflection angles > π.
//
// NOTES FOR AI:
//   - lensing_strength = 0.0 makes this pass a no-op (debug mode).
//   - ghost_threshold prevents rendering very faint secondary images (saves GPU).
//     Set to 0.001 (1/1000 of peak) by default.
//   - The Einstein ring radius in apparent angle: θ_E = sqrt(4GM/c²D)
//     where D is observer-lens distance. In the scene, this maps to a
//     pixel radius that depends on the camera FOV.
//   - This pass reads from the geodesic output texture (written by
//     geodesic_gpu.rs or the LUT). The FrameGraph must declare this
//     dependency so barriers are inserted correctly.
// =============================================================================

use bytemuck::{Pod, Zeroable};
use gargantua_core::{
    errors::CoreError,
    frame::{
        pass::{ComputePass, Pass, PassContext},
        resource::ResourceHandle,
    },
    quality::preset::QualityPreset,
};
use crate::bindgroups::{scene::SceneBindGroup, textures::TexturesBindGroup};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct LensingParams {
    pub mass:             f32,
    pub spin:             f32,
    pub einstein_ring_r:  f32,
    pub observer_dist:    f32,
    pub disk_scale:       f32,
    pub lensing_strength: f32,
    pub ghost_threshold:  f32,
    pub _pad:             f32,
}

pub struct LensingPass {
    pipeline:        wgpu::ComputePipeline,
    params_buffer:   wgpu::Buffer,
    params_bg:       wgpu::BindGroup,
    geodesic_handle: ResourceHandle,
    output_handle:   ResourceHandle,
    reads:           Vec<ResourceHandle>,
    writes:          Vec<ResourceHandle>,
    workgroup_x:     u32,
    workgroup_y:     u32,
}

impl LensingPass {
    pub fn new(
        device:          &wgpu::Device,
        shader:          &wgpu::ShaderModule,
        scene_bg:        &SceneBindGroup,
        tex_bg:          &TexturesBindGroup,
        geodesic_output: ResourceHandle,
        framebuffer:     ResourceHandle,
        params:          LensingParams,
        preset:          &QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &LensingParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

impl Pass for LensingPass {
    fn name(&self) -> &str { "lensing" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}

impl ComputePass for LensingPass {}