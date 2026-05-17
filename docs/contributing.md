# Contributing to Gargantua

Thank you for contributing. This document covers the development workflow, coding standards, commit conventions, CI requirements, and how to add new features to each crate.

---

## Getting Started

### Prerequisites

- Rust stable (see `rust-toolchain.toml` for the exact pinned version)
- Rust nightly (for WASM builds with `-Z build-std`)
- Git LFS (for large asset files)
- Platform tools: see `docs/platform.md` "Build Requirements" section

### First-time setup

```bash
git clone https://github.com/your-org/gargantua-reimagined
cd gargantua-reimagined
git lfs pull                       # download starfield EXR, CoreML model
./build/convert_assets.sh          # generate compiled assets (~5 min first run)
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit   # optional
make install-hooks                 # installs validate_shaders.sh as pre-commit hook
cargo build                        # verify everything compiles
cargo test                         # run all tests
```

### Development builds

```bash
cargo run                          # debug build, hot-reload enabled
cargo run --release                # release build
cargo run --features hot-reload    # explicit hot-reload in release
```

---

## Repository Structure

```
crates/            — all Rust crates (one directory per crate)
shaders/           — WGSL shader source files
assets/
  raw/             — source assets (large files in Git LFS)
  compiled/        — generated at build time, gitignored
build/             — shell scripts: convert_assets, optimize, validate_shaders, wasm-pack
docs/              — this documentation
benches/           — Criterion benchmark entry points
tests/             — integration tests that span multiple crates
.github/workflows/ — CI/CD pipeline definitions
```

---

## Development Workflow

### Branching

- `main` — always deployable; protected branch, requires passing CI
- `feature/<name>` — new features
- `fix/<name>` — bug fixes
- `refactor/<name>` — code quality improvements without behaviour change
- `perf/<name>` — performance improvements

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short description>

[optional body]

[optional footer: BREAKING CHANGE: ...]
```

**Types:** `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`

**Scopes** map to crate names: `core`, `physics`, `bake`, `render`, `camera`, `video`, `ui`, `app`

**Examples:**

```
feat(physics): add Novikov-Thorne luminosity efficiency calculation
fix(render): correct HDR10 tonemap PQ curve constant m2
perf(core): reduce PhysicsSync rebuild frequency by caching last spin value
docs(physics): add Penrose process formula reference
build(wasm): upgrade wasm-opt to Binaryen 117
```

Breaking changes:

```
feat(app)!: change SimState URL schema from v1 to v2

BREAKING CHANGE: existing shared URLs using #v1= prefix will not decode.
Migration: decode v1 format in url_serde.rs decode() using legacy path.
```

---

## Code Style

### Rust Formatting

All Rust code is formatted with `rustfmt`. The project-specific settings are in `rustfmt.toml`:

```toml
max_width = 100
tab_spaces = 4
use_small_heuristics = "Max"
imports_granularity = "Crate"
```

Run before committing: `cargo fmt --all`

### Linting

`clippy.toml` enables strict lints. Zero clippy warnings are allowed on `main`. Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Key enforced lints:
- `clippy::unwrap_used` — use `?` or explicit `expect("reason")`
- `clippy::panic` — no panic! in library code (allowed in tests and benches)
- `clippy::todo` — no unfinished TODO code in non-draft PRs
- `clippy::perf` — all performance suggestions are treated as errors

### Error Handling

- Library crates (`gargantua-physics`, `gargantua-core`, etc.) must return `Result<T, CrateError>` from all fallible public functions. Never panic or `unwrap()` in library code.
- The error type for each crate is defined in `src/errors.rs` using `thiserror`.
- `gargantua-app` aggregates errors into `AppError` via `#[from]` impls.
- Use `?` for error propagation; avoid `match err { ... }` unless you need to handle specific variants.

### Safety

- No `unsafe` code outside of platform HAL files (`platform/macos/` and `platform/windows/`).
- All `unsafe` blocks must have a `// SAFETY:` comment explaining the invariant being upheld.
- Platform FFI bindings use `unsafe extern "C"` correctly with explicit null checks.

### GPU Resources

- Never create GPU objects (pipelines, bind groups, textures) inside `record()` / per-frame methods. Create them at startup in `new()` and cache.
- All `wgpu::Buffer` and `wgpu::Texture` allocations must go through `ResourcePool` (not created directly).
- Label every GPU resource: `wgpu::BufferDescriptor { label: Some("PhysicsUniforms"), ... }`.

---

## Adding a New Render Pass

1. Create `crates/gargantua-render/src/pipelines/my_pass.rs`
2. Implement `Pass` trait (or `ComputePass` / `RenderPass` marker trait)
3. Declare `reads()` and `writes()` resource lists so `FrameGraph::compile()` places it correctly
4. Create the WGSL shader in `shaders/my_pass.wgsl`
5. Register the pass in `gargantua-app/src/lib.rs` (or the appropriate startup function)
6. Run `build/validate_shaders.sh` to verify the shader
7. Add a `cargo test` integration test verifying the pass compiles and records without panicking

**File header comment template** (required for all new `.rs` files — see existing files for the full format):

```rust
// =============================================================================
// FILE: crates/gargantua-render/src/pipelines/my_pass.rs
// CRATE: gargantua-render
// LINES: ~NNN
// PLATFORM: Mac + Windows + WASM
// =============================================================================
//
// PURPOSE:
//   One paragraph describing what this file does and why it exists.
// ...
// =============================================================================
```

---

## Adding a New Quality Preset Tier

