// ============================================================
// FILE: shaders/compute/accumulate.wgsl
// LINES: ~180
// CATEGORY: Shader — Temporal accumulation (TAA pre-pass)
// STAGE: Compute  (@compute @workgroup_size(8, 8, 1))
// ============================================================
//
// PURPOSE:
//   Accumulates multiple rendered samples over time into a single
//   high-quality frame. Each invocation blends the current frame's
//   pixel with a running history buffer using exponential moving
//   average. Drives the multi-sample per pixel (SPP) quality mode
//   and the offline render accumulation pipeline.
//
// SHADER INTERFACE:
//   @group(0) @binding(0) var<uniform>         params: AccumParams;
//   @group(0) @binding(1) var                  current_frame: texture_2d<f32>;
//   @group(0) @binding(2) var                  history:       texture_2d<f32>;
//   @group(0) @binding(3) var<storage, read_write> output:   array<vec4<f32>>;
//
//   struct AccumParams {
//       frame_idx:    u32,   // current frame index (0-based)
//       total_frames: u32,   // total SPP target (for offline render)
//       blend_alpha:  f32,   // EMA blend factor: 0=history only, 1=current only
//       width:        u32,
//       height:       u32,
//       reset:        u32,   // 1 = clear history (camera moved), 0 = accumulate
//   }
//
// ENTRY POINT:
//   @compute @workgroup_size(8, 8, 1)
//   fn main(@builtin(global_invocation_id) gid: vec3<u32>)
//     // if gid.x >= params.width || gid.y >= params.height: return
//     //
//     // coord = vec2<i32>(gid.xy)
//     // curr  = textureLoad(current_frame, coord, 0)
//     // hist  = textureLoad(history, coord, 0)
//     //
//     // if params.reset == 1u:
//     //     blended = curr   (discard history on camera move)
//     // else:
//     //     alpha   = 1.0 / f32(params.frame_idx + 1u)  // exact average
//     //     blended = mix(hist, curr, alpha)             // running mean
//     //
//     // output[gid.y * params.width + gid.x] = blended
//
// USES (imports from):
//   No WGSL imports. Self-contained.
//
// USED BY:
//   crates/gargantua-video/src/offline/accumulator.rs
//     → orchestrates sub-frame GPU accumulation for offline export
//   PLANNED: crates/gargantua-render/src/pipelines/accumulate.rs
//     → Pass wrapper that dispatches this shader after ray_march.wgsl
//     → dispatch: (width/8, height/8, 1)
//
// NOTE FOR AI:
//   blend_alpha = 1/(frame_idx+1) gives exact running mean (no bias).
//   For real-time TAA (anti-aliasing): use fixed blend_alpha=0.1 instead.
//   reset=1 must be sent when camera moves to avoid ghosting artifacts.
//   output is a storage buffer (not a texture) — written as vec4<f32>.
//   Alpha channel: w=1.0 always (fully opaque accumulated frame).
//   History texture is the output of the previous frame's accumulation.
//   Ping-pong: two history textures alternating each frame.
// ============================================================
