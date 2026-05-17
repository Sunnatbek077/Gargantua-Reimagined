# Gargantua — Platform Guide

This document covers all platform-specific behaviour: GPU backend selection, hardware encoder availability, HDR display configuration, memory management, and known limitations per platform. All platform code lives under `crates/gargantua-core/src/platform/`.

---

## Supported Platforms

| Platform | Minimum requirement | GPU backend | Status |
|---|---|---|---|
| macOS (Apple Silicon) | macOS 13, M1 | Metal via wgpu | ✅ Full support |
| macOS (Intel) | macOS 12, Intel Iris | Metal via wgpu | ✅ Supported |
| Windows 11/10 | Windows 10 2004, DX12 | DX12 → Vulkan | ✅ Full support |
| Browser (WASM) | Chrome 113 / Firefox 115 | WebGPU | ✅ Supported |
| Linux | Ubuntu 22.04, Vulkan 1.3 | Vulkan | 🔄 Planned |

---

## macOS

### GPU Initialisation

On macOS, `wgpu` uses the Metal backend automatically. The surface is attached to a `CAMetalLayer` via `platform/macos/gpu/adapter_metal.rs`:

1. The winit `NSView` is retrieved from the window.
2. A `CAMetalLayer` is allocated and set as the view's backing layer (`wantsLayer = YES`).
3. `maximumDrawableCount = 3` enables triple buffering on Apple Silicon.
4. Pixel format: `RGBA16Float` when EDR is available, `BGRA8Unorm_sRGB` otherwise.
5. `wgpu::Instance::create_surface_from_layer()` wraps the Metal layer.

### Apple Silicon Chip Detection

`platform/macos/gpu/chip_detect.rs` reads the IORegistry to identify the exact chip variant, enabling per-chip quality defaults:

```
IOServiceGetMatchingService("chip-id") → numeric chip ID
                            ("gpu-core-count") → verify Pro/Max/Ultra variant
```

Supported chips and their default quality presets:

| Chip | GPU cores | Bandwidth | Default preset |
|---|---|---|---|
| M1 | 8 | 68 GB/s | Medium (SPP 4, steps 64, scale 0.75) |
| M1 Pro | 16 | 200 GB/s | High (SPP 16, steps 128, scale 1.0) |
| M1 Max | 32 | 400 GB/s | Ultra (SPP 32, steps 256, scale 1.0) |
| M2 | 10 | 100 GB/s | High (SPP 8, steps 128, scale 0.9) |
| M2 Pro | 19 | 200 GB/s | High (SPP 16, steps 128, scale 1.0) |
| M2 Max | 38 | 400 GB/s | Ultra (SPP 32, steps 256, scale 1.0) |
| M2 Ultra | 76 | 800 GB/s | Ultra+ (SPP 64, steps 512, scale 1.0) |
| M3 | 10 | 100 GB/s | High (SPP 8, steps 128, scale 1.0) |
| M3 Pro | 18 | 150 GB/s | High (SPP 16, steps 128, scale 1.0) |
| M3 Max | 40 | 400 GB/s | Ultra (SPP 64, steps 256, scale 1.0) |
| M4 | 10 | 120 GB/s | High (SPP 16, steps 128, scale 1.0) |
| M4 Pro | 20 | 273 GB/s | Ultra (SPP 32, steps 256, scale 1.0) |
| M4 Max | 40 | 546 GB/s | Ultra (SPP 128, steps 512, scale 1.0) |
| M5 | ~12 | ~150 GB/s | Ultra (estimated) |
| M5 Pro | ~24 | ~300 GB/s | Ultra (estimated) |
| M5 Max | ~48 | ~600 GB/s | Ultra+ (SPP 256, steps 1024, scale 1.0) |

M5 figures are estimated based on Apple's generation-over-generation scaling trend and will be updated once official specs are published.

### Compute Shader Configuration

Metal threadgroup constraints (`platform/macos/compute/threadgroup.rs`):

```
MAX_THREADS_PER_THREADGROUP = 1024
SIMD_GROUP_WIDTH            = 32
MAX_THREADGROUP_SHARED_MEM  = 32 KB
Default 2D workgroup:       (8, 8, 1) = 64 threads = 2 SIMD groups
Wide-image workgroup (4K+): (16, 4, 1) = 64 threads, L1 cache aligned
```

The WGSL `@workgroup_size` annotation must match the Rust dispatch configuration. Mismatches are caught by `build/validate_shaders.sh` Rule 2.

### Neural Engine Denoising

On Apple Silicon, the Neural Engine (ANE) accelerates the denoising pass via CoreML (`platform/macos/compute/neural_engine.rs`). The compiled model (`gargantua_denoise.mlmodelc`) is in the app bundle under `Resources/`.

