/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/init_clockwise_atomic_workaround.frag.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/init_clockwise_atomic_workaround.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "092a1f498d0f6ff336edecdc73f96c46c1b3a51d494249839fe9934203dd53a3";
pub const PINNED_SOURCE_LINE_COUNT: usize = 33;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1082;

/// Exact pinned upstream source bytes.
pub const PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

// This shader implements a seeming workaround for Qualcomm. Basically, input
// attachment reads of the clip and color buffers don't work unless we first
// draw these buffers into themselves between borrowed coverage and the main
// subpass. This draw is issued with a scissor that only allows one pixel
// through, so the fill rate impact should be negligible.
#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
PLS_BLOCK_END

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    // Draw the clip buffer onto itself.
    PLS_STORE4F(clipBuffer, make_half4(PLS_LOAD4F(clipBuffer).r, .0, .0, 1.));
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    // Draw the color buffer onto itself.
    EMIT_CLOCKWISE_ATOMIC_PLS(PLS_LOAD4F(colorBuffer));
#else
    // This render pass doesn't read the color buffer. Emit 0 (since srcOver
    // blend is enabled) to leave the color buffer unaffected.
    EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(.0));
#endif
}

#endif
"###;

/// Stable source aliases.
pub const PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_SOURCE: &str =
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE;
pub const INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE: &str =
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE
}
