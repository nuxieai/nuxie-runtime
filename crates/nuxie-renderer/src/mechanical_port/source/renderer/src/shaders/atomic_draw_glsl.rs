/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/atomic_draw.glsl.
 *
 * This Phase-1 owner intentionally retains the exact GLSL source and the
 * authority inventories as inert Rust data. It does not parse, minify,
 * compile, generate artifacts, or alter shader behavior.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "fad587733e5990e4ba77e194326dacaf27022026f9621e58a1aac2c131935849";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1104;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/atomic_draw_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub source_path: &'static str,
    pub source_stage: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: u16,
    pub translation_unit: &'static str,
    pub translation_target: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    source_path: PINNED_SOURCE_PATH,
    source_stage: "minify-input-glsl",
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT as u16,
    translation_unit: TRANSLATION_UNIT,
    translation_target: TRANSLATION_TARGET,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Exact pinned source, retained for provenance and line-for-line audit.
pub const PINNED_ATOMIC_DRAW_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

#ifdef @DRAW_PATH
#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0,
     float4,
     @a_patchVertexData); // [localVertexID, outset, fillCoverage, vertexType]
ATTR(1, float4, @a_mirroredVertexData);
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
#ifdef @ENABLE_FEATHER
NO_PERSPECTIVE VARYING(0, float4, v_coverages);
#else
NO_PERSPECTIVE VARYING(0, half2, v_coverages);
#endif
FLAT VARYING(1, ushort, v_pathID);
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_patchVertexData, float4);
    ATTR_UNPACK(_vertexID, attrs, @a_mirroredVertexData, float4);

#ifdef @ENABLE_FEATHER
    VARYING_INIT(v_coverages, float4);
#else
    VARYING_INIT(v_coverages, half2);
#endif
    VARYING_INIT(v_pathID, ushort);

    float4 pos;
    uint pathID;
    float2 vertexPosition;
    float4 coverages;
    if (unpack_tessellated_path_vertex(@a_patchVertexData,
                                       @a_mirroredVertexData,
                                       _instanceID,
                                       pathID,
                                       vertexPosition,
                                       coverages VERTEX_CONTEXT_UNPACK))
    {
#ifdef @ENABLE_FEATHER
        v_coverages = coverages;
#else
        v_coverages.xy = cast_float2_to_half2(coverages.xy);
#endif
        v_pathID = cast_uint_to_ushort(pathID);
        pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);
    }
    else
    {
        pos = float4(uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue);
    }

    VARYING_PACK(v_coverages);
    VARYING_PACK(v_pathID);
    EMIT_VERTEX(pos);
}
#endif // VERTEX
#endif // DRAW_PATH

#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)
#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0, packed_float3, @a_triangleVertex);
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
#ifdef @FEATHER_ATLAS_BLIT
NO_PERSPECTIVE VARYING(0, float2, v_atlasCoord);
#else
@OPTIONALLY_FLAT VARYING(0, half, v_windingWeight);
#endif
FLAT VARYING(1, ushort, v_pathID);
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_triangleVertex, float3);

#ifdef @FEATHER_ATLAS_BLIT
    VARYING_INIT(v_atlasCoord, float2);
#else
    VARYING_INIT(v_windingWeight, half);
#endif
    VARYING_INIT(v_pathID, ushort);

    uint pathID;
    float2 vertexPosition;
#ifdef @FEATHER_ATLAS_BLIT
    vertexPosition =
        unpack_atlas_coverage_vertex(@a_triangleVertex,
                                     pathID,
                                     v_atlasCoord VERTEX_CONTEXT_UNPACK);
#else
    vertexPosition =
        unpack_interior_triangle_vertex(@a_triangleVertex,
                                        pathID,
                                        v_windingWeight VERTEX_CONTEXT_UNPACK);
#endif
    v_pathID = cast_uint_to_ushort(pathID);
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);

#ifdef @FEATHER_ATLAS_BLIT
    VARYING_PACK(v_atlasCoord);
#else
    VARYING_PACK(v_windingWeight);
#endif
    VARYING_PACK(v_pathID);
    EMIT_VERTEX(pos);
}
#endif // @VERTEX
#endif // @DRAW_INTERIOR_TRIANGLES || @FEATHER_ATLAS_BLIT

#ifdef @DRAW_IMAGE_RECT
#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0, float4, @a_imageRectVertex);
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
NO_PERSPECTIVE VARYING(0, float2, v_texCoord);
NO_PERSPECTIVE VARYING(1, half, v_edgeCoverage);
#ifdef @ENABLE_CLIP_RECT
NO_PERSPECTIVE VARYING(2, float4, v_clipRect);
#endif
@OPTIONALLY_FLAT VARYING(3, half, v_imageOpacity);
#ifdef @ENABLE_CLIPPING
FLAT VARYING(4, ushort, v_imageClipID);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
FLAT VARYING(5, ushort, v_imageBlendMode);
#endif
VARYING_BLOCK_END

#ifdef @VERTEX
IMAGE_RECT_VERTEX_MAIN(@drawVertexMain,
                       Attrs,
                       attrs,
                       ImageDrawAttrs,
                       imageDrawAttrs,
                       _vertexID,
                       _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_imageRectVertex, float4);
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

    VARYING_INIT(v_texCoord, float2);
    VARYING_INIT(v_edgeCoverage, half);
#ifdef @ENABLE_CLIP_RECT
    VARYING_INIT(v_clipRect, float4);
#endif
    VARYING_INIT(v_imageOpacity, half);
#ifdef @ENABLE_CLIPPING
    VARYING_INIT(v_imageClipID, ushort);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_INIT(v_imageBlendMode, ushort);
#endif

    bool isOuterVertex =
        @a_imageRectVertex.z == .0 || @a_imageRectVertex.w == .0;
    v_edgeCoverage = isOuterVertex ? .0 : 1.;

    float2 vertexPosition = @a_imageRectVertex.xy;
    float2x2 M = make_float2x2(@a_imageDrawViewMatrix);
    float2x2 MIT = transpose(inverse(M));
    if (!isOuterVertex)
    {
        // Inset the inner vertices to the point where coverage == 1.
        // NOTE: if width/height ever change from 1, these equations need to be
        // updated.
        float aaRadiusX =
            AA_RADIUS * manhattan_width(MIT[1]) / dot(M[1], MIT[1]);
        if (aaRadiusX >= .5)
        {
            vertexPosition.x = .5;
            v_edgeCoverage *= cast_float_to_half(.5 / aaRadiusX);
        }
        else
        {
            vertexPosition.x += aaRadiusX * @a_imageRectVertex.z;
        }
        float aaRadiusY =
            AA_RADIUS * manhattan_width(MIT[0]) / dot(M[0], MIT[0]);
        if (aaRadiusY >= .5)
        {
            vertexPosition.y = .5;
            v_edgeCoverage *= cast_float_to_half(.5 / aaRadiusY);
        }
        else
        {
            vertexPosition.y += aaRadiusY * @a_imageRectVertex.w;
        }
    }

    v_texCoord = vertexPosition;
    vertexPosition = MUL(M, vertexPosition) + @a_imageDrawTranslates.xy;

    if (isOuterVertex)
    {
        // Outset the outer vertices to the point where coverage == 0.
        float2 n = MUL(MIT, @a_imageRectVertex.zw);
        n *= manhattan_width(n) / dot(n, n);
        vertexPosition += AA_RADIUS * n;
    }

#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
        v_clipRect = find_clip_rect_coverage_distances(
            make_float2x2(@a_imageDrawClipRectInverseMatrix),
            @a_imageDrawTranslates.zw,
            vertexPosition);
    }
#endif

    v_imageOpacity = uintBitsToFloat(@a_imageDrawPacked.x);
#ifdef @ENABLE_CLIPPING
    v_imageClipID = cast_uint_to_ushort(@a_imageDrawPacked.y);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    v_imageBlendMode = cast_uint_to_ushort(@a_imageDrawPacked.z);
#endif

    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);

    VARYING_PACK(v_texCoord);
    VARYING_PACK(v_edgeCoverage);
#ifdef @ENABLE_CLIP_RECT
    VARYING_PACK(v_clipRect);
#endif
    VARYING_PACK(v_imageOpacity);
#ifdef @ENABLE_CLIPPING
    VARYING_PACK(v_imageClipID);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_PACK(v_imageBlendMode);
#endif
    EMIT_VERTEX(pos);
}
#endif // VERTEX

#elif defined(@DRAW_IMAGE_MESH)
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
NO_PERSPECTIVE VARYING(0, float2, v_texCoord);
#ifdef @ENABLE_CLIP_RECT
NO_PERSPECTIVE VARYING(1, float4, v_clipRect);
#endif
@OPTIONALLY_FLAT VARYING(3, half, v_imageOpacity);
#ifdef @ENABLE_CLIPPING
FLAT VARYING(4, ushort, v_imageClipID);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
FLAT VARYING(5, ushort, v_imageBlendMode);
#endif
VARYING_BLOCK_END

