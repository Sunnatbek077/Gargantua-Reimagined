// =============================================================================
// crates/gargantua-render/src/bindgroups/textures.rs
// =============================================================================
//
// PURPOSE:
//   Defines the TexturesBindGroup — bind group 1 (group(1) in WGSL) that
//   holds all baked textures needed by the render pipeline. These textures
//   are generated once by gargantua-bake at startup and remain unchanged
//   for the lifetime of the application.
//
//   All WGSL shaders that need baked data declare:
//     @group(1) @binding(0) var geodesic_lut:  texture_2d<f32>;
//     @group(1) @binding(1) var blackbody_lut:  texture_1d<f32>;
//     @group(1) @binding(2) var doppler_lut:    texture_2d<f32>;
//     @group(1) @binding(3) var blue_noise_3d:  texture_3d<f32>;
//     @group(1) @binding(4) var starmap:         texture_2d<f32>;
//     @group(1) @binding(5) var lut_sampler:     sampler;
//     @group(1) @binding(6) var noise_sampler:   sampler;
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_core::gpu::context::GpuContext
//     - gargantua_core::errors::CoreError
//   External:
//     - wgpu::{Device, Texture, TextureView, Sampler, BindGroup,
//              BindGroupLayout, BindGroupLayoutEntry, SamplerDescriptor,
//              AddressMode, FilterMode, ShaderStages, TextureSampleType,
//              SamplerBindingType, TextureViewDimension}
//
// CALLED BY:
//   - crates/gargantua-render/src/pipelines/ray_march.rs  — group(1)
//   - crates/gargantua-render/src/pipelines/accretion.rs  — group(1)
//   - crates/gargantua-render/src/pipelines/starfield.rs  — group(1)
//   - crates/gargantua-render/src/postfx/film_grain.rs    — blue_noise_3d
//   - crates/gargantua-core/src/app.rs
//       — creates TexturesBindGroup once after bake completes
//
// PUBLIC TYPES:
//
//   pub struct BakedTextures {
//     pub geodesic_lut: wgpu::Texture,   // Spin × impact → deflection (2D, Rgba32Float)
//     pub blackbody_lut: wgpu::Texture,  // temperature → RGB (1D, Rgba16Float, 1024 pts)
//     pub doppler_lut:  wgpu::Texture,   // β × λ → shifted λ (2D, Rgba16Float)
//     pub blue_noise_3d: wgpu::Texture,  // 64³ blue noise (3D, R8Unorm)
//     pub starmap:       wgpu::Texture,  // equirect HDR starfield (2D, Rgba16Float)
//   }
//
//   pub struct TexturesBindGroup {
//     bind_group: wgpu::BindGroup,
//     layout:     wgpu::BindGroupLayout,
//     // Samplers stored to keep them alive (BindGroup holds weak refs in some backends)
//     lut_sampler:   wgpu::Sampler,
//     noise_sampler: wgpu::Sampler,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
//     — layout entries:
//         binding 0: texture_2d<f32>  — geodesic_lut   (ClampToEdge, Linear)
//         binding 1: texture_1d<f32>  — blackbody_lut  (ClampToEdge, Linear)
//         binding 2: texture_2d<f32>  — doppler_lut    (ClampToEdge, Linear)
//         binding 3: texture_3d<f32>  — blue_noise_3d  (Repeat, Nearest)
//         binding 4: texture_2d<f32>  — starmap         (Repeat, Linear)
//         binding 5: sampler          — lut_sampler     (Linear, ClampToEdge)
//         binding 6: sampler          — noise_sampler   (Nearest, Repeat)
//     — all textures visible to COMPUTE | FRAGMENT stages.
//
//   pub fn new(
//     device:   &wgpu::Device,
//     textures: &BakedTextures,
//   ) -> Self
//     — creates lut_sampler:
//         AddressMode: ClampToEdge (all axes)
//         MagFilter: Linear, MinFilter: Linear
//         MipmapFilter: Linear
//     — creates noise_sampler:
//         AddressMode: Repeat (all axes) — blue noise tiles seamlessly
//         MagFilter: Nearest, MinFilter: Nearest
//         MipmapFilter: Nearest
//     — creates TextureViews for all 5 textures.
//     — creates BindGroup with 7 entries (5 textures + 2 samplers).
//
//   pub fn bind_group(&self) -> &wgpu::BindGroup      { &self.bind_group }
//   pub fn layout(&self)     -> &wgpu::BindGroupLayout { &self.layout     }
//
// TEXTURE SPECS (must match bake pipeline output):
//   geodesic_lut:
//     Format: Rgba32Float (high precision — deflection angle accuracy critical)
//     Size: 512×512 (spin_param × impact_param — 512 samples each)
//     Mips: 1 (no mip — always sampled at same scale)
//
//   blackbody_lut:
//     Format: Rgba16Float (RGB = XYZ color, A = total luminance)
//     Size: 1024×1 (1D texture — temperature index 0..1 maps 1000K..1e8K log)
//     Mips: 1
//
//   doppler_lut:
//     Format: Rgba16Float
//     Size: 256×256 (β × wavelength_normalized)
//     Mips: 1
//
//   blue_noise_3d:
//     Format: R8Unorm (single channel, 0..1)
//     Size: 64×64×64 (3D — z-axis indexed by frame_idx % 64 in shader)
//     Mips: 1
//     Usage: TEXTURE_BINDING only (no render attachment)
//
//   starmap:
//     Format: Rgba16Float (HDR — bright stars exceed 1.0)
//     Size: 4096×2048 (equirectangular projection)
//     Mips: 8 (mipmapped — avoids aliasing at small solid angles)
//
// NOTES FOR AI:
//   - BakedTextures are created by gargantua-bake and handed to
//     TexturesBindGroup::new(). The render crate does not create textures.
//   - The noise_sampler uses Nearest filtering intentionally — the blue
//     noise pattern must not be blurred (filtering destroys the spectral
//     properties that make it low-discrepancy).
//   - starmap uses Repeat addressing in both U and V because the equirect
//     projection wraps: U=0.0 and U=1.0 are the same longitude (0°/360°).
//   - geodesic_lut uses Rgba32Float (not f16) because small errors in the
//     deflection angle cause visible artifacts near the photon sphere.
//     The extra precision is worth the memory cost (512×512×16 = ~4MB).
//   - All views use default TextureViewDescriptor (full texture, all mips).
// =============================================================================

