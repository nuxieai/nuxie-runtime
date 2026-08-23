/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_raster_order_path.frag.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, includes, exports, functions, and source
 * metadata as literal source-shaped data. It does not compile, evaluate,
 * simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_raster_order_path.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "f4b4f70790ff16aa39870f0fcd848afa69dc52bf4b45fbbed3c1dab645eeb67f";
pub const PINNED_SOURCE_LINE_COUNT: usize = 234;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 8245;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_raster_order_path_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_RASTER_ORDER_PATH_FRAG_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
PLS_DECL4F(SCRATCH_COLOR_PLANE_IDX, scratchColorBuffer);
PLS_DECLUI(COVERAGE_PLANE_IDX, coverageCountBuffer);
PLS_BLOCK_END

PLS_MAIN(@drawFragmentMain)
{
    VARYING_UNPACK(v_paint, float4);

#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_windingWeight, half);
#else
    VARYING_UNPACK(v_coverages, COVERAGE_TYPE);
#endif //@DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_pathID, half);

#ifdef @ENABLE_CLIPPING
    VARYING_UNPACK(v_clipIDs, half2);
#endif
#ifdef @ENABLE_CLIP_RECT
    VARYING_UNPACK(v_clipRect, float4);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_blendMode, half);
#endif

#if !defined(@DRAW_INTERIOR_TRIANGLES)
    // Interior triangles don't overlap, so don't need raster ordering.
    PLS_INTERLOCK_BEGIN;
#endif

    half2 coverageData = unpackHalf2x16(PLS_LOADUI(coverageCountBuffer));
    half coverageBufferID = coverageData.g;
    half coverageCount =
        coverageBufferID == v_pathID ? coverageData.r : make_half(.0);

#ifdef @DRAW_INTERIOR_TRIANGLES
    coverageCount += v_windingWeight;
    // Preserve the coverage buffer even though we don't use it, so it doesn't
    // get overwritten in a way that would corrupt a future draw (e.g., by
    // accidentally writing the next path's id with a bogus coverage.)
    PLS_PRESERVE_UI(coverageCountBuffer);
#else
    coverageCount =
        apply_frag_coverage(coverageCount, v_coverages TEXTURE_CONTEXT_FORWARD);
    // Save the updated coverage.
    PLS_STOREUI(coverageCountBuffer,
                packHalf2x16(make_half2(coverageCount, v_pathID)));
#endif // !@DRAW_INTERIOR_TRIANGLES

    // Convert coverageCount to coverage.
    half coverage;
#ifdef @CLOCKWISE_FILL
    if (@CLOCKWISE_FILL)
    {
        coverage =
            safe_clamp_for_mali(coverageCount, make_half(.0), make_half(1.));
    }
    else
#endif // CLOCKWISE_FILL
    {
        coverage = abs(coverageCount);
#ifdef @ENABLE_EVEN_ODD
        if (@ENABLE_EVEN_ODD && v_pathID < .0 /*even-odd*/)
        {
            coverage = 1. - make_half(abs(fract(coverage * .5) * 2. + -1.));
        }
#endif
        // This also caps stroke coverage, which can be >1.
        coverage = min(coverage, make_half(1.));
    }

#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING && v_clipIDs.x < .0) // Update the clip buffer.
    {
        half clipID = -v_clipIDs.x;
#ifdef @ENABLE_NESTED_CLIPPING
        if (@ENABLE_NESTED_CLIPPING)
        {
            half outerClipID = v_clipIDs.y;
            if (outerClipID != .0)
            {
                // This is a nested clip. Intersect coverage with the enclosing
                // clip (outerClipID).
                half2 clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
                half clipContentID = clipData.g;
                half outerClipCoverage;
                if (clipContentID != clipID)
                {
                    // First hit: either clipBuffer contains outerClipCoverage,
                    // or this pixel is not inside the outer clip and
                    // outerClipCoverage is zero.
                    outerClipCoverage =
                        clipContentID == outerClipID ? clipData.r : .0;
#ifndef @DRAW_INTERIOR_TRIANGLES
                    // Stash outerClipCoverage before overwriting clipBuffer, in
                    // case we hit this pixel again and need it. (Not necessary
                    // when drawing interior triangles because they always go
                    // last and don't overlap.)
                    PLS_STORE4F(scratchColorBuffer,
                                make_half4(outerClipCoverage, .0, .0, .0));
#endif
                }
                else
                {
                    // Subsequent hit: outerClipCoverage is stashed in
                    // scratchColorBuffer.
                    outerClipCoverage = PLS_LOAD4F(scratchColorBuffer).r;
#ifndef @DRAW_INTERIOR_TRIANGLES
                    // Since interior triangles are always last, there's no need
                    // to preserve this value.
                    PLS_PRESERVE_4F(scratchColorBuffer);
#endif
                }
                coverage = min(coverage, outerClipCoverage);
            }
        }
#endif // @ENABLE_NESTED_CLIPPING
        PLS_STOREUI(clipBuffer, packHalf2x16(make_half2(coverage, clipID)));
        PLS_PRESERVE_4F(colorBuffer);
    }
    else // Render to the main framebuffer.