#ifdef @VERTEX
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

    VARYING_INIT(v_texCoord, float2);
#ifdef @ENABLE_CLIP_RECT
    VARYING_INIT(v_clipRect, float4);
#endif
    VARYING_INIT(v_imageOpacity, half);
#ifdef @ENABLE_CLIPPING
    VARYING_INIT(v_imageClipID, ushort);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_INIT(v_imageBlendMode, ushort);
#endif

    float2x2 M = make_float2x2(@a_imageDrawViewMatrix);
    float2 vertexPosition = MUL(M, @a_position) + @a_imageDrawTranslates.xy;
    v_texCoord = @a_texCoord;

#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
        v_clipRect = find_clip_rect_coverage_distances(
            make_float2x2(@a_imageDrawClipRectInverseMatrix),
            @a_imageDrawTranslates.zw,
            vertexPosition);
    }
#endif

    v_imageOpacity = uintBitsToFloat(@a_imageDrawPacked.x);
#ifdef @ENABLE_CLIPPING
    v_imageClipID = cast_uint_to_ushort(@a_imageDrawPacked.y);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    v_imageBlendMode = cast_uint_to_ushort(@a_imageDrawPacked.z);
#endif

    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);

    VARYING_PACK(v_texCoord);
#ifdef @ENABLE_CLIP_RECT
    VARYING_PACK(v_clipRect);
#endif
    VARYING_PACK(v_imageOpacity);
#ifdef @ENABLE_CLIPPING
    VARYING_PACK(v_imageClipID);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_PACK(v_imageBlendMode);
#endif
    EMIT_VERTEX(pos);
}
#endif // VERTEX
#endif // DRAW_IMAGE_MESH

#ifdef @DRAW_RENDER_TARGET_UPDATE_BOUNDS
#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR_BLOCK_END
#endif // VERTEX

VARYING_BLOCK_BEGIN
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    int2 coord;
    coord.x = (_vertexID & 1) == 0 ? uniforms.renderTargetUpdateBounds.x
                                   : uniforms.renderTargetUpdateBounds.z;
    coord.y = (_vertexID & 2) == 0 ? uniforms.renderTargetUpdateBounds.y
                                   : uniforms.renderTargetUpdateBounds.w;
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(float2(coord));
    EMIT_VERTEX(pos);
}
#endif // VERTEX
#endif // DRAW_RENDER_TARGET_UPDATE_BOUNDS

#ifdef @DRAW_IMAGE
#define NEEDS_IMAGE_TEXTURE
#endif

// INITIALIZE_PLS may sample @imageTexture (the previous framebuffer contents
// copied to dstColorTexture) when LoadAction::preserveRenderTarget is
// requested. The spec-const LOAD_COLOR_FROM_DST_TEXTURE is set at runtime. We
// have to declare the binding unconditionally though for non-FFCO
// INITIALIZE_PLS so the spec-const branch can reach it; the caller binds a null
// texture when the branch is off.
#if defined(@INITIALIZE_PLS) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
#define NEEDS_IMAGE_TEXTURE
#endif

#ifdef @FRAGMENT
PLS_BLOCK_BEGIN
// We only bind the framebuffer as PLS when there are blend modes. Otherwise, we
// render to it as a normal color attachment.
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
#ifdef @COLOR_PLANE_IDX_OVERRIDE
// D3D11 doesn't let us bind the framebuffer UAV to slot 0 when there is a
// color output.
#define LOCAL_COLOR_PLANE_IDX @COLOR_PLANE_IDX_OVERRIDE
#else
#define LOCAL_COLOR_PLANE_IDX COLOR_PLANE_IDX
#endif
#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER
PLS_DECL4F_READONLY(LOCAL_COLOR_PLANE_IDX, colorBuffer);
#else
PLS_DECL4F(LOCAL_COLOR_PLANE_IDX, colorBuffer);
#endif
#endif // !FIXED_FUNCTION_COLOR_OUTPUT
#ifdef @PLS_BLEND_SRC_OVER
// When PLS has src-over blending enabled, the clip buffer is RGBA8 so we
// can preserve clip contents by emitting a=0 instead of loading the current
// value. This is also is a hint to the hardware that it doesn't need to
// write anything to the clip attachment.
#define CLIP_VALUE_TYPE half4
#define PLS_LOAD_CLIP_TYPE PLS_LOAD4F
#define MAKE_NON_UPDATING_CLIP_VALUE make_half4(.0)
#define HAS_UPDATED_CLIP_VALUE(X) ((X).a != .0)
#ifdef @ENABLE_CLIPPING
#ifndef @RESOLVE_PLS
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
#else
PLS_DECL4F_READONLY(CLIP_PLANE_IDX, clipBuffer);
#endif
#endif // ENABLE_CLIPPING
#else
// When PLS does not have src-over blending, the clip buffer the usual
// packed R32UI.
#define CLIP_VALUE_TYPE uint
#define MAKE_NON_UPDATING_CLIP_VALUE 0u
#define PLS_LOAD_CLIP_TYPE PLS_LOADUI
#define HAS_UPDATED_CLIP_VALUE(X) ((X) != 0u)
#ifdef @ENABLE_CLIPPING
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
#endif // ENABLE_CLIPPING
#endif // !PLS_BLEND_SRC_OVER
PLS_DECLUI_UAV(COVERAGE_PLANE_IDX, coverageAtomicBuffer);
PLS_BLOCK_END

FRAG_STORAGE_BUFFER_BLOCK_BEGIN
STORAGE_BUFFER_U32x2(PAINT_BUFFER_IDX, PaintBuffer, @paintBuffer);
STORAGE_BUFFER_F32x4(PAINT_AUX_BUFFER_IDX, PaintAuxBuffer, @paintAuxBuffer);
FRAG_STORAGE_BUFFER_BLOCK_END

INLINE uint to_fixed(float x)
{
    return uint(round(x * FIXED_COVERAGE_PRECISION + FIXED_COVERAGE_ZERO));
}

INLINE half from_fixed(uint x)
{
    return cast_float_to_half(
        float(x) * FIXED_COVERAGE_INVERSE_PRECISION +
        (-FIXED_COVERAGE_ZERO * FIXED_COVERAGE_INVERSE_PRECISION));
}

ushort apply_driver_workaround_for_path_id(ushort pathID)
{
#ifdef @NEEDS_PATH_ID_CLAMP_WORKAROUND
    // We have observed that on some hardware, inactive threads or helper lanes
    // appear to issue calls that access storage textures, even though they
    // should be NO-OP. clamping the path ID prevents crashes in these
    // scenarios.
    pathID = min(pathID, uniforms.maxPathId);
#endif
    return pathID;
}

#ifdef @ENABLE_CLIPPING
INLINE void apply_clip(uint clipID,
                       CLIP_VALUE_TYPE clipData,
                       INOUT(half) coverage)
{
#ifdef @PLS_BLEND_SRC_OVER
    // The clipID is packed into r & g of clipData. Use a fuzzy equality test
    // since we lose precision when the clip value gets stored to and from the
    // attachment.
    if (all(lessThan(abs(clipData.rg - unpackUnorm4x8(clipID).rg),
                     make_half2(.25 / 255.))))
        coverage = min(coverage, clipData.b);
    else
        coverage = .0;
#else
    // The clipID is the top 16 bits of clipData.
    if (clipID == clipData >> 16)
        coverage = min(coverage, unpackHalf2x16(clipData).r);
    else
        coverage = .0;
#endif
}
#endif

INLINE void resolve_paint(uint pathID,
                          half coverageCount,
                          OUT(half4) fragColorOut
#if defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)
                          ,
                          INOUT(CLIP_VALUE_TYPE) fragClipOut
#endif
                              FRAGMENT_CONTEXT_DECL PLS_CONTEXT_DECL)
{
    uint2 paintData = STORAGE_BUFFER_LOAD2(@paintBuffer, pathID);
    half coverage = coverageCount;
    if ((paintData.x & (PAINT_FLAG_NON_ZERO_FILL | PAINT_FLAG_EVEN_ODD_FILL)) !=
        0u)
    {
        // This path has a legacy (non-clockwise) fill.
        coverage = abs(coverage);
#ifdef @ENABLE_EVEN_ODD
        if (@ENABLE_EVEN_ODD && (paintData.x & PAINT_FLAG_EVEN_ODD_FILL) != 0u)
        {
            coverage = 1. - abs(fract(coverage * .5) * 2. + -1.);
        }
#endif
    }
    // This also caps stroke coverage, which can be >1.
    coverage = clamp(coverage, make_half(.0), make_half(1.));
#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING)
    {
        uint clipID = paintData.x >> 16u;
        if (clipID != 0u)
        {
            apply_clip(clipID, PLS_LOAD_CLIP_TYPE(clipBuffer), coverage);
        }
    }
#endif // ENABLE_CLIPPING
#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT && (paintData.x & PAINT_FLAG_HAS_CLIP_RECT) != 0u)
    {
        float2x2 M = make_float2x2(
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 2u));
        float4 translate =
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 3u);
        float2 clipCoord = MUL(M, _fragCoord) + translate.xy;
        // translate.zw contains -1 / fwidth(clipCoord), which we use to
        // calculate antialiasing.
        half2 distXY =
            cast_float2_to_half2(abs(clipCoord) * translate.zw - translate.zw);
        half clipRectCoverage = clamp(min(distXY.x, distXY.y) + .5, .0, 1.);
        coverage = min(coverage, clipRectCoverage);
    }