| Chip | ANE throughput | 4K denoise time |
|---|---|---|
| M1 / M2 | ~11 TOPS | ~30 ms |
| M3 / M4 | ~18–38 TOPS | ~9–15 ms |
| M5 (estimated) | ~50+ TOPS | ~6 ms |

ANE denoising is used for offline renders only (not realtime). The realtime path uses À-trous GPU denoising instead to avoid the ANE latency.

### Unified Memory

Apple Silicon has unified memory — the CPU and GPU share the same physical DRAM with no PCIe bus between them. Gargantua exploits this in two ways:

**Zero-copy video capture** (`platform/macos/memory/zero_copy_readback.rs`):
The GPU renders into an `MTLBuffer` in `.shared` storage mode. The CPU reads from the same physical memory with no `memcpy`. At 4K 60 FPS, this avoids copying ~2 GB/s of frame data.

**Write-combined uniform uploads** (`platform/macos/memory/unified_allocator.rs`):
Physics uniforms are written directly to the GPU-accessible buffer via `queue.write_buffer()`. The Metal driver handles the cache coherency with no explicit flush.

**Memory pressure monitoring** (`platform/macos/memory/memory_pressure.rs`):
`os_proc_available_memory()` is polled each frame. When available memory drops below thresholds, `pressure_response.rs` reduces quality automatically:

```
> 512 MB free:  Normal   → no action
128–512 MB:     Warning  → reduce geodesic LUT resolution, lower blue noise mip
< 128 MB:       Critical → SPP=1, disable TAA, disable bloom, evict LUT caches
```

### HDR Display

**EDR (Extended Dynamic Range)** is queried via `NSScreen.maximumExtendedDynamicRangeColorValue` (macOS 12+). The EDR value scales the internal HDR rendering:

```
Pro Display XDR:        max_edr = 16.0  (1600 nit peak in XDR mode)
MacBook Pro (M3/M4):    max_edr ≈ 2.0   (HDR mode via True Tone)
Regular SDR display:    max_edr = 1.0   (no EDR)
```

`tonemap.wgsl` uses `max_edr` as the peak luminance ceiling. On Reference Mode (Pro Display XDR), EDR is locked to 1.0 (perfect calibration, no tone mapping beyond SDR).

**Display P3** is detected via `NSScreen.colorSpace.localizedName`. The tonemap pass applies the Rec2020 → P3 gamut matrix when P3 is active, preserving wide-gamut colours on compatible displays.

### Hardware Encoders (VideoToolbox)

All encoders require the VideoToolbox framework. Performance uses the dedicated media engine on Apple Silicon (M1+):

| Codec | Availability | Notes |
|---|---|---|
| H.264 (AVC) | All Apple Silicon + Intel Mac | `kCMVideoCodecType_H264`, High profile |
| H.265 (HEVC) | All Apple Silicon + Intel Mac | 10-bit + HDR10 metadata on M1+ |
| ProRes 4444 XQ | M1+ (hardware), Intel (software) | Professional editing master |
| ProRes 422 HQ | All Macs | Standard editing quality |
| ProRes RAW | M1+ only | Requires Apple Silicon media engine |
| AV1 | M3+ (hardware), M1/M2 (rav1e software) | `kCMVideoCodecType_AV1` |

---

## Windows

### GPU Adapter Selection

`platform/windows/gpu/adapter_dx12.rs` uses `IDXGIFactory6::EnumAdapterByGpuPreference` with `DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE` to always select the discrete GPU on Optimus/Hybrid systems. `DXGI_ADAPTER_FLAG_SOFTWARE` adapters (Microsoft Basic Render Driver) are skipped.

Fallback chain: DX12 (primary) → Vulkan (secondary for some older AMD configurations) → Software rasteriser (CI/VM only).

### GPU Vendor Detection

`platform/windows/gpu/vendor.rs` reads the PCI vendor ID from `DXGI_ADAPTER_DESC1`:

```
NVIDIA: VendorId = 0x10DE
AMD:    VendorId = 0x1002
Intel:  VendorId = 0x8086
```

The vendor determines:
- Compute workgroup size (warp 32 vs wavefront 64)
- Hardware encoder selection (NVENC vs AMF vs QSV)
- Quality preset tier (see below)

### Windows Quality Presets

**NVIDIA (RTX series):**

| GPU | VRAM | Default preset |
|---|---|---|
| RTX 5090 | 32 GB | Ultra+ (SPP 128, steps 512) |
| RTX 5080 | 16 GB | Ultra (SPP 64, steps 512) |
| RTX 4090 | 24 GB | Ultra (SPP 64, steps 256) |
| RTX 4080 | 16 GB | Ultra (SPP 32, steps 256) |
| RTX 4070 Ti | 12 GB | High (SPP 16, steps 128) |
| RTX 4070 | 12 GB | High (SPP 16, steps 128) |
| RTX 4060 | 8 GB | Medium (SPP 8, steps 64) |

