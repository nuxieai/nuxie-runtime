//! Neutral native-Metal context options shared by the product boundary and
//! the source-owned context constructor.
//!
//! These are configuration DTOs only.  Pipeline/cache behavior remains in
//! the canonical mechanical Metal owner (or in cfg(test) legacy fixtures).

/// Exact behavior choices from pinned upstream `ShaderCompilationMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShaderCompilationMode {
    /// Upstream `standard` is an alias for `allowAsynchronous`.
    #[default]
    AllowAsynchronous,
    AlwaysSynchronous,
    OnlyUbershaders,
}

/// Exact stored values from pinned tools-only `SynthesizedFailureType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NativeMetalSynthesizedFailureType {
    #[default]
    None,
    UbershaderLoad,
    ShaderCompilation,
    PipelineCreation,
}

/// Context options consumed by the canonical source context constructor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeMetalContextOptions {
    pub shader_compilation_mode: ShaderCompilationMode,
    pub disable_framebuffer_reads: bool,
    pub synthesized_failure_type: NativeMetalSynthesizedFailureType,
}
