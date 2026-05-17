// =============================================================================
// crates/gargantua-core/src/platform/windows/hdr/hdr10.rs
// =============================================================================
//
// PURPOSE:
//   Implements HDR10 output on Windows. HDR10 is the standard HDR format
//   for Windows gaming and content: BT.2020 color primaries, PQ (ST.2084)
//   electro-optical transfer function, 10-bit per channel output.
//
//   Gargantua's internal render space is linear ACEScg (scene-referred).
//   This module provides the conversion pipeline: linear ACEScg → BT.2020
//   → PQ encoding → 10-bit output, along with the WGSL uniform data
//   consumed by shaders/postfx/tonemap.wgsl.
//
//   Also sets HDR metadata (MaxCLL, MaxFALL) on the DXGI swap chain so
//   the display can calibrate its tone mapping correctly.
//
// SIZE: ~240 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::display_detect::{DisplayHdrInfo, HdrMode}
//     - crate::errors::CoreError
//   External:
//     - glam::{Mat3, Vec3}
//     - bytemuck::{Pod, Zeroable}
//     - wgpu::{Device, Queue, Buffer, TextureFormat}
//     - windows_sys::Win32::Graphics::Dxgi::{
//         IDXGISwapChain4,
//         DXGI_HDR_METADATA_TYPE_HDR10,
//         DXGI_HDR_METADATA_HDR10,
//         DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020 }
//
// CALLED BY:
//   - crate::gpu::surface::GpuSurface::new()  — Windows HDR branch
//   - crates/gargantua-render/src/postfx/tonemap.rs
//       — calls Hdr10Output::color_uniforms() for bind group data
//   - crates/gargantua-core/src/app.rs
//       — calls Hdr10Output::update_metadata() when scene brightness changes
//
// PUBLIC TYPES:
//
//   #[repr(C)]
//   #[derive(Copy, Clone, Pod, Zeroable)]
//   pub struct Hdr10Uniforms {
//     pub scene_to_bt2020: [[f32; 4]; 3],  // 3x3 matrix (ACEScg → BT.2020), padded
//     pub max_nits:         f32,            // display MaxLuminance from DXGI
//     pub min_nits:         f32,            // display MinLuminance
//     pub ui_nits:          f32,            // target nits for UI elements (203 nits = SDR white)
//     pub _pad:             f32,
//   }
//
//   pub struct Hdr10Output {
//     display:    DisplayHdrInfo,
//     uniforms:   Hdr10Uniforms,
//     max_cll:    u16,    // MaxCLL: max content light level (nits, integer)
//     max_fall:   u16,    // MaxFALL: max average frame light level (nits)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(display: DisplayHdrInfo) -> Self
//     — builds Hdr10Uniforms from the display's luminance values:
//         max_nits = display.max_luminance
//         min_nits = display.min_luminance
//         ui_nits  = 203.0  (SMPTE ST 2084 SDR reference white)
//         scene_to_bt2020 = acescg_to_bt2020_matrix()
//     — sets initial MaxCLL/MaxFALL from display.max_luminance:
//         max_cll  = display.max_luminance as u16
//         max_fall = (display.max_full_frame) as u16
//
//   pub fn color_uniforms(&self) -> &Hdr10Uniforms
//     — returns reference to Hdr10Uniforms for GPU upload.
//     — called each frame by tonemap.rs to write to the uniform buffer.
//
//   pub fn set_swapchain_color_space(
//     &self,
//     surface: &wgpu::Surface<'_>,
//   ) -> Result<(), CoreError>
//     — accesses the DX12 swap chain via wgpu HAL:
//         unsafe { surface.as_hal::<wgpu::hal::api::Dx12, _, _>(|dx12_surface| { ... }) }
//     — casts to IDXGISwapChain4.
//     — calls swap_chain.SetColorSpace1(
//         DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020)
//     — This tells DXGI the pixel values are PQ-encoded BT.2020.
//     — Must be called once after swap chain creation, before first present.
//
//   pub fn update_metadata(
//     &mut self,
//     surface:    &wgpu::Surface<'_>,
//     scene_peak: f32,   // scene peak luminance in nits (from physics simulation)
//   ) -> Result<(), CoreError>
//     — updates max_cll based on scene_peak (clamped to display.max_luminance).
//     — calls swap_chain.SetHDRMetaData(DXGI_HDR_METADATA_TYPE_HDR10, &metadata):
//         metadata.MaxContentLightLevel   = max_cll
//         metadata.MaxFrameAverageLightLevel = max_fall
//         metadata.RedPrimary, GreenPrimary, BluePrimary, WhitePoint
//           = BT.2020 chromaticity coordinates (×50000 as u16):
//             Red:   (0.708, 0.292) → (35400, 14600)
//             Green: (0.170, 0.797) → (8500,  39850)
//             Blue:  (0.131, 0.046) → (6550,  2300)
//             White: (0.3127, 0.3290) → (15635, 16450)  [D65]
//         metadata.MinMasteringLuminance = min_nits * 10000 as u32
//         metadata.MaxMasteringLuminance = max_nits * 10000 as u32 (× 10000 per DXGI spec)
//     — only submits metadata if max_cll changed by > 10 nits (avoid per-frame updates).
//
//   pub fn acescg_to_bt2020_matrix() -> glam::Mat3
//     — returns the standard ACEScg → BT.2020 D65 color primaries matrix.
//     — matrix (column-major):
//         [[ 0.6369, 0.2627, 0.0000],
//          [ 0.1447, 0.6780, 0.0282],
//          [ 0.1688, 0.0593, 1.0610]]
//     — used in both Hdr10Uniforms and tonemap.wgsl.
//
//   pub fn pq_encode(linear_nits: f32) -> f32
//     — applies the PQ (ST.2084) EOTF inverse (OETF):
//         normalized = linear_nits / 10000.0  (PQ reference white = 10000 nits)
//         m1 = 0.1593017578125
//         m2 = 78.84375
//         c1 = 0.8359375
//         c2 = 18.8515625
//         c3 = 18.6875
//         pq = ((c1 + c2 * normalized^m1) / (1 + c3 * normalized^m1))^m2
//     — called in tonemap.wgsl (GPU-side, this Rust fn is for unit tests).
//
// NOTES FOR AI:
//   - PQ encoding happens in tonemap.wgsl on the GPU. This Rust pq_encode()
//     function is only for CPU-side verification in unit tests.
//   - DXGI SetHDRMetaData must be called on the swap chain thread (render thread).
//     It is NOT thread-safe — call only from App::render_frame().
//   - max_nits from DXGI may be 0 on some displays/drivers. If 0, fall back
//     to 1000 nits as a safe default for HDR10 content.
//   - BT.2020 chromaticity coordinates in DXGI are scaled by 50000 (u16),
//     NOT 10000. This is a DXGI quirk — read the DXGI_HDR_METADATA_HDR10 docs.
//   - DXGI MaxMasteringLuminance is in units of 0.0001 nits (× 10000).
//     So 1000 nits → MaxMasteringLuminance = 10,000,000.
// =============================================================================

