/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_image_mesh.vert.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, function entrypoints, direct include
 * inventory, and source metadata as literal source-shaped data. It does not
 * compile, evaluate, simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_image_mesh.vert";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-vert";
pub const PINNED_SOURCE_SHA256: &str =
    "f8c9d0c3a50cd3d42af1e67f8acb4258ac8c05833210d0b4556c95dff3312166";
pub const PINNED_SOURCE_LINE_COUNT: usize = 144;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 4552;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_image_mesh_vert.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned vertex-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_IMAGE_MESH_VERT_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(PositionAttr)
ATTR(0, float2, @a_position);
ATTR_BLOCK_END

ATTR_BLOCK_BEGIN(UVAttr)
ATTR(1, float2, @a_texCoord);
ATTR_BLOCK_END

ATTR_BLOCK_BEGIN(ImageDrawAttrs)
ATTR(IMAGE_VIEW_MATRIX_ATTRIB_IDX, float4, @a_imageDrawViewMatrix);
ATTR(IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX,
     float4,
     @a_imageDrawClipRectInverseMatrix);
ATTR(IMAGE_TRANSLATES_ATTRIB_IDX, float4, @a_imageDrawTranslates);
#ifdef SPLIT_UINT4_ATTRIBUTES
ATTR(IMAGE_SPLIT_OPACITY_ATTRIB_IDX, uint, @a_imageDrawOpacity);
ATTR(IMAGE_SPLIT_CLIP_ID_ATTRIB_IDX, uint, @a_imageDrawClipID);
ATTR(IMAGE_SPLIT_BLEND_MODE_ATTRIB_IDX, uint, @a_imageDrawBlendMode);
ATTR(IMAGE_SPLIT_ZINDEX_ATTRIB_IDX, uint, @a_imageDrawZIndex);
#else
ATTR(IMAGE_PACKED_ATTRIBS_IDX, uint4, @a_imageDrawPacked);
#endif
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
NO_PERSPECTIVE VARYING(0, float2, v_imageTexCoord);
#ifdef @ENABLE_CLIPPING
@OPTIONALLY_FLAT VARYING(1, half, v_clipID);
#endif
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
NO_PERSPECTIVE VARYING(2, float4, v_clipRect);
#endif
@OPTIONALLY_FLAT VARYING(3, half, v_imageOpacity);
#ifdef @ENABLE_ADVANCED_BLEND
FLAT VARYING(4, ushort, v_imageBlendMode);
#endif
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_TEXTURE_BLOCK_BEGIN
VERTEX_TEXTURE_BLOCK_END

IMAGE_MESH_VERTEX_MAIN(@drawVertexMain,
                       PositionAttr,
                       position,
                       UVAttr,
                       uv,
                       ImageDrawAttrs,
                       imageDrawAttrs,
                       _vertexID)
{
    ATTR_UNPACK(_vertexID, position, @a_position, float2);
    ATTR_UNPACK(_vertexID, uv, @a_texCoord, float2);
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawViewMatrix, float4);
    ATTR_UNPACK(_instanceID,
                imageDrawAttrs,
                @a_imageDrawClipRectInverseMatrix,
                float4);
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawTranslates, float4);
#ifdef SPLIT_UINT4_ATTRIBUTES
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawOpacity, uint);
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawClipID, uint);
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawBlendMode, uint);
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawZIndex, uint);
    uint4 @a_imageDrawPacked = uint4(@a_imageDrawOpacity,
                                     @a_imageDrawClipID,
                                     @a_imageDrawBlendMode,
                                     @a_imageDrawZIndex);
#else
    ATTR_UNPACK(_instanceID, imageDrawAttrs, @a_imageDrawPacked, uint4);
#endif

    VARYING_INIT(v_imageTexCoord, float2);
#ifdef @ENABLE_CLIPPING
    VARYING_INIT(v_clipID, half);
#endif
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
    VARYING_INIT(v_clipRect, float4);
#endif
    VARYING_INIT(v_imageOpacity, half);
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_INIT(v_imageBlendMode, ushort);
#endif

    float2 vertexPosition =
        MUL(make_float2x2(@a_imageDrawViewMatrix), @a_position) +
        @a_imageDrawTranslates.xy;
    v_imageTexCoord = @a_texCoord;
#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING)
    {
        v_clipID =
            id_bits_to_f16(@a_imageDrawPacked.y, uniforms.pathIDGranularity);
    }
#endif
#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
#ifndef @RENDER_MODE_MSAA
        v_clipRect = find_clip_rect_coverage_distances(
            make_float2x2(@a_imageDrawClipRectInverseMatrix),
            @a_imageDrawTranslates.zw,
            vertexPosition CLIP_CONTEXT_UNPACK);
