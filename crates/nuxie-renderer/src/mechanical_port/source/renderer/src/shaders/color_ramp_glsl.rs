/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/color_ramp.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, exports, functions, and dependencies as
 * literal source-shaped data. It does not compile, evaluate, simplify, or
 * generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/color_ramp.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "65d0e8610193de1a7bf02722bcc153f04474e0d57644a35fd56291333ee8fde1";
pub const PINNED_SOURCE_LINE_COUNT: usize = 107;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3167;

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_COLOR_RAMP_GLSL_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

// This shader draws horizontal color ramps into a gradient texture, which will
// later be sampled by the renderer for drawing gradients.

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
#ifdef SPLIT_UINT4_ATTRIBUTES
ATTR(0, uint, @a_spanX);
ATTR(1, uint, @a_yWithFlags);
ATTR(2, uint, @a_color0);
ATTR(3, uint, @a_color1);
#else
ATTR(0, uint4, @a_span); // [spanX, y, color0, color1]
#endif
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
NO_PERSPECTIVE VARYING(0, half4, v_rampColor);
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_TEXTURE_BLOCK_BEGIN
VERTEX_TEXTURE_BLOCK_END

VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
VERTEX_STORAGE_BUFFER_BLOCK_END

half4 unpackColorInt(uint color)
{
    return cast_uint4_to_half4(
               (uint4(color, color, color, color) >> uint4(16, 8, 0, 24)) &
               0xffu) /
           255.;
}

VERTEX_MAIN(@colorRampVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
#ifdef SPLIT_UINT4_ATTRIBUTES
    ATTR_UNPACK(_instanceID, attrs, @a_spanX, uint);
    ATTR_UNPACK(_instanceID, attrs, @a_yWithFlags, uint);
    ATTR_UNPACK(_instanceID, attrs, @a_color0, uint);
    ATTR_UNPACK(_instanceID, attrs, @a_color1, uint);
    uint4 @a_span = uint4(@a_spanX, @a_yWithFlags, @a_color0, @a_color1);
#else
    ATTR_UNPACK(_instanceID, attrs, @a_span, uint4);
#endif
    VARYING_INIT(v_rampColor, half4);

    int columnWithinSpan = _vertexID >> 1;
    float x =
        float(columnWithinSpan <= 1 ? @a_span.x & 0xffffu : @a_span.x >> 16) /
        65536.;
    float offsetY = (_vertexID & 1) == 0 ? .0 : 1.;
    if (uniforms.gradInverseViewportY < .0)
    {
        // Swap the top and bottom vertices to make sure we always emit
        // clockwise triangles. vertices.
        offsetY = 1. - offsetY;
    }
    uint yWithFlags = @a_span.y;
    float y = float(yWithFlags & ~GRAD_SPAN_FLAGS_MASK) + offsetY;
    if ((yWithFlags & GRAD_SPAN_FLAG_LEFT_BORDER) != 0u &&
        columnWithinSpan == 0)
    {
        if ((yWithFlags & GRAD_SPAN_FLAG_COMPLEX_BORDER) != 0u)
            x = .0; // Borders of complex gradients go to the far edge.
        else
            // Simple gradients are empty with 1px borders on either side.
            x -= GRAD_TEXTURE_INVERSE_WIDTH;
    }
    if ((yWithFlags & GRAD_SPAN_FLAG_RIGHT_BORDER) != 0u &&
        columnWithinSpan == 3)
    {
        if ((yWithFlags & GRAD_SPAN_FLAG_COMPLEX_BORDER) != 0u)
            x = 1.; // Borders of complex gradients go to the far edge.
        else
            // Simple gradients are empty with 1px borders on either side.
            x += GRAD_TEXTURE_INVERSE_WIDTH;
    }
    v_rampColor = unpackColorInt(columnWithinSpan <= 1 ? @a_span.z : @a_span.w);

    float4 pos = pixel_coord_to_clip_coord(float2(x, y),
                                           2.,
                                           uniforms.gradInverseViewportY);
#ifdef @POST_INVERT_Y
    pos.y = -pos.y;
#endif

    VARYING_PACK(v_rampColor);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT
FRAG_TEXTURE_BLOCK_BEGIN
FRAG_TEXTURE_BLOCK_END

FRAG_DATA_MAIN(half4, @colorRampFragmentMain)
{
    VARYING_UNPACK(v_rampColor, half4);
    EMIT_FRAG_DATA(v_rampColor);
}
#endif
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_COLOR_RAMP_SOURCE: &str = PINNED_COLOR_RAMP_GLSL_SOURCE;
pub const COLOR_RAMP_GLSL_SOURCE: &str = PINNED_COLOR_RAMP_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_COLOR_RAMP_GLSL_SOURCE
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
    target_path:
        "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/color_ramp_glsl.rs",
    translation_disposition: "full-translation-source / source-shaped provenance",
};

