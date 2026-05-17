# Gargantua — Architecture

## Overview

Gargantua is a physically accurate black hole renderer built in Rust. It runs natively on macOS (Apple Silicon + Intel) and Windows (NVIDIA/AMD/Intel GPU), and in the browser via WebAssembly. The codebase is split into eight focused crates that communicate through well-defined interfaces, with zero circular dependencies.

```
gargantua-app          ← top-level orchestrator (binary entry point)
├── gargantua-core     ← GPU context, frame graph, quality, platform HAL
├── gargantua-physics  ← Kerr metric, geodesic integrator, accretion disk
├── gargantua-bake     ← pre-computed LUT generation (geodesic, spectrum, noise)
├── gargantua-render   ← render pipelines, shaders, post-processing
├── gargantua-camera   ← camera modes, paths, relativistic effects
├── gargantua-video    ← hardware encoding, denoising, offline render
└── gargantua-ui       ← HUD, menus, overlays, accessibility
```

---

## Crate Responsibilities

### `gargantua-core`
The engine foundation. Owns the `wgpu` device and queue, the `FrameGraph` (a DAG of render and compute passes), the `Clock` (delta time, simulation time), and the `AdaptiveQuality` system. Contains all platform-specific hardware abstraction (HAL) code under `platform/macos/` and `platform/windows/`.

**Key types:** `App`, `GpuContext`, `FrameGraph`, `ResourcePool`, `QualityPreset`, `GpuProfiler`

### `gargantua-physics`
Pure computation — no GPU code. Implements the Kerr-Newman metric in Boyer-Lindquist coordinates, a 4th-order Runge-Kutta geodesic integrator, the Novikov-Thorne accretion disk temperature model, and relativistic optical effects (Doppler beaming, gravitational redshift, Penrose process). All functions use `f64` for numerical stability near the event horizon.

**Key types:** `KerrMetric`, `Rk4Integrator`, `PhotonState`, `MhdDisk`, `IscoBounds`

### `gargantua-bake`
Offline pre-computation that runs once at startup (or is loaded from disk cache). Generates the geodesic deflection LUT (GPU compute via WGSL), blackbody/Doppler colour LUTs (CPU via physics crate), and 3D blue noise textures (GPU, void-and-cluster algorithm). All outputs are cached on disk with SHA-256 invalidation.

**Key types:** `BakeScheduler`, `BakeCache`, `GeodesicLutBaker`, `BlueNoiseBaker`

### `gargantua-render`
All GPU rendering. Registered render and compute passes: ray marching (`ray_march.wgsl`), accretion disk (`accretion.wgsl`), gravitational lensing (`lensing.wgsl`), starfield, lens flare, bloom, TAA, and tonemapping. Passes are registered into `FrameGraph` from `gargantua-core` and execute in DAG-resolved order every frame.

**Key types:** `RayMarchPipeline`, `AccretionPipeline`, `TaaPipeline`, `TonemapPipeline`, `PhysicsUniforms`

### `gargantua-camera`
Camera system with five modes: Free-flight, Satellite orbit, Plunge (falling into the black hole), Gravity (camera follows a geodesic), and Path (scripted keyframe path). Also handles relativistic FOV correction (aberration), time dilation display, and the `CameraPath` recorder/playback used by the replay system.

**Key types:** `CameraController`, `CameraMode`, `CameraPath`, `AberrationFov`

### `gargantua-video`
Hardware-accelerated video capture and export. Selects the best available encoder per platform (VideoToolbox on Mac, NVENC/AMF/QSV on Windows, rav1e as universal fallback). The offline renderer runs sub-frame accumulation (motion blur), denoising (ANE on Mac, CUDA OIDN on Windows, À-trous GPU fallback), colour space transform (3D LUT), and encoding in a background thread.

**Key types:** `OfflineRenderer`, `RealtimeCapturer`, `Denoiser`, `Encoder`, `ColorTransform`

### `gargantua-ui`
All user interface: the physics/render/camera/accretion/export menu tabs, HUD stats bar, physics readout overlay, and render progress overlay. Built on `egui` (immediate-mode GUI rendered via `wgpu`). Reads and writes `SimState` through the `EventBus`; never calls GPU code directly.

