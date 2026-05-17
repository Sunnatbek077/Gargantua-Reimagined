// =============================================================================
// crates/gargantua-render/src/pipelines/starfield.rs
// =============================================================================
//
// PURPOSE:
//   Renders the background star field from the baked equirectangular HDR
//   starmap texture (generated from the Tycho2 catalogue, 2.5M stars).
//   Runs shaders/render/starfield.wgsl (200 lines).
//
//   The starfield pass renders BEFORE accretion and lensing. It provides
//   the base background that the lensing pass then distorts. Stars in
//   directions unaffected by lensing are drawn directly from the starmap;
//   stars near the black hole are handled by the lensing pass.
//
//   Also reads the spherical harmonic coefficients (sh_coeffs.bin) for
//   a fast low-frequency ambient light approximation at large distances
//   from the photon sphere.
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup        — group(0)
//     - crate::bindgroups::textures::TexturesBindGroup  — group(1) starmap
//     - gargantua_core::frame::pass::{Pass, PassContext, ComputePass}
//     - gargantua_core::frame::resource::ResourceHandle
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Buffer}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs
//       — first pass registered (before accretion and lensing)
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct StarfieldParams {
//     pub exposure:      f32,   // star brightness multiplier (default 1.0)
//     pub aberration:    f32,   // relativistic aberration β (camera velocity / c)
//     pub cutoff_radius: f32,   // angular radius where lensing takes over (radians)
//     pub sh_blend:      f32,   // blend between starmap and SH approximation (0..1)
//     pub color_temp:    f32,   // global color temperature shift in Kelvin (0=neutral)
//     pub _pad:          [f32; 3],
//   }
//
//   pub struct StarfieldPass {
//     pipeline:        wgpu::ComputePipeline,
//     params_buffer:   wgpu::Buffer,
//     params_bg:       wgpu::BindGroup,
//     sh_buffer:       wgpu::Buffer,      // SH coefficients (L0..L2, 9 × vec3)
//     sh_bg:           wgpu::BindGroup,
//     output_handle:   ResourceHandle,
//     reads:           Vec<ResourceHandle>,
//     writes:          Vec<ResourceHandle>,
//     workgroup_x:     u32,
//     workgroup_y:     u32,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:    &wgpu::Device,
//     shader:    &wgpu::ShaderModule,  // starfield.wgsl
//     scene_bg:  &SceneBindGroup,
//     tex_bg:    &TexturesBindGroup,
//     output:    ResourceHandle,
//     sh_coeffs: &[f32],              // 27 floats (9 × RGB) loaded from sh_coeffs.bin
//     params:    StarfieldParams,
//     preset:    &QualityPreset,
//   ) -> Result<Self, CoreError>
//     — uploads sh_coeffs to sh_buffer (UNIFORM | COPY_DST).
//     — pipeline layout:
//         group(0): SceneUniforms
//         group(1): baked textures (starmap at binding 4)
//         group(2): StarfieldParams uniform
//         group(3): SH coefficients uniform buffer
//         group(4): output storage texture (write-only, clears to starfield)
//
//   pub fn update_params(&self, queue: &wgpu::Queue, params: &StarfieldParams)
//
//   impl Pass: name = "starfield"
//     — dispatches starfield.wgsl.
//     — NOTE: this pass WRITES (not adds) to the framebuffer. It runs first
//       and establishes the base color. All subsequent passes add to it.
//
// WGSL INTERFACE (starfield.wgsl):
//   @group(0) @binding(0) var<uniform> scene:    SceneUniforms;
//   @group(1) @binding(4) var starmap:            texture_2d<f32>;
//   @group(1) @binding(5) var lut_sampler:        sampler;
//   @group(2) @binding(0) var<uniform> params:    StarfieldParams;
//   @group(3) @binding(0) var<uniform> sh_coeffs: array<vec4<f32>, 9>;
//   @group(4) @binding(0) var output: texture_storage_2d<rgba16float, write>;
//   // Converts screen UV → world ray direction (via scene.inv_view_proj)
//   // Applies relativistic aberration if params.aberration > 0
//   // Samples starmap at lensed direction for non-lensed region
//   // Uses SH approximation for smooth ambient background
//   // Applies exposure multiplier and writes to output
//
// NOTES FOR AI:
//   - The starmap texture is in equirectangular projection (longitude-latitude).
//     UV = (atan2(dir.z, dir.x)/(2π) + 0.5, acos(dir.y)/π).
//   - aberration param: 0.0 for stationary observer. Set to camera's
//     physical velocity / c for relativistic aberration (used in free_flight mode).
//   - cutoff_radius: pixels where the angular distance to the black hole
//     is less than this value are set to black (handled by lensing pass).
//   - sh_coeffs: 9 vec3 RGB coefficients for L0 (ambient), L1 (directional),
//     and L2 (quadratic) spherical harmonics. Stored as vec4 (w=0) for
//     alignment. Generated by gargantua-bake/src/irradiance/sh_coeffs.rs.
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
pub struct StarfieldParams {
    pub exposure:      f32,
    pub aberration:    f32,
    pub cutoff_radius: f32,
    pub sh_blend:      f32,
    pub color_temp:    f32,
    pub _pad:          [f32; 3],
}

pub struct StarfieldPass {
    pipeline:      wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    params_bg:     wgpu::BindGroup,
    sh_buffer:     wgpu::Buffer,
    sh_bg:         wgpu::BindGroup,
    output_handle: ResourceHandle,
    reads:         Vec<ResourceHandle>,
    writes:        Vec<ResourceHandle>,
    workgroup_x:   u32,
    workgroup_y:   u32,
}

impl StarfieldPass {
    pub fn new(
        device:    &wgpu::Device,
        shader:    &wgpu::ShaderModule,
        scene_bg:  &SceneBindGroup,
        tex_bg:    &TexturesBindGroup,
        output:    ResourceHandle,
        sh_coeffs: &[f32],
        params:    StarfieldParams,
        preset:    &QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &StarfieldParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

impl Pass for StarfieldPass {
    fn name(&self) -> &str { "starfield" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}

impl ComputePass for StarfieldPass {}