#endif // ENABLE_CLIP_RECT
    uint paintType = paintData.x & 0xfu;
    if (paintType <= SOLID_COLOR_PAINT_TYPE) // CLIP_UPDATE_PAINT_TYPE or
                                             // SOLID_COLOR_PAINT_TYPE
    {
        fragColorOut = unpackUnorm4x8(paintData.y);
#ifdef @ENABLE_CLIPPING
        if (@ENABLE_CLIPPING && paintType == CLIP_UPDATE_PAINT_TYPE)
        {
#ifndef @RESOLVE_PLS
#ifdef @PLS_BLEND_SRC_OVER
            // This was actually a clip update. fragColorOut contains the packed
            // clipID.
            fragClipOut.rg = fragColorOut.ba; // Pack the clipID into r & g.
            fragClipOut.b = coverage;         // Put the clipCoverage in b.
            fragClipOut.a =
                1.; // a=1 so we replace the value in the clipBuffer.
#else
            fragClipOut = paintData.y | packHalf2x16(make_half2(coverage, .0));
#endif
#endif
            // Don't update the colorBuffer when this is a clip update.
            fragColorOut = make_half4(.0);
        }
#endif
    }
    else // LINEAR_GRADIENT_PAINT_TYPE or RADIAL_GRADIENT_PAINT_TYPE
    {
        float2x2 M =
            make_float2x2(STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u));
        float4 translate =
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 1u);
        float2 paintCoord = MUL(M, _fragCoord) + translate.xy;
        float t = paintType == LINEAR_GRADIENT_PAINT_TYPE
                      ? /*linear*/ paintCoord.x
                      : /*radial*/ length(paintCoord);
        t = clamp(t, .0, 1.);
        float x = t * translate.z + translate.w;
        float y = uintBitsToFloat(paintData.y);
        fragColorOut =
            TEXTURE_SAMPLE_LOD(@gradTexture, gradSampler, float2(x, y), .0);
    }
    fragColorOut.a *= coverage;

#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND)
    // Apply the advanced blend mode, if applicable.
    ushort blendMode;
    if (@ENABLE_ADVANCED_BLEND && fragColorOut.a != .0 &&
        (blendMode = cast_uint_to_ushort((paintData.x >> 4) & 0xfu)) !=
            BLEND_SRC_OVER)
    {
        half4 dstColorPremul = PLS_LOAD4F(colorBuffer);
        fragColorOut.rgb =
            advanced_color_blend(fragColorOut.rgb, dstColorPremul, blendMode);
    }
#endif // !FIXED_FUNCTION_COLOR_OUTPUT && ENABLE_ADVANCED_BLEND

// Certain platforms give us less control of the format of what we are
// rendering too. Specifically, we are auto converted from linear -> sRGB on
// render target writes in unreal. In those cases we made need to end up in
// linear color space
#if defined(@NEEDS_GAMMA_CORRECTION) &&                                        \
    (defined(@FIXED_FUNCTION_COLOR_OUTPUT) || defined(@RESOLVE_PLS))
    fragColorOut = gamma_to_linear(fragColorOut);
#endif

    fragColorOut.rgb *= fragColorOut.a;
}

#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) &&                                  \
    !defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER)
INLINE void blend_pls_color_src_over(half4 fragColorOut PLS_CONTEXT_DECL)
{
#ifndef @PLS_BLEND_SRC_OVER
    if (fragColorOut.a == .0)
        return;
    float oneMinusSrcAlpha = 1. - fragColorOut.a;
    if (oneMinusSrcAlpha != .0)
        fragColorOut += PLS_LOAD4F(colorBuffer) * oneMinusSrcAlpha;
#endif
    PLS_STORE4F(colorBuffer, fragColorOut);
}
#endif // !@FIXED_FUNCTION_COLOR_OUTPUT &&
       // !@COALESCED_PLS_RESOLVE_AND_TRANSFER

#if defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)
INLINE void emit_pls_clip(CLIP_VALUE_TYPE fragClipOut PLS_CONTEXT_DECL)
{
#ifdef @PLS_BLEND_SRC_OVER
    PLS_STORE4F(clipBuffer, fragClipOut);
#else
    if (fragClipOut != 0u)
        PLS_STOREUI(clipBuffer, fragClipOut);
#endif
}
#endif // ENABLE_CLIPPING && !RESOLVE_PLS

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
#define ATOMIC_PLS_MAIN PLS_FRAG_COLOR_MAIN
#define EMIT_ATOMIC_PLS EMIT_PLS_AND_FRAG_COLOR
#else // !FIXED_FUNCTION_COLOR_OUTPUT
#define ATOMIC_PLS_MAIN PLS_MAIN
#define EMIT_ATOMIC_PLS EMIT_PLS
#endif

#ifdef @DRAW_PATH
ATOMIC_PLS_MAIN(@drawFragmentMain)
{
#ifdef @ENABLE_FEATHER
    VARYING_UNPACK(v_coverages, float4);
#else
    VARYING_UNPACK(v_coverages, half2);
#endif
    VARYING_UNPACK(v_pathID, ushort);

    half fragmentCoverage;

#ifdef @ENABLE_FEATHER
    if (@ENABLE_FEATHER && is_feathered_stroke(v_coverages))
    {
        fragmentCoverage =
            eval_feathered_stroke(v_coverages TEXTURE_CONTEXT_FORWARD);
    }
    else if (@ENABLE_FEATHER && is_feathered_fill(v_coverages))
    {
        fragmentCoverage =
            eval_feathered_fill(v_coverages TEXTURE_CONTEXT_FORWARD);
    }
    else
#endif
    {
        // Cover stroke and fill both in a branchless expression.
        fragmentCoverage =
            min(min(make_half(v_coverages.x), abs(make_half(v_coverages.y))),
                make_half(1.));
    }

    half4 fragColorOut = make_half4(.0);
#ifdef @ENABLE_CLIPPING
    CLIP_VALUE_TYPE fragClipOut = MAKE_NON_UPDATING_CLIP_VALUE;
#endif

    // Since v_pathID increases monotonically with every draw, and since it
    // lives in the most significant bits of the coverage data, an atomic max()
    // function will serve 3 purposes:
    //
    //    * The invocation that changes the pathID is the single first fragment
    //      invocation to hit the new path, and the one that should resolve the
    //      previous path in the framebuffer.
    //    * Properly resets coverage to zero when we do cross over into
    //      processing a new path.
    //    * Accumulates coverage for strokes.
    //
    uint fixedCoverage = to_fixed(fragmentCoverage);
    uint minCoverageData =
        (make_uint(v_pathID) << FIXED_COVERAGE_BIT_COUNT) | fixedCoverage;
    uint lastCoverageData =
        PLS_ATOMIC_MAX(coverageAtomicBuffer, minCoverageData);
    ushort lastPathID =
        cast_uint_to_ushort(lastCoverageData >> FIXED_COVERAGE_BIT_COUNT);

    lastPathID = apply_driver_workaround_for_path_id(lastPathID);

    if (lastPathID == v_pathID)
    {
        // This is not the first fragment of the current path to touch this
        // pixel. We already resolved the previous path, so just update coverage
        // (if we're a fill) and move on.
        if (!is_stroke(v_coverages))
        {
            // Only apply the effect of the min() the first time we cross into a
            // path.
            fixedCoverage +=
                lastCoverageData - max(minCoverageData, lastCoverageData);
            fixedCoverage -=
                FIXED_COVERAGE_ZERO_UINT; // Only apply the zero bias once.
            PLS_ATOMIC_ADD(coverageAtomicBuffer,
                           fixedCoverage); // Count coverage.
        }
    }
    else
    {
        // We crossed into a new path! Resolve the previous path now that we
        // know its exact coverage.
        half coverageCount = from_fixed(lastCoverageData & FIXED_COVERAGE_MASK);
        resolve_paint(lastPathID,
                      coverageCount,
                      fragColorOut
#ifdef @ENABLE_CLIPPING
                      ,
                      fragClipOut
#endif
                          FRAGMENT_CONTEXT_UNPACK PLS_CONTEXT_UNPACK);
    }

    fragColorOut.rgb = add_dither_if_alpha_nonzero(fragColorOut.rgb,
                                                   fragColorOut.a,
                                                   _fragCoord.xy,
                                                   uniforms.ditherScale,
                                                   uniforms.ditherBias);
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    _fragColor = fragColorOut;
#else
    blend_pls_color_src_over(fragColorOut PLS_CONTEXT_UNPACK);
#endif
#ifdef @ENABLE_CLIPPING
    emit_pls_clip(fragClipOut PLS_CONTEXT_UNPACK);
#endif

    EMIT_ATOMIC_PLS
}
#endif // DRAW_PATH

