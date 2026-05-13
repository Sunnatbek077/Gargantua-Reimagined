// =============================================================================
// FILE: crates/gargantua-app/src/systems/physics_sync.rs
// CRATE: gargantua-app
// LINES: ~240
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   Each frame, reads the current SimState, steps the physics simulation
//   forward by DeltaTime.sim, and uploads the resulting parameters to the
//   GPU uniform buffer. Serves as the integration point between the CPU
//   physics layer (gargantua-physics) and the GPU render layer.
//
// WHAT THIS FILE CONTAINS:
//   - `pub struct PhysicsSync`:
//       uniform_buffer:  wgpu::Buffer       — GPU-side PhysicsUniforms buffer
//       metric:          KerrMetric         — cached from last SimState update
//       isco:            IscoBounds         — cached ISCO for disk inner boundary
//       mhd_disk:        MhdDisk            — accretion disk model instance
//       upload_heap:     Option<UploadHeap> — Windows-only write-combined upload
//   - `impl PhysicsSync`:
//       `pub fn new(ctx: &GpuContext, sim: &SimState) -> AppResult<Self>`
//             Builds KerrMetric from sim.to_kerr_metric().
//             Computes IscoBounds via accretion::isco::compute_isco().
//             Creates wgpu::Buffer (UNIFORM | COPY_DST) for PhysicsUniforms.
//             On Windows: creates UploadHeap for write-combined CPU→GPU transfer.
//       `pub fn sync(&mut self, sim: &SimState, dt: DeltaTime,
//                    queue: &wgpu::Queue)`
//             Called every frame from App::tick():
//             1. If sim.mass or sim.spin changed since last frame:
//                  Rebuilds self.metric = KerrMetric::new(sim.mass, sim.spin, sim.charge)
//                  Recomputes self.isco = compute_isco(&self.metric)
//                  Recomputes self.mhd_disk from new metric + accretion_rate
//             2. Advances sim.simulation_time += dt.sim
//             3. Builds PhysicsUniforms from sim.to_gpu_uniforms() + ISCO + MHD params
//             4. Uploads PhysicsUniforms to self.uniform_buffer:
//                  Mac:     queue.write_buffer() (unified memory — zero copy)
//                  Windows: upload_heap.write() + copy_to(uniform_buffer)
//       `pub fn uniform_buffer(&self) -> &wgpu::Buffer`
//             Returns reference to the GPU uniform buffer for render bind groups.
//
// OUTBOUND DEPENDENCIES:
//   - state/sim_state.rs                     → SimState, to_gpu_uniforms()
//   - gargantua_physics::metric::kerr        → KerrMetric
//   - gargantua_physics::accretion::isco     → compute_isco(), IscoBounds
//   - gargantua_physics::accretion::mhd      → MhdDisk
//   - render/bindgroups/physics.rs           → PhysicsUniforms (GPU struct layout)
//   - platform/windows/memory/upload_heap.rs → UploadHeap (Windows only)
//   - gargantua_core::gpu::context           → GpuContext
//   - wgpu (external)                        → Buffer, Queue
//   - errors.rs                              → AppResult
//
// INBOUND:
//   - gargantua_core::app::App → calls physics_sync.sync() once per frame
//                                  in App::tick() before frame_graph.execute()
//
// NOTES:
//   - Rebuilding KerrMetric and ISCO only happens when mass/spin/charge changes
//     (typically on user slider input), NOT every frame. The hot path (no change)
//     skips the rebuild and only updates simulation_time and MHD turbulence seed.
//   - The MHD turbulence seed is incremented each frame to animate disk flicker.
// =============================================================================
