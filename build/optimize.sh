#!/usr/bin/env bash
# =============================================================================
# FILE: build/optimize.sh
# PROJECT: gargantua-reimagined
# LINES: ~160
# PLATFORM: Mac + Linux CI; Windows via Git Bash or WSL2
# =============================================================================
#
# PURPOSE:
#   Post-build binary and asset optimisation pipeline. Runs after
#   `cargo build --release` to produce the smallest, fastest possible
#   distributable artefact. Applies dead-code stripping, LTO verification,
#   WASM size optimisation, and texture re-compression for the final bundle.
#   Should be run as the last step before packaging/signing.
#
# WHAT THIS SCRIPT DOES (in order):
#
#   1. ENVIRONMENT CHECK
#      Verifies required tools are on PATH; exits 1 if any are missing:
#        - strip / llvm-strip    (binary symbol stripping)
#        - wasm-opt              (Binaryen WASM optimiser, for WASM builds)
#        - upx                   (optional: binary packer for Windows .exe)
#        - dsymutil              (Mac only: debug symbol extraction)
#        - codesign              (Mac only: ad-hoc signing for local testing)
#      Prints each tool version for build log traceability.
#
#   2. DETECT BUILD TARGET
#      Reads the target triple from the first argument or CARGO_BUILD_TARGET env:
#        aarch64-apple-darwin    → Mac Apple Silicon
#        x86_64-apple-darwin     → Mac Intel
#        x86_64-pc-windows-msvc  → Windows
#        wasm32-unknown-unknown  → WebAssembly
#      Sets BINARY_PATH to:
#        target/<triple>/release/gargantua          (Mac/Linux)
#        target/<triple>/release/gargantua.exe      (Windows)
#        target/<triple>/release/gargantua_bg.wasm  (WASM)
#      Exits 2 if the binary does not exist (cargo build not yet run).
#
#   3. NATIVE BINARY STRIP (Mac / Linux)
#      Skipped for WASM target.
#        Mac:   dsymutil $BINARY_PATH -o target/release/gargantua.dSYM
#               strip -x $BINARY_PATH   (-x = remove local symbols, keep globals)
#               Saves .dSYM bundle alongside binary for crash report symbolication.
#        Linux: llvm-strip --strip-debug $BINARY_PATH
#               Removes debug info but preserves panic backtraces via addr2line.
#      Prints binary size before and after strip.
#      Expected size reduction: 30–60% for a debug-info-heavy Rust binary.
#
#   4. WASM OPTIMISATION (wasm32 target only)
#      Runs wasm-opt in two passes:
#        Pass 1 (speed):  wasm-opt -O3 --enable-simd $BINARY_PATH -o tmp.wasm
#        Pass 2 (size):   wasm-opt -Oz --strip-debug tmp.wasm -o $BINARY_PATH
#      -O3 = full optimisation (inlining, DCE, constant propagation)
#      -Oz = size optimisation on top (merges duplicate functions, shrinks tables)
#      --enable-simd = enables WASM SIMD instructions for vectorised ray march
#      Prints .wasm size before and after (target: < 5 MB for initial page load).
#      Also runs: wasm-opt --print-function-sizes $BINARY_PATH | head -20
#        Prints the 20 largest functions by size for bloat investigation.
#
#   5. WINDOWS EXE PACKING (Windows target only, optional)
#      Runs only if UPX is available AND env var GARGANTUA_USE_UPX=1:
#        upx --best --lzma $BINARY_PATH
#      UPX compresses the .exe ~50% but adds ~0.5 s startup decompression time.
#      Disabled by default; only recommended for distribution builds where
#      download size matters more than startup time.
#
#   6. MAC AD-HOC CODESIGN (Mac target only)
#      Runs: codesign --force --sign - $BINARY_PATH
#      Ad-hoc signing allows the binary to run on the build machine without
#      a Developer ID certificate. Required for Apple Silicon (arm64) binaries
#      that are not signed by Xcode's build system.
#      For distribution builds, replace "-" with the actual Developer ID:
#        codesign --force --sign "Developer ID Application: ..." $BINARY_PATH
#
#   7. COMPILED ASSET OPTIMISATION
#      Re-compresses any .ktx2 texture in assets/compiled/ that has not been
#      compressed at maximum quality level:
#        basisu -comp_level 5 -q 255 <input.ktx2> -output_file <output.ktx2>
#      Skips textures already at comp_level 5 (checks embedded metadata).
#      convert_assets.sh uses comp_level 2 (fast) for development iteration;
#      this script upgrades to comp_level 5 (slow, best quality) for release.
#
#   8. SIZE REPORT
#      Prints a final summary table to stdout:
#        Binary:           XX.X MB  (stripped)
#        WASM (if built):  X.X MB   (wasm-opt'd)
#        Assets total:     XXX MB
#        Bundle estimate:  XXX MB
#      Also writes the summary to target/release/size_report.txt for CI artefact.
#
# USAGE:
#   ./build/optimize.sh                              — auto-detect target from CARGO_BUILD_TARGET
#   ./build/optimize.sh aarch64-apple-darwin         — explicit target triple
#   ./build/optimize.sh wasm32-unknown-unknown       — WASM optimisation only
#   ./build/optimize.sh --skip-wasm-opt              — skip wasm-opt (faster CI)
#   ./build/optimize.sh --skip-assets                — skip texture re-compression
#
# EXIT CODES:
#   0 — optimisation complete
#   1 — required tool missing
#   2 — binary not found (cargo build not run)
#   3 — wasm-opt or strip returned non-zero
#   4 — codesign failed
#
# ENVIRONMENT VARIABLES RESPECTED:
#   CARGO_BUILD_TARGET      — target triple (set automatically by cargo build)
#   GARGANTUA_USE_UPX=1     — enable UPX packing on Windows (default: disabled)
#   GARGANTUA_SKIP_SIGN=1   — skip codesign step (useful in unsigned CI builds)
#   GARGANTUA_DIST=1        — enable all release optimisations including UPX
#
# CALLED BY:
#   - Makefile target `release`:  make release  (runs cargo build --release then this)
#   - .github/workflows/release.yml: post-build step on tag push
#   - CI does NOT run this on every PR — only on release branch builds
#
# DEPENDENCIES:
#   Mac:     brew install llvm binaryen   (for wasm-opt and llvm-strip)
#   Windows: scoop install upx            (optional)
#   All:     wasm-opt ships with the wasm-pack toolchain (see wasm-pack.sh)
#
# NOTES:
#   - LTO (Link-Time Optimisation) is configured in Cargo.toml [profile.release]:
#       lto = "fat"   — full cross-crate LTO (~20% binary size reduction)
#       codegen-units = 1
#     This script does NOT configure LTO; it only verifies and strips the result.
#   - wasm-opt -O3 can take 2–5 minutes on the full WASM binary. For CI speed,
#     use --skip-wasm-opt on non-release builds.
#   - dsymutil must run BEFORE strip on Mac (strip removes the addresses that
#     dsymutil needs to build the .dSYM bundle).
# =============================================================================