#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)
ATOMIC_PLS_MAIN(@drawFragmentMain)
{
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_UNPACK(v_atlasCoord, float2);
#else
    VARYING_UNPACK(v_windingWeight, half);
#endif
    VARYING_UNPACK(v_pathID, ushort);

    uint lastCoverageData = PLS_LOADUI_UAV(coverageAtomicBuffer);
    ushort lastPathID =
        cast_uint_to_ushort(lastCoverageData >> FIXED_COVERAGE_BIT_COUNT);
    lastPathID = apply_driver_workaround_for_path_id(lastPathID);
    // Update coverageAtomicBuffer with the coverage weight of the current
    // triangle. This does not need to be atomic since interior triangles don't
    // overlap.
    uint currPathCoverageData;
#ifndef @FEATHER_ATLAS_BLIT
    if (lastPathID == v_pathID)
    {
        currPathCoverageData = lastCoverageData;
    }
    else
#endif
    {
        currPathCoverageData =
            (make_uint(v_pathID) << FIXED_COVERAGE_BIT_COUNT) +
            FIXED_COVERAGE_ZERO_UINT;
    }

    half coverage;
#ifdef @FEATHER_ATLAS_BLIT
    coverage = clamp(TEXTURE_SAMPLE_LOD(@featherAtlasTexture,
                                        featherAtlasSampler,
                                        v_atlasCoord,
                                        .0)
                         .r,
                     make_half(.0),
                     make_half(1.));
#else
    coverage = v_windingWeight;
#endif

    int coverageDeltaFixed = int(round(coverage * FIXED_COVERAGE_PRECISION));
    PLS_STOREUI_UAV(coverageAtomicBuffer,
                    currPathCoverageData + uint(coverageDeltaFixed));

    half4 fragColorOut = make_half4(.0);
#ifdef @ENABLE_CLIPPING
    CLIP_VALUE_TYPE fragClipOut = MAKE_NON_UPDATING_CLIP_VALUE;
#endif

#ifndef @FEATHER_ATLAS_BLIT
    // If this is not the first fragment of the current path to touch this
    // pixel, then we've already resolved the previous path and can move on.
    if (lastPathID != v_pathID)
#endif
    {
        // We crossed into a new path! Resolve the previous path now that we
        // know its exact coverage.
        half lastCoverageCount =
            from_fixed(lastCoverageData & FIXED_COVERAGE_MASK);
        resolve_paint(lastPathID,
                      lastCoverageCount,
                      fragColorOut
#ifdef @ENABLE_CLIPPING
                      ,
                      fragClipOut
#endif
                          FRAGMENT_CONTEXT_UNPACK PLS_CONTEXT_UNPACK);
    }

    fragColorOut.rgb = add_dither_if_alpha_nonzero(fragColorOut.rgb,
                                                   fragColorOut.a,
                                                   _fragCoord.xy,
                                                   uniforms.ditherScale,
                                                   uniforms.ditherBias);
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    _fragColor = fragColorOut;
#else
    blend_pls_color_src_over(fragColorOut PLS_CONTEXT_UNPACK);
#endif
#ifdef @ENABLE_CLIPPING
    emit_pls_clip(fragClipOut PLS_CONTEXT_UNPACK);
#endif

    EMIT_ATOMIC_PLS
}
#endif // @DRAW_INTERIOR_TRIANGLES || @FEATHER_ATLAS_BLIT

#ifdef @DRAW_IMAGE
ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    VARYING_UNPACK(v_texCoord, float2);
#ifdef @DRAW_IMAGE_RECT
    VARYING_UNPACK(v_edgeCoverage, half);
#endif
#ifdef @ENABLE_CLIP_RECT
    VARYING_UNPACK(v_clipRect, float4);
#endif
    VARYING_UNPACK(v_imageOpacity, half);
#ifdef @ENABLE_CLIPPING
    VARYING_UNPACK(v_imageClipID, ushort);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_imageBlendMode, ushort);
#endif

    // Start by finding the image color. We have to do this immediately instead
    // of allowing it to get resolved later like other draws because the
    // @imageTexture binding is liable to change, and furthermore in the case of
    // imageMeshes, we can't calculate UV coordinates based on fragment
    // position.
    half4 imageColor =
        TEXTURE_SAMPLE_DYNAMIC(@imageTexture, imageSampler, v_texCoord);
    half imageCoverage = 1.;
#ifdef @DRAW_IMAGE_RECT
    imageCoverage = min(v_edgeCoverage, imageCoverage);
#endif
#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
        half clipRectCoverage = min_component(cast_float4_to_half4(v_clipRect));
        imageCoverage = clamp(clipRectCoverage, make_half(.0), imageCoverage);
    }
#endif

    // Resolve the previous path.
    uint lastCoverageData = PLS_LOADUI_UAV(coverageAtomicBuffer);
    ushort lastPathID =
        cast_uint_to_ushort(lastCoverageData >> FIXED_COVERAGE_BIT_COUNT);
    lastPathID = apply_driver_workaround_for_path_id(lastPathID);
    half lastCoverageCount = from_fixed(lastCoverageData & FIXED_COVERAGE_MASK);
    half4 fragColorOut;
#ifdef @ENABLE_CLIPPING
    CLIP_VALUE_TYPE fragClipOut = MAKE_NON_UPDATING_CLIP_VALUE;
#endif
    // TODO: consider not resolving the prior paint if we're solid and the prior
    // paint is not a clip update: (imageColor.a == 1. &&
    //                              v_imageBlendMode ==
    //                              BLEND_SRC_OVER && priorPaintType !=
    //                              CLIP_UPDATE_PAINT_TYPE)
    resolve_paint(lastPathID,
                  lastCoverageCount,
                  fragColorOut
#ifdef @ENABLE_CLIPPING
                  ,
                  fragClipOut
#endif
                      FRAGMENT_CONTEXT_UNPACK PLS_CONTEXT_UNPACK);

// Clip the image after resolving the previous path, since that can affect
// the clip buffer.
#ifdef @ENABLE_CLIPPING // TODO! ENABLE_IMAGE_CLIPPING in addition to
                        // ENABLE_CLIPPING?
    if (@ENABLE_CLIPPING && v_imageClipID != 0u)
    {
        CLIP_VALUE_TYPE clipData = HAS_UPDATED_CLIP_VALUE(fragClipOut)
                                       ? fragClipOut
                                       : PLS_LOAD_CLIP_TYPE(clipBuffer);
        apply_clip(v_imageClipID, clipData, imageCoverage);
    }
#endif // ENABLE_CLIPPING

// Prepare imageColor for premultiplied src-over blending.
#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND)
    if (@ENABLE_ADVANCED_BLEND && v_imageBlendMode != BLEND_SRC_OVER)
    {
        // Calculate what dstColorPremul will be after applying fragColorOut.
        half4 dstColorPremul =
            PLS_LOAD4F(colorBuffer) * (1. - fragColorOut.a) + fragColorOut;
        // Calculate the imageColor to emit *BEFORE* src-over blending, such
        // that the post-src-over-blend result is equivalent to the blendMode.
        imageColor.rgb = advanced_color_blend(unmultiply_rgb(imageColor),
                                              dstColorPremul,
                                              v_imageBlendMode) *
                         imageColor.a;
    }
#endif // !FIXED_FUNCTION_COLOR_OUTPUT && ENABLE_ADVANCED_BLEND
    imageColor *= imageCoverage * v_imageOpacity;

#if defined(@NEEDS_GAMMA_CORRECTION)
    imageColor = gamma_to_linear(imageColor);
