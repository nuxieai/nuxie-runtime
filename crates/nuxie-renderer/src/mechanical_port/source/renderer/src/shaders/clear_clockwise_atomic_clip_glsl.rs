/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/clear_clockwise_atomic_clip.glsl.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/clear_clockwise_atomic_clip.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "75e9b26beb81bf9279a78e13c0510dca4e60f704ad9710ae368c116b1aa13da6";
pub const PINNED_SOURCE_LINE_COUNT: usize = 36;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 923;

/// Exact pinned upstream source bytes.
pub const PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0, packed_float3, @a_triangleVertex);
ATTR_BLOCK_END

VERTEX_MAIN(@drawVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_triangleVertex, packed_float3);
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(@a_triangleVertex.xy);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT
PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
PLS_BLOCK_END

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    // srcOver blend is enabled: emit an alpha value of 1 to overwrite the
    // existing clip.
    PLS_STORE4F(clipBuffer, make_half4(.0, .0, .0, 1.));

    // srcOver blend is enabled: emit a color of 0 to make sure the framebuffer
    // remains unchanged.
    EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(.0));
}
#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_SOURCE: &str =
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE;
pub const CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE: &str =
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_CLEAR_CLOCKWISE_ATOMIC_CLIP_GLSL_SOURCE
}
