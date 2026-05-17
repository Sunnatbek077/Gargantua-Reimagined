// =============================================================================
// crates/gargantua-render/src/hdr.rs
// =============================================================================
//
// PURPOSE:
//   Cross-platform HDR output coordinator. Abstracts the platform-specific
//   HDR setup (macOS EDR vs Windows HDR10 vs SDR fallback) behind a single
//   HdrOutput enum that tonemap.rs queries each frame.
//
//   Determines at startup which HDR mode is active and provides the correct
//   color space uniforms (ColorSpaceUniforms or Hdr10Uniforms) for the
//   tonemap shader. Also polls each frame for HDR state changes (user toggling
//   HDR in System Settings / Windows Settings).
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_core::gpu::context::GpuContext
//     - gargantua_core::gpu::surface::GpuSurface
//     - crate::errors::RenderError
//     #[cfg(target_os = "macos")]
//     - gargantua_core::platform::macos::hdr::edr::EdrOutput
//     - gargantua_core::platform::macos::hdr::display_p3::ColorSpaceUniforms
//     - gargantua_core::platform::macos::hdr::pro_display_xdr::ProDisplayXdr
//     #[cfg(target_os = "windows")]
//     - gargantua_core::platform::windows::hdr::display_detect::{DisplayHdrInfo, HdrMode}
//     - gargantua_core::platform::windows::hdr::hdr10::Hdr10Output
//     - gargantua_core::platform::windows::hdr::dolby_vision::DolbyVisionOutput
//   External:
//     - wgpu::{Device, Queue, Buffer}
//     - bytemuck::bytes_of
//
// CALLED BY:
//   - crates/gargantua-core/src/app.rs::App::new()   — creates HdrOutput
//   - crates/gargantua-render/src/postfx/tonemap.rs
//       — calls HdrOutput::uniforms_buffer() to get the color space buffer
//   - crates/gargantua-core/src/app.rs::App::render_frame()
//       — calls HdrOutput::poll() to detect HDR state changes
//
// PUBLIC TYPES:
//
//   pub enum HdrOutput {
//     #[cfg(target_os = "macos")]
//     MacOsEdr {
//       edr:      EdrOutput,
//       xdr:      ProDisplayXdr,
//       buffer:   wgpu::Buffer,  // ColorSpaceUniforms GPU buffer
//     },
//     #[cfg(target_os = "windows")]
//     WindowsHdr10 {
//       output:  Hdr10Output,
//       buffer:  wgpu::Buffer,  // Hdr10Uniforms GPU buffer
//     },
//     #[cfg(target_os = "windows")]
//     WindowsDolbyVision {
//       output: DolbyVisionOutput,
//       buffer: wgpu::Buffer,
//     },
//     Sdr {
//       buffer: wgpu::Buffer,  // SDR ColorSpaceUniforms (identity matrices)
//     },
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(
//     device:  &wgpu::Device,
//     ctx:     &GpuContext,
//     surface: &GpuSurface,
//   ) -> Result<Self, RenderError>
//     — macOS: creates EdrOutput::new(&ctx.adapter), ProDisplayXdr::detect().
//       Builds initial ColorSpaceUniforms and uploads to GPU buffer.
//       Returns MacOsEdr { edr, xdr, buffer }.
//     — Windows: enumerates displays via display_detect::primary_display().
//       If HdrMode::Hdr10: creates Hdr10Output, sets swap chain color space.
//       If HdrMode::DolbyVision: creates DolbyVisionOutput.
//       If HdrMode::Sdr: returns Sdr variant.
//     — WASM: always returns Sdr (browser handles color management).
//     — On any platform error: logs warning and falls back to Sdr.
//
//   pub fn uniforms_buffer(&self) -> &wgpu::Buffer
//     — returns reference to the GPU uniform buffer containing color space data.
//     — tonemap.rs binds this buffer at group(2) binding(0).
//
//   pub fn poll(
//     &mut self,
//     device: &wgpu::Device,
//     queue:  &wgpu::Queue,
//     surface: &wgpu::Surface<'_>,
//   ) -> bool
//     — checks for HDR state changes (headroom change, user toggle, display change).
//     — macOS: calls edr.poll_headroom_change() — returns true if headroom changed.
//       Rebuilds ColorSpaceUniforms and uploads to GPU buffer if changed.
//     — Windows: calls display_detect::poll_hdr_state(prev_mode).
//       If mode changed: may need to recreate the HdrOutput variant entirely.
//       Returns true if tonemap pipeline needs rebuilding (mode changed).
//     — Sdr: always returns false (no state to poll).
//     — called once per frame; should be fast (no GPU work, just CPU queries).
//
//   pub fn is_hdr(&self) -> bool
//     — returns true if any HDR variant is active.
//     — used by the UI to show the HDR badge in the stats overlay.
//
//   pub fn peak_nits(&self) -> f32
//     — macOS: edr.peak_nits()
//     — Windows HDR10: hdr10.display.max_luminance
//     — SDR: 203.0 (SMPTE reference white)
//
//   pub fn edr_headroom(&self) -> f32
//     — macOS: edr.edr_headroom()
//     — Windows: display.max_luminance / 203.0
//     — SDR: 1.0
//     — passed to tonemap.wgsl as a uniform for HDR → display mapping.
//
// NOTES FOR AI:
//   - HdrOutput is created once at startup. If the user moves the window to
//     a different display (different HDR capability), poll() detects this
//     via NSWindowDidChangeScreenNotification (macOS) or DXGI display re-enum.
//   - The GPU buffer is UNIFORM | COPY_DST — updated via queue.write_buffer
//     when uniforms change (typically when EDR headroom changes, ~1/second).
//   - On macOS, ColorSpaceUniforms and Hdr10Uniforms have the same size
//     (matched intentionally) so tonemap.rs uses the same binding for both.
//   - RenderError::HdrOutput is non-fatal — if EDR setup fails (Intel Mac,
//     headless server), the renderer falls back to SDR silently.
// =============================================================================

use crate::errors::RenderError;
use gargantua_core::{gpu::{context::GpuContext, surface::GpuSurface}};

pub enum HdrOutput {
    #[cfg(target_os = "macos")]
    MacOsEdr {
        edr:    gargantua_core::platform::macos::hdr::edr::EdrOutput,
        xdr:    gargantua_core::platform::macos::hdr::pro_display_xdr::ProDisplayXdr,
        buffer: wgpu::Buffer,
    },
    #[cfg(target_os = "windows")]
    WindowsHdr10 {
        output: gargantua_core::platform::windows::hdr::hdr10::Hdr10Output,
        buffer: wgpu::Buffer,
    },
    #[cfg(target_os = "windows")]
    WindowsDolbyVision {
        output: gargantua_core::platform::windows::hdr::dolby_vision::DolbyVisionOutput,
        buffer: wgpu::Buffer,
    },
    Sdr {
        buffer: wgpu::Buffer,
    },
}

impl HdrOutput {
    pub fn new(
        device:  &wgpu::Device,
        ctx:     &GpuContext,
        surface: &GpuSurface,
    ) -> Result<Self, RenderError> {
        todo!()
    }

    pub fn uniforms_buffer(&self) -> &wgpu::Buffer {
        match self {
            #[cfg(target_os = "macos")]
            HdrOutput::MacOsEdr { buffer, .. }         => buffer,
            #[cfg(target_os = "windows")]
            HdrOutput::WindowsHdr10 { buffer, .. }     => buffer,
            #[cfg(target_os = "windows")]
            HdrOutput::WindowsDolbyVision { buffer, .. } => buffer,
            HdrOutput::Sdr { buffer }                  => buffer,
        }
    }

    pub fn poll(
        &mut self,
        device:  &wgpu::Device,
        queue:   &wgpu::Queue,
        surface: &wgpu::Surface<'_>,
    ) -> bool {
        todo!()
    }

    pub fn is_hdr(&self) -> bool {
        !matches!(self, HdrOutput::Sdr { .. })
    }

    pub fn peak_nits(&self) -> f32 {
        match self {
            #[cfg(target_os = "macos")]
            HdrOutput::MacOsEdr { edr, .. } => edr.peak_nits(),
            #[cfg(target_os = "windows")]
            HdrOutput::WindowsHdr10 { output, .. } => output.color_uniforms().max_nits,
            _ => 203.0,
        }
    }

    pub fn edr_headroom(&self) -> f32 {
        match self {
            #[cfg(target_os = "macos")]
            HdrOutput::MacOsEdr { edr, .. } => edr.edr_headroom(),
            _ => self.peak_nits() / 203.0,
        }
    }
}