#endif

    // Leverage the property that premultiplied src-over blending is associative
    // and blend the imageColor and fragColorOut before passing them on to the
    // blending pipeline.
    fragColorOut = fragColorOut * (1. - imageColor.a) + imageColor;

    fragColorOut.rgb = add_dither_if_alpha_nonzero(fragColorOut.rgb,
                                                   fragColorOut.a,
                                                   _fragCoord.xy,
                                                   uniforms.ditherScale,
                                                   uniforms.ditherBias);
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    _fragColor = fragColorOut;
#else
    blend_pls_color_src_over(fragColorOut PLS_CONTEXT_UNPACK);
#endif
#ifdef @ENABLE_CLIPPING
    emit_pls_clip(fragClipOut PLS_CONTEXT_UNPACK);
#endif

    // Write out a coverage value of "zero at pathID=0" so a future resolve
    // attempt doesn't affect this pixel.
    PLS_STOREUI_UAV(coverageAtomicBuffer, FIXED_COVERAGE_ZERO_UINT);

    EMIT_ATOMIC_PLS
}
#endif // DRAW_IMAGE

#ifdef @INITIALIZE_PLS

ATOMIC_PLS_MAIN(@drawFragmentMain)
{
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
#ifdef @STORE_COLOR_CLEAR
    if (@STORE_COLOR_CLEAR)
    {
        PLS_STORE4F(colorBuffer, unpackUnorm4x8(uniforms.colorClearValue));
    }
#endif
#ifdef @LOAD_COLOR_FROM_DST_TEXTURE
    if (@LOAD_COLOR_FROM_DST_TEXTURE)
    {
        PLS_STORE4F(colorBuffer, TEXEL_FETCH(@imageTexture, _plsCoord));
    }
#endif
#ifdef @SWIZZLE_COLOR_BGRA_TO_RGBA
    half4 color = PLS_LOAD4F(colorBuffer);
    PLS_STORE4F(colorBuffer, color.bgra);
#endif
#endif
    PLS_STOREUI_UAV(coverageAtomicBuffer, uniforms.coverageClearValue);
#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING)
    {
        PLS_STOREUI(clipBuffer, 0u);
    }
#endif
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    discard;
#endif
    EMIT_ATOMIC_PLS
}

#endif // INITIALIZE_PLS

#ifdef @RESOLVE_PLS

#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER
PLS_FRAG_COLOR_MAIN(@drawFragmentMain)
#else
ATOMIC_PLS_MAIN(@drawFragmentMain)
#endif
{
    uint lastCoverageData = PLS_LOADUI_UAV(coverageAtomicBuffer);
    half coverageCount = from_fixed(lastCoverageData & FIXED_COVERAGE_MASK);
    ushort lastPathID =
        cast_uint_to_ushort(lastCoverageData >> FIXED_COVERAGE_BIT_COUNT);
    lastPathID = apply_driver_workaround_for_path_id(lastPathID);
    half4 fragColorOut;
    resolve_paint(lastPathID,
                  coverageCount,
                  fragColorOut FRAGMENT_CONTEXT_UNPACK PLS_CONTEXT_UNPACK);
#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER
    float oneMinusSrcAlpha = 1. - fragColorOut.a;
    if (oneMinusSrcAlpha != .0)
        fragColorOut += PLS_LOAD4F(colorBuffer) * oneMinusSrcAlpha;
    _fragColor = fragColorOut;
    EMIT_PLS_AND_FRAG_COLOR
#else

    fragColorOut.rgb = add_dither_if_alpha_nonzero(fragColorOut.rgb,
                                                   fragColorOut.a,
                                                   _fragCoord.xy,
                                                   uniforms.ditherScale,
                                                   uniforms.ditherBias);
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    _fragColor = fragColorOut;
#else
    blend_pls_color_src_over(fragColorOut PLS_CONTEXT_UNPACK);
#endif

    EMIT_ATOMIC_PLS
#endif // COALESCED_PLS_RESOLVE_AND_TRANSFER
}
#endif // RESOLVE_PLS
#endif // FRAGMENT
"###;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

