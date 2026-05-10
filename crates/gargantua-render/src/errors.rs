// =============================================================================
// crates/gargantua-render/src/errors.rs
// =============================================================================
//
// PURPOSE:
//   Defines RenderError — the unified error type for the gargantua-render
//   crate. All public functions that can fail return Result<T, RenderError>.
//   Wraps CoreError for pipeline and GPU errors, and adds render-specific
//   variants for shader loading and bind group mismatches.
//
// SIZE: ~80 lines
//
// DEPENDENCIES:
//   Internal:
//     - gargantua_core::errors::CoreError  — wrapped via #[from]
//   External:
//     - thiserror::Error
//
// CALLED BY:
//   - All modules in gargantua-render that return Result<T, RenderError>
//   - crates/gargantua-app/src/errors.rs  — wraps RenderError in AppError
//
// PUBLIC TYPES:
//
//   #[derive(Debug, thiserror::Error)]
//   pub enum RenderError {
//
//     #[error("Core engine error: {0}")]
//     Core(#[from] gargantua_core::errors::CoreError),
//       — propagates all CoreError variants (GPU, surface, OOM, etc.)
//       — #[from] enables ? operator from CoreError in render functions.
//
//     #[error("Shader not found: {path}")]
//     ShaderNotFound { path: String },
//       — returned by shader_reload.rs when a .wgsl file is missing.
//       — path is the relative path from the workspace root.
//
//     #[error("Shader compilation error in {shader}: {message}")]
//     ShaderCompilation { shader: String, message: String },
//       — returned by shader_reload.rs when wgpu reports a pipeline error.
//       — distinct from CoreError::ShaderCompilationFailed so render code
//         can handle it separately (display in UI, not crash).
//
//     #[error("Bind group layout mismatch in {pass}: {detail}")]
//     BindGroupMismatch { pass: String, detail: String },
//       — returned when a pass's shader expects different bindings than
//         the BindGroupLayout provided. Indicates a programming error.
//
//     #[error("Missing baked texture: {name}")]
//     MissingBakedTexture { name: String },
//       — returned by TexturesBindGroup::new() if a required baked
//         texture is None in BakedTextures struct.
//
//     #[error("Post-fx chain error in {pass}: {source}")]
//     PostFx {
//       pass:   String,
//       #[source]
//       source: Box<RenderError>,
//     },
//       — wraps errors from individual post-fx passes with the pass name.
//       — allows the caller to identify which pass failed.
//
//     #[error("HDR output error: {0}")]
//     HdrOutput(String),
//       — returned by hdr.rs when EDR or HDR10 setup fails.
//       — non-fatal: renderer falls back to SDR output.
//   }
//
// NOTES FOR AI:
//   - RenderError is Send + Sync (all fields are Send + Sync).
//   - The #[from] derive on Core variant allows using ? on any CoreError
//     in render functions: fn foo() -> Result<(), RenderError> { ctx.new()?; }
//   - PostFx variant uses Box<RenderError> for indirection — required because
//     RenderError cannot contain itself without heap allocation (recursive type).
//   - ShaderCompilation is intentionally non-fatal in gargantua-render:
//     the render loop catches it, displays the error in the UI overlay,
//     and continues using the last successfully compiled pipeline.
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Core engine error: {0}")]
    Core(#[from] gargantua_core::errors::CoreError),

    #[error("Shader not found: {path}")]
    ShaderNotFound { path: String },

    #[error("Shader compilation error in {shader}: {message}")]
    ShaderCompilation { shader: String, message: String },

    #[error("Bind group layout mismatch in {pass}: {detail}")]
    BindGroupMismatch { pass: String, detail: String },

    #[error("Missing baked texture: {name}")]
    MissingBakedTexture { name: String },

    #[error("Post-fx chain error in {pass}: {source}")]
    PostFx {
        pass:   String,
        #[source]
        source: Box<RenderError>,
    },

    #[error("HDR output error: {0}")]
    HdrOutput(String),
}