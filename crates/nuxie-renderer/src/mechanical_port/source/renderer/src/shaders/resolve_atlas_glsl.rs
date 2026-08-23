/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/resolve_atlas.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, function entrypoints, and direct
 * include inventory as literal source-shaped data. It does not compile,
 * evaluate, simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/resolve_atlas.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "a31d945c9b29dd4ba74cff3c9c9010e108f5cd82bb0b82474b199725e59aa04f";
pub const PINNED_SOURCE_LINE_COUNT: usize = 93;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2615;

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_RESOLVE_ATLAS_GLSL_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

// This shader provides a mechanism for resolving various atlas types into GL_R8
// so they can be sampled linearly.
//
// Additionally, EXT_shader_pixel_local_storage extension does not have a
// "clear" function, so this shader also provides a clear mechanism for PLS.

#ifdef @VERTEX
VERTEX_MAIN(@atlasResolveVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    // Draw a right triangle that covers the entire screen.
    float4 pos;
    pos.x = (_vertexID != 2) ? -1. : 3.;
    pos.y = (_vertexID != 1) ? -1. : 3.;
    pos.zw = float2(.0, 1.);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT

INLINE ivec2 atlas_frag_coord() { return ivec2(floor(gl_FragCoord)); }

#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH

layout(location = 0) inout uint4 coverageCount;
layout(location = 1) out half4 resolvedCoverage;

void main() { resolvedCoverage.r = uintBitsToFloat(coverageCount.r); }

#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)

#ifdef @CLEAR_COVERAGE
__pixel_local_outEXT PLS { layout(r32f) float coverageCount; };
#else
__pixel_local_inEXT PLS { layout(r32f) float coverageCount; };
layout(location = 0) out half4 resolvedCoverage;
#endif

void main()
{
#ifdef @CLEAR_COVERAGE
    coverageCount = .0;
#else
    resolvedCoverage.r = coverageCount;
#endif
}

#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)

layout(binding = 0, r32ui) uniform highp upixelLocalANGLE coverageCount;
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    resolvedCoverage.r = uintBitsToFloat(pixelLocalLoadANGLE(coverageCount).r);
}

#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)

layout(binding = 0, r32i) uniform highp coherent iimage2D _atlasImage;
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    resolvedCoverage.r = float(imageLoad(_atlasImage, atlas_frag_coord()).r) *
                         (1. / ATLAS_R32I_FIXED_POINT_FACTOR);
}

#elif defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)

TEXTURE_RGBA8(PER_FLUSH_BINDINGS_SET, 0, @atlasRenderTexture);
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    // Apply the following weights to the RGBA of each u8x4 coverage value:
    //   - R counts fractional, positive coverage.
    //   - G counts fractional, negative coverage.
    //   - B counts integer, positive coverage.
    //   - A counts integer, negative coverage.
    half4 coverages = TEXEL_FETCH(@atlasRenderTexture, atlas_frag_coord());
    resolvedCoverage.r =
        (coverages.r - coverages.g) * ATLAS_UNORM8_COVERAGE_SCALE_FACTOR +
        (coverages.b - coverages.a) * 255.;
}

#endif

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_RESOLVE_ATLAS_SOURCE: &str = PINNED_RESOLVE_ATLAS_GLSL_SOURCE;
pub const RESOLVE_ATLAS_GLSL_SOURCE: &str = PINNED_RESOLVE_ATLAS_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_RESOLVE_ATLAS_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_disposition: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/resolve_atlas_glsl.rs",
    translation_disposition: "full-translation-source / source-shaped provenance",
};