#endif   // @ENABLE_CLIPPING
    {
#ifdef @ENABLE_CLIPPING
        if (@ENABLE_CLIPPING)
        {
            // Apply the clip.
            half clipID = v_clipIDs.x;
            if (clipID != .0)
            {
                // Clip IDs are not necessarily drawn in monotonically
                // increasing order, so always check exact equality of the
                // clipID.
                half2 clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
                half clipContentID = clipData.g;
                coverage = (clipContentID == clipID) ? min(clipData.r, coverage)
                                                     : make_half(.0);
            }
        }
#endif
#ifdef @ENABLE_CLIP_RECT
        if (@ENABLE_CLIP_RECT)
        {
            half clipRectCoverage =
                min_component(cast_float4_to_half4(v_clipRect));
            coverage = clamp(clipRectCoverage, make_half(.0), coverage);
        }
#endif // ENABLE_CLIP_RECT

        half4 color =
            find_paint_color(v_paint, coverage FRAGMENT_CONTEXT_UNPACK);

        half4 dstColorPremul;
        if (coverageBufferID != v_pathID)
        {
            // This is the first fragment from pathID to touch this pixel.
            dstColorPremul = PLS_LOAD4F(colorBuffer);
#ifndef @DRAW_INTERIOR_TRIANGLES
            // We don't need to store coverage when drawing interior triangles
            // because they always go last and don't overlap, so every fragment
            // is the final one in the path.
            PLS_STORE4F(scratchColorBuffer, dstColorPremul);
#endif
        }
        else
        {
            dstColorPremul = PLS_LOAD4F(scratchColorBuffer);
#ifndef @DRAW_INTERIOR_TRIANGLES
            // Since interior triangles are always last, there's no need to
            // preserve this value.
            PLS_PRESERVE_4F(scratchColorBuffer);
#endif
        }

        // Blend with the framebuffer color.
#ifdef @ENABLE_ADVANCED_BLEND
        if (@ENABLE_ADVANCED_BLEND)
        {
            // GENERATE_PREMULTIPLIED_PAINT_COLORS is false in this case because
            // advanced blend needs unmultiplied colors.
            if (v_blendMode != cast_uint_to_half(BLEND_SRC_OVER))
            {
                color.rgb =
                    advanced_color_blend(color.rgb,
                                         dstColorPremul,
                                         cast_half_to_ushort(v_blendMode));
            }
            // Premultiply alpha now.
            color.rgb *= color.a;
        }
#endif

        // Certain platforms give us less control of the format of what we are
        // rendering too. Specifically, we are auto converted from linear ->
        // sRGB on render target writes in unreal. In those cases we made need
        // to end up in linear color space
#ifdef @NEEDS_GAMMA_CORRECTION
        if (@NEEDS_GAMMA_CORRECTION)
        {
            color = gamma_to_linear(color);
        }
#endif

        // Save paint alpha before destructively updating it with the dstColor.
        half paintAlpha = color.a;
        color += dstColorPremul * (1. - paintAlpha);
        color.rgb = add_dither_if_alpha_nonzero(color.rgb,
                                                paintAlpha,
                                                _fragCoord.xy,
                                                uniforms.ditherScale,
                                                uniforms.ditherBias);

        PLS_STORE4F(colorBuffer, color);
        PLS_PRESERVE_UI(clipBuffer);
    }

#if !defined(@DRAW_INTERIOR_TRIANGLES)
    // Interior triangles don't overlap, so don't need raster ordering.
    PLS_INTERLOCK_END;
