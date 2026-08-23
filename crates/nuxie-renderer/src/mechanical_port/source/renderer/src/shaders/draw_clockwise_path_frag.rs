/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_clockwise_path.frag.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_path.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "f033a35f69ad4d2802fc9afa21f0ca0e06f73bb516d9cd9099a378a553eaa377";
pub const PINNED_SOURCE_LINE_COUNT: usize = 251;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 9698;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_clockwise_path_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_CLOCKWISE_PATH_FRAG_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F_RGB10_A2(SCRATCH_COLOR_PLANE_IDX, blendColorBuffer);
#endif
PLS_DECLUI(COVERAGE_PLANE_IDX, coverageBuffer);
PLS_BLOCK_END

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_FRAG_COLOR_MAIN(@drawFragmentMain)
#else
PLS_MAIN(@drawFragmentMain)
#endif
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

    // Calculate fragment coverage before entering the interlock.
    half fragCoverage =
#ifdef @DRAW_INTERIOR_TRIANGLES
        v_windingWeight;
#else
        find_frag_coverage(v_coverages);
#endif

    half4 paintColor;
    half maxCoverage;
#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
    if (!@BORROWED_COVERAGE_PASS)
#endif
    {
        // Calculate the paint color before entering the interlock.
        paintColor = find_paint_color(v_paint, 1. FRAGMENT_CONTEXT_UNPACK);

        maxCoverage = 1.;
#ifdef @ENABLE_CLIP_RECT
        // Calculate the clip rect before entering the interlock.
        if (@ENABLE_CLIP_RECT)
        {
            half clipRectMin = min_component(cast_float4_to_half4(v_clipRect));
            maxCoverage = min(clipRectMin, maxCoverage);
        }
#endif
    }

    PLS_INTERLOCK_BEGIN;

#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
    if (@BORROWED_COVERAGE_PASS)
    {
        // Interior triangles with borrowed coverage never write color. They're
        // also always the first fragment of the path at their pixel, so just
        // blindly write coverage and move on.
        PLS_STOREUI(coverageBuffer,
                    packHalf2x16(make_half2(fragCoverage, v_pathID)));
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        PLS_PRESERVE_4F(colorBuffer);
#endif
    }
    else
