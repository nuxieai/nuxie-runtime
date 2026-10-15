//! Exact pinned upstream shader source: renderer/src/shaders/draw_depthstencil_mesh.frag.
#![allow(dead_code)]
pub const PINNED_UPSTREAM_COMMIT: &str = "ce891f526a3297f9498a3e524fce6385cf0f2f25";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "52dd839a8034c26aa7582e7e4d58b382ce1f1bd7cd67b47b27e8eea62f49b53e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 118;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3393;
pub const PINNED_DRAW_DEPTHSTENCIL_MESH_FRAG_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

#ifdef @FRAGMENT

// Path draws include draw_path_common.glsl, which declares the textures &
// samplers, so we only need to declare these for image meshes.
#ifdef @DRAW_IMAGE_MESH
FRAG_TEXTURE_BLOCK_BEGIN
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @imageTexture);
#if defined(@ENABLE_ADVANCED_BLEND) && !defined(@RENDER_TO_UAV)
DST_COLOR_TEXTURE(@dstColorTexture);
#endif
FRAG_TEXTURE_BLOCK_END

#ifdef @RENDER_TO_UAV
PLS_BLOCK_BEGIN
PLS_DECL4F_UAV(COLOR_PLANE_IDX, @dstColorUav);
PLS_BLOCK_END
#endif

DYNAMIC_SAMPLER_BLOCK_BEGIN
SAMPLER_DYNAMIC_IMAGE(imageSampler)
DYNAMIC_SAMPLER_BLOCK_END
#endif // @DRAW_IMAGE_MESH

FRAG_DATA_MAIN(half4, @drawFragmentMain)
{
#ifdef @DRAW_IMAGE_MESH
    VARYING_UNPACK(v_imageTexCoord, float2);
    VARYING_UNPACK(v_imageModulatedColor, half4);
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_imageBlendMode, ushort);
#endif
#else
    VARYING_UNPACK(v_paint, float4);
#ifdef @ENABLE_MODULATED_IMAGE
    VARYING_UNPACK(v_image, float3);
#endif
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_UNPACK(v_atlasCoord, float2);
#endif // @FEATHER_ATLAS_BLIT
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_blendMode, half);
#endif
#endif // !@DRAW_IMAGE_MESH

#ifdef @DRAW_IMAGE_MESH
    half4 color = TEXTURE_SAMPLE_DYNAMIC_LODBIAS(@imageTexture,
                                                 imageSampler,
                                                 v_imageTexCoord,
                                                 uniforms.mipMapLODBias) *
                  v_imageModulatedColor;
#else
    half coverage =
#ifdef @FEATHER_ATLAS_BLIT
        clamp(TEXTURE_SAMPLE_LOD(@featherAtlasTexture,
                                 featherAtlasSampler,
                                 v_atlasCoord,
                                 .0)
                  .r,
              make_half(.0),
              make_half(1.));
#else
        1.;
#endif

    half4 color = find_paint_color(
#ifdef @ENABLE_MODULATED_IMAGE
        v_image,
#endif
#ifdef @ENABLE_ADVANCED_BLEND
        cast_half_to_ushort(v_blendMode),
#endif
        v_paint FRAGMENT_CONTEXT_UNPACK);
#endif

// Need to check both flags here because in GL when KHR_blend_equation_advanced
// is supported, it is possible that neither is defined.
#if defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
    // Do the color portion of the blend mode in the shader.
#ifdef @DRAW_IMAGE_MESH
    color.rgb = unmultiply_rgb(color);
    ushort blendMode = v_imageBlendMode;
#else
    ushort blendMode = cast_half_to_ushort(v_blendMode);
#endif
#ifdef @RENDER_TO_UAV
    half4 dstColorPremul = PLS_LOAD4F_UAV(@dstColorUav);
#else
    half4 dstColorPremul = DST_COLOR_FETCH(@dstColorTexture);
#endif
    color.rgb =
        advanced_color_blend(color.rgb, dstColorPremul, blendMode) * color.a;
#endif

#ifndef @DRAW_IMAGE_MESH
    color *= coverage;
#endif

    color.rgb = add_dither_if_alpha_nonzero(color.rgb,
                                            color.a,
                                            _fragCoord.xy,
                                            uniforms.ditherScale,
                                            uniforms.ditherBias);

#ifdef @RENDER_TO_UAV
    if (color.a != .0)
    {
        PLS_STORE4F_UAV(@dstColorUav, blend_src_over(color, dstColorPremul));
    }
#endif

    EMIT_FRAG_DATA(color);
}

#endif // FRAGMENT
"###;
pub const fn pinned_source() -> &'static str { PINNED_DRAW_DEPTHSTENCIL_MESH_FRAG_SOURCE }
