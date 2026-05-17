#!/usr/bin/env bash
# =============================================================================
# FILE: build/validate_shaders.sh
// PROJECT: gargantua-reimagined
# LINES: ~200
# PLATFORM: Mac + Linux CI + Windows (Git Bash / WSL2)
# =============================================================================
#
# PURPOSE:
#   Validates all WGSL shader files in the shaders/ directory before a build.
#   Uses naga (the wgpu shader compiler) to parse, validate, and optionally
#   transpile every .wgsl file to Metal Shading Language (MSL) and HLSL/DXIL,
#   catching shader errors at build time rather than at runtime.
#   Also runs a custom lint pass to enforce project-specific shader conventions.
#   Intended to be run as a pre-commit hook and in CI before `cargo build`.
#
# WHAT THIS SCRIPT DOES (in order):
#
#   1. ENVIRONMENT CHECK
#      Verifies required tools are on PATH; exits 1 if missing:
#        - naga         (wgpu's standalone WGSL validator CLI)
#        - spirv-val    (Khronos SPIRV-Tools validator, optional but recommended)
#        - wgsl-analyzer (optional LSP-style linter for additional diagnostics)
#      Checks naga version >= 0.19 (minimum for wgsl subgroup extensions).
#      Prints tool versions to stdout.
#
#   2. DISCOVER SHADER FILES
#      Finds all .wgsl files recursively under shaders/:
#        SHADERS=$(find shaders/ -name "*.wgsl" | sort)
#      Prints total count: "Found N shader files to validate."
#      Also discovers any .wgsl files embedded in Rust source as include_str!:
#        grep -r 'include_str!.*\.wgsl' crates/ --include="*.rs" -h \
#          | sed 's/.*include_str!("\(.*\)").*/\1/' >> SHADERS
#      De-duplicates the combined list.
#
#   3. WGSL PARSE AND VALIDATION (naga)
#      For each shader file:
#        naga --validate $SHADER_FILE
#      If naga exits non-zero:
#        Prints "FAIL: $SHADER_FILE" with the naga error message.
#        Increments FAIL_COUNT.
#      If naga exits 0:
#        Prints "OK:   $SHADER_FILE"
#      Continues to next file (does not abort on first failure, so all errors
#      are reported in a single run).
#
#   4. MSL TRANSPILATION CHECK (Mac builds only)
#      Runs only when [[ "$(uname)" == "Darwin" ]] or GARGANTUA_CHECK_MSL=1:
#      For each shader file:
#        naga --validate --output-msl /dev/null $SHADER_FILE
#      Verifies that naga can generate valid MSL from the WGSL source.
#      A shader can be valid WGSL but untranspilatable to MSL if it uses
#      features not yet supported by naga's Metal backend (e.g. certain
#      subgroup operations, 64-bit atomics).
#      Failures here are marked as WARN (not FAIL) since they may be
#      acceptable if the feature is only used on non-Metal platforms.
#
#   5. HLSL TRANSPILATION CHECK (Windows builds)
#      Runs when GARGANTUA_CHECK_HLSL=1 or running on Windows:
#        naga --validate --output-hlsl /dev/null $SHADER_FILE
#      Same warning-vs-error logic as MSL check above.
#
#   6. SPIRV VALIDATION (optional, if spirv-val is available)
#      For each shader:
#        naga --output-spv /tmp/shader_validate.spv $SHADER_FILE
#        spirv-val /tmp/shader_validate.spv
#      spirv-val applies stricter checks than naga alone (e.g. memory model
#      consistency, decoration conflicts). Failures are FAIL (not WARN).
#
#   7. PROJECT-SPECIFIC LINT RULES
#      Applies custom checks via grep/awk for project conventions:
#
#      RULE 1 — No f32 literals without suffix in uniforms:
#        Checks that all struct fields in @group bind groups use explicit
#        types (vec4<f32>, not vec4). Pattern: grep for bare "vec[234]," in
#        @group struct declarations. Warns on violations.
#
#      RULE 2 — workgroup_size must match platform threadgroup config:
#        Checks that @workgroup_size annotations in compute shaders match the
#        values defined in platform/macos/compute/threadgroup.rs and
//        platform/windows/compute/workgroup.rs. Uses a Python helper:
#          python3 scripts/check_workgroup_sizes.py shaders/ crates/
#        Fails if a mismatch is found (build-time guard against divergence).
#
#      RULE 3 — No hardcoded physical constants in shaders:
#        Checks that values like 6.674e-11 (G), 2.998e8 (c), 1.989e30 (M_sun)
#        do not appear as literals in WGSL files. Physical constants must come
#        from the PhysicsUniforms uniform buffer, not be hardcoded.
#        Pattern: grep -n '[0-9]\.[0-9]*e[-+][0-9]\{2,\}' shaders/
#        Prints file:line for each violation.
#
#      RULE 4 — All compute shaders must have a @group(0) @binding(0) block:
#        Ensures no compute shader was written without a uniform buffer binding.
#        A compute shader without uniforms is likely missing its physics params.
#
#   8. SUMMARY REPORT
#      Prints a final summary:
#        "Shader validation complete: N passed, F failed, W warnings"
#      If FAIL_COUNT > 0: exits with code 2.
#      If only warnings: exits with code 0 (warnings do not fail CI).
#      Writes full report to target/shader_validation_report.txt.
#
# USAGE:
#   ./build/validate_shaders.sh                    — validate all shaders
#   ./build/validate_shaders.sh shaders/ray_march.wgsl  — validate single file
#   ./build/validate_shaders.sh --msl              — force MSL check on Linux
#   ./build/validate_shaders.sh --no-spirv         — skip spirv-val pass
#   ./build/validate_shaders.sh --strict           — treat warnings as errors
#
# EXIT CODES:
#   0 — all shaders valid (warnings allowed unless --strict)
#   1 — required tool (naga) missing
#   2 — one or more shaders failed validation
#   3 — lint rule violation found (only with --strict)
#
# ENVIRONMENT VARIABLES RESPECTED:
#   GARGANTUA_CHECK_MSL=1    — force MSL transpilation check (default: Mac only)
#   GARGANTUA_CHECK_HLSL=1   — force HLSL check (default: Windows only)
#   GARGANTUA_STRICT=1       — treat warnings as errors (same as --strict flag)
#
# CALLED BY:
#   - .git/hooks/pre-commit   — installed by `make install-hooks`
#   - .github/workflows/ci.yml — runs as a separate "shader-validate" job
#   - Makefile target `shaders`: make shaders
#   - platform/common/shader_reload.rs → calls this script in dev builds when
#     a .wgsl file changes (via std::process::Command)
#
# DEPENDENCIES:
#   cargo install naga-cli          — naga CLI tool
#   cargo install spirv-tools       — spirv-val (optional)
#   pip3 install wgsl-analyzer      — optional LSP linter
#
# NOTES:
#   - naga is the same parser/validator that wgpu uses internally. Passing
#     this script guarantees the shaders will compile at runtime (on the
#     wgpu path). Metal/DX12 driver-specific errors are not caught here.
#   - The workgroup_size lint (Rule 2) is the most important: a mismatch
#     between the Rust dispatch size and the WGSL @workgroup_size causes
#     silent under-utilisation or out-of-bounds access on the GPU.
#   - Run with --strict in release CI; run without --strict in pre-commit
#     hooks so that work-in-progress shaders don't block commits.
# =============================================================================