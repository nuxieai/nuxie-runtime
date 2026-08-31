/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_path.vert.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, includes, exports, functions, and source
 * metadata as literal source-shaped data. It does not compile, evaluate,
 * simplify, or generate shader artifacts.
 *
 * Upstream source revision: 2b2203f45a67f813cb662272962192ecfdfd923e
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2b2203f45a67f813cb662272962192ecfdfd923e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-vert";
pub const PINNED_SOURCE_SHA256: &str =
    "fbf1a2dcc7674eaf044275476c402db700d7de3a4f74fc4ac475b051e451f326";
pub const PINNED_SOURCE_LINE_COUNT: usize = 520;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18139;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_path_vert.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned vertex-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_PATH_VERT_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

// undef GENERATE_UNMULTIPLIED_PAINT_COLORS first because this file gets
// included multiple times with different defines in the Metal library.
#undef GENERATE_UNMULTIPLIED_PAINT_COLORS

#ifdef @NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS
// The specific fragment shader we're being compiled for expects un-multiplied
// paint colors all the time.
#define GENERATE_UNMULTIPLIED_PAINT_COLORS true
#elif defined(@ENABLE_ADVANCED_BLEND)
// If advanced blend is enabled, we generate unmultiplied paint colors in the
// shader. Otherwise we would have to just turn around and unmultiply them in
// order to run the blend equation.
#define GENERATE_UNMULTIPLIED_PAINT_COLORS @ENABLE_ADVANCED_BLEND
#else
// As long as advanced blend is not enabled, it's more efficient for the shader
// to generate premultiplied paint colors from the start.
#define GENERATE_UNMULTIPLIED_PAINT_COLORS false
#endif

// undef COVERAGE_TYPE first because this file gets included multiple times with
// different defines in the Metal library.
#undef COVERAGE_TYPE
#ifdef @ENABLE_FEATHER
#define COVERAGE_TYPE float4
#else
#define COVERAGE_TYPE half2
#endif

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)
ATTR(0, packed_float3, @a_triangleVertex);
#else
ATTR(0,
     float4,
     @a_patchVertexData); // [localVertexID, outset, fillCoverage, vertexType]
ATTR(1, float4, @a_mirroredVertexData);
#endif
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
NO_PERSPECTIVE VARYING(0, float4, v_paint);

#ifdef @FEATHER_ATLAS_BLIT
NO_PERSPECTIVE VARYING(1, float2, v_atlasCoord);
#elif !defined(@RENDER_MODE_MSAA)
#ifdef @DRAW_INTERIOR_TRIANGLES
@OPTIONALLY_FLAT VARYING(1, half, v_windingWeight);
#else
NO_PERSPECTIVE VARYING(2, COVERAGE_TYPE, v_coverages);
#endif //@DRAW_INTERIOR_TRIANGLES
@OPTIONALLY_FLAT VARYING(3, half, v_pathID);
#endif // !@RENDER_MODE_MSAA

#ifdef @ENABLE_CLIPPING
#ifdef @FEATHER_ATLAS_BLIT
@OPTIONALLY_FLAT VARYING(4, half, v_clipID); // [clipID, outerClipID]
#else
@OPTIONALLY_FLAT VARYING(4, half2, v_clipIDs); // [clipID, outerClipID]
#endif
#endif // @ENABLE_CLIPPING
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
NO_PERSPECTIVE VARYING(5, float4, v_clipRect);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
@OPTIONALLY_FLAT VARYING(6, half, v_blendMode);
#endif

#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC
FLAT VARYING(7, uint2, v_coveragePlacement);
VARYING(8, float2, v_coverageCoord);
#endif

VARYING_BLOCK_END

#ifdef @VERTEX

#ifdef @EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
// Emulation for VK_EXT_color_write_enable.
// 1 writes color normally; 0 suppresses it by outputting v_paint == 0 (which
// then gets discarded at the blend step).
// NOTE: This is intentionally declared inside "#ifdef @VERTEX" so it doesn't
// get needlessly added to fragment shaders.
layout(push_constant) uniform PushConstants { float colorWriteEnable; }
pushConstants;
#endif

VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)
    ATTR_UNPACK(_vertexID, attrs, @a_triangleVertex, float3);
#else
    ATTR_UNPACK(_vertexID, attrs, @a_patchVertexData, float4);
    ATTR_UNPACK(_vertexID, attrs, @a_mirroredVertexData, float4);
#endif

    VARYING_INIT(v_paint, float4);

#ifdef @FEATHER_ATLAS_BLIT
    VARYING_INIT(v_atlasCoord, float2);
#elif !defined(@RENDER_MODE_MSAA)
#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_INIT(v_windingWeight, half);
#else
    VARYING_INIT(v_coverages, COVERAGE_TYPE);
#endif //@DRAW_INTERIOR_TRIANGLES
    VARYING_INIT(v_pathID, half);
#endif // !@RENDER_MODE_MSAA

#ifdef @ENABLE_CLIPPING
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_INIT(v_clipID, half);
#else
    VARYING_INIT(v_clipIDs, half2);
