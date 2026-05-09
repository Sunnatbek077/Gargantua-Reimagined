// =============================================================================
// crates/gargantua-core/src/platform/macos/hdr/display_p3.rs
// =============================================================================
//
// PURPOSE:
//   Implements the color space transformation from the internal linear
//   scene-referred working space (ACEScg / linear Rec.2020) to the
//   Display P3 color gamut used by all Apple Silicon Mac displays since 2016.
//
//   Display P3 has a wider gamut than sRGB (covers ~26% more colors) and
//   is the native color space of all Apple Retina displays. Rendering in
//   linear light and converting to Display P3 at output ensures accurate
//   color reproduction of the accretion disk's thermal emission spectrum.
//
//   The WGSL tonemap shader (shaders/postfx/tonemap.wgsl) calls these
//   transform matrices as uniform data uploaded by this module.
//
// SIZE: ~160 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - glam::{Mat3, Vec3}    — 3×3 color matrix math
//     - wgpu::{Device, Queue, Buffer, BufferUsages}
//     - bytemuck::{Pod, Zeroable}
//
// CALLED BY:
//   - crate::platform::macos::hdr::edr::EdrOutput::new()
//       — queries p3_transform_matrix() for the tonemap uniform
//   - crates/gargantua-render/src/postfx/tonemap.rs
//       — uploads ColorSpaceUniforms to the tonemap bind group
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct ColorSpaceUniforms {
//     pub scene_to_p3:   [[f32; 4]; 3],   // 3×3 matrix padded to 3×4 for WGSL
//     pub p3_to_linear:  [[f32; 4]; 3],   // inverse (used for TAA history blend)
//     pub p3_white_point: [f32; 4],       // D65 white point in P3 (padded)
//     pub peak_nits:      f32,            // display peak luminance (nits)
//     pub sdr_white_nits: f32,            // SDR white level (typically 200 nits)
//     pub _pad:          [f32; 2],
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn p3_transform_matrix() -> glam::Mat3
//     — returns the 3×3 linear RGB → Display P3 transform matrix.
//     — matrix values (column-major, standard ICC transform):
//         ACEScg linear → XYZ D65 → Display P3
//         [[ 0.4865,  0.2657,  0.1982],
//          [ 0.2290,  0.6917,  0.0793],
//          [ 0.0000,  0.0451,  1.0439]]
//     — this is the standard IEC 61966-2-1 P3-D65 primaries matrix.
//
//   pub fn linear_to_p3(color: glam::Vec3) -> glam::Vec3
//     — applies p3_transform_matrix() to a linear RGB color.
//     — used in unit tests to verify the matrix is correct.
//     — NOT called at runtime (GPU does this in tonemap.wgsl).
//
//   pub fn p3_gamma_encode(linear: f32) -> f32
//     — applies Display P3 gamma (same curve as sRGB: γ≈2.2 with linear toe):
//         if linear <= 0.0031308: return linear * 12.92
//         else: return 1.055 * linear.powf(1.0/2.4) - 0.055
//     — used for CPU-side verification and LUT generation.
//
//   pub fn build_color_uniforms(
//     peak_nits:      f32,
//     sdr_white_nits: f32,
//   ) -> ColorSpaceUniforms
//     — constructs the ColorSpaceUniforms struct from:
//         p3_transform_matrix() → scene_to_p3
//         p3_transform_matrix().inverse() → p3_to_linear
//         D65 white point → p3_white_point
//         peak_nits, sdr_white_nits from arguments
//     — peak_nits: queried from EDR display at runtime via edr.rs
//         (typical values: 1000 nits for XDR, 500 nits for MBP built-in)
//     — sdr_white_nits: 200 nits (macOS EDR default for SDR content)
//
//   pub fn upload_color_uniforms(
//     device: &wgpu::Device,
//     queue:  &wgpu::Queue,
//     uniforms: &ColorSpaceUniforms,
//     buffer: &wgpu::Buffer,
//   )
//     — calls queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniforms)).
//     — buffer must have been created with BufferUsages::UNIFORM | COPY_DST.
//     — called once per frame if peak_nits changes (e.g., EDR headroom update).
//
// NOTES FOR AI:
//   - All color math is in linear light (no gamma). Gamma encoding happens
//     only at the very end in tonemap.wgsl after all post-processing.
//   - The WGSL matrix layout is column-major [[f32; 4]; 3] (3 columns of 4).
//     The 4th element of each column is padding (WGSL vec4 alignment rules).
//   - peak_nits is not constant — it changes as the OS adjusts EDR headroom
//     based on display brightness and content. Update ColorSpaceUniforms
//     when edr.rs signals a headroom change.
//   - D65 white point in P3 chromaticity: x=0.3127, y=0.3290.
// =============================================================================

#![cfg(target_os = "macos")]

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ColorSpaceUniforms {
    pub scene_to_p3:    [[f32; 4]; 3],
    pub p3_to_linear:   [[f32; 4]; 3],
    pub p3_white_point: [f32; 4],
    pub peak_nits:      f32,
    pub sdr_white_nits: f32,
    pub _pad:           [f32; 2],
}

pub fn p3_transform_matrix() -> Mat3 {
    // ACEScg linear → XYZ D65 → Display P3 (column-major)
    Mat3::from_cols(
        Vec3::new( 0.4865,  0.2657,  0.1982),
        Vec3::new( 0.2290,  0.6917,  0.0793),
        Vec3::new( 0.0000,  0.0451,  1.0439),
    )
}

pub fn linear_to_p3(color: Vec3) -> Vec3 {
    p3_transform_matrix() * color
}

pub fn p3_gamma_encode(linear: f32) -> f32 {
    if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

pub fn build_color_uniforms(peak_nits: f32, sdr_white_nits: f32) -> ColorSpaceUniforms {
    todo!()
}

pub fn upload_color_uniforms(
    device:   &wgpu::Device,
    queue:    &wgpu::Queue,
    uniforms: &ColorSpaceUniforms,
    buffer:   &wgpu::Buffer,
) {
    queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniforms));
}