**AMD (RDNA 3/4):**

| GPU | VRAM | Default preset |
|---|---|---|
| RX 9070 XT | 16 GB | Ultra (SPP 32, steps 256) |
| RX 7900 XTX | 24 GB | Ultra (SPP 32, steps 256) |
| RX 7800 XT | 16 GB | High (SPP 16, steps 128) |
| RX 7700 XT | 12 GB | High (SPP 16, steps 128) |
| RX 7600 | 8 GB | Medium (SPP 8, steps 64) |

**Intel Arc:**

| GPU | VRAM | Default preset |
|---|---|---|
| Arc B580 | 12 GB | Medium (SPP 4, steps 64) |
| Arc A770 | 16 GB | Medium (SPP 8, steps 64) |
| iGPU (Xe) | Shared | Potato (SPP 1, steps 16) |

For unlisted GPUs, `from_vram_bytes()` falls back to a VRAM-based tier: > 16 GB → Ultra, 8–16 GB → High, 4–8 GB → Medium, < 4 GB → Low.

### Compute Shader Configuration

`platform/windows/compute/workgroup.rs` selects optimal workgroup sizes:

```
NVIDIA (warp size 32):  (8, 4, 1) = 32 threads = 1 warp
AMD (wavefront size 64): (8, 8, 1) = 64 threads = 1 wavefront
Intel (EU threads 32):  (8, 4, 1) = 32 threads
```

Shared memory limits per workgroup (`platform/windows/compute/shared_mem.rs`):

```
NVIDIA Ampere/Ada/Blackwell:  49 152 bytes (48 KB)
AMD RDNA 2/3/4:               65 536 bytes (64 KB)
Intel Arc:                    32 768 bytes (32 KB)
Safe budget (75% of max):     used for var<workgroup> sizing
```

### VRAM Budget Monitoring

`IDXGIAdapter3::QueryVideoMemoryInfo(DXGI_MEMORY_SEGMENT_GROUP_LOCAL)` is polled each frame to track VRAM usage:

```
headroom > 20%:   Normal   → no action
10–20%:           Warning  → reduce LUT resolution
< 10%:            Critical → SPP=1, evict non-essential textures
```

This mirrors the macOS unified memory pressure system but targets discrete VRAM specifically.

### Memory Architecture

Windows has a discrete GPU with GDDR VRAM separated from system RAM by PCIe. Gargantua uses a triple-buffered staging pool (`platform/windows/memory/staging_pool.rs`) for GPU → CPU video capture:

```
Frame N:   GPU renders into write_buffer (buffer[cursor])
Frame N-1: GPU → CPU copy in flight (buffer[(cursor+1)%3])
Frame N-2: CPU maps and reads (buffer[(cursor+2)%3]) → encoder
```

This triple-buffer pattern ensures the GPU is never waiting for the CPU and the CPU is never waiting for the GPU.

Uniform buffer uploads use `D3D12_HEAP_TYPE_UPLOAD` write-combined memory (`platform/windows/memory/upload_heap.rs`). Write-combined memory is fast for sequential CPU writes (cache-line streaming) but must never be read back on the CPU.

### HDR Display

**HDR10** is the standard Windows HDR format. `platform/windows/hdr/hdr10.rs` enables it via:

```
IDXGIOutput6::GetDesc1() → check ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020
IDXGISwapChain4::SetColorSpace1(DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020)
```

The ST 2084 PQ transfer function constants (m1, m2, c1, c2, c3) are passed to `tonemap.wgsl` as a push constant. MaxCLL and MaxFALL are read from the monitor's `OutputDesc1` and embedded as HDR metadata in the encoded video output.

**Dolby Vision** is detected via `Windows.Devices.Display.DisplayMonitor` WinRT API when available (Windows 11 22H2+). DV Profile 8 (compatible with HDR10 players) is preferred over Profile 5.

### Hardware Encoders

Encoder selection priority: hardware > software fallback.

**NVIDIA NVENC** (`platform/windows/video/nvenc.rs`):
- Loaded dynamically from `nvEncodeAPI64.dll`
- RTX 30+ series: H.264, H.265, AV1 (Ada/Blackwell)
- RTX 50 (Blackwell): dual NVENC engines → double-speed or parallel streams
- Requires CUDA driver 522.25+ for AV1 support

**AMD AMF** (`platform/windows/video/amf.rs`):
- Loaded from `amfrt64.dll`
- RX 5000+: H.264, H.265
- RX 6000+: H.264, H.265, AV1