#endif
#endif // @ENABLE_CLIPPING
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
    VARYING_INIT(v_clipRect, float4);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_INIT(v_blendMode, half);
#endif
#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC
    VARYING_INIT(v_coveragePlacement, uint2);
    VARYING_INIT(v_coverageCoord, float2);
#endif

    bool shouldDiscardVertex = false;
    uint pathID;
    float2 vertexPosition;
#ifdef @RENDER_MODE_MSAA
    ushort pathZIndex;
#endif

#ifdef @FEATHER_ATLAS_BLIT
    vertexPosition =
        unpack_atlas_coverage_vertex(@a_triangleVertex,
                                     pathID,
#ifdef @RENDER_MODE_MSAA
                                     pathZIndex,
#endif
                                     v_atlasCoord VERTEX_CONTEXT_UNPACK);
#elif defined(@DRAW_INTERIOR_TRIANGLES)
    vertexPosition = unpack_interior_triangle_vertex(@a_triangleVertex,
                                                     pathID
#ifdef @RENDER_MODE_MSAA
                                                     ,
                                                     pathZIndex
#else
                                                     ,
                                                     v_windingWeight
#endif
                                                         VERTEX_CONTEXT_UNPACK);
#else // !@DRAW_INTERIOR_TRIANGLES
    float4 coverages;
    shouldDiscardVertex =
        !unpack_tessellated_path_vertex(@a_patchVertexData,
                                        @a_mirroredVertexData,
                                        _instanceID,
                                        pathID,
                                        vertexPosition
#ifndef @RENDER_MODE_MSAA
                                        ,
                                        coverages
#else
                                        ,
                                        pathZIndex
#endif
                                            VERTEX_CONTEXT_UNPACK);
#ifndef @RENDER_MODE_MSAA
#ifdef @ENABLE_FEATHER
    v_coverages = coverages;
#else
    v_coverages.xy = cast_float2_to_half2(coverages.xy);
#endif
#endif
#endif // !DRAW_INTERIOR_TRIANGLES

    uint2 paintData = STORAGE_BUFFER_LOAD2(@paintBuffer, pathID);

#if !defined(@FEATHER_ATLAS_BLIT) && !defined(@RENDER_MODE_MSAA)
    // Encode the integral pathID as a "half" that we know the hardware will see
    // as a unique value in the fragment shader.
    v_pathID = id_bits_to_f16(pathID, uniforms.pathIDGranularity);

    // Indicate even-odd fill rule by making pathID negative.
    if ((paintData.x & PAINT_FLAG_EVEN_ODD_FILL) != 0u)
        v_pathID = -v_pathID;
#endif // !@FEATHER_ATLAS_BLIT && !@RENDER_MODE_MSAA

    uint paintType = paintData.x & 0xfu;
#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING)
    {
        uint clipIDBits =
            (paintType == CLIP_UPDATE_PAINT_TYPE ? paintData.y : paintData.x) >>
            16;
        half clipID = id_bits_to_f16(clipIDBits, uniforms.pathIDGranularity);
        // Negative clipID means to update the clip buffer instead of the color
        // buffer.
        if (paintType == CLIP_UPDATE_PAINT_TYPE)
            clipID = -clipID;
#ifdef @FEATHER_ATLAS_BLIT
        v_clipID = clipID;
#else
        v_clipIDs.x = clipID;
#endif
    }
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    if (@ENABLE_ADVANCED_BLEND)
    {
        v_blendMode = float((paintData.x >> 4) & 0xfu);
    }
#endif

    // Paint matrices operate on the fragment shader's "_fragCoord", which is
    // bottom-up in GL.
    float2 fragCoord = vertexPosition;
#ifdef @FRAMEBUFFER_BOTTOM_UP
    fragCoord.y = float(uniforms.renderTargetHeight) - fragCoord.y;
#endif

#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
        // clipRectInverseMatrix transforms from pixel coordinates to a space
        // where the clipRect is the normalized rectangle: [-1, -1, 1, 1].
        float2x2 clipRectInverseMatrix = make_float2x2(
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 2u));
        float4 clipRectInverseTranslate =
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 3u);
#ifndef @RENDER_MODE_MSAA
        v_clipRect =
            find_clip_rect_coverage_distances(clipRectInverseMatrix,
                                              clipRectInverseTranslate.xy,
                                              fragCoord);
#else  // !@RENDER_MODE_MSAA => @RENDER_MODE_MSAA
        set_clip_rect_plane_distances(clipRectInverseMatrix,
                                      clipRectInverseTranslate.xy,
                                      fragCoord CLIP_CONTEXT_UNPACK);
#endif // @RENDER_MODE_MSAA
    }
