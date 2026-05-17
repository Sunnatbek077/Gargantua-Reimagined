// =============================================================================
// crates/gargantua-render/src/postfx/bloom.rs
// =============================================================================
//
// PURPOSE:
//   Dual Kawase bloom filter — physically-based glow for bright light sources.
//   Simulates lens/sensor scattering of photons from extremely bright pixels
//   (accretion disk, photon sphere). Makes bright regions appear to emit
//   a realistic glow that spreads into the surrounding darkness.
//
//   Shaders: shaders/postfx/bloom_down.wgsl (140 lines)
//            shaders/postfx/bloom_up.wgsl   (140 lines)
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup      — group(0)
//     - gargantua_core::frame::pass::{Pass, PassContext, ComputePass}
//     - gargantua_core::frame::resource::{ResourceHandle, ResourcePool}
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Sampler}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs — registered after TAA in post-fx chain
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct BloomParams {
//     pub threshold:   f32,   // pixels below this brightness contribute zero bloom (default 1.0)
//     pub knee:        f32,   // soft knee width around threshold (default 0.1)
//     pub intensity:   f32,   // final bloom intensity multiplier (default 0.04)
//     pub scatter:     f32,   // bloom scatter radius (0..1, default 0.7)
//     pub dirt_mask:   u32,   // 1 = apply dirt/lens mask texture, 0 = skip
//     pub _pad:        [u32; 3],
//   }
//
//   pub struct BloomPass {
//     down_pipeline:  wgpu::ComputePipeline,  // bloom_down.wgsl
//     up_pipeline:    wgpu::ComputePipeline,  // bloom_up.wgsl
//     params_buffer:  wgpu::Buffer,
//     params_bg:      wgpu::BindGroup,
//     linear_sampler: wgpu::Sampler,
//     mip_handles:    [ResourceHandle; 6],    // 6-level pyramid (transient each frame)
//     input_handle:   ResourceHandle,
//     output_handle:  ResourceHandle,
//     reads:          Vec<ResourceHandle>,
//     writes:         Vec<ResourceHandle>,
//     levels:         u32,    // bloom pyramid levels (default 6)
//     workgroup_x:    u32,
//     workgroup_y:    u32,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:        &wgpu::Device,
//     down_shader:   &wgpu::ShaderModule,  // bloom_down.wgsl
//     up_shader:     &wgpu::ShaderModule,  // bloom_up.wgsl
//     scene_bg:      &SceneBindGroup,
//     input_handle:  ResourceHandle,
//     output_handle: ResourceHandle,
//     params:        BloomParams,
//     preset:        &QualityPreset,
//   ) -> Result<Self, CoreError>
//     — creates two pipelines (downsample and upsample).
//     — creates linear_sampler (ClampToEdge, Linear filtering).
//     — levels = 6 (covers 64× radius at native resolution).
//     — mip_handles[i] allocated each frame from ResourcePool (transient).
//
//   pub fn update_params(&self, queue: &wgpu::Queue, params: &BloomParams)
//
//   impl Pass for BloomPass:
//     fn name(&self) -> &str { "bloom" }
//     fn record(&mut self, ctx: &mut PassContext) -> Result<(), CoreError>
//
//       DOWNSAMPLE CHAIN (bloom_down.wgsl):
//         Input: hdr framebuffer (after TAA)
//         Level 0: full res → half res  (dispatch at half res)
//         Level 1: half res → quarter   (dispatch at quarter res)
//         ...
//         Level 5: 1/32 res → 1/64 res  (dispatch at 1/64 resolution)
//         Each level: dual Kawase kernel (4 bilinear taps offset by ±0.5 and ±1.5 pixels)
//
//       UPSAMPLE CHAIN (bloom_up.wgsl):
//         Level 5: 1/64 res → 1/32 res  (dispatch at 1/32 res, accumulate)
//         ...
//         Level 0: half res → full res   (dispatch at full res, add to input)
//         Each level: tent filter (4 taps, 3×3 footprint)
//         Accumulate = add current level to upsampled result from previous level
//
//       FINAL: full-res bloom output added to framebuffer in last upsample pass.
//
// DUAL KAWASE KERNEL (each downsample tap):
//   Sample at: (uv + offset * (1.0 ± 0.5)) for 4 diagonally offset positions
//   Average the 4 samples. Repeat for the 4 alternating variants.
//   This 8-tap kernel approximates a Gaussian at a fraction of the cost.
//
// NOTES FOR AI:
//   - mip_handles[i] are TRANSIENT — allocated from ResourcePool each frame
//     and released at frame end. Each mip is half the previous resolution:
//       mip[0]: width/2  × height/2  (Rgba16Float)
//       mip[1]: width/4  × height/4
//       ...
//       mip[5]: width/64 × height/64
//   - The dispatch size for each level must use div_ceil to cover all pixels.
//   - BloomParams.threshold softening uses a quadratic knee:
//       if luma < threshold - knee: contribution = 0
//       if luma > threshold + knee: contribution = luma - threshold
//       else: quadratic blend (smooth transition)
//   - Disable bloom under memory pressure: saves ~120MB at 4K (6 mip textures).
// =============================================================================

use bytemuck::{Pod, Zeroable};
use gargantua_core::{
    errors::CoreError,
    frame::{
        pass::{ComputePass, Pass, PassContext},
        resource::{ResourceHandle, ResourcePool},
    },
    quality::preset::QualityPreset,
};
use crate::bindgroups::scene::SceneBindGroup;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BloomParams {
    pub threshold: f32,
    pub knee:      f32,
    pub intensity: f32,
    pub scatter:   f32,
    pub dirt_mask: u32,
    pub _pad:      [u32; 3],
}

pub struct BloomPass {
    down_pipeline:  wgpu::ComputePipeline,
    up_pipeline:    wgpu::ComputePipeline,
    params_buffer:  wgpu::Buffer,
    params_bg:      wgpu::BindGroup,
    linear_sampler: wgpu::Sampler,
    mip_handles:    [ResourceHandle; 6],
    input_handle:   ResourceHandle,
    output_handle:  ResourceHandle,
    reads:          Vec<ResourceHandle>,
    writes:         Vec<ResourceHandle>,
    levels:         u32,
    workgroup_x:    u32,
    workgroup_y:    u32,
}

impl BloomPass {
    pub fn new(
        device:        &wgpu::Device,
        down_shader:   &wgpu::ShaderModule,
        up_shader:     &wgpu::ShaderModule,
        scene_bg:      &SceneBindGroup,
        input_handle:  ResourceHandle,
        output_handle: ResourceHandle,
        params:        BloomParams,
        preset:        &QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &BloomParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

impl Pass for BloomPass {
    fn name(&self) -> &str { "bloom" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}

impl ComputePass for BloomPass {}