**Intel Quick Sync** (`platform/windows/video/qsv.rs`):
- Via Intel Media SDK / oneVPL
- All modern Intel CPUs: H.264, H.265
- Arc discrete GPU: + AV1 (oneVPL 2.6+)

**Software fallback** (always available):
- H.264: x264 (libx264 FFI)
- H.265: x265 (libx265 FFI)
- AV1: rav1e (pure Rust)

---

## WebAssembly (Browser)

### Build

The WASM build is produced by `build/wasm-pack.sh` using `wasm-pack build --target web`. Key Cargo feature flags:

```
--features "wasm"
-Z build-std=std,panic_abort
-Z build-std-features=panic_immediate_abort
```

The `wasm` feature gates disable all platform-specific code (VideoToolbox, NVENC, ANE, D3D12) and enables WASM-specific paths (WebCodecs, web_sys URL hash reading, wasm-tracing-subscriber).

### WebGPU

The browser WebGPU API is more restrictive than native wgpu:

| Feature | Native | WASM (WebGPU) |
|---|---|---|
| Timestamp queries | ✅ | ❌ (not available) |
| Max texture size | 16384 | 8192 |
| Compute workgroup shared mem | 48–64 KB | 16 KB |
| Persistent buffer mapping | ✅ | Limited |
| HDR swapchain | ✅ | ❌ (SDR only) |

`GpuLimits::negotiate()` handles all these restrictions transparently — it reads the adapter's actual limits and caps Gargantua's requests accordingly.

### Quality

On WASM, `GpuTierDetector::detect()` always returns `QualityPreset::medium()` as a conservative default. The user can manually increase quality; the adaptive quality system will reduce it if FPS drops.

### Denoiser

WASM uses the À-trous wavelet GPU denoiser (5 passes, inline WGSL). ANE (CoreML) and CUDA OIDN are not available in the browser.

### Video Recording

WASM uses the browser `VideoEncoder` API (WebCodecs, Chrome 94+, Firefox 130+):

```javascript
const encoder = new VideoEncoder({
  output: (chunk) => { /* handle encoded chunk */ },
  error: (e) => console.error(e),
});
encoder.configure({ codec: 'avc1.42E01E', width, height, bitrate });
```

The `RealtimeCapturer` sends `CapturedFrame` objects to `WebCodecsEncoder` (`video/realtime/webcodecs.rs`) which wraps them in `VideoFrame` objects for the browser encoder.

### URL State Sharing

The share URL hash (`#v1=...`) is read at startup via:

```rust
web_sys::window()
    .and_then(|w| w.location().hash().ok())
```

This works in all browsers with no server-side support. The compressed, base64url-encoded `SimState` is decoded and applied as the initial simulation state.

---

## Common: Shader Hot-Reload (Development)

`platform/common/shader_reload.rs` uses the `notify` crate to watch `shaders/` for `.wgsl` file changes:

```
Mac:     FSEvents API
Windows: ReadDirectoryChangesW
```

On change, `naga` validates the shader in-process. If valid, the affected pipeline is recreated without restarting the app. This typically completes in < 100 ms.

Hot-reload is compiled only in debug builds or when `feature = "hot-reload"` is set. Release builds have this code stripped entirely.

---

## Common: Software Fallback

`platform/common/fallback.rs` handles the case where no real GPU is available (CI runners, VMs, headless servers). It requests `force_fallback_adapter = true` from wgpu, selecting the software rasteriser (wgpu's own lavapipe/WARP/Metal reference device).

In fallback mode:
- `QualityPreset::potato()` is forced regardless of user settings.
- The `GpuProfiler` returns zeroed timestamps (software rasteriser does not support timestamp queries).
- This mode is primarily used for CI integration tests to verify render pipeline compilation without a GPU.

---

## Build Requirements by Platform

### macOS

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown   # for WASM builds

# Asset pipeline tools
brew install openimageio ffmpeg
pip3 install numpy

# basisu (Basis Universal)
# Download binary from https://github.com/BinomialLLC/basis_universal/releases

# WASM tools (optional)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
brew install binaryen  # for wasm-opt
```

### Windows

```powershell
# Rust toolchain
winget install Rustlang.Rustup

# Build tools
winget install Microsoft.VisualStudio.2022.BuildTools

# WASM tools (optional)
scoop install binaryen
npm install -g wasm-pack

# Hardware encoder SDKs (optional, for development)
# NVENC: Install NVIDIA CUDA Toolkit 12.x
# AMF: Install AMD Radeon Software
# QSV: Install Intel Media SDK
```

### First-time setup (all platforms)

```bash
# 1. Pull Git LFS assets (starfield EXR, etc.)
git lfs pull

# 2. Generate compiled assets
./build/convert_assets.sh

# 3. Build and run
cargo run --release
```