#endif // ENABLE_CLIP_RECT

    // Unpack the paint once we have a position.
    if (paintType == SOLID_COLOR_PAINT_TYPE)
    {
        half4 color = unpackUnorm4x8(paintData.y);
        if (GENERATE_UNMULTIPLIED_PAINT_COLORS)
        {
            // naga can't handle "if (!SpecConst)" when transpiling spv to wgsl.
            // Use this if -> else construct instead so we don't have to negate
            // a specialization constant.
        }
        else
        {
            color.rgb *= color.a;
        }
        v_paint = float4(color);
    }
#if defined(@ENABLE_CLIPPING) && !defined(@FEATHER_ATLAS_BLIT)
    else if (@ENABLE_CLIPPING && paintType == CLIP_UPDATE_PAINT_TYPE)
    {
        half outerClipID =
            id_bits_to_f16(paintData.x >> 16, uniforms.pathIDGranularity);
        v_clipIDs.y = outerClipID;
    }
#endif
    else
    {
        float2x2 paintMatrix =
            make_float2x2(STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u));
        float4 paintTranslate =
            STORAGE_BUFFER_LOAD4(@paintAuxBuffer, pathID * 4u + 1u);
        float2 paintCoord = MUL(paintMatrix, fragCoord) + paintTranslate.xy;
        if (paintType == LINEAR_GRADIENT_PAINT_TYPE ||
            paintType == RADIAL_GRADIENT_PAINT_TYPE)
        {
            // v_paint.a contains "-row" of the gradient ramp at texel center,
            // in normalized space.
            v_paint.a = -uintBitsToFloat(paintData.y);
            // abs(v_paint.b) contains either:
            //   - 2 if the gradient ramp spans an entire row.
            //   - x0 of the gradient ramp in normalized space, if it's a simple
            //   2-texel ramp.
            float gradientSpan = paintTranslate.z;
            // gradientSpan is either ~1 (complex gradients span the whole width
            // of the texture minus 1px), or 1/GRAD_TEXTURE_WIDTH (simple
            // gradients span 1px).
            if (gradientSpan > .9)
            {
                // Complex ramps span an entire row. Set it to 2 to convey this.
                v_paint.b = 2.;
            }
            else
            {
                // This is a simple ramp.
                v_paint.b = paintTranslate.w;
            }
            if (paintType == LINEAR_GRADIENT_PAINT_TYPE)
            {
                // The paint is a linear gradient.
                v_paint.g = .0;
                v_paint.r = paintCoord.x;
            }
            else
            {
                // The paint is a radial gradient. Mark v_paint.b negative to
                // indicate this to the fragment shader. (v_paint.b can't be
                // zero because the gradient ramp is aligned on pixel centers,
                // so negating it will always produce a negative number.)
                v_paint.b = -v_paint.b;
                v_paint.rg = paintCoord.xy;
            }
        }
        else // IMAGE_PAINT_TYPE
        {
            // v_paint.a <= -1. signals that the paint is an image.
            // -v_paint.a - 2 is the texture mipmap level-of-detail.
            // v_paint.b is the image opacity.
            // v_paint.rg is the normalized image texture coordinate (built into
            // the paintMatrix).
            float opacity = uintBitsToFloat(paintData.y);
            float lod = paintTranslate.z;
            v_paint = float4(paintCoord.x, paintCoord.y, opacity, -2. - lod);
        }
    }
#ifdef @EMULATE_DYNAMIC_COLOR_WRITE_DISABLE
    if (@EMULATE_DYNAMIC_COLOR_WRITE_DISABLE)
    {
        // Zeroing v_paint is all we need to disable color write; float4(0) gets
        // interpreted by the fragment shader as a fully transparent
        // SOLID_COLOR_PAINT_TYPE, and then discarded at the blend step.
        v_paint *= pushConstants.colorWriteEnable;
    }
#endif

    float4 pos;
    if (!shouldDiscardVertex)
    {
        pos = RENDER_TARGET_COORD_TO_CLIP_COORD(vertexPosition);
#ifdef @POST_INVERT_Y
        pos.y = -pos.y;
#endif
#ifdef @RENDER_MODE_MSAA
        pos.z = normalize_z_index(pathZIndex);
#elif defined(@RENDER_MODE_CLOCKWISE_ATOMIC)
        uint4 coverageData =
            STORAGE_BUFFER_LOAD4(@pathBuffer, pathID * 4u + 3u);
        v_coveragePlacement = coverageData.xy;
        v_coverageCoord = vertexPosition + uintBitsToFloat(coverageData.zw);
#endif
    }
    else
    {
        pos = float4(uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue);
    }

    VARYING_PACK(v_paint);
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_PACK(v_atlasCoord);
#elif !defined(@RENDER_MODE_MSAA)
#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_PACK(v_windingWeight);
#else
    VARYING_PACK(v_coverages);
#endif //@DRAW_INTERIOR_TRIANGLES
    VARYING_PACK(v_pathID);
#endif // !@RENDER_MODE_MSAA

#ifdef @ENABLE_CLIPPING
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_PACK(v_clipID);
#else
    VARYING_PACK(v_clipIDs);
#endif
#endif // @ENABLE_CLIPPING
#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)
    VARYING_PACK(v_clipRect);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_PACK(v_blendMode);
