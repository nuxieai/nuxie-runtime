/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/blit_texture_as_draw.glsl.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/blit_texture_as_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "c9d6ab3c8911900a246d22484ad4dbda0a050ba76d74353c9a514d3ca7da3515";
pub const PINNED_SOURCE_LINE_COUNT: usize = 72;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1976;

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE: &str = r###"/*
 * Copyright 2024 Rive
 */

VARYING_BLOCK_BEGIN
#ifdef @USE_FILTERING
NO_PERSPECTIVE VARYING(0, float2, v_texCoord);
#endif
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_TEXTURE_BLOCK_BEGIN
VERTEX_TEXTURE_BLOCK_END

VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
VERTEX_STORAGE_BUFFER_BLOCK_END

ATTR_BLOCK_BEGIN(Attrs)
ATTR_BLOCK_END

VERTEX_MAIN(@blitVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    // Fill the entire screen. The caller will use a scissor test to control the
    // bounds being drawn.
    float2 coord;
    coord.x = (_vertexID & 1) == 0 ? -1. : 1.;
    coord.y = (_vertexID & 2) == 0 ? -1. : 1.;
#ifdef @USE_FILTERING
    VARYING_INIT(v_texCoord, float2);
    v_texCoord.x = coord.x * .5 + .5;
    v_texCoord.y = coord.y * -.5 + .5;
    VARYING_PACK(v_texCoord);
#endif
    float4 pos = float4(coord, 0, 1);
    EMIT_VERTEX(pos);
}
#endif // @VERTEX

#ifdef @FRAGMENT
FRAG_TEXTURE_BLOCK_BEGIN
#ifdef @SOURCE_TEXTURE_MSAA
TEXTURE_RGBA8_MS(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @sourceTexture);
#else
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @sourceTexture);
#endif
FRAG_TEXTURE_BLOCK_END

#ifdef @USE_FILTERING
DYNAMIC_SAMPLER_BLOCK_BEGIN
SAMPLER_DYNAMIC_IMAGE(blitSampler)
DYNAMIC_SAMPLER_BLOCK_END
#endif

FRAG_DATA_MAIN(half4, @blitFragmentMain)
{
    half4 srcColor;
#ifdef @USE_FILTERING
    VARYING_UNPACK(v_texCoord, float2);
    srcColor =
        TEXTURE_SAMPLE_DYNAMIC_LOD(@sourceTexture, blitSampler, v_texCoord, .0);
#elif defined(@SOURCE_TEXTURE_MSAA)
    srcColor = (TEXEL_FETCH_MS(@sourceTexture, 0, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 1, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 2, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 3, int2(floor(_fragCoord.xy)))) *
               0.25;
#else
    srcColor = TEXEL_FETCH(@sourceTexture, int2(floor(_fragCoord.xy)));
#endif
    EMIT_FRAG_DATA(srcColor);
}
#endif // @FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_BLIT_TEXTURE_AS_DRAW_SOURCE: &str = PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE;
pub const BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE: &str = PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE
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
    target_path: "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/blit_texture_as_draw_glsl.rs",
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
        block_id: "pp-0225",
        block_start: 6,
        block_end: 8,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0226",
        block_start: 11,
        block_end: 37,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0227",
        block_start: 28,
        block_end: 33,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0228",
        block_start: 39,
        block_end: 72,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0229",
        block_start: 41,
        block_end: 45,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0230",
        block_start: 48,
        block_end: 52,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0231",
        block_start: 57,
        block_end: 69,
        block_depth: 1,
        branch_count: 3,
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
        block_id: "pp-0225",
        branch_ordinal: 1,
        branch_line: 6,
        directive: "#ifdef @USE_FILTERING",
        active_branch_path: "(defined(@USE_FILTERING))",
    },
    ConditionalBranch {
        block_id: "pp-0226",
        branch_ordinal: 1,
        branch_line: 11,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0227",
        branch_ordinal: 1,
        branch_line: 28,
        directive: "#ifdef @USE_FILTERING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@USE_FILTERING))",
    },
    ConditionalBranch {
        block_id: "pp-0228",
        branch_ordinal: 1,
        branch_line: 39,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0229",
        branch_ordinal: 1,
        branch_line: 41,
        directive: "#ifdef @SOURCE_TEXTURE_MSAA",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@SOURCE_TEXTURE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0229",
        branch_ordinal: 2,
        branch_line: 43,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@SOURCE_TEXTURE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0230",
        branch_ordinal: 1,
        branch_line: 48,
        directive: "#ifdef @USE_FILTERING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@USE_FILTERING))",
    },
    ConditionalBranch {
        block_id: "pp-0231",
        branch_ordinal: 1,
        branch_line: 57,
        directive: "#ifdef @USE_FILTERING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@USE_FILTERING))",
    },
    ConditionalBranch {
        block_id: "pp-0231",
        branch_ordinal: 2,
        branch_line: 61,
        directive: "#elif defined(@SOURCE_TEXTURE_MSAA)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@USE_FILTERING))) && (defined(@SOURCE_TEXTURE_MSAA)))",
    },
    ConditionalBranch {
        block_id: "pp-0231",
        branch_ordinal: 3,
        branch_line: 67,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@USE_FILTERING)) || (defined(@SOURCE_TEXTURE_MSAA))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The seven @-prefixed identifiers occurring directly in this source. The
