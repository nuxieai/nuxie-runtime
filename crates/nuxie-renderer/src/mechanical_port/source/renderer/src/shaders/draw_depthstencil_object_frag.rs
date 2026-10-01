/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_depthstencil_object.frag.
 *
 * Upstream source revision: 0aadd4c65084a38dbeae3bd05814ead3f743ed77
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "0aadd4c65084a38dbeae3bd05814ead3f743ed77";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_object.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "e62f45bc3e811a504e1bf844ae579fb16173e742e530bffbd459eeff0c8c11d6";
pub const PINNED_SOURCE_LINE_COUNT: usize = 101;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3017;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_DEPTHSTENCIL_OBJECT_FRAG_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

#ifdef @FRAGMENT

// Path draws include draw_path_common.glsl, which declares the textures &
// samplers, so we only need to declare these for image meshes.
#ifdef @DRAW_IMAGE_MESH
FRAG_TEXTURE_BLOCK_BEGIN
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @imageTexture);
#ifdef @ENABLE_ADVANCED_BLEND
DST_COLOR_TEXTURE(@dstColorTexture);
#endif
FRAG_TEXTURE_BLOCK_END

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
    half4 dstColorPremul = DST_COLOR_FETCH(@dstColorTexture);
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

    EMIT_FRAG_DATA(color);
}

#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_DRAW_DEPTHSTENCIL_OBJECT_SOURCE: &str = PINNED_DRAW_DEPTHSTENCIL_OBJECT_FRAG_SOURCE;
pub const DRAW_DEPTHSTENCIL_OBJECT_FRAG_SOURCE: &str = PINNED_DRAW_DEPTHSTENCIL_OBJECT_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_DEPTHSTENCIL_OBJECT_FRAG_SOURCE
}
