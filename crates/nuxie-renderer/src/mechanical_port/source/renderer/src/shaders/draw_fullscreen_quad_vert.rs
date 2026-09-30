/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_fullscreen_quad.vert.
 *
 * Upstream source revision: c14cb2510071bd4cfa08d52ba5cd44d98c362237
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c14cb2510071bd4cfa08d52ba5cd44d98c362237";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_fullscreen_quad.vert";
pub const PINNED_SOURCE_SHA256: &str =
    "44e538fa18b814c1643e046b3d3c2a0e2e54bb74521667f989bdbe7a0978253e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 21;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 492;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
// No attributes: the quad comes from the vertex index.
ATTR_BLOCK_END

VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexIdx, _instanceIdx)
{
    // Fill the entire screen. The caller will use a scissor test to control the
    // bounds being drawn.
    float4 pos;
    pos.x = (_vertexIdx & 1) == 0 ? -1. : 1.;
    pos.y = (_vertexIdx & 2) == 0 ? -1. : 1.;
    pos.z = 0.;
    pos.w = 1.;
    EMIT_VERTEX(pos);
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