#endif
#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC
    VARYING_PACK(v_coveragePlacement);
    VARYING_PACK(v_coverageCoord);
#endif
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT

FRAG_STORAGE_BUFFER_BLOCK_BEGIN
FRAG_STORAGE_BUFFER_BLOCK_END

// Add a function here for fragments to unpack the paint since we're the ones
// who packed it in the vertex shader.
INLINE half4 find_paint_color(float4 paint,
                              float coverage FRAGMENT_CONTEXT_DECL)
{
    half4 color;
    if (paint.a >= .0) // Is the paint a solid color?
    {
        // The vertex shader will have premultiplied 'paint' (or not) based on
        // GENERATE_UNMULTIPLIED_PAINT_COLORS.
        color = cast_float4_to_half4(paint);
        if (GENERATE_UNMULTIPLIED_PAINT_COLORS)
            color.a *= coverage;
        else
            color *= coverage;
    }
    else if (paint.a > -1.) // Is paint is a gradient (linear or radial)?
    {
        float t =
            paint.b > .0 ? /*linear*/ paint.r : /*radial*/ length(paint.rg);
        t = clamp(t, .0, 1.);
        float span = abs(paint.b);
        float x = span > 1.
                      ? /*entire row*/ (1. - 1. / GRAD_TEXTURE_WIDTH) * t +
                            (.5 / GRAD_TEXTURE_WIDTH)
                      : /*two texels*/ (1. / GRAD_TEXTURE_WIDTH) * t + span;
        float row = -paint.a;
        // Our gradient texture is not mipmapped. Issue a texture-sample that
        // explicitly does not find derivatives for LOD computation.
        color =
            TEXTURE_SAMPLE_LOD(@gradTexture, gradSampler, float2(x, row), .0);
        color.a *= coverage;
        // Gradients are always unmultiplied so we don't lose color data while
        // doing the hardware filter.
        if (GENERATE_UNMULTIPLIED_PAINT_COLORS)
        {
            // naga can't handle "if (!SpecConst)" when transpiling spv to wgsl.
            // Use this if -> else construct instead so we don't have to
            // negate a specialization constant.
        }
        else
        {
            color.rgb *= color.a;
        }
    }
    else // The paint is an image.
    {
        half lod = -paint.a - 2.;
        color = TEXTURE_SAMPLE_DYNAMIC_LOD(@imageTexture,
                                           imageSampler,
                                           paint.rg,
                                           lod);
        half opacity = paint.b * coverage;
        // Images are always premultiplied so the (transparent) background color
        // doesn't bleed into the edges during the hardware filter.
        if (GENERATE_UNMULTIPLIED_PAINT_COLORS)
            color = make_half4(unmultiply_rgb(color), color.a * opacity);
        else
            color *= opacity;
    }
    return color;
}

#if !defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT)

// Add functions here for fragments to unpack and evaluate coverage since we're
// the ones who packed the coverage components in the vertex shader.
INLINE half find_stroke_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)
{
#ifdef @ENABLE_FEATHER
    if (@ENABLE_FEATHER && is_feathered_stroke(coverages))
        return eval_feathered_stroke(coverages TEXTURE_CONTEXT_FORWARD);
    else
#endif // @ENABLE_FEATHER
        return min(coverages.x, coverages.y);
}

INLINE half find_fill_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)
{
#if defined(@ENABLE_FEATHER)
    if (@ENABLE_FEATHER && is_feathered_fill(coverages))
        return eval_feathered_fill(coverages TEXTURE_CONTEXT_FORWARD);
    else
#endif // @ENABLE_FEATHER
        return coverages.x;
}

INLINE half find_frag_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)
{
    if (is_stroke(coverages))
        return find_stroke_coverage(coverages TEXTURE_CONTEXT_FORWARD);
    else // Fill. (Back-face culling handles the sign of coverages.x.)
        return find_fill_coverage(coverages TEXTURE_CONTEXT_FORWARD);
}

INLINE half apply_frag_coverage(half initialCoverage,
                                COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)
{
    if (is_stroke(coverages))
    {
        half fragCoverage =
            find_stroke_coverage(coverages TEXTURE_CONTEXT_FORWARD);
        return max(fragCoverage, initialCoverage);
    }
    else // Fill. (Back-face culling handles the sign of coverages.x.)
    {
        half fragCoverage =
            find_fill_coverage(coverages TEXTURE_CONTEXT_FORWARD);
        return initialCoverage + fragCoverage;
    }
}

#endif // !@DRAW_INTERIOR_TRIANGLES && !@FEATHER_ATLAS_BLIT

