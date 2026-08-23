/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/flush_uniforms.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, functions, and dependencies as literal
 * source-shaped data. It does not compile, evaluate, simplify, or generate
 * shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "1c22659c0e40233b0b06515287e122e06b73a4428d8e78721ca71e3419db961e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 58;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2454;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/flush_uniforms_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_FLUSH_UNIFORMS_GLSL_SOURCE: &str = r###"#ifndef DECLARE_UNIFORM_FLOAT
#define DECLARE_UNIFORM_FLOAT(UNIFORM_NAME) float UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_UINT
#define DECLARE_UNIFORM_UINT(UNIFORM_NAME) uint UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_INT4
#define DECLARE_UNIFORM_INT4(UNIFORM_NAME) int4 UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_FLOAT2
#define DECLARE_UNIFORM_FLOAT2(UNIFORM_NAME) float2 UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_FLOAT4
#define DECLARE_UNIFORM_FLOAT4(UNIFORM_NAME) float4 UNIFORM_NAME;
#endif

#ifndef FLUSH_UNIFORMS_NAME
#define FLUSH_UNIFORMS_NAME @FlushUniforms
#endif

UNIFORM_BLOCK_BEGIN(FLUSH_UNIFORM_BUFFER_IDX, FLUSH_UNIFORMS_NAME)
DECLARE_UNIFORM_FLOAT(gradInverseViewportY)
DECLARE_UNIFORM_FLOAT(tessInverseViewportY)
DECLARE_UNIFORM_FLOAT(renderTargetInverseViewportX)
DECLARE_UNIFORM_FLOAT(renderTargetInverseViewportY)
DECLARE_UNIFORM_UINT(renderTargetWidth)
DECLARE_UNIFORM_UINT(renderTargetHeight)
// Only used if clears are implemented as draws.
DECLARE_UNIFORM_UINT(colorClearValue)
// Only used if clears are implemented as draws.
DECLARE_UNIFORM_UINT(coverageClearValue)
// drawBounds, or renderTargetBounds if there is a clear. (LTRB.)
DECLARE_UNIFORM_INT4(renderTargetUpdateBounds)
// 1 / [atlasWidth, atlasHeight]
DECLARE_UNIFORM_FLOAT2(atlasTextureInverseSize)
// 2 / atlasContentBounds
DECLARE_UNIFORM_FLOAT2(atlasContentInverseViewport)
DECLARE_UNIFORM_UINT(coverageBufferPrefix)
// GLSL doesn't appear to provide a lightweight, region-local barrier for memory
// ordering outside of memoryBarrier*(), which have severe consequences for
// tiling. When we are already relying on other API level barriers and only need
// to guard against instruction reordering, we can multiply by a tiny epsilon
// instead, and introduce artifical dependencies that enforce ordering but don't
// actually have an effect on the final outcome.
DECLARE_UNIFORM_FLOAT(epsilonForPseudoMemoryBarrier)
// Spacing between adjacent path IDs (1 if IEEE compliant).
DECLARE_UNIFORM_UINT(pathIDGranularity)
DECLARE_UNIFORM_FLOAT(vertexDiscardValue)
DECLARE_UNIFORM_FLOAT(mipMapLODBias)
DECLARE_UNIFORM_UINT(maxPathId)
DECLARE_UNIFORM_FLOAT(ditherScale)
DECLARE_UNIFORM_FLOAT(ditherBias)
// Amount by which to multiply a computed dither value when storing as RGB10 (as
// opposed to writing it out to the framebuffer).
DECLARE_UNIFORM_FLOAT(ditherConversionToRGB10)
// Debugging.
DECLARE_UNIFORM_UINT(wireframeEnabled)
UNIFORM_BLOCK_END(uniforms)"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_FLUSH_UNIFORMS_SOURCE: &str = PINNED_FLUSH_UNIFORMS_GLSL_SOURCE;
pub const FLUSH_UNIFORMS_GLSL_SOURCE: &str = PINNED_FLUSH_UNIFORMS_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_FLUSH_UNIFORMS_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Every semantic preprocessor block in the pinned source, in source order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[
    ConditionalBlock {
        block_id: "pp-0257",
        block_start: 1,
        block_end: 3,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0258",
        block_start: 4,
        block_end: 6,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0259",
        block_start: 7,
        block_end: 9,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0260",
        block_start: 10,
        block_end: 12,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0261",
        block_start: 13,
        block_end: 15,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0262",
        block_start: 17,
        block_end: 19,
        block_depth: 0,
        branch_count: 1,
    },
];

/// Every branch entry remains literal, in authority/source order. The active
/// paths are ledger spellings; they are not evaluated as Rust cfg expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[
    ConditionalBranch {
        block_id: "pp-0257",
        branch_ordinal: 1,
        branch_line: 1,
        directive: "#ifndef DECLARE_UNIFORM_FLOAT",
        active_branch_path: "(!defined(DECLARE_UNIFORM_FLOAT))",
    },
    ConditionalBranch {
        block_id: "pp-0258",
        branch_ordinal: 1,
        branch_line: 4,
        directive: "#ifndef DECLARE_UNIFORM_UINT",
        active_branch_path: "(!defined(DECLARE_UNIFORM_UINT))",
    },
    ConditionalBranch {
        block_id: "pp-0259",
        branch_ordinal: 1,
        branch_line: 7,
        directive: "#ifndef DECLARE_UNIFORM_INT4",
        active_branch_path: "(!defined(DECLARE_UNIFORM_INT4))",
    },
    ConditionalBranch {
        block_id: "pp-0260",
        branch_ordinal: 1,
        branch_line: 10,
        directive: "#ifndef DECLARE_UNIFORM_FLOAT2",
        active_branch_path: "(!defined(DECLARE_UNIFORM_FLOAT2))",
    },
    ConditionalBranch {
        block_id: "pp-0261",
        branch_ordinal: 1,
        branch_line: 13,
        directive: "#ifndef DECLARE_UNIFORM_FLOAT4",
        active_branch_path: "(!defined(DECLARE_UNIFORM_FLOAT4))",
    },
    ConditionalBranch {
        block_id: "pp-0262",
        branch_ordinal: 1,
        branch_line: 17,
        directive: "#ifndef FLUSH_UNIFORMS_NAME",
        active_branch_path: "(!defined(FLUSH_UNIFORMS_NAME))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The single @-prefixed identifier occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[ExportedSymbol {
    source_line: 18,
    source_name: "@FlushUniforms",
    generated_name: "NB",
    generated_header_name: "GLSL_FlushUniforms",
}];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &["FlushUniforms"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// This shader source has no function declarations.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[];
pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncludeDependency {
    pub including_source: &'static str,
    pub include_line: u16,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub translation_disposition: &'static str,
}

/// This shader source has no direct #include/#import directive. These incoming
/// generated-source edges are retained from the include/source dependency
/// authorities because they determine its artifact consumers.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 9,
        include_token: "generated/shaders/flush_uniforms.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/color_ramp.metal",
        include_line: 9,
        include_token: "flush_uniforms.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/draw.metal",
        include_line: 13,
        include_token: "flush_uniforms.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/tessellate.metal",
        include_line: 9,
        include_token: "flush_uniforms.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