#else
        set_clip_rect_plane_distances(
            make_float2x2(@a_imageDrawClipRectInverseMatrix),
            @a_imageDrawTranslates.zw,
            vertexPosition CLIP_CONTEXT_UNPACK);
#endif
    }
#endif // ENABLE_CLIP_RECT
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);
#ifdef @POST_INVERT_Y
    pos.y = -pos.y;
#endif
#ifdef @RENDER_MODE_MSAA
    pos.z = normalize_z_index(@a_imageDrawPacked.w);
#endif

    v_imageOpacity = uintBitsToFloat(@a_imageDrawPacked.x);
#ifdef @ENABLE_ADVANCED_BLEND
    v_imageBlendMode = cast_uint_to_ushort(@a_imageDrawPacked.z);
#endif

    VARYING_PACK(v_imageTexCoord);
#ifdef @ENABLE_CLIPPING
    VARYING_PACK(v_clipID);
#endif
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
    VARYING_PACK(v_clipRect);
#endif
    VARYING_PACK(v_imageOpacity);
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_PACK(v_imageBlendMode);
#endif
    EMIT_VERTEX(pos);
}
#endif
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_IMAGE_MESH_SOURCE: &str = PINNED_DRAW_IMAGE_MESH_VERT_SOURCE;
pub const DRAW_IMAGE_MESH_VERT_SOURCE: &str = PINNED_DRAW_IMAGE_MESH_VERT_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_IMAGE_MESH_VERT_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_stage: &'static str,
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
    source_stage: PINNED_SOURCE_STAGE,
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
        block_id: "pp-0328",
        block_start: 5,
        block_end: 29,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0329",
        block_start: 20,
        block_end: 27,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0330",
        block_start: 33,
        block_end: 35,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0331",
        block_start: 36,
        block_end: 38,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0332",
        block_start: 40,
        block_end: 42,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0333",
        block_start: 45,
        block_end: 144,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0334",
        block_start: 66,
        block_end: 77,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0335",
        block_start: 80,
        block_end: 82,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0336",
        block_start: 83,
        block_end: 85,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0337",
        block_start: 87,
        block_end: 89,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0338",
        block_start: 95,
        block_end: 101,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0339",
        block_start: 102,
        block_end: 117,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0340",
        block_start: 105,
        block_end: 115,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0341",
        block_start: 119,
        block_end: 121,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0342",
        block_start: 122,
        block_end: 124,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0343",
        block_start: 127,
        block_end: 129,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0344",
        block_start: 132,
        block_end: 134,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0345",
        block_start: 135,
        block_end: 137,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0346",
        block_start: 139,
        block_end: 141,
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
        block_id: "pp-0328",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0329",
        branch_ordinal: 1,
        branch_line: 20,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0329",
        branch_ordinal: 2,
        branch_line: 25,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0330",
        branch_ordinal: 1,
        branch_line: 33,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0331",
        branch_ordinal: 1,
        branch_line: 36,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0332",
        branch_ordinal: 1,
        branch_line: 40,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0333",
        branch_ordinal: 1,
        branch_line: 45,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0334",
        branch_ordinal: 1,
        branch_line: 66,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0334",
        branch_ordinal: 2,
        branch_line: 75,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0335",
        branch_ordinal: 1,
        branch_line: 80,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0336",
        branch_ordinal: 1,
        branch_line: 83,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0337",
        branch_ordinal: 1,
        branch_line: 87,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0338",
        branch_ordinal: 1,
        branch_line: 95,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0339",
        branch_ordinal: 1,
        branch_line: 102,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0340",
        branch_ordinal: 1,
        branch_line: 105,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0340",
        branch_ordinal: 2,
        branch_line: 110,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT)) && (!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0341",
        branch_ordinal: 1,
        branch_line: 119,
        directive: "#ifdef @POST_INVERT_Y",
        active_branch_path: "(defined(@VERTEX)) && (defined(@POST_INVERT_Y))",
    },
    ConditionalBranch {
        block_id: "pp-0342",
        branch_ordinal: 1,
        branch_line: 122,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0343",
        branch_ordinal: 1,
        branch_line: 127,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0344",
        branch_ordinal: 1,
        branch_line: 132,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0345",
        branch_ordinal: 1,
        branch_line: 135,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0346",
        branch_ordinal: 1,
        branch_line: 139,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The direct @-prefixed identifiers occurring in draw_image_mesh.vert, in
/// first-occurrence source order. Generated names are the pinned batch-minifier
/// outputs for the complete wildcard-expanded shader input set.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 7,
        source_name: "@a_position",
        generated_name: "OC",
        generated_header_name: "GLSL_a_position",
    },
    ExportedSymbol {
        source_line: 11,
        source_name: "@a_texCoord",
        generated_name: "PC",
        generated_header_name: "GLSL_a_texCoord",
    },
    ExportedSymbol {
        source_line: 15,
        source_name: "@a_imageDrawViewMatrix",
        generated_name: "WB",
        generated_header_name: "GLSL_a_imageDrawViewMatrix",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@a_imageDrawClipRectInverseMatrix",
        generated_name: "QB",
        generated_header_name: "GLSL_a_imageDrawClipRectInverseMatrix",
    },
    ExportedSymbol {
        source_line: 19,
        source_name: "@a_imageDrawTranslates",
        generated_name: "NB",
        generated_header_name: "GLSL_a_imageDrawTranslates",
    },
    ExportedSymbol {
        source_line: 21,
        source_name: "@a_imageDrawOpacity",
        generated_name: "XB",
        generated_header_name: "GLSL_a_imageDrawOpacity",
    },
    ExportedSymbol {
        source_line: 22,
        source_name: "@a_imageDrawClipID",
        generated_name: "YB",
        generated_header_name: "GLSL_a_imageDrawClipID",
    },
    ExportedSymbol {
        source_line: 23,
        source_name: "@a_imageDrawBlendMode",
        generated_name: "ZB",
        generated_header_name: "GLSL_a_imageDrawBlendMode",
    },
    ExportedSymbol {
        source_line: 24,
        source_name: "@a_imageDrawZIndex",
        generated_name: "AC",
        generated_header_name: "GLSL_a_imageDrawZIndex",
    },
    ExportedSymbol {
        source_line: 26,
        source_name: "@a_imageDrawPacked",
        generated_name: "IB",
        generated_header_name: "GLSL_a_imageDrawPacked",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "I",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "MB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "BB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "CB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 49,
        source_name: "@drawVertexMain",
        generated_name: "GC",
        generated_header_name: "GLSL_drawVertexMain",
    },
    ExportedSymbol {
        source_line: 119,
        source_name: "@POST_INVERT_Y",
        generated_name: "RC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
];