#endif // @FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_PATH_SOURCE: &str = PINNED_DRAW_PATH_VERT_SOURCE;
pub const DRAW_PATH_VERT_SOURCE: &str = PINNED_DRAW_PATH_VERT_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_PATH_VERT_SOURCE
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
        block_id: "pp-0393",
        block_start: 9,
        block_end: 22,
        block_depth: 0,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0394",
        block_start: 27,
        block_end: 31,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0395",
        block_start: 33,
        block_end: 44,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0396",
        block_start: 35,
        block_end: 42,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0397",
        block_start: 49,
        block_end: 58,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0398",
        block_start: 52,
        block_end: 56,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0399",
        block_start: 60,
        block_end: 66,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0400",
        block_start: 61,
        block_end: 65,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0401",
        block_start: 67,
        block_end: 69,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0402",
        block_start: 70,
        block_end: 72,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0403",
        block_start: 74,
        block_end: 77,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0404",
        block_start: 81,
        block_end: 399,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-sync-83",
        block_start: 83,
        block_end: 91,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0405",
        block_start: 95,
        block_end: 100,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0406",
        block_start: 104,
        block_end: 113,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0407",
        block_start: 107,
        block_end: 111,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0408",
        block_start: 115,
        block_end: 121,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0409",
        block_start: 116,
        block_end: 120,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0410",
        block_start: 122,
        block_end: 124,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0411",
        block_start: 125,
        block_end: 127,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0412",
        block_start: 128,
        block_end: 131,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0413",
        block_start: 136,
        block_end: 138,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0414",
        block_start: 140,
        block_end: 182,
        block_depth: 1,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0415",
        block_start: 144,
        block_end: 146,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0416",
        block_start: 151,
        block_end: 157,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0417",
        block_start: 167,
        block_end: 173,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0418",
        block_start: 175,
        block_end: 181,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0419",
        block_start: 176,
        block_end: 180,
        block_depth: 3,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0420",
        block_start: 186,
        block_end: 194,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0421",
        block_start: 197,
        block_end: 214,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0422",
        block_start: 208,
        block_end: 212,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0423",
        block_start: 215,
        block_end: 220,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0424",
        block_start: 225,
        block_end: 227,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0425",
        block_start: 229,
        block_end: 249,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0426",
        block_start: 238,
        block_end: 247,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0427",
        block_start: 267,
        block_end: 274,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-sync-334",
        block_start: 334,
        block_end: 342,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0428",
        block_start: 348,
        block_end: 350,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0429",
        block_start: 351,
        block_end: 358,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0430",
        block_start: 369,
        block_end: 378,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0431",
        block_start: 372,
        block_end: 376,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0432",
        block_start: 380,
        block_end: 386,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0433",
        block_start: 381,
        block_end: 385,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0434",
        block_start: 387,
        block_end: 389,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0435",
        block_start: 390,
        block_end: 392,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0436",
        block_start: 393,
        block_end: 396,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0437",
        block_start: 401,
        block_end: 520,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0438",
        block_start: 469,
        block_end: 518,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0439",
        block_start: 475,
        block_end: 479,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0440",
        block_start: 485,
        block_end: 489,
        block_depth: 2,
        branch_count: 1,
    },
];