#endif

    EMIT_PLS;
}

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_RASTER_ORDER_PATH_SOURCE: &str = PINNED_DRAW_RASTER_ORDER_PATH_FRAG_SOURCE;
pub const DRAW_RASTER_ORDER_PATH_FRAG_SOURCE: &str = PINNED_DRAW_RASTER_ORDER_PATH_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_RASTER_ORDER_PATH_FRAG_SOURCE
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
        block_id: "pp-0471",
        block_start: 5,
        block_end: 234,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0472",
        block_start: 18,
        block_end: 22,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0473",
        block_start: 25,
        block_end: 27,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0474",
        block_start: 28,
        block_end: 30,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0475",
        block_start: 31,
        block_end: 33,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0476",
        block_start: 35,
        block_end: 38,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0477",
        block_start: 45,
        block_end: 57,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0478",
        block_start: 61,
        block_end: 68,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0479",
        block_start: 71,
        block_end: 76,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0480",
        block_start: 81,
        block_end: 131,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0481",
        block_start: 85,
        block_end: 126,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0482",
        block_start: 103,
        block_end: 110,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0483",
        block_start: 117,
        block_end: 121,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0484",
        block_start: 133,
        block_end: 149,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0485",
        block_start: 150,
        block_end: 157,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0486",
        block_start: 167,
        block_end: 172,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0487",
        block_start: 177,
        block_end: 181,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0488",
        block_start: 185,
        block_end: 200,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0489",
        block_start: 206,
        block_end: 211,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0490",
        block_start: 226,
        block_end: 229,
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
        block_id: "pp-0471",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0472",
        branch_ordinal: 1,
        branch_line: 18,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0472",
        branch_ordinal: 2,
        branch_line: 20,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0473",
        branch_ordinal: 1,
        branch_line: 25,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0474",
        branch_ordinal: 1,
        branch_line: 28,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0475",
        branch_ordinal: 1,
        branch_line: 31,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0476",
        branch_ordinal: 1,
        branch_line: 35,
        directive: "#if !defined(@DRAW_INTERIOR_TRIANGLES)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0477",
        branch_ordinal: 1,
        branch_line: 45,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0477",
        branch_ordinal: 2,
        branch_line: 51,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0478",
        branch_ordinal: 1,
        branch_line: 61,
        directive: "#ifdef @CLOCKWISE_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@CLOCKWISE_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0479",
        branch_ordinal: 1,
        branch_line: 71,
        directive: "#ifdef @ENABLE_EVEN_ODD",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_EVEN_ODD))",
    },
    ConditionalBranch {
        block_id: "pp-0480",
        branch_ordinal: 1,
        branch_line: 81,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0481",
        branch_ordinal: 1,
        branch_line: 85,
        directive: "#ifdef @ENABLE_NESTED_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (defined(@ENABLE_NESTED_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0482",
        branch_ordinal: 1,
        branch_line: 103,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (defined(@ENABLE_NESTED_CLIPPING)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0483",
        branch_ordinal: 1,
        branch_line: 117,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (defined(@ENABLE_NESTED_CLIPPING)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0484",
        branch_ordinal: 1,
        branch_line: 133,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0485",
        branch_ordinal: 1,
        branch_line: 150,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0486",
        branch_ordinal: 1,
        branch_line: 167,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0487",
        branch_ordinal: 1,
        branch_line: 177,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0488",
        branch_ordinal: 1,
        branch_line: 185,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0489",
        branch_ordinal: 1,
        branch_line: 206,
        directive: "#ifdef @NEEDS_GAMMA_CORRECTION",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
    },
    ConditionalBranch {
        block_id: "pp-0490",
        branch_ordinal: 1,
        branch_line: 226,
        directive: "#if !defined(@DRAW_INTERIOR_TRIANGLES)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The ten @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 14,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 25,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 28,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 61,
        source_name: "@CLOCKWISE_FILL",
        generated_name: "UD",
        generated_header_name: "GLSL_CLOCKWISE_FILL",
    },
    ExportedSymbol {
        source_line: 71,
        source_name: "@ENABLE_EVEN_ODD",
        generated_name: "PC",
        generated_header_name: "GLSL_ENABLE_EVEN_ODD",
    },
    ExportedSymbol {
        source_line: 85,
        source_name: "@ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
        generated_header_name: "GLSL_ENABLE_NESTED_CLIPPING",
    },
    ExportedSymbol {
        source_line: 206,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 25,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 28,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 61,
        source_name: "@CLOCKWISE_FILL",
        generated_name: "UD",
        generated_header_name: "GLSL_CLOCKWISE_FILL",
    },
    ExportedSymbol {
        source_line: 71,
        source_name: "@ENABLE_EVEN_ODD",
        generated_name: "PC",
        generated_header_name: "GLSL_ENABLE_EVEN_ODD",
    },
    ExportedSymbol {
        source_line: 85,
        source_name: "@ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
        generated_header_name: "GLSL_ENABLE_NESTED_CLIPPING",
    },
    ExportedSymbol {
        source_line: 206,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
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

/// The macro-defined fragment entrypoint is retained as a source spelling and
/// range. Its body remains in the pinned fragment source rather than becoming
/// an executable Rust function.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[ShaderFunction {
    source_line: 14,
    end_line: 232,
    name: "drawFragmentMain",
    signature: "PLS_MAIN(@drawFragmentMain)",
    guard_path: "(defined(@FRAGMENT))",
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
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
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
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "CLOCKWISE_FILL",
        generated_name: "UD",
    },
    ExportedIdentifier {
        source_name: "ENABLE_EVEN_ODD",
        generated_name: "PC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
    },
    ExportedIdentifier {
        source_name: "NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "drawFragmentMain",
    "DRAW_INTERIOR_TRIANGLES",
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "ENABLE_ADVANCED_BLEND",
    "CLOCKWISE_FILL",
    "ENABLE_EVEN_ODD",
    "ENABLE_NESTED_CLIPPING",
    "NEEDS_GAMMA_CORRECTION",
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

/// draw_raster_order_path.frag has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source include edge retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[ShaderInclude {
    upstream_file: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 14,
    directive: "include",
    include_token: "generated/shaders/draw_raster_order_path.frag.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_raster_order_path.frag",
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
    include_line: 14,
    include_token: "generated/shaders/draw_raster_order_path.frag.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_raster_order_path.frag",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    translation_disposition: "preserve-source-dependency",
}];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
