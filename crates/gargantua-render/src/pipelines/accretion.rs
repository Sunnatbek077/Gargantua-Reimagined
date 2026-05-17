// =============================================================================
// crates/gargantua-render/src/pipelines/accretion.rs
// =============================================================================
//
// PURPOSE:
//   Renders the accretion disk emission using the Novikov-Thorne thin disk
//   model. Computes the temperature profile T(r) → blackbody spectrum →
//   relativistic Doppler shift → observed intensity for each point on the disk.
//
//   The accretion disk is the primary light source in the scene — the bright
//   ring around the black hole. Physical accuracy is critical here:
//   the temperature gradient, Doppler beaming, and gravitational redshift
//   must match the Novikov-Thorne model (as used in the 2019 EHT M87* paper).
//
//   Runs the WGSL shader: shaders/render/accretion_disk.wgsl (440 lines)
//
// SIZE: ~320 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::bindgroups::scene::SceneBindGroup            — group(0)
//     - crate::bindgroups::textures::TexturesBindGroup      — group(1)
//     - gargantua_core::frame::pass::{Pass, PassContext, ComputePass}
//     - gargantua_core::frame::resource::ResourceHandle
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, ComputePipeline, Buffer, BufferUsages}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs
//       — registered after GeodesicGpuPass in the FrameGraph
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct AccretionParams {
//     pub mass:           f32,   // black hole mass M
//     pub spin:           f32,   // Kerr spin a/M
//     pub accretion_rate: f32,   // mass accretion rate Mdot (Eddington units)
//     pub r_isco:         f32,   // inner disk edge = ISCO radius
//     pub r_outer:        f32,   // outer disk edge (typically 20-50 M)
//     pub disk_height:    f32,   // half-thickness h/r (typically 0.01-0.1)
//     pub efficiency:     f32,   // radiative efficiency η (NT model: ~0.057 for a=0)
//     pub corona_frac:    f32,   // fraction of luminosity in corona (0.0..1.0)
//     pub turbulence:     f32,   // MHD turbulence amplitude (0.0..1.0)
//     pub inner_blend:    f32,   // smooth blend width at ISCO edge (in M units)
//     pub _pad:           [f32; 2],
//   }
//
//   pub struct AccretionPass {
//     pipeline:      wgpu::ComputePipeline,
//     params_buffer: wgpu::Buffer,
//     params_bg:     wgpu::BindGroup,
//     output_handle: ResourceHandle,   // accumulates into ray_march output
//     reads:         Vec<ResourceHandle>,
//     writes:        Vec<ResourceHandle>,
//     workgroup_x:   u32,
//     workgroup_y:   u32,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:   &wgpu::Device,
//     shader:   &wgpu::ShaderModule,  // accretion_disk.wgsl
//     scene_bg: &SceneBindGroup,
//     tex_bg:   &TexturesBindGroup,
//     output:   ResourceHandle,
//     params:   AccretionParams,
//     preset:   &QualityPreset,
//   ) -> Result<Self, CoreError>
//     — pipeline layout:
//         group(0): SceneUniforms (camera, timing, quality)
//         group(1): baked textures (blackbody_lut, doppler_lut, blue_noise_3d)
//         group(2): AccretionParams uniform
//         group(3): output storage texture (Rgba16Float, read_write)
//           NOTE: accretion pass ADDS to the framebuffer (accumulate mode)
//           so output texture needs StorageTextureAccess::ReadWrite.
//
//   pub fn update_params(&self, queue: &wgpu::Queue, params: &AccretionParams)
//     — queue.write_buffer for AccretionParams uniform.
//     — called when preset changes (e.g., M87 → Sgr A*).
//
//   impl Pass for AccretionPass:
//     fn name(&self) -> &str { "accretion" }
//     fn record(&mut self, ctx: &mut PassContext) -> Result<(), CoreError>
//       — dispatches accretion_disk.wgsl to add disk emission to framebuffer.
//
// PHYSICS MODEL (Novikov-Thorne, implemented in accretion_disk.wgsl):
//
//   Temperature profile:
//     T(r) = T_max × f(r) where:
//     T_max = (3 G M Mdot / (8π σ r_isco³))^(1/4) × correction_factor
//     f(r) = [(r/r_isco)^(-3) × (1 - sqrt(r_isco/r))]^(1/4)  — NT profile
//     f(r) → 0 at r = r_isco (inner edge boundary condition)
//     f(r) → r^(-3/4) at large r (Stefan-Boltzmann radiation)
//
//   Doppler beaming:
//     The disk rotates at Keplerian velocity v_φ = sqrt(M/r³)/(1+...) (Kerr correction)
//     Relativistic beaming factor: δ = 1/(γ(1 - β cos θ))^4  (4th power for intensity)
//     β = v_φ / c, γ = 1/sqrt(1-β²), θ = angle between velocity and line of sight
//
//   Gravitational redshift:
//     z_grav = 1/sqrt(1 - r_s/r) where r_s = 2M (Schwarzschild radius)
//     In Kerr metric: more complex — uses full g_tt + g_tφ² / g_φφ term
//
//   Combined observed temperature: T_obs = T(r) × δ / (1+z_grav)
//   Observed intensity: I_obs = B_ν(T_obs) × (1+z_grav)^(-4)  (Liouville's theorem)
//
// NOTES FOR AI:
//   - accretion_disk.wgsl uses blackbody_lut (group(1), binding 1) to convert
//     T_obs → RGB color. The LUT covers 1000K to 1e8K on a log scale.
//   - doppler_lut (group(1), binding 2) is used for wavelength-dependent
//     Doppler shifts (spectrally accurate color shift, not just brightness).
//   - blue_noise_3d (group(1), binding 3) is used to add turbulence to the
//     disk surface density for a more realistic appearance.
//   - r_outer is typically set to 50M in presets. Beyond this distance,
//     the disk emission is negligible and wastes compute time.
//   - corona_frac > 0 adds a hot, diffuse corona above the disk.
//     corona emission is grey (blackbody at T_corona ≈ 1e8 K → X-ray regime,
//     remapped to visible blue).
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
pub struct AccretionParams {
    pub mass:           f32,
    pub spin:           f32,
    pub accretion_rate: f32,
    pub r_isco:         f32,
    pub r_outer:        f32,
    pub disk_height:    f32,
    pub efficiency:     f32,
    pub corona_frac:    f32,
    pub turbulence:     f32,
    pub inner_blend:    f32,
    pub _pad:           [f32; 2],
}

impl AccretionParams {
    /// Novikov-Thorne radiative efficiency η for Kerr parameter a/M
    pub fn nt_efficiency(spin_normalized: f32) -> f32 {
        // Approximate formula: η ≈ 1 - sqrt(1 - 2/(3 r_isco/M))
        // For a/M=0 (Schwarzschild): η ≈ 0.0572
        // For a/M=1 (extreme Kerr):  η ≈ 0.4238
        let r_isco_over_m = Self::isco_radius_normalized(spin_normalized);
        1.0 - (1.0 - 2.0 / (3.0 * r_isco_over_m)).sqrt()
    }

    /// ISCO radius in units of M for prograde orbit
    pub fn isco_radius_normalized(spin: f32) -> f32 {
        // Bardeen-Press-Teukolsky formula (approximate for prograde)
        let z1 = 1.0 + (1.0 - spin * spin).powf(1.0 / 3.0)
            * ((1.0 + spin).powf(1.0 / 3.0) + (1.0 - spin).powf(1.0 / 3.0));
        let z2 = (3.0 * spin * spin + z1 * z1).sqrt();
        3.0 + z2 - ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt()
    }
}

pub struct AccretionPass {
    pipeline:      wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    params_bg:     wgpu::BindGroup,
    output_handle: ResourceHandle,
    reads:         Vec<ResourceHandle>,
    writes:        Vec<ResourceHandle>,
    workgroup_x:   u32,
    workgroup_y:   u32,
}

impl AccretionPass {
    pub fn new(
        device:   &wgpu::Device,
        shader:   &wgpu::ShaderModule,
        scene_bg: &SceneBindGroup,
        tex_bg:   &TexturesBindGroup,
        output:   ResourceHandle,
        params:   AccretionParams,
        preset:   &QualityPreset,
    ) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &AccretionParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

impl Pass for AccretionPass {
    fn name(&self) -> &str { "accretion" }

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError> {
        todo!()
    }

    fn read_resources(&self)  -> &[ResourceHandle] { &self.reads  }
    fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
}

impl ComputePass for AccretionPass {}