/// Every branch entry remains literal, in authority/source order. Active
/// paths describe source branches; they are not evaluated as Rust cfg expressions.
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
        block_id: "pp-0393",
        branch_ordinal: 1,
        branch_line: 9,
        directive: "#ifdef @NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        active_branch_path: "(defined(@NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS))",
    },
    ConditionalBranch {
        block_id: "pp-0393",
        branch_ordinal: 2,
        branch_line: 13,
        directive: "#elif defined(@ENABLE_ADVANCED_BLEND)",
        active_branch_path: "(!((defined(@NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS))) && (defined(@ENABLE_ADVANCED_BLEND)))",
    },
    ConditionalBranch {
        block_id: "pp-0393",
        branch_ordinal: 3,
        branch_line: 18,
        directive: "#else",
        active_branch_path: "(!((defined(@NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS)) || (defined(@ENABLE_ADVANCED_BLEND))))",
    },
    ConditionalBranch {
        block_id: "pp-0394",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0394",
        branch_ordinal: 2,
        branch_line: 29,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0395",
        branch_ordinal: 1,
        branch_line: 33,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0396",
        branch_ordinal: 1,
        branch_line: 35,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0396",
        branch_ordinal: 2,
        branch_line: 37,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0397",
        branch_ordinal: 1,
        branch_line: 49,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0397",
        branch_ordinal: 2,
        branch_line: 51,
        directive: "#elif !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(!((defined(@FEATHER_ATLAS_BLIT))) && (!defined(@RENDER_MODE_MSAA)))",
    },
    ConditionalBranch {
        block_id: "pp-0398",
        branch_ordinal: 1,
        branch_line: 52,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(!defined(@RENDER_MODE_MSAA)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0398",
        branch_ordinal: 2,
        branch_line: 54,
        directive: "#else",
        active_branch_path: "((!defined(@RENDER_MODE_MSAA))) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0399",
        branch_ordinal: 1,
        branch_line: 60,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0400",
        branch_ordinal: 1,
        branch_line: 61,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@ENABLE_CLIPPING)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0400",
        branch_ordinal: 2,
        branch_line: 63,
        directive: "#else",
        active_branch_path: "((defined(@ENABLE_CLIPPING))) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0401",
        branch_ordinal: 1,
        branch_line: 67,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0402",
        branch_ordinal: 1,
        branch_line: 70,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0403",
        branch_ordinal: 1,
        branch_line: 74,
        directive: "#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0404",
        branch_ordinal: 1,
        branch_line: 81,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-sync-83",
        branch_ordinal: 1,
        branch_line: 83,
        directive: "#ifdef @EMULATE_DYNAMIC_COLOR_WRITE_DISABLE",
        active_branch_path: "(defined(@VERTEX)) && (defined(@EMULATE_DYNAMIC_COLOR_WRITE_DISABLE))",
    },
    ConditionalBranch {
        block_id: "pp-0405",
        branch_ordinal: 1,
        branch_line: 95,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0405",
        branch_ordinal: 2,
        branch_line: 97,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@DRAW_INTERIOR_TRIANGLES) || defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0406",
        branch_ordinal: 1,
        branch_line: 104,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0406",
        branch_ordinal: 2,
        branch_line: 106,
        directive: "#elif !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@FEATHER_ATLAS_BLIT))) && (!defined(@RENDER_MODE_MSAA)))",
    },
    ConditionalBranch {
        block_id: "pp-0407",
        branch_ordinal: 1,
        branch_line: 107,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0407",
        branch_ordinal: 2,
        branch_line: 109,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA))) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0408",
        branch_ordinal: 1,
        branch_line: 115,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0409",
        branch_ordinal: 1,
        branch_line: 116,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0409",
        branch_ordinal: 2,
        branch_line: 118,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0410",
        branch_ordinal: 1,
        branch_line: 122,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0411",
        branch_ordinal: 1,
        branch_line: 125,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0412",
        branch_ordinal: 1,
        branch_line: 128,
        directive: "#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@VERTEX)) && (defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0413",
        branch_ordinal: 1,
        branch_line: 136,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0414",
        branch_ordinal: 1,
        branch_line: 140,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0415",
        branch_ordinal: 1,
        branch_line: 144,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0414",
        branch_ordinal: 2,
        branch_line: 148,
        directive: "#elif defined(@DRAW_INTERIOR_TRIANGLES)",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@FEATHER_ATLAS_BLIT))) && (defined(@DRAW_INTERIOR_TRIANGLES)))",
    },
    ConditionalBranch {
        block_id: "pp-0416",
        branch_ordinal: 1,
        branch_line: 151,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@DRAW_INTERIOR_TRIANGLES)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0416",
        branch_ordinal: 2,
        branch_line: 154,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (defined(@DRAW_INTERIOR_TRIANGLES))) && (!((defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0414",
        branch_ordinal: 3,
        branch_line: 159,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@FEATHER_ATLAS_BLIT)) || (defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0417",
        branch_ordinal: 1,
        branch_line: 167,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (true) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0417",
        branch_ordinal: 2,
        branch_line: 170,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (true)) && (!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0418",
        branch_ordinal: 1,
        branch_line: 175,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (true) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0419",
        branch_ordinal: 1,
        branch_line: 176,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@VERTEX)) && (true) && (!defined(@RENDER_MODE_MSAA)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0419",
        branch_ordinal: 2,
        branch_line: 178,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (true) && (!defined(@RENDER_MODE_MSAA))) && (!((defined(@ENABLE_FEATHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0420",
        branch_ordinal: 1,
        branch_line: 186,
        directive: "#if !defined(@FEATHER_ATLAS_BLIT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@VERTEX)) && (!defined(@FEATHER_ATLAS_BLIT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0421",
        branch_ordinal: 1,
        branch_line: 197,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0422",
        branch_ordinal: 1,
        branch_line: 208,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0422",
        branch_ordinal: 2,
        branch_line: 210,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0423",
        branch_ordinal: 1,
        branch_line: 215,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0424",
        branch_ordinal: 1,
        branch_line: 225,
        directive: "#ifdef @FRAMEBUFFER_BOTTOM_UP",
        active_branch_path: "(defined(@VERTEX)) && (defined(@FRAMEBUFFER_BOTTOM_UP))",
    },
    ConditionalBranch {
        block_id: "pp-0425",
        branch_ordinal: 1,
        branch_line: 229,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0426",
        branch_ordinal: 1,
        branch_line: 238,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0426",
        branch_ordinal: 2,
        branch_line: 243,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT))) && (!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0427",
        branch_ordinal: 1,
        branch_line: 267,
        directive: "#if defined(@ENABLE_CLIPPING) && !defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING) && !defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-sync-334",
        branch_ordinal: 1,
        branch_line: 334,
        directive: "#ifdef @EMULATE_DYNAMIC_COLOR_WRITE_DISABLE",
        active_branch_path: "(defined(@VERTEX)) && (defined(@EMULATE_DYNAMIC_COLOR_WRITE_DISABLE))",
    },
    ConditionalBranch {
        block_id: "pp-0428",
        branch_ordinal: 1,
        branch_line: 348,
        directive: "#ifdef @POST_INVERT_Y",
        active_branch_path: "(defined(@VERTEX)) && (defined(@POST_INVERT_Y))",
    },
    ConditionalBranch {
        block_id: "pp-0429",
        branch_ordinal: 1,
        branch_line: 351,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0429",
        branch_ordinal: 2,
        branch_line: 353,
        directive: "#elif defined(@RENDER_MODE_CLOCKWISE_ATOMIC)",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@RENDER_MODE_MSAA))) && (defined(@RENDER_MODE_CLOCKWISE_ATOMIC)))",
    },
    ConditionalBranch {
        block_id: "pp-0430",
        branch_ordinal: 1,
        branch_line: 369,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0430",
        branch_ordinal: 2,
        branch_line: 371,
        directive: "#elif !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "((defined(@VERTEX))) && (!((defined(@FEATHER_ATLAS_BLIT))) && (!defined(@RENDER_MODE_MSAA)))",
    },
    ConditionalBranch {
        block_id: "pp-0431",
        branch_ordinal: 1,
        branch_line: 372,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0431",
        branch_ordinal: 2,
        branch_line: 374,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA))) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0432",
        branch_ordinal: 1,
        branch_line: 380,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0433",
        branch_ordinal: 1,
        branch_line: 381,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIPPING)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0433",
        branch_ordinal: 2,
        branch_line: 383,
        directive: "#else",
        active_branch_path: "((defined(@VERTEX)) && (defined(@ENABLE_CLIPPING))) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0434",
        branch_ordinal: 1,
        branch_line: 387,
        directive: "#if defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_CLIP_RECT) && !defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0435",
        branch_ordinal: 1,
        branch_line: 390,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0436",
        branch_ordinal: 1,
        branch_line: 393,
        directive: "#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@VERTEX)) && (defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0437",
        branch_ordinal: 1,
        branch_line: 401,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0438",
        branch_ordinal: 1,
        branch_line: 469,
        directive: "#if !defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0439",
        branch_ordinal: 1,
        branch_line: 475,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0440",
        branch_ordinal: 1,
        branch_line: 485,
        directive: "#if defined(@ENABLE_FEATHER)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT)) && (defined(@ENABLE_FEATHER))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// Direct @-prefixed exports occurring in the pinned source, in source order.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 9,
        source_name: "@NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        generated_name: "ZF",
        generated_header_name: "GLSL_NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 27,
        source_name: "@ENABLE_FEATHER",
        generated_name: "HB",
        generated_header_name: "GLSL_ENABLE_FEATHER",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "EB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "FB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@a_triangleVertex",
        generated_name: "KB",
        generated_header_name: "GLSL_a_triangleVertex",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@a_patchVertexData",
        generated_name: "UB",
        generated_header_name: "GLSL_a_patchVertexData",
    },
    ExportedSymbol {
        source_line: 41,
        source_name: "@a_mirroredVertexData",
        generated_name: "VB",
        generated_header_name: "GLSL_a_mirroredVertexData",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "CB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 53,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "MB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 60,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "I",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "BB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 74,
        source_name: "@RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "SB",
        generated_header_name: "GLSL_RENDER_MODE_CLOCKWISE_ATOMIC",
    },
    ExportedSymbol {
        source_line: 93,
        source_name: "@drawVertexMain",
        generated_name: "GC",
        generated_header_name: "GLSL_drawVertexMain",
    },
    ExportedSymbol {
        source_line: 184,
        source_name: "@paintBuffer",
        generated_name: "AD",
        generated_header_name: "GLSL_paintBuffer",
    },
    ExportedSymbol {
        source_line: 225,
        source_name: "@FRAMEBUFFER_BOTTOM_UP",
        generated_name: "AG",
        generated_header_name: "GLSL_FRAMEBUFFER_BOTTOM_UP",
    },
    ExportedSymbol {
        source_line: 235,
        source_name: "@paintAuxBuffer",
        generated_name: "RB",
        generated_header_name: "GLSL_paintAuxBuffer",
    },
    ExportedSymbol {
        source_line: 348,
        source_name: "@POST_INVERT_Y",
        generated_name: "RC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 355,
        source_name: "@pathBuffer",
        generated_name: "PB",
        generated_header_name: "GLSL_pathBuffer",
    },
    ExportedSymbol {
        source_line: 401,
        source_name: "@FRAGMENT",
        generated_name: "GB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 436,
        source_name: "@gradTexture",
        generated_name: "LD",
        generated_header_name: "GLSL_gradTexture",
    },
    ExportedSymbol {
        source_line: 454,
        source_name: "@imageTexture",
        generated_name: "IC",
        generated_header_name: "GLSL_imageTexture",
    },
    ExportedSymbol {
        source_line: 83,
        source_name: "@EMULATE_DYNAMIC_COLOR_WRITE_DISABLE",
        generated_name: "GD",
        generated_header_name: "GLSL_EMULATE_DYNAMIC_COLOR_WRITE_DISABLE",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 9,
        source_name: "@NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        generated_name: "ZF",
        generated_header_name: "GLSL_NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 27,
        source_name: "@ENABLE_FEATHER",
        generated_name: "HB",
        generated_header_name: "GLSL_ENABLE_FEATHER",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@VERTEX",
        generated_name: "DB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "EB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "FB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "CB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 60,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "I",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "BB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 74,
        source_name: "@RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "SB",
        generated_header_name: "GLSL_RENDER_MODE_CLOCKWISE_ATOMIC",
    },
    ExportedSymbol {
        source_line: 225,
        source_name: "@FRAMEBUFFER_BOTTOM_UP",
        generated_name: "AG",
        generated_header_name: "GLSL_FRAMEBUFFER_BOTTOM_UP",
    },
    ExportedSymbol {
        source_line: 348,
        source_name: "@POST_INVERT_Y",
        generated_name: "RC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 401,
        source_name: "@FRAGMENT",
        generated_name: "GB",
        generated_header_name: "GLSL_FRAGMENT",
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

/// Named function declarations and macro-defined entrypoints remain literal.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 93,
        end_line: 399,
        name: "drawVertexMain",
        signature: "VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 408,
        end_line: 467,
        name: "find_paint_color",
        signature: "INLINE half4 find_paint_color(float4 paint, float coverage FRAGMENT_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 473,
        end_line: 481,
        name: "find_stroke_coverage",
        signature: "INLINE half find_stroke_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 483,
        end_line: 491,
        name: "find_fill_coverage",
        signature: "INLINE half find_fill_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 493,
        end_line: 499,
        name: "find_frag_coverage",
        signature: "INLINE half find_frag_coverage(COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 501,
        end_line: 516,
        name: "apply_frag_coverage",
        signature: "INLINE half apply_frag_coverage(half initialCoverage, COVERAGE_TYPE coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES) && !defined(@FEATHER_ATLAS_BLIT))",
        inline_qualifier: "INLINE",
    },
];
pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings without the leading @.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        generated_name: "ZF",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "AB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_FEATHER",
        generated_name: "HB",
    },
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "EB",
    },
    ExportedIdentifier {
        source_name: "FEATHER_ATLAS_BLIT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "a_triangleVertex",
        generated_name: "KB",
    },
    ExportedIdentifier {
        source_name: "a_patchVertexData",
        generated_name: "UB",
    },
    ExportedIdentifier {
        source_name: "a_mirroredVertexData",
        generated_name: "VB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "OPTIONALLY_FLAT",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIPPING",
        generated_name: "I",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIP_RECT",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "SB",
    },
    ExportedIdentifier {
        source_name: "drawVertexMain",
        generated_name: "GC",
    },
    ExportedIdentifier {
        source_name: "paintBuffer",
        generated_name: "AD",
    },
    ExportedIdentifier {
        source_name: "FRAMEBUFFER_BOTTOM_UP",
        generated_name: "AG",
    },
    ExportedIdentifier {
        source_name: "paintAuxBuffer",
        generated_name: "RB",
    },
    ExportedIdentifier {
        source_name: "POST_INVERT_Y",
        generated_name: "RC",
    },
    ExportedIdentifier {
        source_name: "pathBuffer",
        generated_name: "PB",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "gradTexture",
        generated_name: "LD",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "IC",
    },
    ExportedIdentifier {
        source_name: "EMULATE_DYNAMIC_COLOR_WRITE_DISABLE",
        generated_name: "GD",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
    "ENABLE_ADVANCED_BLEND",
    "ENABLE_FEATHER",
    "VERTEX",
    "DRAW_INTERIOR_TRIANGLES",
    "FEATHER_ATLAS_BLIT",
    "a_triangleVertex",
    "a_patchVertexData",
    "a_mirroredVertexData",
    "RENDER_MODE_MSAA",
    "OPTIONALLY_FLAT",
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "RENDER_MODE_CLOCKWISE_ATOMIC",
    "drawVertexMain",
    "paintBuffer",
    "FRAMEBUFFER_BOTTOM_UP",
    "paintAuxBuffer",
    "POST_INVERT_Y",
    "pathBuffer",
    "FRAGMENT",
    "gradTexture",
    "imageTexture",
];

/// No source spelling maps ambiguously in this owner.
pub const EXPORT_MAPPING_AMBIGUITIES: &[(&str, &str, &str)] = &[];

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

/// draw_path.vert has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source include edges retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[
    ShaderInclude {
        upstream_file: "renderer/src/gpu.cpp",
        include_line: 14,
        directive: "include",
        include_token: "generated/shaders/draw_path.vert.exports.h",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path.vert",
        source_unit: "generic-gpu-implementation",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 13,
        directive: "include",
        include_token: "generated/shaders/draw_path.vert.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path.vert",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/gpu.cpp",
        include_line: 14,
        include_token: "generated/shaders/draw_path.vert.exports.h",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path.vert",
        source_unit: "generic-gpu-implementation",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 13,
        include_token: "generated/shaders/draw_path.vert.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path.vert",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
