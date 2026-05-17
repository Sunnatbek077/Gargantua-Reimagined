// =============================================================================
// crates/gargantua-core/src/platform/windows/hdr/display_detect.rs
// =============================================================================
//
// PURPOSE:
//   Detects HDR-capable displays connected on Windows and queries their
//   capabilities: maximum luminance, minimum luminance, color gamut, and
//   whether HDR output is currently enabled by the user in Windows Settings.
//
//   Windows requires the user to manually enable HDR per-display in
//   Settings → System → Display → HDR. This module checks that state and
//   notifies the render pipeline so it can switch between SDR and HDR
//   tonemapping paths accordingly.
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::hdr10::Hdr10Output
//     - super::dolby_vision::DolbyVisionOutput
//     - crate::errors::CoreError
//   External:
//     - windows_sys::Win32::Graphics::Dxgi::{
//         IDXGIOutput6, DXGI_OUTPUT_DESC1,
//         DXGI_COLOR_SPACE_TYPE,
//         DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020,  // HDR10
//         DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709 }    // SDR sRGB
//     - windows_sys::Win32::Graphics::Gdi::HMONITOR
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()   — Windows branch
//   - crates/gargantua-core/src/app.rs
//       — calls poll_hdr_state() each frame to detect HDR toggle
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum HdrMode {
//     Sdr,          // HDR disabled by user in Windows Settings
//     Hdr10,        // HDR10 (PQ/ST.2084 + BT.2020)
//     DolbyVision,  // Dolby Vision (if display + driver support it)
//   }
//
//   pub struct DisplayHdrInfo {
//     pub mode:              HdrMode,
//     pub max_luminance:     f32,    // nits — peak brightness the display supports
//     pub min_luminance:     f32,    // nits — black level
//     pub max_full_frame:    f32,    // nits — sustained brightness (full white screen)
//     pub color_space:       ColorSpace,
//     pub monitor_handle:    usize,  // HMONITOR as usize (opaque)
//     pub display_name:      String, // e.g. "LG OLED C3 27\""
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum ColorSpace {
//     SrgbLinear,   // BT.709 primaries, linear gamma — internal render space
//     SrgbGamma,    // BT.709 primaries, sRGB gamma  — SDR display output
//     Bt2020Pq,     // BT.2020 primaries, PQ gamma   — HDR10 output
//     DisplayP3,    // P3 primaries, sRGB gamma       — wide gamut SDR
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn enumerate_displays() -> Vec<DisplayHdrInfo>
//     — enumerates all connected displays via IDXGIOutput6:
//         factory.EnumOutputs(i, &mut output_ptr)
//         output6 = output.cast::<IDXGIOutput6>()
//         output6.GetDesc1(&mut desc)
//     — for each display, reads DXGI_OUTPUT_DESC1:
//         .MaxLuminance          → max_luminance
//         .MinLuminance          → min_luminance
//         .MaxFullFrameLuminance → max_full_frame
//         .ColorSpace            → maps to HdrMode and ColorSpace
//     — HDR10 condition: desc.ColorSpace ==
//         DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020
//     — SDR condition: desc.ColorSpace ==
//         DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709
//     — returns empty Vec if no displays found.
//
//   pub fn primary_display() -> Option<DisplayHdrInfo>
//     — returns the display where the Gargantua window is located.
//     — determined by matching HMONITOR of the winit window handle
//       with the HMONITOR in DXGI_OUTPUT_DESC1.Monitor.
//     — returns None if the window is not yet shown (unlikely but safe).
//
//   pub fn is_hdr_enabled() -> bool
//     — returns true if primary_display().mode == HdrMode::Hdr10
//       or HdrMode::DolbyVision.
//     — called each frame to detect HDR state changes.
//
//   pub fn poll_hdr_state(previous: HdrMode) -> Option<HdrMode>
//     — re-enumerates displays and checks if mode changed from previous.
//     — returns Some(new_mode) if changed (e.g., user toggled HDR in Settings).
//     — returns None if unchanged.
//     — cheap: DXGI_OUTPUT_DESC1 query is fast (no GPU work).
//
// NOTES FOR AI:
//   - IDXGIOutput6 requires DXGI 1.6 (Windows 10 1803+). Safe on modern systems.
//   - HMONITOR must be compared by value, not pointer — it is an opaque handle.
//     Use MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) to get the
//     window's HMONITOR and compare to desc.Monitor from DXGI.
//   - HDR can be toggled by the user at any time without app restart.
//     poll_hdr_state() must be called every frame (cheap operation).
//     When a toggle is detected, the tonemap pipeline must be rebuilt
//     (different shader path for HDR10 vs SDR).
//   - max_luminance from DXGI is the display's physical capability, not the
//     current headroom like macOS EDR. The headroom concept does not apply
//     on Windows — the app must manually tone-map to max_luminance.
//   - On displays with max_luminance = 0: HDR is not supported or the driver
//     did not fill the field. Treat as SDR (max_luminance = 203 nits standard).
// =============================================================================

#![cfg(target_os = "windows")]

use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrMode {
    Sdr,
    Hdr10,
    DolbyVision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    SrgbLinear,
    SrgbGamma,
    Bt2020Pq,
    DisplayP3,
}

pub struct DisplayHdrInfo {
    pub mode:           HdrMode,
    pub max_luminance:  f32,
    pub min_luminance:  f32,
    pub max_full_frame: f32,
    pub color_space:    ColorSpace,
    pub monitor_handle: usize,
    pub display_name:   String,
}

pub fn enumerate_displays() -> Vec<DisplayHdrInfo> {
    todo!()
}

pub fn primary_display() -> Option<DisplayHdrInfo> {
    todo!()
}

pub fn is_hdr_enabled() -> bool {
    primary_display()
        .map(|d| d.mode != HdrMode::Sdr)
        .unwrap_or(false)
}

pub fn poll_hdr_state(previous: HdrMode) -> Option<HdrMode> {
    let current = primary_display()?.mode;
    if current != previous { Some(current) } else { None }
}