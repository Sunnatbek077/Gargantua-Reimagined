// =============================================================================
// crates/gargantua-render/src/postfx/chromatic.rs
// =============================================================================
//
// PURPOSE:
//   Chromatic aberration — simulates lens color fringing at image periphery.
//   Separates R, G, B channels by radially increasing UV offsets so that
//   colors bleed apart toward screen corners, mimicking real camera optics.
//
//   Shader: shaders/postfx/chromatic.wgsl (120 lines)
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup      — group(0)
//     - gargantua_core::frame::pass::{Pass, PassContext, ComputePass}
//     - gargantua_core::frame::resource::ResourceHandle
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Buffer, Sampler}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs
//       — registered after bloom in the post-fx chain, before film_grain
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct ChromaticParams {
//     pub strength:       f32,   // channel separation intensity (default 0.003)
//     pub radial_power:   f32,   // falloff exponent (1.0=linear, 2.0=quadratic, default 2.0)
//     pub barrel_distort: f32,   // barrel distortion amount (0.0=none, default 0.001)
//     pub _pad:           f32,
//   }
//
//   pub struct ChromaticPass {
//     pipeline:       wgpu::ComputePipeline,
//     params_buffer:  wgpu::Buffer,
//     params_bg:      wgpu::BindGroup,
//     linear_sampler: wgpu::Sampler,
//     input_handle:   ResourceHandle,   // reads from: TAA or bloom output
//     output_handle:  ResourceHandle,   // writes to: chromatic output texture
//     reads:          Vec<ResourceHandle>,
//     writes:         Vec<ResourceHandle>,
//     workgroup_x:    u32,
//     workgroup_y:    u32,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:        &wgpu::Device,
//     shader:        &wgpu::ShaderModule,  // chromatic.wgsl
//     scene_bg:      &SceneBindGroup,
//     input_handle:  ResourceHandle,
//     output_handle: ResourceHandle,
//     params:        ChromaticParams,
//     preset:        &QualityPreset,
//   ) -> Result<Self, CoreError>
//     — creates linear_sampler (ClampToEdge, Linear) for channel sampling.
//     — pipeline layout:
//         group(0): SceneUniforms (width, height, inv_width, inv_height)
//         group(1): input texture (read-only) + linear_sampler
//         group(2): ChromaticParams uniform
//         group(3): output storage texture (write-only)
//
//   pub fn update_params(&self, queue: &wgpu::Queue, params: &ChromaticParams)
//     — queue.write_buffer for ChromaticParams.
//
//   impl Pass for ChromaticPass:
//     fn name(&self) -> &str { "chromatic" }
//     fn record(&mut self, ctx: &mut PassContext) -> Result<(), CoreError>
//       — resolves input and output textures from ctx.resources.
//       — creates bind groups for this frame.
//       — dispatches chromatic.wgsl.
//
// WGSL ALGORITHM (chromatic.wgsl):
//   For each pixel (u, v) in [0,1]²:
//     let center = vec2(0.5, 0.5);
//     let offset = uv - center;                         // direction from center
//     let dist   = length(offset);
//     let power  = pow(dist, params.radial_power);      // radial falloff
//     let sep    = params.strength * power;             // channel separation amount
//
//     // Barrel distortion pre-warp (optional)
//     let barrel = offset * (1.0 + params.barrel_distort * dist * dist);
//     let base_uv = center + barrel;
//
//     // Sample each channel at slightly different UV
//     let r = textureSampleLevel(input, sampler, base_uv + offset * sep * 1.0,  0.0).r;
//     let g = textureSampleLevel(input, sampler, base_uv,                        0.0).g;
//     let b = textureSampleLevel(input, sampler, base_uv + offset * sep * -1.0, 0.0).b;
//     let a = textureSampleLevel(input, sampler, base_uv,                        0.0).a;
//     textureStore(output, coord, vec4(r, g, b, a));
//
// NOTES FOR AI:
//   - strength = 0.003 is subtle but visible — a realistic camera lens value.
//     strength > 0.01 becomes obvious artistic exaggeration.
//   - Input and output MUST be different ResourceHandles — cannot read and
//     write the same texture in the same pass (undefined behavior in WGSL).
//   - The linear_sampler uses ClampToEdge so edge pixels don't wrap.
//     Out-of-bounds samples (near corners) simply clamp to the edge color.
//   - radial_power = 2.0 means the aberration is negligible at screen center
//     and peaks at corners, which matches real lens behavior.
//   - If strength = 0.0: the pass still runs but produces identity output.
//     For a true no-op, check strength before registering this pass.
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
use crate::bindgroups::scene::SceneBindGroup;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ChromaticParams {
    pub strength:       f32,
    pub radial_power:   f32,
    pub barrel_distort: f32,
    pub _pad:           f32,
}

pub struct ChromaticPass {
    pipeline:       wgpu::ComputePipeline,
    params_buffer:  wgpu::Buffer,
    params_bg:      wgpu::BindGroup,
    linear_sampler: wgpu::Sampler,
    input_handle:   ResourceHandle,
    output_handle:  ResourceHandle,
    reads:          Vec<ResourceHandle>,
    writes:         Vec<ResourceHandle>,
    workgroup_x:    u32,
    workgroup_y:    u32,
}

impl ChromaticPass {
    pub fn new(
        device:        &wgpu::Device,
        shader:        &wgpu::ShaderModule,
        scene_bg:      &SceneBindGroup,
        input_handle:  ResourceHandle,
        output_handle: ResourceHandle,
        params:        ChromaticParams,
        preset:        &QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &ChromaticParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

impl Pass for ChromaticPass {
    fn name(&self) -> &str { "chromatic" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}

impl ComputePass for ChromaticPass {}