/// Every conditional block and branch entry recorded for this source by the
/// pinned preprocessor authority, in source order.
pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[
    ConditionalBranch {
        block_id: "pp-0105",
        block_start: 5,
        block_end: 69,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @DRAW_PATH",
        active_branch_path: "(defined(@DRAW_PATH))",
    },
    ConditionalBranch {
        block_id: "pp-0106",
        block_start: 6,
        block_end: 13,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 6,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0107",
        block_start: 16,
        block_end: 20,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 16,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0107",
        block_start: 16,
        block_end: 20,
        block_depth: 1,
        branch_ordinal: 2,
        branch_line: 18,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_PATH)) && (!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0108",
        block_start: 24,
        block_end: 68,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 24,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0109",
        block_start: 30,
        block_end: 34,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 30,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0109",
        block_start: 30,
        block_end: 34,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 32,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX)) && (!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0110",
        block_start: 48,
        block_end: 52,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 48,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0110",
        block_start: 48,
        block_end: 52,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 50,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_PATH)) && (defined(@VERTEX)) && (!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0111",
        block_start: 71,
        block_end: 124,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 71,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0112",
        block_start: 72,
        block_end: 76,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 72,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0113",
        block_start: 79,
        block_end: 83,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 79,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0113",
        block_start: 79,
        block_end: 83,
        block_depth: 1,
        branch_ordinal: 2,
        branch_line: 81,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0114",
        block_start: 87,
        block_end: 123,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 87,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0115",
        block_start: 92,
        block_end: 96,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 92,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0115",
        block_start: 92,
        block_end: 96,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 94,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0116",
        block_start: 101,
        block_end: 111,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 101,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0116",
        block_start: 101,
        block_end: 111,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 106,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0117",
        block_start: 115,
        block_end: 119,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 115,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0117",
        block_start: 115,
        block_end: 119,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 117,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@VERTEX)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0118",
        block_start: 126,
        block_end: 411,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 126,
        directive: "#ifdef @DRAW_IMAGE_RECT",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0118",
        block_start: 126,
        block_end: 411,
        block_depth: 0,
        branch_ordinal: 2,
        branch_line: 289,
        directive: "#elif defined(@DRAW_IMAGE_MESH)",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH)))",
    },
    ConditionalBranch {
        block_id: "pp-0119",
        block_start: 127,
        block_end: 147,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 127,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0120",
        block_start: 138,
        block_end: 145,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 138,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0120",
        block_start: 138,
        block_end: 145,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 143,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0121",
        block_start: 152,
        block_end: 154,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 152,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0122",
        block_start: 156,
        block_end: 158,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 156,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0123",
        block_start: 159,
        block_end: 161,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 159,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0124",
        block_start: 164,
        block_end: 287,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 164,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0125",
        block_start: 180,
        block_end: 191,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 180,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0125",
        block_start: 180,
        block_end: 191,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 189,
        directive: "#else",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0126",
        block_start: 195,
        block_end: 197,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 195,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0127",
        block_start: 199,
        block_end: 201,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 199,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0128",
        block_start: 202,
        block_end: 204,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 202,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0129",
        block_start: 253,
        block_end: 261,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 253,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0130",
        block_start: 264,
        block_end: 266,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 264,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0131",
        block_start: 267,
        block_end: 269,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 267,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0132",
        block_start: 275,
        block_end: 277,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 275,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0133",
        block_start: 279,
        block_end: 281,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 279,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0134",
        block_start: 282,
        block_end: 284,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 282,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@DRAW_IMAGE_RECT)) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0135",
        block_start: 290,
        block_end: 314,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 290,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0136",
        block_start: 305,
        block_end: 312,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 305,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0136",
        block_start: 305,
        block_end: 312,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 310,
        directive: "#else",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0137",
        block_start: 318,
        block_end: 320,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 318,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0138",
        block_start: 322,
        block_end: 324,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 322,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0139",
        block_start: 325,
        block_end: 327,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 325,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0140",
        block_start: 330,
        block_end: 410,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 330,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0141",
        block_start: 348,
        block_end: 359,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 348,
        directive: "#ifdef SPLIT_UINT4_ATTRIBUTES",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(SPLIT_UINT4_ATTRIBUTES))",
    },
    ConditionalBranch {
        block_id: "pp-0141",
        block_start: 348,
        block_end: 359,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 357,
        directive: "#else",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (!((defined(SPLIT_UINT4_ATTRIBUTES))))",
    },
    ConditionalBranch {
        block_id: "pp-0142",
        block_start: 362,
        block_end: 364,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 362,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0143",
        block_start: 366,
        block_end: 368,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 366,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0144",
        block_start: 369,
        block_end: 371,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 369,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0145",
        block_start: 377,
        block_end: 385,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 377,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0146",
        block_start: 388,
        block_end: 390,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 388,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0147",
        block_start: 391,
        block_end: 393,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 391,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0148",
        block_start: 398,
        block_end: 400,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 398,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0149",
        block_start: 402,
        block_end: 404,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 402,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0150",
        block_start: 405,
        block_end: 407,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 405,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(!((defined(@DRAW_IMAGE_RECT))) && (defined(@DRAW_IMAGE_MESH))) && (defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0151",
        block_start: 413,
        block_end: 434,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 413,
        directive: "#ifdef @DRAW_RENDER_TARGET_UPDATE_BOUNDS",
        active_branch_path: "(defined(@DRAW_RENDER_TARGET_UPDATE_BOUNDS))",
    },
    ConditionalBranch {
        block_id: "pp-0152",
        block_start: 414,
        block_end: 417,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 414,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_RENDER_TARGET_UPDATE_BOUNDS)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0153",
        block_start: 422,
        block_end: 433,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 422,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@DRAW_RENDER_TARGET_UPDATE_BOUNDS)) && (defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0154",
        block_start: 436,
        block_end: 438,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 436,
        directive: "#ifdef @DRAW_IMAGE",
        active_branch_path: "(defined(@DRAW_IMAGE))",
    },
    ConditionalBranch {
        block_id: "pp-0155",
        block_start: 446,
        block_end: 448,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 446,
        directive: "#if defined(@INITIALIZE_PLS) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)",
        active_branch_path: "(defined(@INITIALIZE_PLS) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0156",
        block_start: 450,
        block_end: 1104,
        block_depth: 0,
        branch_ordinal: 1,
        branch_line: 450,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0157",
        block_start: 454,
        block_end: 467,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 454,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0158",
        block_start: 455,
        block_end: 461,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 455,
        directive: "#ifdef @COLOR_PLANE_IDX_OVERRIDE",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@COLOR_PLANE_IDX_OVERRIDE))",
    },
    ConditionalBranch {
        block_id: "pp-0158",
        block_start: 455,
        block_end: 461,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 459,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!((defined(@COLOR_PLANE_IDX_OVERRIDE))))",
    },
    ConditionalBranch {
        block_id: "pp-0159",
        block_start: 462,
        block_end: 466,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 462,
        directive: "#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))",
    },
    ConditionalBranch {
        block_id: "pp-0159",
        block_start: 462,
        block_end: 466,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 464,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!((defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))))",
    },
    ConditionalBranch {
        block_id: "pp-0160",
        block_start: 468,
        block_end: 494,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 468,
        directive: "#ifdef @PLS_BLEND_SRC_OVER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@PLS_BLEND_SRC_OVER))",
    },
    ConditionalBranch {
        block_id: "pp-0160",
        block_start: 468,
        block_end: 494,
        block_depth: 1,
        branch_ordinal: 2,
        branch_line: 484,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@PLS_BLEND_SRC_OVER))))",
    },
    ConditionalBranch {
        block_id: "pp-0161",
        block_start: 477,
        block_end: 483,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 477,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@PLS_BLEND_SRC_OVER)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0162",
        block_start: 478,
        block_end: 482,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 478,
        directive: "#ifndef @RESOLVE_PLS",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@PLS_BLEND_SRC_OVER)) && (defined(@ENABLE_CLIPPING)) && (!defined(@RESOLVE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0162",
        block_start: 478,
        block_end: 482,
        block_depth: 3,
        branch_ordinal: 2,
        branch_line: 480,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@PLS_BLEND_SRC_OVER)) && (defined(@ENABLE_CLIPPING)) && (!((!defined(@RESOLVE_PLS))))",
    },
    ConditionalBranch {
        block_id: "pp-0163",
        block_start: 491,
        block_end: 493,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 491,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@PLS_BLEND_SRC_OVER)))) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0164",
        block_start: 517,
        block_end: 523,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 517,
        directive: "#ifdef @NEEDS_PATH_ID_CLAMP_WORKAROUND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_PATH_ID_CLAMP_WORKAROUND))",
    },
    ConditionalBranch {
        block_id: "pp-0165",
        block_start: 527,
        block_end: 549,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 527,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0166",
        block_start: 532,
        block_end: 547,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 532,
        directive: "#ifdef @PLS_BLEND_SRC_OVER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (defined(@PLS_BLEND_SRC_OVER))",
    },
    ConditionalBranch {
        block_id: "pp-0166",
        block_start: 532,
        block_end: 547,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 541,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!((defined(@PLS_BLEND_SRC_OVER))))",
    },
    ConditionalBranch {
        block_id: "pp-0167",
        block_start: 554,
        block_end: 557,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 554,
        directive: "#if defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0168",
        block_start: 567,
        block_end: 572,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 567,
        directive: "#ifdef @ENABLE_EVEN_ODD",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_EVEN_ODD))",
    },
    ConditionalBranch {
        block_id: "pp-0169",
        block_start: 576,
        block_end: 585,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 576,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0170",
        block_start: 586,
        block_end: 601,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 586,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0171",
        block_start: 607,
        block_end: 625,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 607,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0172",
        block_start: 610,
        block_end: 621,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 610,
        directive: "#ifndef @RESOLVE_PLS",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!defined(@RESOLVE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0173",
        block_start: 611,
        block_end: 620,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 611,
        directive: "#ifdef @PLS_BLEND_SRC_OVER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!defined(@RESOLVE_PLS)) && (defined(@PLS_BLEND_SRC_OVER))",
    },
    ConditionalBranch {
        block_id: "pp-0173",
        block_start: 611,
        block_end: 620,
        block_depth: 3,
        branch_ordinal: 2,
        branch_line: 618,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!defined(@RESOLVE_PLS)) && (!((defined(@PLS_BLEND_SRC_OVER))))",
    },
    ConditionalBranch {
        block_id: "pp-0174",
        block_start: 645,
        block_end: 656,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 645,
        directive: "#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0175",
        block_start: 662,
        block_end: 665,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 662,
        directive: "#if defined(@NEEDS_GAMMA_CORRECTION) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT) || defined(@RESOLVE_PLS))",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT) || defined(@RESOLVE_PLS)))",
    },
    ConditionalBranch {
        block_id: "pp-0176",
        block_start: 670,
        block_end: 683,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 670,
        directive: "#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))",
    },
    ConditionalBranch {
        block_id: "pp-0177",
        block_start: 674,
        block_end: 680,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 674,
        directive: "#ifndef @PLS_BLEND_SRC_OVER",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER)) && (!defined(@PLS_BLEND_SRC_OVER))",
    },
    ConditionalBranch {
        block_id: "pp-0178",
        block_start: 686,
        block_end: 696,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 686,
        directive: "#if defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0179",
        block_start: 689,
        block_end: 694,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 689,
        directive: "#ifdef @PLS_BLEND_SRC_OVER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)) && (defined(@PLS_BLEND_SRC_OVER))",
    },
    ConditionalBranch {
        block_id: "pp-0179",
        block_start: 689,
        block_end: 694,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 691,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING) && !defined(@RESOLVE_PLS)) && (!((defined(@PLS_BLEND_SRC_OVER))))",
    },
    ConditionalBranch {
        block_id: "pp-0180",
        block_start: 698,
        block_end: 704,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 698,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0180",
        block_start: 698,
        block_end: 704,
        block_depth: 1,
        branch_ordinal: 2,
        branch_line: 701,
        directive: "#else // !FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0181",
        block_start: 706,
        block_end: 812,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 706,
        directive: "#ifdef @DRAW_PATH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH))",
    },
    ConditionalBranch {
        block_id: "pp-0182",
        block_start: 709,
        block_end: 713,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 709,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0182",
        block_start: 709,
        block_end: 713,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 711,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0183",
        block_start: 718,
        block_end: 730,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 718,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0184",
        block_start: 739,
        block_end: 741,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 739,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0185",
        block_start: 789,
        block_end: 792,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 789,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0186",
        block_start: 801,
        block_end: 805,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 801,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0186",
        block_start: 801,
        block_end: 805,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 803,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0187",
        block_start: 806,
        block_end: 808,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 806,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_PATH)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0188",
        block_start: 814,
        block_end: 903,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 814,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0189",
        block_start: 817,
        block_end: 821,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 817,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0189",
        block_start: 817,
        block_end: 821,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 819,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0190",
        block_start: 832,
        block_end: 838,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 832,
        directive: "#ifndef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0191",
        block_start: 846,
        block_end: 856,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 846,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0191",
        block_start: 846,
        block_end: 856,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 854,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0192",
        block_start: 863,
        block_end: 865,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 863,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0193",
        block_start: 867,
        block_end: 871,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 867,
        directive: "#ifndef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0194",
        block_start: 880,
        block_end: 883,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 880,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0195",
        block_start: 892,
        block_end: 896,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 892,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0195",
        block_start: 892,
        block_end: 896,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 894,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0196",
        block_start: 897,
        block_end: 899,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 897,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0197",
        block_start: 905,
        block_end: 1025,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 905,
        directive: "#ifdef @DRAW_IMAGE",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE))",
    },
    ConditionalBranch {
        block_id: "pp-0198",
        block_start: 909,
        block_end: 911,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 909,
        directive: "#ifdef @DRAW_IMAGE_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@DRAW_IMAGE_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0199",
        block_start: 912,
        block_end: 914,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 912,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0200",
        block_start: 916,
        block_end: 918,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 916,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0201",
        block_start: 919,
        block_end: 921,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 919,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0202",
        block_start: 931,
        block_end: 933,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 931,
        directive: "#ifdef @DRAW_IMAGE_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@DRAW_IMAGE_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0203",
        block_start: 934,
        block_end: 940,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 934,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0204",
        block_start: 949,
        block_end: 951,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 949,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0205",
        block_start: 960,
        block_end: 963,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 960,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0206",
        block_start: 968,
        block_end: 977,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 968,
        directive: "#ifdef @ENABLE_CLIPPING // TODO! ENABLE_IMAGE_CLIPPING in addition to",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIPPING // TODO! ENABLE_IMAGE_CLIPPING in addition to))",
    },
    ConditionalBranch {
        block_id: "pp-0207",
        block_start: 980,
        block_end: 993,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 980,
        directive: "#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0208",
        block_start: 996,
        block_end: 998,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 996,
        directive: "#if defined(@NEEDS_GAMMA_CORRECTION)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@NEEDS_GAMMA_CORRECTION))",
    },
    ConditionalBranch {
        block_id: "pp-0209",
        block_start: 1010,
        block_end: 1014,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1010,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0209",
        block_start: 1010,
        block_end: 1014,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 1012,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0210",
        block_start: 1015,
        block_end: 1017,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1015,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0211",
        block_start: 1027,
        block_end: 1062,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 1027,
        directive: "#ifdef @INITIALIZE_PLS",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0212",
        block_start: 1031,
        block_end: 1048,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1031,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0213",
        block_start: 1032,
        block_end: 1037,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 1032,
        directive: "#ifdef @STORE_COLOR_CLEAR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@STORE_COLOR_CLEAR))",
    },
    ConditionalBranch {
        block_id: "pp-0214",
        block_start: 1038,
        block_end: 1043,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 1038,
        directive: "#ifdef @LOAD_COLOR_FROM_DST_TEXTURE",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@LOAD_COLOR_FROM_DST_TEXTURE))",
    },
    ConditionalBranch {
        block_id: "pp-0215",
        block_start: 1044,
        block_end: 1047,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 1044,
        directive: "#ifdef @SWIZZLE_COLOR_BGRA_TO_RGBA",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@SWIZZLE_COLOR_BGRA_TO_RGBA))",
    },
    ConditionalBranch {
        block_id: "pp-0216",
        block_start: 1050,
        block_end: 1055,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1050,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0217",
        block_start: 1056,
        block_end: 1058,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1056,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@INITIALIZE_PLS)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0218",
        block_start: 1064,
        block_end: 1103,
        block_depth: 1,
        branch_ordinal: 1,
        branch_line: 1064,
        directive: "#ifdef @RESOLVE_PLS",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS))",
    },
    ConditionalBranch {
        block_id: "pp-0219",
        block_start: 1066,
        block_end: 1070,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1066,
        directive: "#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))",
    },
    ConditionalBranch {
        block_id: "pp-0219",
        block_start: 1066,
        block_end: 1070,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 1068,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (!((defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))))",
    },
    ConditionalBranch {
        block_id: "pp-0220",
        block_start: 1081,
        block_end: 1101,
        block_depth: 2,
        branch_ordinal: 1,
        branch_line: 1081,
        directive: "#ifdef @COALESCED_PLS_RESOLVE_AND_TRANSFER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))",
    },
    ConditionalBranch {
        block_id: "pp-0220",
        block_start: 1081,
        block_end: 1101,
        block_depth: 2,
        branch_ordinal: 2,
        branch_line: 1087,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (!((defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER))))",
    },
    ConditionalBranch {
        block_id: "pp-0221",
        block_start: 1094,
        block_end: 1098,
        block_depth: 3,
        branch_ordinal: 1,
        branch_line: 1094,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (!((defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER)))) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0221",
        block_start: 1094,
        block_end: 1098,
        block_depth: 3,
        branch_ordinal: 2,
        branch_line: 1096,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RESOLVE_PLS)) && (!((defined(@COALESCED_PLS_RESOLVE_AND_TRANSFER)))) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
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