1. Add a variant to `QualityLevel` in `crates/gargantua-core/src/quality/preset.rs`
2. Add a constructor function in `preset.rs` with the SPP/steps/scale values
3. Add `from_level()` dispatch
4. Update the Mac chip tier files that should use the new preset (e.g. `m5_series.rs`)
5. Update the Windows vendor preset files (`nvidia_presets.rs`, etc.)
6. Add the new level to the UI selector in `gargantua-ui/src/menu/tabs/render_tab.rs`

---

## Adding a New Physics Effect

1. Create `crates/gargantua-physics/src/effects/my_effect.rs`
2. Write pure Rust functions using `f64` (no GPU code in physics crate)
3. Expose any GPU-needed parameters via the `PhysicsUniforms` struct (add fields to `render/src/bindgroups/physics.rs` and update the WGSL `@group(0) @binding(0)` struct)
4. Write unit tests in `crates/gargantua-physics/tests/` with known analytical values
5. Write the file header comment following the project standard

---

## Adding a New Platform

To support a new OS or GPU backend:

1. Create `crates/gargantua-core/src/platform/<platform>/` directory
2. Add `#[cfg(target_os = "<os>")] pub mod <platform>;` to `platform/mod.rs`
3. Implement GPU adapter selection, compute configuration, and memory model
4. Add a quality detector path in `quality/detector.rs`
5. Add platform-specific encoder support in `gargantua-video/src/encode/`
6. Update `docs/platform.md` with the new platform's requirements and limitations

---

## CI Requirements

All pull requests must pass the following CI checks (`.github/workflows/ci.yml`):

| Check | Command | Runs on |
|---|---|---|
| Format | `cargo fmt --all -- --check` | Ubuntu |
| Clippy | `cargo clippy --all-targets -D warnings` | Ubuntu, macOS, Windows |
| Tests | `cargo test --all` | Ubuntu, macOS, Windows |
| WASM build | `wasm-pack build --target web --features wasm` | Ubuntu |
| Shader validation | `./build/validate_shaders.sh --strict` | Ubuntu |
| Dependency audit | `cargo deny check` | Ubuntu |

The CI runs on three operating systems. Clippy and tests must pass on all three.

### GPU Tests

Tests that require a physical GPU are annotated `#[ignore]` and run on self-hosted runners:

```rust
#[test]
#[ignore = "requires physical GPU"]
fn test_ray_march_produces_nonzero_output() { ... }
```

Run locally with: `cargo test -- --include-ignored`

### Benchmark Baseline

Before submitting performance-sensitive changes, run benchmarks and include results in the PR description:

```bash
cargo bench --bench geodesic_rk4 > before.txt
# make your changes
cargo bench --bench geodesic_rk4 > after.txt
cargo benchcmp before.txt after.txt
```

---

## Writing Tests

### Unit tests

Place in the same file as the code under test (inside `#[cfg(test)] mod tests { ... }`).

### Integration tests

Place in `crates/<crate>/tests/<test_name>.rs`. Use the file header comment format documenting what each test verifies, expected values, and any platform restrictions.

### Test helper pattern

```rust
// In crates/gargantua-physics/tests/geodesic.rs:
fn make_kerr(spin: f64) -> KerrMetric {
    KerrMetric::new(1.0, spin, 0.0).expect("valid Kerr metric")
}
```

Never use `unwrap()` in tests without an `expect("reason")` that explains what the test assumes.

---

## Shader Development

### WGSL Style

- One `@group(N) @binding(M)` resource per line, with a comment explaining what it is
- All physical constants come from `PhysicsUniforms`, never as WGSL literals
- `@workgroup_size` must match the Rust dispatch config in the corresponding platform file
- Use `let` for intermediate values; avoid deeply nested expressions
- Comments for non-obvious mathematical steps (e.g. geodesic equation terms)

### Shader Testing

WGSL shaders cannot be unit-tested directly. Instead:

1. Use `build/validate_shaders.sh` to catch parse errors and MSL/HLSL transpilation issues
2. Write render integration tests that dispatch the shader and check the output texture is non-zero (via `GpuContext::new_headless()` in test setup)
3. Use the hot-reload feature during development to iterate quickly without restarting

---

## Documentation

- Every public Rust type and function must have a `///` doc comment.
- Every `.rs` file must have the full file header comment block (as described above and shown in all existing files).
- `docs/` markdown files should be updated when architecture or APIs change.
- Physics formulae should be typeset in code blocks using standard mathematical notation (not LaTeX, since GitHub renders plain markdown).

---

## Release Process

Releases are tagged on `main` with a semantic version (`v1.2.3`). The `release.yml` CI workflow runs automatically on tag push:

1. `cargo test --all --release` on all three platforms
2. `./build/validate_shaders.sh --strict`
3. `cargo build --release` for Mac (arm64 + x86_64 universal binary)
4. `cargo build --release` for Windows (x86_64-pc-windows-msvc)
5. `./build/wasm-pack.sh` for the browser bundle
6. `./build/optimize.sh` for binary stripping and WASM optimisation
7. Upload artefacts to GitHub Releases
8. Deploy WASM bundle to CDN (configured in `netlify.toml`)

For Mac distribution builds, replace the ad-hoc codesign in `optimize.sh` with a Developer ID certificate and run `xcrun notarytool` before release.

---

## Getting Help

- Open a GitHub Discussion for design questions or feature proposals
- Open a GitHub Issue for bugs, with a minimal reproduction case
- For physics questions, reference `docs/physics.md` and the papers listed there
- For platform-specific questions, reference `docs/platform.md`