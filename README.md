# Gargantua Reimagined

Physically accurate Kerr black hole renderer (Rust, wgpu, macOS / Windows / WASM).

## Status

**Specification phase:** most modules are documented via file-header comments. Implementation is filled in crate-by-crate; see `docs/ai-guide.md` for path conventions.

## Quick start (after implementation)

```bash
git lfs pull
./build/convert_assets.sh
cargo run -p gargantua-app
```

## Docs

- [Architecture](docs/architecture.md)
- [AI agent guide](docs/ai-guide.md)
- [Contributing](docs/contributing.md)
- [Physics](docs/physics.md)
- [Platform](docs/platform.md)

