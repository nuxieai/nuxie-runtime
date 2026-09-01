/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_fullscreen_quad.vert.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_fullscreen_quad.vert";
pub const PINNED_SOURCE_SHA256: &str =
    "6a9842803e8472ab8f756a191c6a6d60a7c28db5587ee22c5e9bddb000c49cc2";
pub const PINNED_SOURCE_LINE_COUNT: usize = 15;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 335;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @VERTEX
void main()
{
    // Fill the entire screen. The caller will use a scissor test to control the
    // bounds being drawn.
    gl_Position.x = (gl_VertexID & 1) == 0 ? -1. : 1.;
    gl_Position.y = (gl_VertexID & 2) == 0 ? -1. : 1.;
    gl_Position.z = 0.;
    gl_Position.w = 1.;
}
#endif
"###;

/// Stable source aliases.
pub const PINNED_DRAW_FULLSCREEN_QUAD_SOURCE: &str = PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE;
pub const DRAW_FULLSCREEN_QUAD_VERT_SOURCE: &str = PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE
}
