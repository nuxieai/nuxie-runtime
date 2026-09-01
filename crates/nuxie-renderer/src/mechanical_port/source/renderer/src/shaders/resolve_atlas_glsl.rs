/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/resolve_atlas.glsl.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/resolve_atlas.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "a31d945c9b29dd4ba74cff3c9c9010e108f5cd82bb0b82474b199725e59aa04f";
pub const PINNED_SOURCE_LINE_COUNT: usize = 93;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2615;

/// Exact pinned upstream source bytes.
pub const PINNED_RESOLVE_ATLAS_GLSL_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

// This shader provides a mechanism for resolving various atlas types into GL_R8
// so they can be sampled linearly.
//
// Additionally, EXT_shader_pixel_local_storage extension does not have a
// "clear" function, so this shader also provides a clear mechanism for PLS.

#ifdef @VERTEX
VERTEX_MAIN(@atlasResolveVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    // Draw a right triangle that covers the entire screen.
    float4 pos;
    pos.x = (_vertexID != 2) ? -1. : 3.;
    pos.y = (_vertexID != 1) ? -1. : 3.;
    pos.zw = float2(.0, 1.);
    EMIT_VERTEX(pos);
}
#endif

#ifdef @FRAGMENT

INLINE ivec2 atlas_frag_coord() { return ivec2(floor(gl_FragCoord)); }

#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH

layout(location = 0) inout uint4 coverageCount;
layout(location = 1) out half4 resolvedCoverage;

void main() { resolvedCoverage.r = uintBitsToFloat(coverageCount.r); }

#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)

#ifdef @CLEAR_COVERAGE
__pixel_local_outEXT PLS { layout(r32f) float coverageCount; };
#else
__pixel_local_inEXT PLS { layout(r32f) float coverageCount; };
layout(location = 0) out half4 resolvedCoverage;
#endif

void main()
{
#ifdef @CLEAR_COVERAGE
    coverageCount = .0;
#else
    resolvedCoverage.r = coverageCount;
#endif
}

#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)

layout(binding = 0, r32ui) uniform highp upixelLocalANGLE coverageCount;
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    resolvedCoverage.r = uintBitsToFloat(pixelLocalLoadANGLE(coverageCount).r);
}

#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)

layout(binding = 0, r32i) uniform highp coherent iimage2D _atlasImage;
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    resolvedCoverage.r = float(imageLoad(_atlasImage, atlas_frag_coord()).r) *
                         (1. / ATLAS_R32I_FIXED_POINT_FACTOR);
}

#elif defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)

TEXTURE_RGBA8(PER_FLUSH_BINDINGS_SET, 0, @atlasRenderTexture);
layout(location = 0) out half4 resolvedCoverage;

void main()
{
    // Apply the following weights to the RGBA of each u8x4 coverage value:
    //   - R counts fractional, positive coverage.
    //   - G counts fractional, negative coverage.
    //   - B counts integer, positive coverage.
    //   - A counts integer, negative coverage.
    half4 coverages = TEXEL_FETCH(@atlasRenderTexture, atlas_frag_coord());
    resolvedCoverage.r =
        (coverages.r - coverages.g) * ATLAS_UNORM8_COVERAGE_SCALE_FACTOR +
        (coverages.b - coverages.a) * 255.;
}

#endif

#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_RESOLVE_ATLAS_SOURCE: &str = PINNED_RESOLVE_ATLAS_GLSL_SOURCE;
pub const RESOLVE_ATLAS_GLSL_SOURCE: &str = PINNED_RESOLVE_ATLAS_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_RESOLVE_ATLAS_GLSL_SOURCE
}
