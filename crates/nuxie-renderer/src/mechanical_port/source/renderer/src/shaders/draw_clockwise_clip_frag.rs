/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_clockwise_clip.frag.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_clip.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "85c10fcfd0c66df4796ade0f0c3b33f1d633138cfd2548462e545e804e18307e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 101;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3061;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F_RGB10_A2(SCRATCH_COLOR_PLANE_IDX, scratchColorBuffer);
#endif
PLS_DECLUI(COVERAGE_PLANE_IDX, coverageBuffer);
PLS_BLOCK_END

PLS_MAIN(@drawFragmentMain)
{
    VARYING_UNPACK(v_clipIDs, half2);
    half clipID = -v_clipIDs.x;

#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_windingWeight, half);
    half fragCoverage = v_windingWeight;
#else
    VARYING_UNPACK(v_coverages, COVERAGE_TYPE);
    half fragCoverage = v_coverages.x;
#endif //@DRAW_INTERIOR_TRIANGLES

    PLS_INTERLOCK_BEGIN;

    half2 clipData;
    half clipBufferID, clipCoverage;
#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
    if (@BORROWED_COVERAGE_PASS)
    {
        // Interior triangles with borrowed coverage are always the first
        // fragment of the path at their pixel, so we don't need to check the
        // current coverage value.
        clipCoverage = fragCoverage;
    }
    else
#endif
    {
        clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
        clipBufferID = clipData.g;
        half initialCoverage =
            clipBufferID == clipID ? clipData.r : make_half(.0);
        clipCoverage = initialCoverage + fragCoverage;
    }

#ifdef @ENABLE_NESTED_CLIPPING
    half outerClipID = v_clipIDs.y;
    if (@ENABLE_NESTED_CLIPPING && outerClipID != .0)
    {
        half outerClipCoverage = .0;
#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
        if (@BORROWED_COVERAGE_PASS)
        {
            // Interior triangles with borrowed coverage did not load the clip
            // buffer already, so do that now.
            clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
            clipBufferID = clipData.g;
        }
#endif
        if (clipBufferID != clipID)
        {
            outerClipCoverage = clipBufferID == outerClipID ? clipData.r : .0;
            // The coverage buffer is a free resource when rendering clip
            // because we don't use it to track coverage. Use it instead to
            // temporarily save the outer clip for nested clips. But make sure
            // to write an invalid pathID so we don't corrupt any future paths
            // that will be drawn.
            PLS_STOREUI(
                coverageBuffer,
                packHalf2x16(make_half2(outerClipCoverage, INVALID_PATH_ID)));
        }
        else
        {
            outerClipCoverage = unpackHalf2x16(PLS_LOADUI(coverageBuffer)).r;
            PLS_PRESERVE_UI(coverageBuffer);
        }
        clipCoverage = min(clipCoverage, outerClipCoverage);
    }
    else
#endif
    {
        PLS_PRESERVE_UI(coverageBuffer);
    }

    PLS_STOREUI(clipBuffer, packHalf2x16(make_half2(clipCoverage, clipID)));
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    PLS_PRESERVE_4F(colorBuffer);
#endif
    PLS_INTERLOCK_END;

    EMIT_PLS;
}

#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_DRAW_CLOCKWISE_CLIP_SOURCE: &str = PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_CLIP_FRAG_SOURCE: &str = PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE
}