/// atomic_draw.glsl has no direct #include/#import directives.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// The generated atomic_draw.glsl.hpp consumer edge is retained separately
/// because it belongs to the background compiler, not to this GLSL source.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[ShaderInclude {
    upstream_file: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 21,
    directive: "include",
    include_token: "generated/shaders/atomic_draw.glsl.hpp",
    include_syntax: "quote",
    active_branch_path: "(!defined(RIVE_IOS))",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/atomic_draw.glsl",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    correspondence_owner: "-",
    mapping_status: "prepared",
    translation_status: "pending",
    translation_disposition: "required-source-edge",
}];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Export inventory from the pinned batch minifier's generated
/// atomic_draw.glsl.exports.h. The shared export set is preserved exactly.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "ATLAS_BLIT",
        generated_name: "EB",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_FILL",
        generated_name: "FC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_STROKE",
        generated_name: "MC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "ME",
    },
    ExportedIdentifier {
        source_name: "BASE_INSTANCE_UNIFORM_NAME",
        generated_name: "RD",
    },
    ExportedIdentifier {
        source_name: "BORROWED_COVERAGE_PASS",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "CLEAR_CLIP",
        generated_name: "JF",
    },
    ExportedIdentifier {
        source_name: "CLEAR_COLOR",
        generated_name: "JE",
    },
    ExportedIdentifier {
        source_name: "CLEAR_COVERAGE",
        generated_name: "TD",
    },
    ExportedIdentifier {
        source_name: "CLOCKWISE_FILL",
        generated_name: "UD",
    },
    ExportedIdentifier {
        source_name: "COALESCED_PLS_RESOLVE_AND_TRANSFER",
        generated_name: "SC",
    },
    ExportedIdentifier {
        source_name: "COLOR_PLANE_IDX_OVERRIDE",
        generated_name: "CE",
    },
    ExportedIdentifier {
        source_name: "DISABLE_ADVANCED_BLEND",
        generated_name: "NF",
    },
    ExportedIdentifier {
        source_name: "DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        generated_name: "EE",
    },
    ExportedIdentifier {
        source_name: "DISABLE_SHADER_STORAGE_BUFFERS",
        generated_name: "CF",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE",
        generated_name: "AE",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_MESH",
        generated_name: "LB",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_RECT",
        generated_name: "CD",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "DRAW_PATH",
        generated_name: "BD",
    },
    ExportedIdentifier {
        source_name: "DRAW_RENDER_TARGET_UPDATE_BOUNDS",
        generated_name: "TE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIPPING",
        generated_name: "O",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIP_RECT",
        generated_name: "AB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_DITHER",
        generated_name: "JB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_EVEN_ODD",
        generated_name: "PC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_FEATHER",
        generated_name: "HB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_HSL_BLEND_MODES",
        generated_name: "XB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_INSTANCE_INDEX",
        generated_name: "GE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_KHR_BLEND",
        generated_name: "ZD",
    },
    ExportedIdentifier {
        source_name: "ENABLE_MIN_16_PRECISION",
        generated_name: "HE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_RASTERIZER_ORDERED_VIEWS",
        generated_name: "IE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_TYPED_UAV_LOAD_STORE",
        generated_name: "KC",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "FRAMEBUFFER_BOTTOM_UP",
        generated_name: "SF",
    },
    ExportedIdentifier {
        source_name: "FlushUniforms",
        generated_name: "NB",
    },
    ExportedIdentifier {
        source_name: "GLSL_VERSION",
        generated_name: "EC",
    },
    ExportedIdentifier {
        source_name: "GL_RENDERER_MALI",
        generated_name: "AF",
    },
    ExportedIdentifier {
        source_name: "INITIALIZE_PLS",
        generated_name: "BE",
    },
    ExportedIdentifier {
        source_name: "INPUT_ATTACHMENT_BINDING",
        generated_name: "SE",
    },
    ExportedIdentifier {
        source_name: "ImageDrawUniforms",
        generated_name: "LC",
    },
    ExportedIdentifier {
        source_name: "LOAD_COLOR",
        generated_name: "LE",
    },
    ExportedIdentifier {
        source_name: "LOAD_COLOR_FROM_DST_TEXTURE",
        generated_name: "FD",
    },
    ExportedIdentifier {
        source_name: "NEEDS_CLIP_DISTANCE",
        generated_name: "QE",
    },
    ExportedIdentifier {
        source_name: "NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
    },
    ExportedIdentifier {
        source_name: "NEEDS_PATH_ID_CLAMP_WORKAROUND",
        generated_name: "UE",
    },
    ExportedIdentifier {
        source_name: "NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
    },
    ExportedIdentifier {
        source_name: "NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        generated_name: "RF",
    },
    ExportedIdentifier {
        source_name: "NO_VARYING",
        generated_name: "RE",
    },
    ExportedIdentifier {
        source_name: "OPTIONALLY_FLAT",
        generated_name: "OB",
    },
    ExportedIdentifier {
        source_name: "PLS_BLEND_SRC_OVER",
        generated_name: "OC",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_ANGLE",
        generated_name: "EXPORTED_PLS_IMPL_ANGLE",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_DEVICE_BUFFER",
        generated_name: "HF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        generated_name: "IF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_EXT_NATIVE",
        generated_name: "EF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_NONE",
        generated_name: "GF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_STORAGE_BUFFER",
        generated_name: "PD",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_STORAGE_TEXTURE",
        generated_name: "QD",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_SUBPASS_LOAD",
        generated_name: "FF",
    },
    ExportedIdentifier {
        source_name: "POST_INVERT_Y",
        generated_name: "JC",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "RESOLVE_PLS",
        generated_name: "IC",
    },
    ExportedIdentifier {
        source_name: "SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
    },
    ExportedIdentifier {
        source_name: "STORE_COLOR",
        generated_name: "SD",
    },
    ExportedIdentifier {
        source_name: "STORE_COLOR_CLEAR",
        generated_name: "ED",
    },
    ExportedIdentifier {
        source_name: "SUPPORTS_SUBPASS_LOAD",
        generated_name: "MF",
    },
    ExportedIdentifier {
        source_name: "SWIZZLE_COLOR_BGRA_TO_RGBA",
        generated_name: "VE",
    },
    ExportedIdentifier {
        source_name: "TARGET_SPIRV",
        generated_name: "VB",
    },
    ExportedIdentifier {
        source_name: "TARGET_WGSL",
        generated_name: "FE",
    },
    ExportedIdentifier {
        source_name: "TESS_TEXTURE_FLOATING_POINT",
        generated_name: "ZE",
    },
    ExportedIdentifier {
        source_name: "USE_FILTERING",
        generated_name: "VC",
    },
    ExportedIdentifier {
        source_name: "USE_WEBGPU_SAMPLERS",
        generated_name: "BF",
    },
    ExportedIdentifier {
        source_name: "USING_PLS_STORAGE_TEXTURES",
        generated_name: "DF",
    },
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "VULKAN_VENDOR_ARM",
        generated_name: "WC",
    },
    ExportedIdentifier {
        source_name: "a_args",
        generated_name: "RB",
    },
    ExportedIdentifier {
        source_name: "a_color0",
        generated_name: "JD",
    },
    ExportedIdentifier {
        source_name: "a_color1",
        generated_name: "KD",
    },
    ExportedIdentifier {
        source_name: "a_contourIDWithFlags",
        generated_name: "YD",
    },
    ExportedIdentifier {
        source_name: "a_imageRectVertex",
        generated_name: "ZB",
    },
    ExportedIdentifier {
        source_name: "a_joinTan_and_ys",
        generated_name: "NC",
    },
    ExportedIdentifier {
        source_name: "a_mirroredVertexData",
        generated_name: "TB",
    },
    ExportedIdentifier {
        source_name: "a_p0p1_",
        generated_name: "ZC",
    },
    ExportedIdentifier {
        source_name: "a_p2p3_",
        generated_name: "AD",
    },
    ExportedIdentifier {
        source_name: "a_patchVertexData",
        generated_name: "SB",
    },
    ExportedIdentifier {
        source_name: "a_position",
        generated_name: "GC",
    },
    ExportedIdentifier {
        source_name: "a_reflectionX0X1",
        generated_name: "WD",
    },
    ExportedIdentifier {
        source_name: "a_segmentCounts",
        generated_name: "XD",
    },
    ExportedIdentifier {
        source_name: "a_span",
        generated_name: "CC",
    },
    ExportedIdentifier {
        source_name: "a_spanX",
        generated_name: "HD",
    },
    ExportedIdentifier {
        source_name: "a_texCoord",
        generated_name: "HC",
    },
    ExportedIdentifier {
        source_name: "a_triangleVertex",
        generated_name: "KB",
    },
    ExportedIdentifier {
        source_name: "a_x0x1",
        generated_name: "VD",
    },
    ExportedIdentifier {
        source_name: "a_yWithFlags",
        generated_name: "ID",
    },
    ExportedIdentifier {
        source_name: "atlasFillFragmentMain",
        generated_name: "NE",
    },
    ExportedIdentifier {
        source_name: "atlasRenderTexture",
        generated_name: "PE",
    },
    ExportedIdentifier {
        source_name: "atlasResolveVertexMain",
        generated_name: "LF",
    },
    ExportedIdentifier {
        source_name: "atlasStrokeFragmentMain",
        generated_name: "OE",
    },
    ExportedIdentifier {
        source_name: "atlasTexture",
        generated_name: "UC",
    },
    ExportedIdentifier {
        source_name: "atlasVertexMain",
        generated_name: "KF",
    },
    ExportedIdentifier {
        source_name: "blitFragmentMain",
        generated_name: "DE",
    },
    ExportedIdentifier {
        source_name: "blitVertexMain",
        generated_name: "WE",
    },
    ExportedIdentifier {
        source_name: "clearColor",
        generated_name: "KE",
    },
    ExportedIdentifier {
        source_name: "colorRampFragmentMain",
        generated_name: "YE",
    },
    ExportedIdentifier {
        source_name: "colorRampVertexMain",
        generated_name: "XE",
    },
    ExportedIdentifier {
        source_name: "contourBuffer",
        generated_name: "XC",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "drawVertexMain",
        generated_name: "YB",
    },
    ExportedIdentifier {
        source_name: "dstColorTexture",
        generated_name: "LD",
    },
    ExportedIdentifier {
        source_name: "featherTexture",
        generated_name: "QC",
    },
    ExportedIdentifier {
        source_name: "gradTexture",
        generated_name: "DD",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "paintAuxBuffer",
        generated_name: "PB",
    },
    ExportedIdentifier {
        source_name: "paintBuffer",
        generated_name: "TC",
    },
    ExportedIdentifier {
        source_name: "pathBuffer",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "sourceTexture",
        generated_name: "BC",
    },
    ExportedIdentifier {
        source_name: "stencilVertexMain",
        generated_name: "OF",
    },
    ExportedIdentifier {
        source_name: "tessVertexTexture",
        generated_name: "DC",
    },
    ExportedIdentifier {
        source_name: "tessellateFragmentMain",
        generated_name: "QF",
    },
    ExportedIdentifier {
        source_name: "tessellateVertexMain",
        generated_name: "PF",
    },
];

