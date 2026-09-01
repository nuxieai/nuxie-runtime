/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/blit_texture_as_draw.glsl.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/blit_texture_as_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "c9d6ab3c8911900a246d22484ad4dbda0a050ba76d74353c9a514d3ca7da3515";
pub const PINNED_SOURCE_LINE_COUNT: usize = 72;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1976;

/// Exact pinned upstream source bytes.
pub const PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE: &str = r###"/*
 * Copyright 2024 Rive
 */

VARYING_BLOCK_BEGIN
#ifdef @USE_FILTERING
NO_PERSPECTIVE VARYING(0, float2, v_texCoord);
#endif
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_TEXTURE_BLOCK_BEGIN
VERTEX_TEXTURE_BLOCK_END

VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
VERTEX_STORAGE_BUFFER_BLOCK_END

ATTR_BLOCK_BEGIN(Attrs)
ATTR_BLOCK_END

VERTEX_MAIN(@blitVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    // Fill the entire screen. The caller will use a scissor test to control the
    // bounds being drawn.
    float2 coord;
    coord.x = (_vertexID & 1) == 0 ? -1. : 1.;
    coord.y = (_vertexID & 2) == 0 ? -1. : 1.;
#ifdef @USE_FILTERING
    VARYING_INIT(v_texCoord, float2);
    v_texCoord.x = coord.x * .5 + .5;
    v_texCoord.y = coord.y * -.5 + .5;
    VARYING_PACK(v_texCoord);
#endif
    float4 pos = float4(coord, 0, 1);
    EMIT_VERTEX(pos);
}
#endif // @VERTEX

#ifdef @FRAGMENT
FRAG_TEXTURE_BLOCK_BEGIN
#ifdef @SOURCE_TEXTURE_MSAA
TEXTURE_RGBA8_MS(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @sourceTexture);
#else
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @sourceTexture);
#endif
FRAG_TEXTURE_BLOCK_END

#ifdef @USE_FILTERING
DYNAMIC_SAMPLER_BLOCK_BEGIN
SAMPLER_DYNAMIC_IMAGE(blitSampler)
DYNAMIC_SAMPLER_BLOCK_END
#endif

FRAG_DATA_MAIN(half4, @blitFragmentMain)
{
    half4 srcColor;
#ifdef @USE_FILTERING
    VARYING_UNPACK(v_texCoord, float2);
    srcColor =
        TEXTURE_SAMPLE_DYNAMIC_LOD(@sourceTexture, blitSampler, v_texCoord, .0);
#elif defined(@SOURCE_TEXTURE_MSAA)
    srcColor = (TEXEL_FETCH_MS(@sourceTexture, 0, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 1, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 2, int2(floor(_fragCoord.xy))) +
                TEXEL_FETCH_MS(@sourceTexture, 3, int2(floor(_fragCoord.xy)))) *
               0.25;
#else
    srcColor = TEXEL_FETCH(@sourceTexture, int2(floor(_fragCoord.xy)));
#endif
    EMIT_FRAG_DATA(srcColor);
}
#endif // @FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_BLIT_TEXTURE_AS_DRAW_SOURCE: &str = PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE;
pub const BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE: &str = PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_BLIT_TEXTURE_AS_DRAW_GLSL_SOURCE
}