**Key types:** `MainMenu`, `StatsBar`, `PhysicsReadout`, `RenderProgress`, `PresetSelector`

### `gargantua-app`
Top-level orchestrator. Wires all crates together: registers render passes into `FrameGraph`, drives `PhysicsSync` each frame, handles `UndoHistory`, manages the `PluginRegistry` (Lua scripting via `mlua`), and implements URL-based state sharing (`url_serde`). Contains the binary `main.rs` entry point.

**Key types:** `SimState`, `EventBus`, `UndoHistory`, `PhysicsSync`, `ReplaySystem`, `ScriptingPlugin`

---

## Frame Lifecycle

Every rendered frame follows this sequence inside `App::tick()`:

```
1. Clock::tick()                  — advance wall time, compute DeltaTime
2. InputSystem::tick()            — process held keys, camera movement
3. PluginRegistry::tick_all()     — run Lua scripts / plugin on_frame()
4. PhysicsSync::sync()            — rebuild KerrMetric if params changed,
                                    upload PhysicsUniforms to GPU
5. AdaptiveQuality::evaluate()    — adjust SPP/scale if FPS off-target
6. FrameGraph::execute():
   ├── ray_march.wgsl             — trace photon geodesics (compute)
   ├── accretion.wgsl             — disk temperature → colour (compute)
   ├── lensing.wgsl               — gravitational lensing warp (compute)
   ├── starfield.wgsl             — background stars with aberration
   ├── lens_flare.wgsl            — optical flare (render)
   ├── bloom.wgsl                 — multi-pass bloom (compute)
   ├── taa.wgsl                   — temporal anti-aliasing (compute)
   └── tonemap.wgsl               — HDR → display (EDR/HDR10/SDR)
7. egui render pass               — UI on top of scene
8. Surface::present()             — flip swapchain buffer
9. GpuProfiler::read_results()    — read last frame's GPU timestamps
```

---

## Data Flow: Physics → GPU

`SimState` (plain Rust data, Clone + Serialize) is the single source of truth. It is never stored inside GPU objects. Each frame, `PhysicsSync` converts it to `PhysicsUniforms` (a `bytemuck::Pod` struct) and uploads it to a `wgpu::Buffer`. Every render pass reads from this buffer via `@group(0) @binding(0)` in WGSL.

```
SimState (CPU, f64)
    └── PhysicsSync::sync()
            └── PhysicsUniforms (Pod, f32)
                    └── wgpu::Buffer (GPU uniform)
                            ├── ray_march.wgsl  @group(0) @binding(0)
                            ├── accretion.wgsl  @group(0) @binding(0)
                            └── lensing.wgsl    @group(0) @binding(0)
```

---

## Platform Abstraction

Platform-specific code is isolated under `gargantua-core/src/platform/`. Feature detection happens at startup; runtime `if cfg!(target_os = "macos")` checks are avoided in favour of compile-time gates.

| Subsystem | macOS | Windows | WASM |
|---|---|---|---|
| GPU backend | Metal (wgpu) | DX12 → Vulkan fallback | WebGPU |
| Swapchain HDR | EDR (NSScreen) | HDR10 / Dolby Vision | SDR only |
| Compute config | Metal threadgroups (8×8×1) | NVIDIA warp / AMD wavefront | WebGPU default |
| Memory model | Unified (zero-copy readback) | PCIe staging pool (triple-buf) | JS ArrayBuffer |
| Denoiser | ANE via CoreML | CUDA OIDN → CPU OIDN | À-trous (GPU) |
| Hardware encoder | VideoToolbox (H.264/H.265/ProRes/AV1) | NVENC / AMF / QSV | WebCodecs |
| GPU tier detection | IORegistry chip-id (M1–M5) | DXGI VRAM + PCI vendor | Medium preset |

---

## FrameGraph: DAG Execution

