# AI Agent Guide — Gargantua Reimagined

This repository is being built **specification-first**: many `.rs` files contain file-header comments and type signatures before full implementations. **Read the header comments in each file before editing it.** This document is the canonical map of paths and naming rules so agents do not invent wrong modules.

---

## Crate roles (where logic lives)

| Crate | Responsibility | Do **not** put here |
|---|---|---|
| `gargantua-core` | `App`, GPU, `FrameGraph`, platform HAL, clock, quality | `SimState`, UI, physics math |
| `gargantua-physics` | Kerr metric, geodesics, accretion, effects (CPU `f64`) | GPU, wgpu, egui |
| `gargantua-bake` | Offline LUT generation, disk cache | Per-frame render |
| `gargantua-render` | Pipelines, postfx, bind groups, WGSL dispatch | Window / input |
| `gargantua-camera` | Camera modes, paths, relativistic camera FX | Physics integrator |
| `gargantua-video` | Encode, denoise, offline export | Scene rendering |
| `gargantua-ui` | egui HUD, menus, presets (reads `SimState` via events) | Direct GPU calls |
| `gargantua-app` | `SimState`, plugins, `PhysicsSync`, input, undo, URL share | Low-level GPU / `FrameGraph` |

---

## Critical paths (memorise these)

```
Engine loop:     crates/gargantua-core/src/app.rs
Core modules:    crates/gargantua-core/src/lib.rs
Composition:     crates/gargantua-app/src/lib.rs
Binary: crates/gargantua-app/src/main.rs (stub until event loop is wired)

SimState:        crates/gargantua-app/src/state/sim_state.rs
Event bus:       crates/gargantua-app/src/state/event_bus.rs
Input:           crates/gargantua-app/src/systems/input.rs
Physics → GPU:   crates/gargantua-app/src/systems/physics_sync.rs
Physics uniforms crates/gargantua-render/src/bindgroups/physics.rs

Bake scheduler:  crates/gargantua-bake/src/scheduler.rs
Geodesic LUT:    crates/gargantua-bake/src/geodesic/let_baker.rs   ← module name
Bake tests:      crates/gargantua-bake/tests/lut_baker.rs          ← test binary name

Scene passes:    crates/gargantua-render/src/pipelines/*.rs
PostFX passes:   crates/gargantua-render/src/postfx/*.rs
WGSL sources:    shaders/**/*.wgsl
Shader reload:   crates/gargantua-render/src/shader_reload.rs

Video encode:    crates/gargantua-video/src/encode/
```

---

## Paths that do **not** exist (common mistakes)

| Wrong path | Use instead |
|---|---|
| `gargantua-app/src/app.rs` | `gargantua-core/src/app.rs` |
| `gargantua-app/src/settings.rs` | `gargantua-app/src/state/sim_state.rs` |
| `gargantua-app/src/input/input_router.rs` | `gargantua-app/src/systems/input.rs` |
| `gargantua-app/src/systems/bake_runner.rs` | `gargantua-bake/src/scheduler.rs` + `gargantua-app/src/lib.rs` |
| `gargantua-render/src/pipelines/postfx.rs` | `gargantua-render/src/postfx/<pass>.rs` |
| `gargantua-render/src/pipelines/bloom.rs` | `gargantua-render/src/postfx/bloom.rs` |
| `gargantua-bake/src/geodesic/lut_baker.rs` | `gargantua-bake/src/geodesic/let_baker.rs` |
| `gargantua-video/src/encoder/*` | `gargantua-video/src/encode/*` |
| `platform/common/shader_reload.rs` | `gargantua-render/src/shader_reload.rs` |
| `tests/foo.rs` (bare) | `crates/<crate>/tests/foo.rs` |
| `github/workflows/` | `.github/workflows/` |

---

## File header comment template

When adding or fixing headers, use **full repo-relative paths** and one of these formats.

### Style A — `FILE:` (app, physics, bake, ui, video)

```rust
// ============================================================
// FILE: crates/gargantua-<crate>/src/module/file.rs
// LINES: ~NNN
// CATEGORY: <area> — short label
// PLATFORM: cross-platform | Mac + Windows | etc.
// ============================================================
//
// PURPOSE:
//   ...
//
// USES (imports from):
//   ...
//
// USED BY:
//   crates/.../consumer.rs
//   PLANNED: crates/.../future.rs
//
// NOTE FOR AI:
//   ...
// ============================================================
```

### Style B — banner (core, render, camera)

```rust
// =============================================================================
// crates/gargantua-<crate>/src/module/file.rs
// =============================================================================
//
// PURPOSE:
//   ...
//
// DEPENDENCIES: / CALLED BY: / NOTES FOR AI:
//   ...
// =============================================================================
```

### Cross-reference rules

1. List only real files, or prefix with **`PLANNED:`**.
2. One `FILE:` line per physical file — do not stack multiple file headers in one `.rs` file.
3. Shaders: `USED BY` must point at the Rust pass that loads the WGSL (usually `postfx/` or `pipelines/`).
4. On-disk typo: `metric/schwarzshild.rs` implements **Schwarzschild** — do not rename without a dedicated refactor.

---

## Frame tick order (implementation target)

Defined in `gargantua_core::app::App::tick()`; domain systems are called from `gargantua-app` wiring:

1. `Clock::tick()`
2. `InputSystem::handle_event` / `tick` (`gargantua-app`)
3. `PluginRegistry::tick_all`
4. `PhysicsSync::sync` → uploads `PhysicsUniforms`
5. `AdaptiveQuality::evaluate`
6. `FrameGraph::execute` (render + compute passes)
7. egui UI pass (`gargantua-ui`, driven from `App`)
8. `Surface::present`
9. `GpuProfiler::read_results`

---

## Adding new code (checklist)

| Task | Create | Register / wire |
|---|---|---|
| Physics formula | `gargantua-physics/src/...` | `physics/src/lib.rs` `pub mod` |
| Render pass | `gargantua-render/src/pipelines/` or `postfx/` | Pass registration in `core/src/app.rs` callback |
| WGSL shader | `shaders/...` | Matching Rust pass + `USED BY` in shader header |
| UI panel | `gargantua-ui/src/menu/tabs/` | `ui/src/menu/mod.rs` |
| App state field | `sim_state.rs` | `physics_sync.rs`, URL serde, undo snapshots |
| Platform HAL | `core/src/platform/<os>/` | `platform/mod.rs` |

---

## Further reading

- [architecture.md](./architecture.md) — system design, data flow, dependency graph
- [contributing.md](./contributing.md) — style, CI, PR workflow
- [physics.md](./physics.md) — equations and validation values
- [platform.md](./platform.md) — macOS / Windows / WASM specifics
