/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_msaa_object.frag.
 *
 * Upstream source revision: 3ed35ee0ded0d58fb8d380930a156041a4624a2f
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3ed35ee0ded0d58fb8d380930a156041a4624a2f";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_object.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "3b61972533dfebe2c908d98ef42b50c615d4ead4115fecc43a53cca6007de64f";
pub const PINNED_SOURCE_LINE_COUNT: usize = 110;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3616;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE: &str = r###"/*
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
    VARYING_UNPACK(v_imageOpacity, half);
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
                  v_imageOpacity;
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
    half4 color = find_paint_color(v_paint,
#ifdef @ENABLE_MODULATED_IMAGE
                                   v_image,
#endif
                                   coverage FRAGMENT_CONTEXT_UNPACK);
#endif

// Need to check both flags here because in GL when KHR_blend_equation_advanced
// is supported, it is possible that neither is defined.
#if defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
    // Do the color portion of the blend mode in the shader.
#ifdef @DRAW_IMAGE_MESH
    color.rgb = unmultiply_rgb(color);
    ushort blendMode = v_imageBlendMode;
#else
    // NOTE: for non-image-meshes, "color" is already unmultiplied because
    // GENERATE_PREMULTIPLIED_PAINT_COLORS is false when using advanced
    // blend.
    ushort blendMode = cast_half_to_ushort(v_blendMode);
#endif
    half4 dstColorPremul = DST_COLOR_FETCH(@dstColorTexture);
    color.rgb = advanced_color_blend(color.rgb, dstColorPremul, blendMode);

    // Src-over blending is enabled, so just premultiply and let the HW
    // finish the the the alpha portion of the blend mode.
    color.rgb *= color.a;
#endif

    // Certain platforms give us less control of the format of what we are
    // rendering too. Specifically, we are auto converted from linear -> sRGB on
    // render target writes in unreal. In those cases we made need to end up in
    // linear color space
#ifdef @NEEDS_GAMMA_CORRECTION
    if (@NEEDS_GAMMA_CORRECTION)
    {
        color = gamma_to_linear(color);
    }
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
pub const PINNED_DRAW_MSAA_OBJECT_SOURCE: &str = PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE;
pub const DRAW_MSAA_OBJECT_FRAG_SOURCE: &str = PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE
}