/// two macroized entrypoints and the texture identifier are exports alongside
/// the four preprocessor switches.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 6,
        source_name: "@USE_FILTERING",
        generated_name: "VC",
        generated_header_name: "GLSL_USE_FILTERING",
    },
    ExportedSymbol {
        source_line: 11,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 21,
        source_name: "@blitVertexMain",
        generated_name: "WE",
        generated_header_name: "GLSL_blitVertexMain",
    },
    ExportedSymbol {
        source_line: 39,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 41,
        source_name: "@SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
        generated_header_name: "GLSL_SOURCE_TEXTURE_MSAA",
    },
    ExportedSymbol {
        source_line: 42,
        source_name: "@sourceTexture",
        generated_name: "BC",
        generated_header_name: "GLSL_sourceTexture",
    },
    ExportedSymbol {
        source_line: 54,
        source_name: "@blitFragmentMain",
        generated_name: "DE",
        generated_header_name: "GLSL_blitFragmentMain",
    },
];

/// The four switch exports are retained separately from macroized entrypoints
/// and resource identifiers.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 6,
        source_name: "@USE_FILTERING",
        generated_name: "VC",
        generated_header_name: "GLSL_USE_FILTERING",
    },
    ExportedSymbol {
        source_line: 11,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 39,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 41,
        source_name: "@SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
        generated_header_name: "GLSL_SOURCE_TEXTURE_MSAA",
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

/// The two macroized shader entrypoints are retained as source spellings and
/// ranges. Their bodies remain in the pinned GLSL source rather than being
/// translated into executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 21,
        end_line: 36,
        name: "blitVertexMain",
        signature: "VERTEX_MAIN(@blitVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 54,
        end_line: 71,
        name: "blitFragmentMain",
        signature: "FRAG_DATA_MAIN(half4, @blitFragmentMain)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "",
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
        source_name: "USE_FILTERING",
        generated_name: "VC",
    },
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "blitVertexMain",
        generated_name: "WE",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
    },
    ExportedIdentifier {
        source_name: "sourceTexture",
        generated_name: "BC",
    },
    ExportedIdentifier {
        source_name: "blitFragmentMain",
        generated_name: "DE",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "USE_FILTERING",
    "VERTEX",
    "blitVertexMain",
    "FRAGMENT",
    "SOURCE_TEXTURE_MSAA",
    "sourceTexture",
    "blitFragmentMain",
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

/// blit_texture_as_draw.glsl has no direct #include/#import directives.
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