/// The 47 @-prefixed identifiers occurring directly in atomic_draw.glsl.
pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "COALESCED_PLS_RESOLVE_AND_TRANSFER",
    "COLOR_PLANE_IDX_OVERRIDE",
    "DRAW_IMAGE",
    "DRAW_IMAGE_MESH",
    "DRAW_IMAGE_RECT",
    "DRAW_INTERIOR_TRIANGLES",
    "DRAW_PATH",
    "DRAW_RENDER_TARGET_UPDATE_BOUNDS",
    "ENABLE_ADVANCED_BLEND",
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "ENABLE_EVEN_ODD",
    "ENABLE_FEATHER",
    "FEATHER_ATLAS_BLIT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "FRAGMENT",
    "INITIALIZE_PLS",
    "LOAD_COLOR_FROM_DST_TEXTURE",
    "NEEDS_GAMMA_CORRECTION",
    "NEEDS_PATH_ID_CLAMP_WORKAROUND",
    "OPTIONALLY_FLAT",
    "PLS_BLEND_SRC_OVER",
    "RESOLVE_PLS",
    "STORE_COLOR_CLEAR",
    "SWIZZLE_COLOR_BGRA_TO_RGBA",
    "VERTEX",
    "a_imageDrawBlendMode",
    "a_imageDrawClipID",
    "a_imageDrawClipRectInverseMatrix",
    "a_imageDrawOpacity",
    "a_imageDrawPacked",
    "a_imageDrawTranslates",
    "a_imageDrawViewMatrix",
    "a_imageDrawZIndex",
    "a_imageRectVertex",
    "a_mirroredVertexData",
    "a_patchVertexData",
    "a_position",
    "a_texCoord",
    "a_triangleVertex",
    "drawFragmentMain",
    "drawVertexMain",
    "featherAtlasTexture",
    "gradTexture",
    "imageTexture",
    "paintAuxBuffer",
    "paintBuffer",
];
