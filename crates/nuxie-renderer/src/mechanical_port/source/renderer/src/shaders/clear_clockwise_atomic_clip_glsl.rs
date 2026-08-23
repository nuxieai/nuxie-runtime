/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/clear_clockwise_atomic_clip.glsl.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/clear_clockwise_atomic_clip.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "75e9b26beb81bf9279a78e13c0510dca4e60f704ad9710ae368c116b1aa13da6";
pub const PINNED_SOURCE_LINE_COUNT: usize = 36;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 923;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/clear_clockwise_atomic_clip_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0, packed_float3, @a_triangleVertex);
ATTR_BLOCK_END

VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_triangleVertex, packed_float3);
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(@a_triangleVertex.xy);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT
PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
PLS_BLOCK_END

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    // srcOver blend is enabled: emit an alpha value of 1 to overwrite the
    // existing clip.
    PLS_STORE4F(clipBuffer, make_half4(.0, .0, .0, 1.));

    // srcOver blend is enabled: emit a color of 0 to make sure the framebuffer
    // remains unchanged.
    EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(.0));
}
#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_SOURCE: &str =
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE;
pub const CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE: &str =
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE
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
        block_id: "pp-0232",
        block_start: 5,
        block_end: 16,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0233",
        block_start: 18,
        block_end: 36,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0234",
        block_start: 20,
        block_end: 22,
        block_depth: 1,
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
        block_id: "pp-0232",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0233",
        branch_ordinal: 1,
        branch_line: 18,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0234",
        branch_ordinal: 1,
        branch_line: 20,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The six @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 7,
        source_name: "@a_triangleVertex",
        generated_name: "KB",
        generated_header_name: "GLSL_a_triangleVertex",
    },
    ExportedSymbol {
        source_line: 10,
        source_name: "@drawVertexMain",
        generated_name: "YB",
        generated_header_name: "GLSL_drawVertexMain",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 20,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 26,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "a_triangleVertex",
    "drawVertexMain",
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "drawFragmentMain",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Macro-defined entry functions are retained as source spellings and ranges.
/// Their bodies remain in the pinned GLSL source rather than becoming
/// executable Rust functions.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 10,
        end_line: 15,
        name: "drawVertexMain",
        signature: "VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 26,
        end_line: 35,
        name: "drawFragmentMain",
        signature: "CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "",
    },
];

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

/// This shader source has no direct #include/#import directive.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
