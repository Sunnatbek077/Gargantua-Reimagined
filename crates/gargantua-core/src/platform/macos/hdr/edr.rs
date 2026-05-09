// =============================================================================
// crates/gargantua-core/src/platform/macos/hdr/edr.rs
// =============================================================================
//
// PURPOSE:
//   Manages macOS Extended Dynamic Range (EDR) output. EDR allows pixel
//   values above 1.0 (standard white) to drive the display to peak
//   brightness (up to 1600 nits on Pro Display XDR, ~1000 nits on M-series
//   built-in display).
//
//   In Gargantua, EDR is used to render the accretion disk's innermost
//   photon sphere at full physical brightness — brighter than any SDR
//   display can show, creating a visually striking effect that matches
//   the scientific simulation.
//
//   EDR headroom (the ratio of peak brightness to SDR white) is dynamic —
//   macOS adjusts it based on display settings, ambient light, and content.
//   This module monitors headroom changes and notifies the render pipeline.
//
// SIZE: ~220 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::display_p3::{ColorSpaceUniforms, build_color_uniforms}
//     - super::pro_display_xdr::ProDisplayXdr           — XDR-specific config
//     - crate::platform::macos::gpu::adapter_metal::configure_metal_layer
//     - crate::errors::CoreError
//   External:
//     - objc2_foundation::{NSNotificationCenter, NSNotification, NSString}
//     - objc2::{msg_send, class}
//     - core_graphics::{CGDirectDisplayID, CGDisplayCopyDisplayInfo}
//     - std::sync::{Arc, Mutex}
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()
//       — creates EdrOutput after surface configuration
//   - crates/gargantua-render/src/postfx/tonemap.rs
//       — queries edr_headroom() each frame
//   - crates/gargantua-app/src/app.rs
//       — calls poll_headroom_change() each frame
//
// PUBLIC TYPES:
//
//   pub struct EdrOutput {
//     headroom:        Arc<Mutex<f32>>,  // current EDR headroom (peak/SDR ratio)
//     peak_nits:       Arc<Mutex<f32>>,  // absolute peak luminance in nits
//     is_xdr_display:  bool,             // true for Pro Display XDR
//     color_uniforms:  ColorSpaceUniforms,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(adapter: &wgpu::Adapter) -> Result<Self, CoreError>
//     — queries the current EDR headroom from the NSScreen:
//         let screen = NSScreen::mainScreen();
//         let headroom = screen.maximumExtendedDynamicRangeColorComponentValue();
//         — returns f32 >= 1.0:
//             non-EDR display: 1.0
//             M-series built-in display: 1.0 to ~5.0 (brightness-dependent)
//             Pro Display XDR: 1.0 to ~8.0
//     — detects if connected to Pro Display XDR via display ID check.
//     — registers for NSWindowDidChangeScreenNotification to detect
//       when the window moves to a different display.
//     — registers for
//       NSApplicationDidChangeScreenParametersNotification for headroom changes.
//     — builds initial ColorSpaceUniforms via build_color_uniforms().
//     — returns CoreError::PlatformError if NSScreen query fails.
//
//   pub fn edr_headroom(&self) -> f32
//     — returns the current headroom value (thread-safe).
//     — called every frame by tonemap.rs to set the headroom uniform.
//     — headroom = peak_nits / sdr_white_nits
//         e.g. 1000 nits peak / 200 nits SDR white = 5.0 headroom
//
//   pub fn peak_nits(&self) -> f32
//     — returns absolute peak luminance in nits.
//     — used for physical calibration of accretion disk brightness.
//
//   pub fn is_edr_active(&self) -> bool
//     — returns true if headroom > 1.0 (display is capable of EDR output).
//     — used by the UI to show/hide the EDR badge in the stats bar.
//
//   pub fn poll_headroom_change(&mut self) -> Option<f32>
//     — checks if the headroom has changed since last call.
//     — returns Some(new_headroom) if changed, None if stable.
//     — called once per frame by App. If Some, tonemap.rs rebuilds its
//       uniforms and uploads ColorSpaceUniforms to the GPU.
//
//   pub fn color_uniforms(&self) -> &ColorSpaceUniforms
//     — returns the current ColorSpaceUniforms for upload to the GPU.
//     — updated whenever headroom changes via poll_headroom_change().
//
// NOTES FOR AI:
//   - EDR headroom is a float returned by NSScreen. It is NOT the same as
//     HDR10 MaxCLL/MaxFALL metadata — macOS manages this automatically.
//   - The tonemap WGSL shader reads headroom as a uniform and scales
//     bright pixels (above 1.0 in scene linear light) proportionally:
//       output_linear = scene_linear / headroom  (maps peak to display peak)
//   - On non-EDR displays (headroom = 1.0), Gargantua uses ACES RRT/ODT
//     to compress scene values into the 0..1 SDR range (tonemap.wgsl).
//   - NSScreen and NSNotificationCenter are ObjC APIs — use objc2 crate.
//     All msg_send! calls are unsafe; document each with a SAFETY comment.
//   - headroom changes smoothly as macOS adjusts it. Update ColorSpaceUniforms
//     every frame when headroom != previous_headroom (not just on notification).
// =============================================================================

#![cfg(target_os = "macos")]

use std::sync::{Arc, Mutex};
use crate::{errors::CoreError, platform::macos::hdr::display_p3::ColorSpaceUniforms};

pub struct EdrOutput {
    headroom:       Arc<Mutex<f32>>,
    peak_nits:      Arc<Mutex<f32>>,
    is_xdr_display: bool,
    color_uniforms: ColorSpaceUniforms,
    last_headroom:  f32,
}

impl EdrOutput {
    pub fn new(adapter: &wgpu::Adapter) -> Result<Self, CoreError> {
        todo!()
    }

    pub fn edr_headroom(&self) -> f32 {
        *self.headroom.lock().unwrap()
    }

    pub fn peak_nits(&self) -> f32 {
        *self.peak_nits.lock().unwrap()
    }

    pub fn is_edr_active(&self) -> bool {
        self.edr_headroom() > 1.0
    }

    pub fn poll_headroom_change(&mut self) -> Option<f32> {
        let current = self.edr_headroom();
        if (current - self.last_headroom).abs() > 0.001 {
            self.last_headroom = current;
            Some(current)
        } else {
            None
        }
    }

    pub fn color_uniforms(&self) -> &ColorSpaceUniforms {
        &self.color_uniforms
    }
}