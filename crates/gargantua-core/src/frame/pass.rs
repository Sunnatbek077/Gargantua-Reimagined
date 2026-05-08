// =============================================================================
// crates/gargantua-core/src/frame/pass.rs
// =============================================================================
//
// PURPOSE:
//   Defines the Pass trait that every render and compute pass in the frame
//   graph must implement. Also defines PassContext — the per-pass data bundle
//   passed to Pass::record() containing the wgpu encoder, resolved resource
//   handles, and device reference.
//
//   All rendering work in Gargantua is expressed as Pass implementors:
//     - RayMarchPass   (gargantua-render/src/pipelines/ray_march.rs)
//     - AccretionPass  (gargantua-render/src/pipelines/accretion.rs)
//     - TaaPass        (gargantua-render/src/postfx/taa.rs)
//     - BloomPass      (gargantua-render/src/postfx/bloom.rs)
//     - TonemapPass    (gargantua-render/src/postfx/tonemap.rs)
//     - ... and all other render/compute passes
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - super::resource::{ResourceHandle, ResourcePool}
//     - crate::errors::CoreError
//   External:
//     - wgpu::{CommandEncoder, Device, Queue, RenderPipeline, ComputePipeline,
//              BindGroup, TextureView, Buffer}
//
// CALLED BY:
//   - frame_graph.rs::FrameGraph::record_commands()  — calls pass.record()
//   - All pass implementors in gargantua-render and gargantua-bake
//
// PUBLIC TYPES:
//
//   pub trait Pass: Send {
//     — must be Send so passes can be constructed on any thread.
//
//     fn name(&self) -> &str
//       — human-readable name for debugging and GPU profiler labels.
//       — example: "ray_march", "taa", "bloom_down"
//
//     fn record(&mut self, ctx: &mut PassContext) -> Result<(), CoreError>
//       — core method: encodes GPU commands into ctx.encoder.
//       — for compute passes: begins a compute pass, sets pipeline and
//         bind groups, dispatches workgroups, ends the compute pass.
//       — for render passes: begins a render pass with the correct
//         color/depth attachments, sets pipeline, draws, ends render pass.
//       — must NOT submit the encoder — FrameGraph does that after all passes.
//
//     fn read_resources(&self) -> &[ResourceHandle]
//       — returns the list of resources this pass reads.
//       — used by FrameGraph during DAG construction to set up dependencies.
//
//     fn write_resources(&self) -> &[ResourceHandle]
//       — returns the list of resources this pass writes to.
//   }
//
//   pub struct PassContext<'a> {
//     pub encoder:   &'a mut wgpu::CommandEncoder,
//     pub device:    &'a wgpu::Device,
//     pub queue:     &'a wgpu::Queue,
//     pub resources: &'a ResourcePool,
//     pub frame_idx: u64,    // monotonically increasing frame counter
//     pub delta_t:   f32,    // frame delta time in seconds (from clock.rs)
//   }
//
//   pub trait RenderPass: Pass {
//     — marker trait for passes that use wgpu::RenderPassDescriptor.
//     — no additional methods; used for type-level distinction.
//   }
//
//   pub trait ComputePass: Pass {
//     — marker trait for passes that use wgpu::ComputePassDescriptor.
//   }
//
// PUBLIC FUNCTIONS: none (trait definitions only)
//
// IMPLEMENTATION PATTERN (for AI writing a concrete pass):
//
//   pub struct MyComputePass {
//     pipeline:   wgpu::ComputePipeline,
//     bind_group: wgpu::BindGroup,
//     reads:      Vec<ResourceHandle>,
//     writes:     Vec<ResourceHandle>,
//   }
//
//   impl Pass for MyComputePass {
//     fn name(&self) -> &str { "my_compute" }
//
//     fn record(&mut self, ctx: &mut PassContext) -> Result<(), CoreError> {
//       let mut cpass = ctx.encoder.begin_compute_pass(
//         &wgpu::ComputePassDescriptor { label: Some(self.name()), .. }
//       );
//       cpass.set_pipeline(&self.pipeline);
//       cpass.set_bind_group(0, &self.bind_group, &[]);
//       cpass.dispatch_workgroups(width / 8, height / 8, 1);
//       Ok(())
//     }
//
//     fn read_resources(&self)  -> &[ResourceHandle] { &self.reads }
//     fn write_resources(&self) -> &[ResourceHandle] { &self.writes }
//   }
//
// NOTES FOR AI:
//   - Pass::record() receives a mutable PassContext, not &mut self on the
//     encoder directly — this enforces that all GPU work goes through the
//     single CommandEncoder owned by FrameGraph.
//   - Passes must NOT cache wgpu::TextureView between frames; always resolve
//     from ctx.resources each frame because transient resources are recycled.
//   - The Send bound on Pass is required because passes are stored in a Vec
//     inside FrameGraph, which may be moved across threads at startup.
//   - frame_idx in PassContext is used by TAA (taa.wgsl) for jitter sequences
//     and by film_grain.wgsl for per-frame noise offset.
// =============================================================================

use crate::{errors::CoreError, frame::resource::{ResourceHandle, ResourcePool}};

pub struct PassContext<'a> {
    pub encoder:   &'a mut wgpu::CommandEncoder,
    pub device:    &'a wgpu::Device,
    pub queue:     &'a wgpu::Queue,
    pub resources: &'a ResourcePool,
    pub frame_idx: u64,
    pub delta_t:   f32,
}

pub trait Pass: Send {
    fn name(&self) -> &str;

    fn record(&mut self, ctx: &mut PassContext<'_>) -> Result<(), CoreError>;

    fn read_resources(&self) -> &[ResourceHandle];

    fn write_resources(&self) -> &[ResourceHandle];
}

pub trait RenderPass: Pass {}

pub trait ComputePass: Pass {}