#endif // !@DRAW_INTERIOR_TRIANGLES && @BORROWED_COVERAGE_PASS
    {
        half2 coverageData = unpackHalf2x16(PLS_LOADUI(coverageBuffer));
        half coverageBufferID = coverageData.g;
        half initialCoverage =
            coverageBufferID == v_pathID ? coverageData.r : make_half(.0);
        half finalCoverage =
#ifndef @DRAW_INTERIOR_TRIANGLES
            is_stroke(v_coverages) ? max(initialCoverage, fragCoverage) :
#endif
                                   initialCoverage + fragCoverage;

#ifdef @ENABLE_CLIPPING
        if (@ENABLE_CLIPPING && v_clipIDs.x != .0)
        {
            half2 clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
            half clipBufferID = clipData.g;
            half clip =
                clipBufferID == v_clipIDs.x ? clipData.r : make_half(.0);
            maxCoverage = min(clip, maxCoverage);
        }
#endif

        // Find the coverage delta (c0 -> c1) that this fragment will apply,
        // where c0 is the coverage with which "paintColor" is already blended
        // into the framebuffer, and c1 is the total coverage with which we
        // *want* it to be blended after this fragment. The geometry is ordered
        // such that if c1 > 0, c1 >= c0 as well.
        maxCoverage = max(maxCoverage, .0);
        half c0 = safe_clamp_for_mali(initialCoverage, .0, maxCoverage);
        half c1 = safe_clamp_for_mali(finalCoverage, .0, maxCoverage);

#ifdef @ENABLE_DITHER
        half dither;
        if (@ENABLE_DITHER)
        {
            dither = get_dither(_fragCoord.xy,
                                uniforms.ditherScale,
                                uniforms.ditherBias);
        }
#endif

#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        half4 dstColorPremul = PLS_LOAD4F(colorBuffer);
#ifdef @ENABLE_ADVANCED_BLEND
        if (@ENABLE_ADVANCED_BLEND)
        {
            // Don't bother with advanced blend until coverage becomes > 0. This
            // way, cutout regions don't pay the cost of advanced blend.
            if (v_blendMode != cast_uint_to_half(BLEND_SRC_OVER) && c1 != .0)
            {
                if (c0 == .0)
                {
                    // This is the first fragment of the path to apply the blend
                    // mode, meaning, the current dstColor is the correct value
                    // we need to pass to advanced_color_blend(). Calculate the
                    // color-blended paint color before coverage. Coverage can
                    // be applied later as a simple src-over operation.
                    paintColor.rgb =
                        advanced_color_blend(paintColor.rgb,
                                             dstColorPremul,
                                             cast_half_to_ushort(v_blendMode));
                    // Normally we need to save the color-blended paint color
                    // for any future fragments at this same pixel because once
                    // we blend this fragment, the original dstColor will be
                    // destroyed. However, there are 2 exceptions:
                    //
                    // * No need to save the color-blended paint color if we're
                    // a
                    //   (clockwise) interior triangle, because those are always
                    //   guaranteed to be the final fragment of the path at a
                    //   given pixel.
                    //
                    // * No need to save the color-blended paint color once
                    // coverage
                    //   is maxed out, out because once it's maxed, any future
                    //   fragments will effectively be no-ops (since c1 - c0 ==
                    //   0).
#ifndef @DRAW_INTERIOR_TRIANGLES
                    if (c1 < maxCoverage)
                    {
                        half3 blendRGBToSave = paintColor.rgb;
#ifdef @ENABLE_DITHER
                        if (@ENABLE_DITHER)
                        {
                            blendRGBToSave +=
                                dither * uniforms.ditherConversionToRGB10;
                        }
#endif
                        PLS_STORE4F(blendColorBuffer,
                                    make_half4(blendRGBToSave, 0.0));
                    }
#endif
                }
                else
                {
                    // This is not the first fragment of the path to apply the
                    // blend mode, meaning, the current dstColor is no longer
                    // the correct value we need to pass to
                    // advanced_color_blend(). Instead, the first fragment saved
                    // its result of advanced_color_blend() to the
                    // blendColorBuffer, which we can pull back up and use to
                    // apply our fragment's coverage contribution.
                    paintColor.rgb = PLS_LOAD4F(blendColorBuffer).rgb;
                    PLS_PRESERVE_4F(blendColorBuffer);
                }
            }
            // GENERATE_PREMULTIPLIED_PAINT_COLORS is false when
            // @ENABLE_ADVANCED_BLEND is defined because advanced blend needs
            // unmultiplied colors. Premultiply alpha now.
            paintColor.rgb *= paintColor.a;
        }
#endif // @ENABLE_ADVANCED_BLEND
#endif // @FIXED_FUNCTION_COLOR_OUTPUT

        // Emit a paint color whose post-src-over-blend result is algebraically
        // equivalent to applying the c0 -> c1 coverage delta.
        paintColor *= incremental_clockwise_coverage(c0, c1, paintColor.a);
#ifdef @ENABLE_DITHER
        paintColor.rgb =
            add_dither_if_alpha_nonzero(paintColor.rgb, paintColor.a, dither);
#endif
#ifndef @DRAW_INTERIOR_TRIANGLES
        // Update the coverage buffer with our final value if we aren't an
        // interior triangle, because another fragment from this same path might
        // come along at this pixel. The only exception is if we're src-over and
        // fully opaque, because at that point the next fragment will
        // effectively be a no-op (since any color blended with itself is a
        // no-op).
#ifdef @ENABLE_ADVANCED_BLEND
        // We can't skip the write for advanced blends either because they use
        // the ID in the coverage buffer to detect the first fragment of the
        // path for dst reads.
#define COVERAGE_UPDATE_OPTIONAL                                               \
    (!@ENABLE_ADVANCED_BLEND ||                                                \
     v_blendMode == cast_uint_to_half(BLEND_SRC_OVER)) &&                      \
        paintColor.a >= 1.
#else
#define COVERAGE_UPDATE_OPTIONAL paintColor.a >= 1.
#endif
        PLS_STOREUI_OPTIONAL_IF(
            COVERAGE_UPDATE_OPTIONAL,
            coverageBuffer,
            packHalf2x16(make_half2(finalCoverage, v_pathID)));
#else // -> @DRAW_INTERIOR_TRIANGLES
        PLS_PRESERVE_UI(coverageBuffer);
#endif

#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        PLS_STORE4F_OPTIONAL_IF(paintColor.a == .0,
                                colorBuffer,
                                dstColorPremul * (1. - paintColor.a) +
                                    paintColor);
#endif
    }

    PLS_PRESERVE_UI(clipBuffer);
    PLS_INTERLOCK_END;

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    _fragColor = paintColor;
    EMIT_PLS_AND_FRAG_COLOR
#else
    EMIT_PLS;
#endif
}

