#!/usr/bin/env bash
# =============================================================================
# FILE: build/wasm-pack.sh
# PROJECT: gargantua-reimagined
# LINES: ~180
# PLATFORM: Mac + Linux CI (produces WASM output for browser deployment)
# =============================================================================
#
# PURPOSE:
#   Builds the Gargantua WebAssembly bundle for browser deployment using
#   wasm-pack. Compiles the Rust codebase to wasm32-unknown-unknown, runs
#   wasm-bindgen to generate JavaScript/TypeScript glue code, applies
#   wasm-opt optimisations, and assembles the final web/ output directory
#   ready for static hosting (Netlify, GitHub Pages, Cloudflare Pages).
#
# WHAT THIS SCRIPT DOES (in order):
#
#   1. ENVIRONMENT CHECK
#      Verifies required tools; exits 1 if missing:
#        - wasm-pack >= 0.12.0   (Rust → WASM + JS glue generator)
#        - wasm-opt >= 116       (Binaryen WASM optimiser)
#        - node >= 18            (for bundling and asset copying)
#        - npm or pnpm           (JS package manager for web frontend)
#      Checks that wasm32-unknown-unknown target is installed:
#        rustup target list --installed | grep wasm32-unknown-unknown
#      Prints all versions.
#
#   2. CLEAN (optional, --clean flag)
#      If --clean is passed:
#        rm -rf pkg/ web/wasm/
#      Ensures a fully fresh build. Not run by default (incremental is faster).
#
#   3. WASM COMPILATION (wasm-pack build)
#      Runs wasm-pack with the following flags:
#        wasm-pack build \
#          --target web \
#          --out-dir pkg \
#          --release \
#          -- \
#          --features "wasm" \
#          -Z build-std=std,panic_abort \
#          -Z build-std-features=panic_immediate_abort
#
#      Flag explanations:
#        --target web       → generates ES module (import { init } from './pkg/...js')
#                             rather than Node.js or bundler format
#        --out-dir pkg      → output JS + WASM files to pkg/ directory
#        --release          → cargo --release (full optimisations + LTO)
#        --features "wasm"  → enables cfg(target_arch="wasm32") code paths:
#                               - tracing_wasm instead of tracing_subscriber
#                               - webcodecs encoder instead of VideoToolbox/NVENC
#                               - web_sys::window() for URL hash reading
#                               - wasm_bindgen_futures for async GPU init
#        -Z build-std       → recompiles std with panic_immediate_abort to
#                             remove panic formatting code (~100 KB savings)
#
#      Expected output in pkg/:
#        gargantua_bg.wasm         — compiled WASM binary (~8–15 MB before opt)
#        gargantua.js              — ES module JS glue
#        gargantua.d.ts            — TypeScript type definitions
#        package.json              — npm package metadata
#
#   4. WASM OPTIMISATION
#      Runs two wasm-opt passes (same as optimize.sh WASM section):
#        wasm-opt -O3 --enable-simd pkg/gargantua_bg.wasm -o pkg/tmp.wasm
#        wasm-opt -Oz --strip-debug pkg/tmp.wasm -o pkg/gargantua_bg.wasm
#        rm pkg/tmp.wasm
#      Prints size before and after. Target: < 5 MB final .wasm.
#
#   5. SHADER VALIDATION (pre-bundle check)
#      Calls ./build/validate_shaders.sh --no-spirv
#      WGSL shaders are embedded in the binary as include_str!; any shader
#      error would only appear at runtime in the browser. Validate here first.
#      Exits 2 if validation fails.
#
#   6. ASSET COPYING
#      Copies compiled assets into the web output directory:
#        mkdir -p web/wasm/
#        cp pkg/gargantua_bg.wasm      web/wasm/
#        cp pkg/gargantua.js           web/wasm/
#        cp pkg/gargantua.d.ts         web/wasm/
#        cp -r assets/compiled/luts/   web/assets/luts/
#        cp -r assets/compiled/textures/ web/assets/textures/
#        cp web/index.html             web/  (already there, no-op)
#      Note: CoreML models are NOT copied (browser cannot use them).
#
#   7. JAVASCRIPT FRONTEND BUILD (if web/package.json exists)
#      cd web/ && npm ci && npm run build
#      The web frontend (TypeScript + Vite) imports the wasm pkg and bundles:
#        - UI overlay HTML/CSS
#        - WebCodecs recording UI
#        - Share link handler (reads URL hash → decode → SimState)
#      Output: web/dist/ (static files ready for deployment)
#
#   8. SERVICE WORKER AND CACHING HEADERS
#      Generates a service worker manifest for offline support:
#        python3 scripts/gen_sw_manifest.py web/dist/ > web/dist/sw_manifest.json
#      The service worker (web/sw.js) pre-caches the .wasm file so returning
#      visitors do not re-download 5 MB on every visit.
#
#   9. DEPLOYMENT SUMMARY
#      Prints final bundle sizes:
#        WASM binary:    X.X MB
#        JS glue:        XX KB
#        Assets:         XXX MB
#        Total transfer: XXX MB (estimated, without compression)
#      Writes summary to web/dist/build_info.json for display in the app footer.
#
# USAGE:
#   ./build/wasm-pack.sh                  — full WASM build + frontend bundle
#   ./build/wasm-pack.sh --clean          — clean build (removes pkg/ first)
#   ./build/wasm-pack.sh --no-opt         — skip wasm-opt (faster dev builds)
#   ./build/wasm-pack.sh --no-frontend    — skip npm build (WASM only)
#   ./build/wasm-pack.sh --dev            — debug build (no --release, no wasm-opt)
#
# EXIT CODES:
#   0 — WASM build and frontend bundle succeeded
#   1 — required tool missing (wasm-pack, node, wasm-opt)
#   2 — wasm-pack build failed (Rust compile error)
#   3 — wasm-opt failed
#   4 — shader validation failed
#   5 — npm build failed
#
# ENVIRONMENT VARIABLES RESPECTED:
#   GARGANTUA_WASM_NO_OPT=1    — skip wasm-opt (same as --no-opt)
#   GARGANTUA_DEPLOY_BASE=URL  — base URL for asset paths in index.html
#                                 (default: "/" for root deployment)
#   RUSTFLAGS                  — forwarded to cargo; e.g. -C target-feature=+simd128
#
# CALLED BY:
#   - .github/workflows/deploy-web.yml  — on push to main branch
#   - Makefile target `wasm`:  make wasm
#   - Netlify build command:   ./build/wasm-pack.sh (configured in netlify.toml)
#
# DEPENDENCIES:
#   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-opt   (or: brew install binaryen)
#   node >= 18  (brew install node / nvm install 20)
#
# NOTES:
#   - The "wasm" Cargo feature gate disables platform-specific code
#     (VideoToolbox, NVENC, ANE, D3D12) that cannot compile to wasm32.
#     Every #[cfg(target_arch="wasm32")] block in the codebase is enabled
#     by this feature; see each module's NOTES section for details.
#   - WASM SIMD (--enable-simd) requires Chrome 91+ or Firefox 89+.
#     Falls back to scalar code automatically via the wasm feature detection
#     in web/wasm_init.js if the browser does not support SIMD.
#   - The -Z build-std flags require Rust nightly. The rust-toolchain.toml
#     in the repo root pins the nightly version used by CI.
#   - web/dist/ is gitignored. The Netlify deploy uploads this directory
#     directly without committing it to the repository.
# =============================================================================