#![cfg(target_os = "windows")]

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec3};
use crate::{errors::CoreError, platform::windows::hdr::display_detect::DisplayHdrInfo};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Hdr10Uniforms {
    pub scene_to_bt2020: [[f32; 4]; 3],
    pub max_nits:        f32,
    pub min_nits:        f32,
    pub ui_nits:         f32,
    pub _pad:            f32,
}

pub struct Hdr10Output {
    display:  DisplayHdrInfo,
    uniforms: Hdr10Uniforms,
    max_cll:  u16,
    max_fall: u16,
}

impl Hdr10Output {
    pub fn new(display: DisplayHdrInfo) -> Self {
        todo!()
    }

    pub fn color_uniforms(&self) -> &Hdr10Uniforms {
        &self.uniforms
    }

    pub fn set_swapchain_color_space(
        &self,
        surface: &wgpu::Surface<'_>,
    ) -> Result<(), CoreError> {
        todo!()
    }

    pub fn update_metadata(
        &mut self,
        surface:    &wgpu::Surface<'_>,
        scene_peak: f32,
    ) -> Result<(), CoreError> {
        todo!()
    }
}

pub fn acescg_to_bt2020_matrix() -> Mat3 {
    Mat3::from_cols(
        Vec3::new(0.6369, 0.2627, 0.0000),
        Vec3::new(0.1447, 0.6780, 0.0282),
        Vec3::new(0.1688, 0.0593, 1.0610),
    )
}

pub fn pq_encode(linear_nits: f32) -> f32 {
    let normalized = linear_nits / 10000.0;
    let m1: f32 = 0.159_301_758;
    let m2: f32 = 78.843_75;
    let c1: f32 = 0.835_937_5;
    let c2: f32 = 18.851_562_5;
    let c3: f32 = 18.687_5;
    let n = normalized.powf(m1);
    ((c1 + c2 * n) / (1.0 + c3 * n)).powf(m2)
}