/// Every semantic preprocessor block in the pinned source remains literal.
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
        block_id: "pp-0235",
        block_start: 8,
        block_end: 19,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0236",
        block_start: 10,
        block_end: 17,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0237",
        block_start: 25,
        block_end: 96,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0238",
        block_start: 42,
        block_end: 50,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0239",
        block_start: 89,
        block_end: 91,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0240",
        block_start: 98,
        block_end: 107,
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
        block_id: "pp-0235",
        branch_ordinal: 1,
        branch_line: 8,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0236",
        branch_ordinal: 1,
        branch_line: 10,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0236",
        branch_ordinal: 2,
        branch_line: 15,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0237",
        branch_ordinal: 1,
        branch_line: 25,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0238",
        branch_ordinal: 1,
        branch_line: 42,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0238",
        branch_ordinal: 2,
        branch_line: 48,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0239",
        branch_ordinal: 1,
        branch_line: 89,
        directive: "#ifdef @POST_INVERT_Y",
        active_branch_path: "(defined(@VERTEX)) && (defined(@POST_INVERT_Y))",
    },
    ConditionalBranch {
        block_id: "pp-0240",
        branch_ordinal: 1,
        branch_line: 98,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The @-prefixed identifiers occurring directly in color_ramp.glsl. Their
/// generated names are the pinned batch-minifier outputs.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 8,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 11,
        source_name: "@a_spanX",
        generated_name: "HD",
        generated_header_name: "GLSL_a_spanX",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@a_yWithFlags",
        generated_name: "ID",
        generated_header_name: "GLSL_a_yWithFlags",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@a_color0",
        generated_name: "JD",
        generated_header_name: "GLSL_a_color0",
    },
    ExportedSymbol {
        source_line: 14,
        source_name: "@a_color1",
        generated_name: "KD",
        generated_header_name: "GLSL_a_color1",
    },
    ExportedSymbol {
        source_line: 16,
        source_name: "@a_span",
        generated_name: "CC",
        generated_header_name: "GLSL_a_span",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@colorRampVertexMain",
        generated_name: "XE",
        generated_header_name: "GLSL_colorRampVertexMain",
    },
    ExportedSymbol {
        source_line: 89,
        source_name: "@POST_INVERT_Y",
        generated_name: "JC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 98,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 102,
        source_name: "@colorRampFragmentMain",
        generated_name: "YE",
        generated_header_name: "GLSL_colorRampFragmentMain",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Function declarations and source macro entry points are retained as source
/// spellings and ranges. Their bodies remain in the pinned source above rather
/// than being translated into executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 32,
        end_line: 38,
        name: "unpackColorInt",
        signature: "half4 unpackColorInt(uint color)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 40,
        end_line: 95,
        name: "colorRampVertexMain",
        signature: "VERTEX_MAIN(@colorRampVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 102,
        end_line: 106,
        name: "colorRampFragmentMain",
        signature: "FRAG_DATA_MAIN(half4, @colorRampFragmentMain)",
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

/// The color_ramp owner has no direct #include/#import directive. These
/// incoming generated-source edges are retained from the include/source
/// dependency authorities because they determine its artifact consumers.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/render_context_metal_impl.mm",
        include_line: 20,
        include_token: "generated/shaders/color_ramp.glsl.exports.h",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/color_ramp.glsl",
        source_unit: "metal-render-context-implementation",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/color_ramp.metal",
        include_line: 11,
        include_token: "color_ramp.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/color_ramp.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