use gargantua_core::errors::CoreError;
use wgpu::{
    AddressMode, BindGroupLayout, Device, FilterMode, SamplerDescriptor,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

pub struct BakedTextures {
    pub geodesic_lut:  wgpu::Texture,
    pub blackbody_lut: wgpu::Texture,
    pub doppler_lut:   wgpu::Texture,
    pub blue_noise_3d: wgpu::Texture,
    pub starmap:       wgpu::Texture,
}

pub struct TexturesBindGroup {
    bind_group:    wgpu::BindGroup,
    layout:        wgpu::BindGroupLayout,
    lut_sampler:   wgpu::Sampler,
    noise_sampler: wgpu::Sampler,
}

impl TexturesBindGroup {
    pub fn bind_group_layout(device: &Device) -> BindGroupLayout {
        use wgpu::{BindGroupLayoutEntry, BindingType, SamplerBindingType};
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("textures_bind_group_layout"),
            entries: &[
                // binding 0 — geodesic_lut (2D)
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None,
                },
                // binding 1 — blackbody_lut (1D)
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D1,
                        multisampled:   false,
                    },
                    count: None,
                },
                // binding 2 — doppler_lut (2D)
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None,
                },
                // binding 3 — blue_noise_3d (3D)
                BindGroupLayoutEntry {
                    binding:    3,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D3,
                        multisampled:   false,
                    },
                    count: None,
                },
                // binding 4 — starmap (2D)
                BindGroupLayoutEntry {
                    binding:    4,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None,
                },
                // binding 5 — lut_sampler
                BindGroupLayoutEntry {
                    binding:    5,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty:         BindingType::Sampler(SamplerBindingType::Filtering),
                    count:      None,
                },
                // binding 6 — noise_sampler
                BindGroupLayoutEntry {
                    binding:    6,
                    visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                    ty:         BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count:      None,
                },
            ],
        })
    }

    pub fn new(device: &Device, textures: &BakedTextures) -> Self {
        let layout = Self::bind_group_layout(device);

        let lut_sampler = device.create_sampler(&SamplerDescriptor {
            label:            Some("lut_sampler"),
            address_mode_u:   AddressMode::ClampToEdge,
            address_mode_v:   AddressMode::ClampToEdge,
            address_mode_w:   AddressMode::ClampToEdge,
            mag_filter:       FilterMode::Linear,
            min_filter:       FilterMode::Linear,
            mipmap_filter:    wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let noise_sampler = device.create_sampler(&SamplerDescriptor {
            label:            Some("noise_sampler"),
            address_mode_u:   AddressMode::Repeat,
            address_mode_v:   AddressMode::Repeat,
            address_mode_w:   AddressMode::Repeat,
            mag_filter:       FilterMode::Nearest,
            min_filter:       FilterMode::Nearest,
            mipmap_filter:    wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let geodesic_view  = textures.geodesic_lut.create_view(&Default::default());
        let blackbody_view = textures.blackbody_lut.create_view(&Default::default());
        let doppler_view   = textures.doppler_lut.create_view(&Default::default());
        let noise_view     = textures.blue_noise_3d.create_view(&Default::default());
        let starmap_view   = textures.starmap.create_view(&Default::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("textures_bind_group"),
            layout:  &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&geodesic_view)  },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&blackbody_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&doppler_view)   },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&noise_view)     },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&starmap_view)   },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&lut_sampler)        },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&noise_sampler)      },
            ],
        });

        Self { bind_group, layout, lut_sampler, noise_sampler }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup      { &self.bind_group }
    pub fn layout(&self)     -> &wgpu::BindGroupLayout { &self.layout     }
}