/// Every semantic preprocessor block in the pinned source remains literal,
/// including nested and mutually exclusive branch alternatives.
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
        block_id: "pp-0594",
        block_start: 11,
        block_end: 21,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0595",
        block_start: 23,
        block_end: 93,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0596",
        block_start: 27,
        block_end: 91,
        block_depth: 1,
        branch_count: 5,
    },
    ConditionalBlock {
        block_id: "pp-0597",
        block_start: 36,
        block_end: 41,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0598",
        block_start: 45,
        block_end: 49,
        block_depth: 2,
        branch_count: 2,
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
        block_id: "pp-0594",
        branch_ordinal: 1,
        branch_line: 11,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0595",
        branch_ordinal: 1,
        branch_line: 23,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0596",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))",
    },
    ConditionalBranch {
        block_id: "pp-0596",
        branch_ordinal: 2,
        branch_line: 34,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)))",
    },
    ConditionalBranch {
        block_id: "pp-0596",
        branch_ordinal: 3,
        branch_line: 52,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)))",
    },
    ConditionalBranch {
        block_id: "pp-0596",
        branch_ordinal: 4,
        branch_line: 62,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)))",
    },
    ConditionalBranch {
        block_id: "pp-0596",
        branch_ordinal: 5,
        branch_line: 73,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))",
    },
    ConditionalBranch {
        block_id: "pp-0597",
        branch_ordinal: 1,
        branch_line: 36,
        directive: "#ifdef @CLEAR_COVERAGE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@CLEAR_COVERAGE))",
    },
    ConditionalBranch {
        block_id: "pp-0597",
        branch_ordinal: 2,
        branch_line: 38,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (!((defined(@CLEAR_COVERAGE))))",
    },
    ConditionalBranch {
        block_id: "pp-0598",
        branch_ordinal: 1,
        branch_line: 45,
        directive: "#ifdef @CLEAR_COVERAGE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@CLEAR_COVERAGE))",
    },
    ConditionalBranch {
        block_id: "pp-0598",
        branch_ordinal: 2,
        branch_line: 47,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (!((defined(@CLEAR_COVERAGE))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The ten @-prefixed identifiers occurring directly in this source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 11,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@atlasResolveVertexMain",
        generated_name: "SF",
        generated_header_name: "GLSL_atlasResolveVertexMain",
    },
    ExportedSymbol {
        source_line: 23,
        source_name: "@FRAGMENT",
        generated_name: "GB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 27,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "TD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "UD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@CLEAR_COVERAGE",
        generated_name: "AE",
        generated_header_name: "GLSL_CLEAR_COVERAGE",
    },
    ExportedSymbol {
        source_line: 52,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "VD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    },
    ExportedSymbol {
        source_line: 73,
        source_name: "@ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "TE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM",
    },
    ExportedSymbol {
        source_line: 75,
        source_name: "@atlasRenderTexture",
        generated_name: "WE",
        generated_header_name: "GLSL_atlasRenderTexture",
    },
];

/// The eight switch exports are retained separately from the macroized
/// entrypoint and resource identifier.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 11,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 23,
        source_name: "@FRAGMENT",
        generated_name: "GB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 27,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "TD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "UD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@CLEAR_COVERAGE",
        generated_name: "AE",
        generated_header_name: "GLSL_CLEAR_COVERAGE",
    },
    ExportedSymbol {
        source_line: 52,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "VD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    },
    ExportedSymbol {
        source_line: 73,
        source_name: "@ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "TE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM",
    },
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

/// Function declarations are retained as source spellings and ranges. Their
/// bodies remain in PINNED_RESOLVE_ATLAS_GLSL_SOURCE rather than being
/// translated into executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 12,
        end_line: 20,
        name: "atlasResolveVertexMain",
        signature: "VERTEX_MAIN(@atlasResolveVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 25,
        end_line: 25,
        name: "atlas_frag_coord",
        signature: "INLINE ivec2 atlas_frag_coord() { return ivec2(floor(gl_FragCoord)); }",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings (without the leading @) and
/// the generated names assigned by the pinned batch minifier.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "atlasResolveVertexMain",
        generated_name: "SF",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "TD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "UD",
    },
    ExportedIdentifier {
        source_name: "CLEAR_COVERAGE",
        generated_name: "AE",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "VD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "TE",
    },
    ExportedIdentifier {
        source_name: "atlasRenderTexture",
        generated_name: "WE",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "atlasResolveVertexMain",
    "FRAGMENT",
    "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    "ATLAS_RENDER_TARGET_R8_PLS_EXT",
    "CLEAR_COVERAGE",
    "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    "ATLAS_RENDER_TARGET_RGBA8_UNORM",
    "atlasRenderTexture",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderInclude {
    pub upstream_file: &'static str,
    pub include_line: u16,
    pub directive: &'static str,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub correspondence_owner: &'static str,
    pub mapping_status: &'static str,
    pub translation_status: &'static str,
    pub translation_disposition: &'static str,
}

/// resolve_atlas.glsl has no direct #include/#import directives.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// No incoming generated-source edge is recorded for this owner in the pinned
/// include/dependency authorities; direct include inventory remains empty.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
