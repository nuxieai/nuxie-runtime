/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/flush_uniforms.glsl.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "1c22659c0e40233b0b06515287e122e06b73a4428d8e78721ca71e3419db961e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 58;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2454;

/// Exact pinned upstream source bytes.
pub const PINNED_FLUSH_UNIFORMS_GLSL_SOURCE: &str = r###"#ifndef DECLARE_UNIFORM_FLOAT
#define DECLARE_UNIFORM_FLOAT(UNIFORM_NAME) float UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_UINT
#define DECLARE_UNIFORM_UINT(UNIFORM_NAME) uint UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_INT4
#define DECLARE_UNIFORM_INT4(UNIFORM_NAME) int4 UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_FLOAT2
#define DECLARE_UNIFORM_FLOAT2(UNIFORM_NAME) float2 UNIFORM_NAME;
#endif
#ifndef DECLARE_UNIFORM_FLOAT4
#define DECLARE_UNIFORM_FLOAT4(UNIFORM_NAME) float4 UNIFORM_NAME;
#endif

#ifndef FLUSH_UNIFORMS_NAME
#define FLUSH_UNIFORMS_NAME @FlushUniforms
#endif

UNIFORM_BLOCK_BEGIN(FLUSH_UNIFORM_BUFFER_IDX, FLUSH_UNIFORMS_NAME)
DECLARE_UNIFORM_FLOAT(gradInverseViewportY)
DECLARE_UNIFORM_FLOAT(tessInverseViewportY)
DECLARE_UNIFORM_FLOAT(renderTargetInverseViewportX)
DECLARE_UNIFORM_FLOAT(renderTargetInverseViewportY)
DECLARE_UNIFORM_UINT(renderTargetWidth)
DECLARE_UNIFORM_UINT(renderTargetHeight)
// Only used if clears are implemented as draws.
DECLARE_UNIFORM_UINT(colorClearValue)
// Only used if clears are implemented as draws.
DECLARE_UNIFORM_UINT(coverageClearValue)
// drawBounds, or renderTargetBounds if there is a clear. (LTRB.)
DECLARE_UNIFORM_INT4(renderTargetUpdateBounds)
// 1 / [atlasWidth, atlasHeight]
DECLARE_UNIFORM_FLOAT2(atlasTextureInverseSize)
// 2 / atlasContentBounds
DECLARE_UNIFORM_FLOAT2(atlasContentInverseViewport)
DECLARE_UNIFORM_UINT(coverageBufferPrefix)
// GLSL doesn't appear to provide a lightweight, region-local barrier for memory
// ordering outside of memoryBarrier*(), which have severe consequences for
// tiling. When we are already relying on other API level barriers and only need
// to guard against instruction reordering, we can multiply by a tiny epsilon
// instead, and introduce artifical dependencies that enforce ordering but don't
// actually have an effect on the final outcome.
DECLARE_UNIFORM_FLOAT(epsilonForPseudoMemoryBarrier)
// Spacing between adjacent path IDs (1 if IEEE compliant).
DECLARE_UNIFORM_UINT(pathIDGranularity)
DECLARE_UNIFORM_FLOAT(vertexDiscardValue)
DECLARE_UNIFORM_FLOAT(mipMapLODBias)
DECLARE_UNIFORM_UINT(maxPathId)
DECLARE_UNIFORM_FLOAT(ditherScale)
DECLARE_UNIFORM_FLOAT(ditherBias)
// Amount by which to multiply a computed dither value when storing as RGB10 (as
// opposed to writing it out to the framebuffer).
DECLARE_UNIFORM_FLOAT(ditherConversionToRGB10)
// Debugging.
DECLARE_UNIFORM_UINT(wireframeEnabled)
UNIFORM_BLOCK_END(uniforms)"###;

/// Stable source aliases.
pub const PINNED_FLUSH_UNIFORMS_SOURCE: &str = PINNED_FLUSH_UNIFORMS_GLSL_SOURCE;
pub const FLUSH_UNIFORMS_GLSL_SOURCE: &str = PINNED_FLUSH_UNIFORMS_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_FLUSH_UNIFORMS_GLSL_SOURCE
}
