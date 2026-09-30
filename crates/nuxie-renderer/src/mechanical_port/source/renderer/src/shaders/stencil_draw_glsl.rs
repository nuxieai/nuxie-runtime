/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/stencil_draw.glsl.
 *
 * Upstream source revision: 57dddb3727306e284773ec20c653cf686c45abee
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "57dddb3727306e284773ec20c653cf686c45abee";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/stencil_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "f05d05ee97d8bcb65284e92a9b102ea6fc8e494dee5575a73a0e733cb5684202";
pub const PINNED_SOURCE_LINE_COUNT: usize = 31;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 772;

/// Exact pinned upstream source bytes.
pub const PINNED_STENCIL_DRAW_GLSL_SOURCE: &str = r###"/*
 * Copyright 2024 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
ATTR(0, packed_float3, @a_triangleVertex);
ATTR_BLOCK_END

VERTEX_TEXTURE_BLOCK_BEGIN
VERTEX_TEXTURE_BLOCK_END

VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
VERTEX_STORAGE_BUFFER_BLOCK_END

VERTEX_MAIN(@stencilVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_triangleVertex, packed_float3);
    float4 pos = RENDER_TARGET_COORD_TO_CLIP_COORD(@a_triangleVertex.xy);
    uint zIndex = floatBitsToUint(@a_triangleVertex.z) & 0xffffu;
    pos.z = packNormalizedDepth(zIndex, 0xffu);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT
FRAG_TEXTURE_BLOCK_BEGIN
FRAG_TEXTURE_BLOCK_END

FRAG_DATA_MAIN(half4, @blitFragmentMain) { EMIT_FRAG_DATA(make_half4(.0)); }
#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_STENCIL_DRAW_SOURCE: &str = PINNED_STENCIL_DRAW_GLSL_SOURCE;
pub const STENCIL_DRAW_GLSL_SOURCE: &str = PINNED_STENCIL_DRAW_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_STENCIL_DRAW_GLSL_SOURCE
}