/// The preprocessor-switch subset of the direct exports.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "I",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "BB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "CB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 119,
        source_name: "@POST_INVERT_Y",
        generated_name: "RC",
        generated_header_name: "GLSL_POST_INVERT_Y",
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

/// The macro-defined vertex entrypoint is retained as a source spelling and
/// range. Its body remains in the pinned source rather than becoming an
/// executable Rust function.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[ShaderFunction {
    source_line: 49,
    end_line: 143,
    name: "drawVertexMain",
    signature: "IMAGE_MESH_VERTEX_MAIN(@drawVertexMain, PositionAttr, position, UVAttr, uv, ImageDrawAttrs, imageDrawAttrs, _vertexID)",
    guard_path: "(defined(@VERTEX))",
    inline_qualifier: "",
}];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings without the leading @ and
/// the generated names assigned by the pinned batch minifier.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "a_position",
        generated_name: "OC",
    },
    ExportedIdentifier {
        source_name: "a_texCoord",
        generated_name: "PC",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawViewMatrix",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawClipRectInverseMatrix",
        generated_name: "QB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawTranslates",
        generated_name: "NB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawOpacity",
        generated_name: "XB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawClipID",
        generated_name: "YB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawBlendMode",
        generated_name: "ZB",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawZIndex",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "a_imageDrawPacked",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIPPING",
        generated_name: "I",
    },
    ExportedIdentifier {
        source_name: "OPTIONALLY_FLAT",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIP_RECT",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
    },
    ExportedIdentifier {
        source_name: "drawVertexMain",
        generated_name: "GC",
    },
    ExportedIdentifier {
        source_name: "POST_INVERT_Y",
        generated_name: "RC",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "a_position",
    "a_texCoord",
    "a_imageDrawViewMatrix",
    "a_imageDrawClipRectInverseMatrix",
    "a_imageDrawTranslates",
    "a_imageDrawOpacity",
    "a_imageDrawClipID",
    "a_imageDrawBlendMode",
    "a_imageDrawZIndex",
    "a_imageDrawPacked",
    "ENABLE_CLIPPING",
    "OPTIONALLY_FLAT",
    "ENABLE_CLIP_RECT",
    "RENDER_MODE_MSAA",
    "ENABLE_ADVANCED_BLEND",
    "drawVertexMain",
    "POST_INVERT_Y",
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

/// draw_image_mesh.vert has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source include edge retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[ShaderInclude {
    upstream_file: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 15,
    directive: "include",
    include_token: "generated/shaders/draw_image_mesh.vert.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_image_mesh.vert",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    correspondence_owner: "-",
    mapping_status: "prepared",
    translation_status: "pending",
    translation_disposition: "required-source-edge",
}];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[IncludeDependency {
    including_source: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 15,
    include_token: "generated/shaders/draw_image_mesh.vert.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_image_mesh.vert",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    translation_disposition: "preserve-source-dependency",
}];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
