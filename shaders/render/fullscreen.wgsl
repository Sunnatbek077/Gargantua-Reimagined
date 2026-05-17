// ============================================================
// FILE: shaders/render/fullscreen.wgsl
// LINES: ~40
// CATEGORY: Shader — Fullscreen triangle vertex shader
// STAGE: Vertex
// ============================================================
//
// PURPOSE:
//   Minimal vertex shader that generates a fullscreen clip-space
//   triangle without any vertex buffer. Used by ALL fragment-stage
//   post-processing shaders (bloom, tonemap, TAA, chromatic, etc.).
//   One draw call: DrawPrimitive(3, 1) — no buffers needed.
//
// SHADER INTERFACE:
//   No vertex buffers. No uniform bindings.
//   Input: @builtin(vertex_index) vert_idx: u32
//
// ENTRY POINT:
//   @vertex
//   fn vs_main(@builtin(vertex_index) vert_idx: u32)
//       -> @builtin(position) vec4<f32>
//     // Generates a single triangle that covers the entire screen:
//     // vert_idx=0: (-1, -1)  bottom-left
//     // vert_idx=1: ( 3, -1)  far right (outside clip space)
//     // vert_idx=2: (-1,  3)  far top   (outside clip space)
//     // The oversized triangle covers the [-1,1]×[-1,1] NDC quad.
//     //
//     // Implementation:
//     //   let x = f32(i32(vert_idx) * 2 - 1);   // -1, 3, -1
//     //   let y = f32(1 - i32(vert_idx & 1u) * 4);  // -1, -1, 3
//     //   return vec4(x, y, 0.0, 1.0);
//
// USES (imports from):
//   No WGSL imports. Entirely self-contained.
//
// USED BY (as vertex stage):
//   postfx/bloom_down.wgsl, postfx/bloom_up.wgsl
//   postfx/chromatic.wgsl, postfx/film_grain.wgsl
//   postfx/motion_blur.wgsl, postfx/taa.wgsl, postfx/tonemap.wgsl
//   render/accretion_disk.wgsl, render/lensing.wgsl, render/starfield.wgsl
//   → All fragment shaders use this as their vertex stage
//
// NOTE FOR AI:
//   This pattern (oversized triangle trick) avoids the diagonal seam
//   that occurs when using two triangles to cover the screen.
//   No vertex buffer needed: DrawPrimitive(vertex_count=3, instance_count=1).
//   UV coordinates: frag_pos.xy / screen_size in fragment shader.
//   WGSL spec: @builtin(vertex_index) starts at 0 for non-indexed draws.
// ============================================================
