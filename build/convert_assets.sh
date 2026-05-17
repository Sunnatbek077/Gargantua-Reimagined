#!/usr/bin/env bash
# =============================================================================
# FILE: build/convert_assets.sh
# PROJECT: gargantua-reimagined
# LINES: ~180
# PLATFORM: Mac + Linux (CI runner); Windows via Git Bash or WSL2
# =============================================================================
#
# PURPOSE:
#   Pre-build asset pipeline. Converts raw source assets (EXR textures,
#   HDR environment maps, GLTF models, raw audio) into the optimised
#   runtime formats expected by the engine. Must be run once before the
#   first `cargo build`, and re-run whenever source assets change.
#   Produces files into assets/compiled/ which is gitignored.
#
# WHAT THIS SCRIPT DOES (in order):
#
#   1. ENVIRONMENT CHECK
#      Verifies required tools are on PATH; exits with error if missing:
#        - ffmpeg >= 5.0       (video/image conversion)
#        - oiiotool            (OpenImageIO, for EXR processing)
#        - basisu              (Basis Universal texture compressor)
#        - gltf-transform      (Node.js GLTF optimiser)
#        - python3             (for LUT generation scripts)
#      Prints each tool's version for build log traceability.
#
#   2. DIRECTORY SETUP
#      Creates output directories if they don't exist:
#        assets/compiled/textures/
#        assets/compiled/luts/
#        assets/compiled/env/
#        assets/compiled/audio/
#
#   3. STARFIELD EXR → COMPRESSED CUBEMAP
#      Input:  assets/raw/starfield_milkyway_16k.exr  (16K × 8K, 32-bit float)
#      Steps:
#        a. oiiotool: resize to 8192 × 4096 (equirectangular)
#        b. oiiotool: convert to 6-face cubemap layout (--envlayout cubecross)
#        c. oiiotool: downsample to 6 mip levels (--mipmaps)
#        d. basisu: compress to UASTC (high-quality BC7 equivalent for GPU)
#           Output: assets/compiled/textures/starfield_cubemap.ktx2
#      Skips if output .ktx2 is newer than input .exr (incremental build).
#
#   4. BLACKBODY LUT GENERATION
#      Runs: python3 scripts/gen_blackbody_lut.py
#        Input:  CIE 1931 2° colour matching functions (embedded in script)
#        Output: assets/compiled/luts/blackbody_1d.exr
#                  — 1D LUT: 1024 × 1 RGBA16F, temperature axis 1000 K–1e8 K
#        Also generates: assets/compiled/luts/blackbody_1d.png (preview)
#      Skips if output .exr is newer than gen_blackbody_lut.py.
#
#   5. DOPPLER LUT GENERATION
#      Runs: python3 scripts/gen_doppler_lut.py
#        Output: assets/compiled/luts/doppler_2d.exr
#                  — 2D LUT: 256 (beta) × 256 (wavelength) RGBA16F
#      Skips if up to date.
#
#   6. COLOUR SPACE 3D LUT CONVERSION
#      Input:  assets/raw/luts/rec2020_to_p3.cube  (33³ Adobe .cube)
#              assets/raw/luts/aces_rrt.cube        (65³ ACES RRT)
#      oiiotool: convert .cube → .exr (3D texture, 33³ or 65³ RGBA16F)
#      Output: assets/compiled/luts/rec2020_to_p3_33.exr
#              assets/compiled/luts/aces_rrt_65.exr
#
#   7. CORE ML MODEL COMPILATION (Mac only)
#      Runs only when: [[ "$(uname)" == "Darwin" ]]
#        xcrun coremlc compile \
#          assets/raw/models/gargantua_denoise.mlmodel \
#          assets/compiled/models/
#        Output: assets/compiled/models/gargantua_denoise.mlmodelc/
#      Skips if compiled model is newer than source .mlmodel.
#      On non-Mac: prints "Skipping CoreML compilation (not macOS)" and continues.
#
#   8. AUDIO ASSETS (future placeholder)
#      Currently a no-op loop over assets/raw/audio/*.wav.
#      Will run ffmpeg to convert WAV → Opus at 128 kbps when audio is added.
#
#   9. ASSET MANIFEST
#      Writes assets/compiled/manifest.json listing:
#        { "generated_at": "<ISO timestamp>",
#          "files": [ { "path": "...", "sha256": "...", "size_bytes": N }, ... ] }
#      Used by the engine at startup to verify asset integrity.
#      python3 scripts/gen_manifest.py assets/compiled/ > assets/compiled/manifest.json
#
# USAGE:
#   ./build/convert_assets.sh              — full conversion (incremental)
#   ./build/convert_assets.sh --force      — rebuild all outputs regardless of timestamps
#   ./build/convert_assets.sh --check      — dry-run: print what would be rebuilt, exit 0
#   ./build/convert_assets.sh --no-luts    — skip LUT generation (faster for texture-only changes)
#
# EXIT CODES:
#   0 — all assets converted successfully (or already up to date)
#   1 — missing required tool (printed to stderr)
#   2 — conversion step failed (oiiotool / basisu / python3 returned non-zero)
#   3 — output directory creation failed (permission error)
#
# ENVIRONMENT VARIABLES RESPECTED:
#   GARGANTUA_ASSET_THREADS=N   — parallelism for basisu compression (default: nproc)
#   GARGANTUA_SKIP_COREML=1     — skip CoreML compilation even on Mac
#   OIIO_LIBRARY_PATH           — path to OpenImageIO dylibs if not on standard PATH
#
# CALLED BY:
#   - Makefile target `assets`:  make assets
#   - .github/workflows/ci.yml:  runs before `cargo build` on the asset-build job
#   - README.md "Getting Started" section instructs developers to run this first
#
# DEPENDENCIES (must be installed separately — not managed by Cargo):
#   brew install openimageio           # Mac
#   brew install ffmpeg
#   npm install -g @gltf-transform/cli
#   pip3 install numpy                 # for LUT generation scripts
#   # basisu: download from https://github.com/BinomialLLC/basis_universal
#
# NOTES:
#   - All conversions are incremental: a file is only re-processed if its
#     source is newer than its output (standard Make-style dependency checking
#     via Bash `[ source -nt output ]` comparisons).
#   - The --force flag bypasses all timestamp checks and rebuilds everything.
#   - assets/compiled/ is listed in .gitignore; compiled assets are NOT
#     committed to the repository (too large). CI regenerates them from source.
#   - On Apple Silicon, basisu uses NEON SIMD automatically via the arm64 binary.
#   - The starfield EXR source file (800 MB) is stored in Git LFS.
#     Run `git lfs pull` before running this script if the file is a pointer.
# =============================================================================