#endif // @FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_CLOCKWISE_PATH_SOURCE: &str = PINNED_DRAW_CLOCKWISE_PATH_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_PATH_FRAG_SOURCE: &str = PINNED_DRAW_CLOCKWISE_PATH_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_PATH_FRAG_SOURCE
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
        block_id: "pp-0302",
        block_start: 5,
        block_end: 251,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0303",
        block_start: 8,
        block_end: 10,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0304",
        block_start: 12,
        block_end: 14,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0305",
        block_start: 18,
        block_end: 22,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0306",
        block_start: 25,
        block_end: 29,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0307",
        block_start: 31,
        block_end: 33,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0308",
        block_start: 34,
        block_end: 36,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0309",
        block_start: 37,
        block_end: 39,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0310",
        block_start: 43,
        block_end: 47,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0311",
        block_start: 51,
        block_end: 53,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0312",
        block_start: 59,
        block_end: 66,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0313",
        block_start: 71,
        block_end: 84,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0314",
        block_start: 79,
        block_end: 81,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0315",
        block_start: 91,
        block_end: 93,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0316",
        block_start: 96,
        block_end: 105,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0317",
        block_start: 116,
        block_end: 124,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0318",
        block_start: 126,
        block_end: 197,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0319",
        block_start: 128,
        block_end: 196,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0320",
        block_start: 162,
        block_end: 176,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0321",
        block_start: 166,
        block_end: 172,
        block_depth: 4,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0322",
        block_start: 202,
        block_end: 205,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0323",
        block_start: 206,
        block_end: 230,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0324",
        block_start: 213,
        block_end: 223,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0325",
        block_start: 232,
        block_end: 237,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0326",
        block_start: 243,
        block_end: 248,
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
        block_id: "pp-0302",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0303",
        branch_ordinal: 1,
        branch_line: 8,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0304",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0305",
        branch_ordinal: 1,
        branch_line: 18,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0305",
        branch_ordinal: 2,
        branch_line: 20,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0306",
        branch_ordinal: 1,
        branch_line: 25,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0306",
        branch_ordinal: 2,
        branch_line: 27,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0307",
        branch_ordinal: 1,
        branch_line: 31,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0308",
        branch_ordinal: 1,
        branch_line: 34,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0309",
        branch_ordinal: 1,
        branch_line: 37,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0310",
        branch_ordinal: 1,
        branch_line: 43,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0310",
        branch_ordinal: 2,
        branch_line: 45,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0311",
        branch_ordinal: 1,
        branch_line: 51,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS))",
    },
    ConditionalBranch {
        block_id: "pp-0312",
        branch_ordinal: 1,
        branch_line: 59,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0313",
        branch_ordinal: 1,
        branch_line: 71,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS))",
    },
    ConditionalBranch {
        block_id: "pp-0314",
        branch_ordinal: 1,
        branch_line: 79,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0315",
        branch_ordinal: 1,
        branch_line: 91,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0316",
        branch_ordinal: 1,
        branch_line: 96,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0317",
        branch_ordinal: 1,
        branch_line: 116,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0318",
        branch_ordinal: 1,
        branch_line: 126,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0319",
        branch_ordinal: 1,
        branch_line: 128,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0320",
        branch_ordinal: 1,
        branch_line: 162,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0321",
        branch_ordinal: 1,
        branch_line: 166,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (!defined(@DRAW_INTERIOR_TRIANGLES)) && (defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0322",
        branch_ordinal: 1,
        branch_line: 202,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0323",
        branch_ordinal: 1,
        branch_line: 206,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0323",
        branch_ordinal: 2,
        branch_line: 228,
        directive: "#else // -> @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!((!defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0324",
        branch_ordinal: 1,
        branch_line: 213,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0324",
        branch_ordinal: 2,
        branch_line: 221,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES)) && (!((defined(@ENABLE_ADVANCED_BLEND))))",
    },
    ConditionalBranch {
        block_id: "pp-0325",
        branch_ordinal: 1,
        branch_line: 232,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0326",
        branch_ordinal: 1,
        branch_line: 243,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0326",
        branch_ordinal: 2,
        branch_line: 246,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The nine @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 8,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 19,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 25,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 37,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@BORROWED_COVERAGE_PASS",
        generated_name: "WB",
        generated_header_name: "GLSL_BORROWED_COVERAGE_PASS",
    },
    ExportedSymbol {
        source_line: 116,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
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
        source_line: 8,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 25,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 37,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@BORROWED_COVERAGE_PASS",
        generated_name: "WB",
        generated_header_name: "GLSL_BORROWED_COVERAGE_PASS",
    },
    ExportedSymbol {
        source_line: 116,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
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

/// Both conditional macro-defined fragment entrypoint declarations are
/// retained as source spellings and ranges. Their shared body remains in the
/// pinned source rather than becoming an executable Rust function.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 19,
        end_line: 249,
        name: "drawFragmentMain",
        signature: "PLS_FRAG_COLOR_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 21,
        end_line: 249,
        name: "drawFragmentMain",
        signature: "PLS_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
        inline_qualifier: "",
    },
];

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
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
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
        source_name: "BORROWED_COVERAGE_PASS",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_DITHER",
        generated_name: "JB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "drawFragmentMain",
    "DRAW_INTERIOR_TRIANGLES",
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "ENABLE_ADVANCED_BLEND",
    "BORROWED_COVERAGE_PASS",
    "ENABLE_DITHER",
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

/// draw_clockwise_path.frag has no direct #include/#import directive.
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

/// This shader source has no direct #include/#import directive.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