`FrameGraph` stores render passes as a directed acyclic graph. Edges are derived automatically from each pass's declared `reads()` and `writes()` resource lists. At startup, `compile()` runs Kahn's topological sort to find the execution order, inserts `BarrierPass` nodes between passes that share resources, and aliases non-overlapping intermediate textures to reduce peak VRAM usage.

```
ray_march ──writes──► hdr_accum ──reads──► taa ──writes──► taa_history
accretion ──writes──► accretion_buf ──reads──► ray_march (albedo input)
                                    └──reads──► tonemap
tonemap ──writes──► swapchain_backbuffer
```

If a cycle is detected in the declared dependencies, `compile()` returns `CoreError::FrameGraphCycle` and the application refuses to start (caught in tests).

---

## Shader Pipeline

All shaders are written in WGSL 1.0 and compiled by `naga` (wgpu's built-in compiler). There is no pre-compilation step for shaders — `naga` validates and transpiles to MSL / DXIL / SPIRV at runtime on first launch, then caches the compiled variants.

In development builds (`feature = "hot-reload"`), `platform/common/shader_reload.rs` watches the `shaders/` directory with `notify` and triggers pipeline recreation on `.wgsl` file changes without restarting the app.

Shader validation is also run as a CI step via `build/validate_shaders.sh`, which calls `naga --validate` on every `.wgsl` file and checks for project-specific lint rules (no hardcoded physical constants, workgroup size consistency).

---

## Memory Budget (typical 4K frame, High preset)

| Resource | Format | Size |
|---|---|---|
| HDR accumulation buffer | RGBA16Float | 32 MB |
| TAA history (×2) | RGBA16Float | 64 MB |
| Velocity buffer | RG16Float | 16 MB |
| Geodesic LUT (512×512) | RGBA32Float | 4 MB |
| Blackbody LUT (1024×1) | RGBA16Float | 8 KB |
| Starfield cubemap (8K, 6 mip) | UASTC KTX2 | ~48 MB |
| 3D blue noise (64³) | R8Unorm | 256 KB |
| Physics uniforms | Pod struct | 256 B |
| **Total (approx.)** | | **~165 MB** |

On M1 (8 GB unified), the full asset set fits with ~6.8 GB headroom for the OS and other apps. `MemoryPressureMonitor` evicts non-essential cached mip levels when available memory drops below 512 MB.

---

## Dependency Graph (crate level)

```
gargantua-app
    ├── gargantua-core
    │       └── (wgpu, winit, tracing, bytemuck, naga)
    ├── gargantua-physics
    │       └── (thiserror, serde, bytemuck)
    ├── gargantua-bake
    │       ├── gargantua-physics
    │       └── gargantua-core
    ├── gargantua-render
    │       ├── gargantua-core
    │       └── gargantua-physics
    ├── gargantua-camera
    │       ├── gargantua-core
    │       └── gargantua-physics
    ├── gargantua-video
    │       └── gargantua-core
    └── gargantua-ui
            ├── gargantua-core
            └── (egui, egui-wgpu)
```

No crate depends on `gargantua-app`. The app crate is the composition root only.

---

## Testing Strategy

| Layer | What is tested | Tool |
|---|---|---|
| Physics correctness | Kerr metric, ISCO, redshift formulae | `cargo test` + known analytical values |
| Geodesic integrator | Photon sphere radius, circular orbits | `cargo test` (gargantua-physics/tests/) |
| Frame graph | DAG sort, cycle detection, resource aliasing | `cargo test` (gargantua-core/tests/) |
| URL serialisation | Round-trip, invalid input, schema stability | `cargo test` (gargantua-app/tests/) |
| Undo/redo | Stack ordering, max-depth, redo invalidation | `cargo test` |
| Shader validation | WGSL parse, MSL/HLSL transpile, lint rules | `build/validate_shaders.sh` (CI) |
| GPU benchmarks | Ray march ms/frame, geodesic steps/sec | `cargo bench` (Criterion) |
| Codec availability | Hardware encoder init, software fallback | `cargo test` (gargantua-video/tests/) |
| Colour accuracy | LUT round-trip, matrix white-point | `cargo test` |

Integration tests requiring a physical GPU are marked `#[ignore]` in CI and run on self-hosted runners with real hardware.