/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/common.glsl.
 *
 * This Phase-1 owner retains the exact GLSL source and exposes the
 * authority-ledger conditionals, include dependencies, exported symbols,
 * and function declarations as inert Rust data. It does not compile,
 * evaluate, simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "37d9f72c2ec84a9a24b42d8798c56c77e396c7b57a39f24edece8c95fe8b3881";
pub const PINNED_SOURCE_LINE_COUNT: usize = 494;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16550;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/common_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_COMMON_GLSL_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

// Common definitions and functions shared by multiple shaders.

#define PI 3.14159265359
#define _2PI 6.28318530718
#define PI_OVER_2 1.57079632679
#define ONE_OVER_SQRT_2 0.70710678118 // 1/sqrt(2)

#ifndef @RENDER_MODE_MSAA
#define AA_RADIUS float(.5)
#else
#define AA_RADIUS float(.0)
#endif

// Defined as a macro because 'uniforms' isn't always available at global scope.
#define RENDER_TARGET_COORD_TO_CLIP_COORD(COORD)                               \
    pixel_coord_to_clip_coord(COORD,                                           \
                              uniforms.renderTargetInverseViewportX,           \
                              uniforms.renderTargetInverseViewportY)

#ifdef @TESS_TEXTURE_FLOATING_POINT
#define TEXTURE_TESSDATA4(SET, IDX, NAME) TEXTURE_RGBA32F(SET, IDX, NAME)
#define TESSDATA4 float4
#define FLOAT_AS_TESSDATA(X) X
#define TESSDATA_AS_FLOAT(X) X
#define UINT_AS_TESSDATA(X) uintBitsToFloat(X)
#define TESSDATA_AS_UINT(X) floatBitsToUint(X)
#else
#define TEXTURE_TESSDATA4(SET, IDX, NAME) TEXTURE_RGBA32UI(SET, IDX, NAME)
#define TESSDATA4 uint4
#define FLOAT_AS_TESSDATA(X) floatBitsToUint(X)
#define TESSDATA_AS_FLOAT(X) uintBitsToFloat(X)
#define UINT_AS_TESSDATA(X) X
#define TESSDATA_AS_UINT(X) X
#endif

// Gathers a 4xN matrix of texels, in the same order as the textureGather() API.
// clang-format off
#define TEXTURE_GATHER_MATRIX(NAME, COORD, COMPONENTS)                         \
    TEXEL_FETCH(NAME, int2(COORD) + int2(-1, 0))COMPONENTS,                    \
        TEXEL_FETCH(NAME, int2(COORD) + int2(0, 0))COMPONENTS,                 \
        TEXEL_FETCH(NAME, int2(COORD) + int2(0, -1))COMPONENTS,                \
        TEXEL_FETCH(NAME, int2(COORD) + int2(-1, -1))COMPONENTS
// clang-format on

// This is a macro because we can't (at least for now) forward texture refs to a
// function in a way that works in all the languages we support.
// This is a macro because we can't (at least for now) forward texture refs to a
// function in a way that works in all the languages we support.
#define FEATHER(X)                                                             \
    TEXTURE_SAMPLE_LOD_1D_ARRAY(@gaussianIntegralTexture,                      \
                                gaussianIntegralSampler,                       \
                                X,                                             \
                                FEATHER_FUNCTION_ARRAY_INDEX,                  \
                                float(FEATHER_FUNCTION_ARRAY_INDEX),           \
                                .0)                                            \
        .r
#define INVERSE_FEATHER(X)                                                     \
    TEXTURE_SAMPLE_LOD_1D_ARRAY(@gaussianIntegralTexture,                      \
                                gaussianIntegralSampler,                       \
                                X,                                             \
                                FEATHER_INVERSE_FUNCTION_ARRAY_INDEX,          \
                                float(FEATHER_INVERSE_FUNCTION_ARRAY_INDEX),   \
                                .0)                                            \
        .r

#ifdef GLSL
// GLSL has different semantics around precision. Normalize type conversions
// across languages with "cast_*_to_*()" methods.
INLINE half cast_float_to_half(float x) { return x; }
INLINE half cast_uint_to_half(uint x) { return float(x); }
INLINE half cast_ushort_to_half(ushort x) { return float(x); }
INLINE half cast_int_to_half(int x) { return float(x); }
INLINE half4 cast_float4_to_half4(float4 xyzw) { return xyzw; }
INLINE half2 cast_float2_to_half2(float2 xy) { return xy; }
INLINE half4 cast_uint4_to_half4(uint4 xyzw) { return vec4(xyzw); }
INLINE ushort cast_half_to_ushort(half x) { return uint(x); }
INLINE ushort cast_uint_to_ushort(uint x) { return x; }
#else
INLINE half cast_float_to_half(float x) { return (half)x; }
INLINE half cast_uint_to_half(uint x) { return (half)x; }
INLINE half cast_ushort_to_half(ushort x) { return (half)x; }
INLINE half cast_int_to_half(int x) { return (half)x; }
INLINE half4 cast_float4_to_half4(float4 xyzw) { return (half4)xyzw; }
INLINE half2 cast_float2_to_half2(float2 xy) { return (half2)xy; }
INLINE half4 cast_uint4_to_half4(uint4 xyzw) { return (half4)xyzw; }
INLINE ushort cast_half_to_ushort(half x) { return (ushort)x; }
INLINE ushort cast_uint_to_ushort(uint x) { return (ushort)x; }
#endif

INLINE half make_half(half x) { return x; }

INLINE half2 make_half2(half2 xy) { return xy; }

INLINE half2 make_half2(half x, half y)
{
    half2 ret;
    ret.x = x, ret.y = y;
    return ret;
}

INLINE half2 make_half2(half x)
{
    half2 ret;
    ret.x = x, ret.y = x;
    return ret;
}

INLINE float2 make_float2(float x) { return float2(x, x); }

INLINE half3 make_half3(half x, half y, half z)
{
    half3 ret;
    ret.x = x, ret.y = y, ret.z = z;
    return ret;
}

INLINE half3 make_half3(half x)
{
    half3 ret;
    ret.x = x, ret.y = x, ret.z = x;
    return ret;
}

INLINE half4 make_half4(half x, half y, half z, half w)
{
    half4 ret;
    ret.x = x, ret.y = y, ret.z = z, ret.w = w;
    return ret;
}

INLINE half4 make_half4(half3 xyz, half w)
{
    half4 ret;
    ret.xyz = xyz;
    ret.w = w;
    return ret;
}

INLINE half4 make_half4(half x)
{
    half4 ret;
    ret.x = x, ret.y = x, ret.z = x, ret.w = x;
    return ret;
}

INLINE half4 make_half4(half4 x) { return x; }

INLINE bool2 make_bool2(bool b) { return bool2(b, b); }

INLINE half3x3 make_half3x3(half3 a, half3 b, half3 c)
{
    half3x3 ret;
    ret[0] = a;
    ret[1] = b;
    ret[2] = c;
    return ret;
}

INLINE half2x3 make_half2x3(half3 a, half3 b)
{
    half2x3 ret;
    ret[0] = a;
    ret[1] = b;
    return ret;
}

INLINE half4x4 make_half4x4(half4 a, half4 b, half4 c, half4 d)
{
    half4x4 ret;
    ret[0] = a;
    ret[1] = b;
    ret[2] = c;
    ret[3] = d;
    return ret;
}

INLINE float2x2 make_float2x2(float4 x) { return float2x2(x.xy, x.zw); }

INLINE uint make_uint(ushort x) { return x; }

INLINE float2 unchecked_mix(float2 a, float2 b, float t)
{
    return (b - a) * t + a;
}

INLINE half id_bits_to_f16(uint idBits, uint pathIDGranularity)
{
    return idBits == 0u
               ? .0
               : unpackHalf2x16((idBits + MAX_DENORM_F16) * pathIDGranularity)
                     .r;
}

INLINE float atan2(float2 v)
{
    v = normalize(v);
    float theta = acos(clamp(v.x, -1., 1.));
    return v.y >= .0 ? theta : -theta;
}

INLINE half4 premultiply(half4 color)
{
    return make_half4(color.rgb * color.a, color.a);
}

INLINE half3 unmultiply_rgb(half4 premul)
{
    // We *could* return preciesly 1 when premul.rgb == premul.a, but we can
    // also be approximate here. The blend modes that depend on this exact level
    // of precision (colordodge and colorburn) account for it with dstPremul.
    return premul.rgb * (premul.a != .0 ? 1. / premul.a : .0);
}

INLINE half min_component(half2 min2) { return min(min2.x, min2.y); }

INLINE half min_component(half3 min3)
{
    return min(min_component(min3.xy), min3.z);
}

INLINE half min_component(half4 min4)
{
    half2 min2 = min(min4.xy, min4.zw);
    half min1 = min(min2.x, min2.y);
    return min1;
}

INLINE half max_component(half2 max2) { return max(max2.x, max2.y); }

INLINE half max_component(half3 max3)
{
    return max(max_component(max3.xy), max3.z);
}

INLINE half max_component(half4 max4)
{
    half2 max2 = max(max4.xy, max4.zw);
    half max1 = max(max2.x, max2.y);
    return max1;
}

INLINE float manhattan_width(float2 x) { return abs(x.x) + abs(x.y); }

// ARM Mali has experienced multiple errors for us when calling clamp(), in both
// GL and Vulkan.
INLINE half safe_clamp_for_mali(half x, half lo, half hi)
{
#if defined(@GL_RENDERER_MALI) || defined(@VULKAN_VENDOR_ARM)
#ifdef @VULKAN_VENDOR_ARM
    if (@VULKAN_VENDOR_ARM)
#endif
    {
        if (x < hi)
            if (x > lo)
                return x;
            else
                return lo;
        else
            return hi;
    }
#endif // @GL_RENDERER_MALI || @VULKAN_VENDOR_ARM
    return clamp(x, lo, hi);
}

INLINE half interleaved_gradient_noise(float2 fragCoord, half scale, half bias)
{
    half v1 = fract(0.06711056 * fragCoord.x + 0.00583715 * fragCoord.y);
    half v2 = fract(52.9829189 * v1);
    return (v2 * scale) + bias;
}

#if 0
// Bayer 4x4 and Bayer 2x2 variants included for reference,
// but not currently used.
INLINE half bayer4x4f(float2 fragCoord, float scale, float bias)
{
    int x = int(fragCoord.x);
    int y = int(fragCoord.y);

    int xxory = (x ^ y);
    int b = (y >> 1) & 1;
    b |= (xxory & 2);
    b |= (y & 1) << 2;
    b |= (xxory & 1) << 3;
    float fb = float(b);
    half hb = cast_float_to_half(fb) / 16.0;
    return (hb * scale) + bias;
}

INLINE half bayer2x2f(float2 fragCoord, float scale, float bias)
{
    fragCoord.y *= 0.5;
    fragCoord.x = fract(fragCoord.x * 0.5 + fragCoord.y);
    fragCoord.y = fract(fragCoord.y);
    float n = (fragCoord.y * 0.5 + fragCoord.x);
    return (n * scale) + bias;
}
#endif

#ifdef @ENABLE_DITHER
INLINE half get_dither(float2 fragCoord, half scale, half bias)
{
    return @ENABLE_DITHER ? interleaved_gradient_noise(fragCoord, scale, bias)
                          : .0;
}

INLINE half3 add_dither_if_alpha_nonzero(half3 color,
                                         half alpha,
                                         float2 fragCoord,
                                         half scale,
                                         half bias)
{
    // Skip dither at alpha == 0, where src-over is an identity on an already
    // quantized destination -- there is no rounding to randomize, and the noise
    // would land in the framebuffer undiluted. It only varies with fragCoord,
    // so that error accumulates with overdraw rather than averaging out.
    return (@ENABLE_DITHER && alpha != .0)
               ? (interleaved_gradient_noise(fragCoord, scale, bias) + color)
               : color;
}

INLINE half3 add_dither_if_alpha_nonzero(half3 color,
                                         half alpha,
                                         half precomputedDither)
{
    // Skip dither at alpha == 0, where src-over is an identity on an already
    // quantized destination -- there is no rounding to randomize, and the noise
    // would land in the framebuffer undiluted. It only varies with fragCoord,
    // so that error accumulates with overdraw rather than averaging out.
    return (@ENABLE_DITHER && alpha != .0) ? (precomputedDither + color)
                                           : color;
}
#else

INLINE half get_dither(float2 fragCoord, float scale, float bias) { return 0.; }

INLINE half3 add_dither_if_alpha_nonzero(half3 color,
                                         half alpha,
                                         float2 fragCoord,
                                         half scale,
                                         half bias)
{
    return color;
}

INLINE half3 add_dither_if_alpha_nonzero(half3 color,
                                         half alpha,
                                         half precomputedDither)
{
    return color;
}
#endif

#ifdef @VERTEX

INLINE float4 pixel_coord_to_clip_coord(float2 pixelCoord,
                                        float inverseViewportX,
                                        float inverseViewportY)
{
    return float4(pixelCoord.x * inverseViewportX - 1.,
                  pixelCoord.y * inverseViewportY - sign(inverseViewportY),
                  0.,
                  1.);
}

#ifndef @RENDER_MODE_MSAA
// Calculates the Manhattan distance in pixels from the given pixelPosition, to
// the point at each edge of the clipRect where coverage = 0.
//
// clipRectInverseMatrix transforms from pixel coordinates to a space where the
// clipRect is the normalized rectangle: [-1, -1, 1, 1].
INLINE float4 find_clip_rect_coverage_distances(float2x2 clipRectInverseMatrix,
                                                float2 clipRectInverseTranslate,
                                                float2 pixelPosition)
{
    float2 clipRectAAWidth =
        abs(clipRectInverseMatrix[0]) + abs(clipRectInverseMatrix[1]);
    if (clipRectAAWidth.x != .0 && clipRectAAWidth.y != .0)
    {
        float2 r = 1. / clipRectAAWidth;
        float2 clipRectCoord = MUL(clipRectInverseMatrix, pixelPosition) +
                               clipRectInverseTranslate;
        // When the center of a pixel falls exactly on an edge, coverage should
        // be .5.
        const float coverageWhenDistanceIsZero = .5;
        return float4(clipRectCoord, -clipRectCoord) * r.xyxy + r.xyxy +
               coverageWhenDistanceIsZero;
    }
    else
    {
        // The caller gave us a singular clipRectInverseMatrix. This is a
        // special case where we are expected to use tx and ty as uniform
        // coverage.
        return clipRectInverseTranslate.xyxy;
    }
}

#else // !@RENDER_MODE_MSAA => @RENDER_MODE_MSAA

INLINE float normalize_z_index(uint zIndex)
{
    return 1. - float(zIndex) * (2. / 32768.);
}

#ifdef @ENABLE_CLIP_RECT
INLINE void set_clip_rect_plane_distances(float2x2 clipRectInverseMatrix,
                                          float2 clipRectInverseTranslate,
                                          float2 pixelPosition
                                              CLIP_CONTEXT_FORWARD)
{
// MSAA uses gl_ClipDistance when ENABLE_CLIP_RECT is set, but since SPIRV uses
// specialization constants (as opposed to compile-time flags), it means that
// the usage of them is in the compiled shader even if that codepath is not
// going to be taken, which ends up as a validation failure on systems that do
// not support that extension. In those cases, we compile separate SPIRV
// binaries with gl_ClipDistance explicitly disabled.
#ifndef @DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
    if (any(notEqual(float4(clipRectInverseMatrix), float4(.0, .0, .0, .0))))
    {
        float2 clipRectCoord = MUL(clipRectInverseMatrix, pixelPosition) +
                               clipRectInverseTranslate.xy;
        gl_ClipDistance[0] = clipRectCoord.x + 1.;
        gl_ClipDistance[1] = clipRectCoord.y + 1.;
        gl_ClipDistance[2] = 1. - clipRectCoord.x;
        gl_ClipDistance[3] = 1. - clipRectCoord.y;
    }
    else
    {
        // "clipRectInverseMatrix == 0" is a special case:
        //     "clipRectInverseTranslate.x == 1" => all in.
        //     "clipRectInverseTranslate.x == 0" => all out.
        gl_ClipDistance[0] = gl_ClipDistance[1] = gl_ClipDistance[2] =
            gl_ClipDistance[3] = clipRectInverseTranslate.x - .5;
    }
#endif // !@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
}
#endif // ENABLE_CLIP_RECT

#endif // @RENDER_MODE_MSAA
#endif // VERTEX

#ifdef @FRAGMENT
#ifdef @NEEDS_GAMMA_CORRECTION
INLINE half gamma_to_linear(half color)
{
    return (color <= 0.04045) ? color / 12.92
                              : pow(abs((color + 0.055) / 1.055), 2.4);
}

INLINE half3 gamma_to_linear(half3 color)
{
    return make_half3(gamma_to_linear(color.r),
                      gamma_to_linear(color.g),
                      gamma_to_linear(color.b));
}

INLINE half4 gamma_to_linear(half4 color)
{
    return make_half4(gamma_to_linear(color.rgb), color.a);
}
#endif // NEEDS_GAMMA_CORRECTION
#endif // FRAGMENT

// The Qualcomm compiler can't handle line breaks in #ifs.
// clang-format off
#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
// clang-format on
INLINE half4 dst_color_fetch(half4x4 dstSamples, int sampleMask)
{
    if (sampleMask == 0xf)
    {
        // Average together all samples for this fragment.
        return (dstSamples[0] + dstSamples[1] + dstSamples[2] + dstSamples[3]) *
               .25;
    }
    else
    {
        // Average together only the samples that are inside the sample mask.
        half4 mask =
            float4(notEqual(sampleMask & int4(1, 2, 4, 8), int4(0, 0, 0, 0)));
        half4 ret = MUL(dstSamples, mask);
        // Since the sample mask can only have 4 bits, counting them is faster
        // this way on Galaxy S24 than calling bitCount().
        int numSamples = (sampleMask & 5) + ((sampleMask >> 1) & 5);
        numSamples = (numSamples & 3) + (numSamples >> 2);
        ret *= 1. / float(numSamples);
        return ret;
    }
}
#endif // @FRAGMENT && @RENDER_MODE_MSAA && !@FIXED_FUNCTION_COLOR_OUTPUT
"###;

pub const PINNED_COMMON_SOURCE: &str = PINNED_COMMON_GLSL_SOURCE;
pub const COMMON_GLSL_SOURCE: &str = PINNED_COMMON_GLSL_SOURCE;
pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_COMMON_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[
    ConditionalBlock {
        block_id: "pp-0241",
        block_start: 12,
        block_end: 16,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0242",
        block_start: 24,
        block_end: 38,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0243",
        block_start: 70,
        block_end: 92,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0244",
        block_start: 252,
        block_end: 265,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0245",
        block_start: 253,
        block_end: 255,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0246",
        block_start: 276,
        block_end: 302,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0247",
        block_start: 304,
        block_end: 356,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0248",
        block_start: 358,
        block_end: 444,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0249",
        block_start: 370,
        block_end: 443,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0250",
        block_start: 409,
        block_end: 441,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0251",
        block_start: 421,
        block_end: 439,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0252",
        block_start: 446,
        block_end: 466,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0253",
        block_start: 447,
        block_end: 465,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0254",
        block_start: 470,
        block_end: 494,
        block_depth: 0,
        branch_count: 1,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[
    ConditionalBranch {
        block_id: "pp-0241",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0241",
        branch_ordinal: 2,
        branch_line: 14,
        directive: "#else",
        active_branch_path: "(!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0242",
        branch_ordinal: 1,
        branch_line: 24,
        directive: "#ifdef @TESS_TEXTURE_FLOATING_POINT",
        active_branch_path: "(defined(@TESS_TEXTURE_FLOATING_POINT))",
    },
    ConditionalBranch {
        block_id: "pp-0242",
        branch_ordinal: 2,
        branch_line: 31,
        directive: "#else",
        active_branch_path: "(!((defined(@TESS_TEXTURE_FLOATING_POINT))))",
    },
    ConditionalBranch {
        block_id: "pp-0243",
        branch_ordinal: 1,
        branch_line: 70,
        directive: "#ifdef GLSL",
        active_branch_path: "(defined(GLSL))",
    },
    ConditionalBranch {
        block_id: "pp-0243",
        branch_ordinal: 2,
        branch_line: 82,
        directive: "#else",
        active_branch_path: "(!((defined(GLSL))))",
    },
    ConditionalBranch {
        block_id: "pp-0244",
        branch_ordinal: 1,
        branch_line: 252,
        directive: "#if defined(@GL_RENDERER_MALI) || defined(@VULKAN_VENDOR_ARM)",
        active_branch_path: "(defined(@GL_RENDERER_MALI) || defined(@VULKAN_VENDOR_ARM))",
    },
    ConditionalBranch {
        block_id: "pp-0245",
        branch_ordinal: 1,
        branch_line: 253,
        directive: "#ifdef @VULKAN_VENDOR_ARM",
        active_branch_path: "(defined(@GL_RENDERER_MALI) || defined(@VULKAN_VENDOR_ARM)) && (defined(@VULKAN_VENDOR_ARM))",
    },
    ConditionalBranch {
        block_id: "pp-0246",
        branch_ordinal: 1,
        branch_line: 276,
        directive: "#if 0",
        active_branch_path: "(0)",
    },
    ConditionalBranch {
        block_id: "pp-0247",
        branch_ordinal: 1,
        branch_line: 304,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0247",
        branch_ordinal: 2,
        branch_line: 337,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_DITHER))))",
    },
    ConditionalBranch {
        block_id: "pp-0248",
        branch_ordinal: 1,
        branch_line: 358,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0249",
        branch_ordinal: 1,
        branch_line: 370,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0249",
        branch_ordinal: 2,
        branch_line: 402,
        directive: "#else // !@RENDER_MODE_MSAA => @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX)) && (!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0250",
        branch_ordinal: 1,
        branch_line: 409,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@VERTEX)) && (!((!defined(@RENDER_MODE_MSAA)))) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0251",
        branch_ordinal: 1,
        branch_line: 421,
        directive: "#ifndef @DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        active_branch_path: "(defined(@VERTEX)) && (!((!defined(@RENDER_MODE_MSAA)))) && (defined(@ENABLE_CLIP_RECT)) && (!defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))",
    },
    ConditionalBranch {
        block_id: "pp-0252",
        branch_ordinal: 1,
        branch_line: 446,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0253",
        branch_ordinal: 1,
        branch_line: 447,
        directive: "#ifdef @NEEDS_GAMMA_CORRECTION",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
    },
    ConditionalBranch {
        block_id: "pp-0254",
        branch_ordinal: 1,
        branch_line: 470,
        directive: "#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)",
        active_branch_path: "(defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 12,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 24,
        source_name: "@TESS_TEXTURE_FLOATING_POINT",
        generated_name: "ZE",
        generated_header_name: "GLSL_TESS_TEXTURE_FLOATING_POINT",
    },
    ExportedSymbol {
        source_line: 54,
        source_name: "@gaussianIntegralTexture",
        generated_name: "XC",
        generated_header_name: "GLSL_gaussianIntegralTexture",
    },
    ExportedSymbol {
        source_line: 252,
        source_name: "@GL_RENDERER_MALI",
        generated_name: "AF",
        generated_header_name: "GLSL_GL_RENDERER_MALI",
    },
    ExportedSymbol {
        source_line: 252,
        source_name: "@VULKAN_VENDOR_ARM",
        generated_name: "WC",
        generated_header_name: "GLSL_VULKAN_VENDOR_ARM",
    },
    ExportedSymbol {
        source_line: 304,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
    },
    ExportedSymbol {
        source_line: 358,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 409,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 421,
        source_name: "@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        generated_name: "EE",
        generated_header_name: "GLSL_DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
    },
    ExportedSymbol {
        source_line: 446,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 447,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
    },
    ExportedSymbol {
        source_line: 470,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 12,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 24,
        source_name: "@TESS_TEXTURE_FLOATING_POINT",
        generated_name: "ZE",
        generated_header_name: "GLSL_TESS_TEXTURE_FLOATING_POINT",
    },
    ExportedSymbol {
        source_line: 252,
        source_name: "@GL_RENDERER_MALI",
        generated_name: "AF",
        generated_header_name: "GLSL_GL_RENDERER_MALI",
    },
    ExportedSymbol {
        source_line: 252,
        source_name: "@VULKAN_VENDOR_ARM",
        generated_name: "WC",
        generated_header_name: "GLSL_VULKAN_VENDOR_ARM",
    },
    ExportedSymbol {
        source_line: 304,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
    },
    ExportedSymbol {
        source_line: 358,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 409,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 421,
        source_name: "@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        generated_name: "EE",
        generated_header_name: "GLSL_DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
    },
    ExportedSymbol {
        source_line: 446,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 447,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
    },
    ExportedSymbol {
        source_line: 470,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "RENDER_MODE_MSAA",
    "TESS_TEXTURE_FLOATING_POINT",
    "gaussianIntegralTexture",
    "GL_RENDERER_MALI",
    "VULKAN_VENDOR_ARM",
    "ENABLE_DITHER",
    "VERTEX",
    "ENABLE_CLIP_RECT",
    "DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
    "FRAGMENT",
    "NEEDS_GAMMA_CORRECTION",
    "FIXED_FUNCTION_COLOR_OUTPUT",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 73,
        end_line: 73,
        name: "cast_float_to_half",
        signature: "INLINE half cast_float_to_half(float x) { return x; }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 74,
        end_line: 74,
        name: "cast_uint_to_half",
        signature: "INLINE half cast_uint_to_half(uint x) { return float(x); }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 75,
        end_line: 75,
        name: "cast_ushort_to_half",
        signature: "INLINE half cast_ushort_to_half(ushort x) { return float(x); }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 76,
        end_line: 76,
        name: "cast_int_to_half",
        signature: "INLINE half cast_int_to_half(int x) { return float(x); }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 77,
        end_line: 77,
        name: "cast_float4_to_half4",
        signature: "INLINE half4 cast_float4_to_half4(float4 xyzw) { return xyzw; }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 78,
        end_line: 78,
        name: "cast_float2_to_half2",
        signature: "INLINE half2 cast_float2_to_half2(float2 xy) { return xy; }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 79,
        end_line: 79,
        name: "cast_uint4_to_half4",
        signature: "INLINE half4 cast_uint4_to_half4(uint4 xyzw) { return vec4(xyzw); }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 80,
        end_line: 80,
        name: "cast_half_to_ushort",
        signature: "INLINE ushort cast_half_to_ushort(half x) { return uint(x); }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 81,
        end_line: 81,
        name: "cast_uint_to_ushort",
        signature: "INLINE ushort cast_uint_to_ushort(uint x) { return x; }",
        guard_path: "(defined(GLSL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 83,
        end_line: 83,
        name: "cast_float_to_half",
        signature: "INLINE half cast_float_to_half(float x) { return (half)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 84,
        end_line: 84,
        name: "cast_uint_to_half",
        signature: "INLINE half cast_uint_to_half(uint x) { return (half)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 85,
        end_line: 85,
        name: "cast_ushort_to_half",
        signature: "INLINE half cast_ushort_to_half(ushort x) { return (half)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 86,
        end_line: 86,
        name: "cast_int_to_half",
        signature: "INLINE half cast_int_to_half(int x) { return (half)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 87,
        end_line: 87,
        name: "cast_float4_to_half4",
        signature: "INLINE half4 cast_float4_to_half4(float4 xyzw) { return (half4)xyzw; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 88,
        end_line: 88,
        name: "cast_float2_to_half2",
        signature: "INLINE half2 cast_float2_to_half2(float2 xy) { return (half2)xy; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 89,
        end_line: 89,
        name: "cast_uint4_to_half4",
        signature: "INLINE half4 cast_uint4_to_half4(uint4 xyzw) { return (half4)xyzw; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 90,
        end_line: 90,
        name: "cast_half_to_ushort",
        signature: "INLINE ushort cast_half_to_ushort(half x) { return (ushort)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 91,
        end_line: 91,
        name: "cast_uint_to_ushort",
        signature: "INLINE ushort cast_uint_to_ushort(uint x) { return (ushort)x; }",
        guard_path: "(!((defined(GLSL))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 94,
        end_line: 94,
        name: "make_half",
        signature: "INLINE half make_half(half x) { return x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 96,
        end_line: 96,
        name: "make_half2",
        signature: "INLINE half2 make_half2(half2 xy) { return xy; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 98,
        end_line: 103,
        name: "make_half2",
        signature: "INLINE half2 make_half2(half x, half y)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 105,
        end_line: 110,
        name: "make_half2",
        signature: "INLINE half2 make_half2(half x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 112,
        end_line: 112,
        name: "make_float2",
        signature: "INLINE float2 make_float2(float x) { return float2(x, x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 114,
        end_line: 119,
        name: "make_half3",
        signature: "INLINE half3 make_half3(half x, half y, half z)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 121,
        end_line: 126,
        name: "make_half3",
        signature: "INLINE half3 make_half3(half x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 128,
        end_line: 133,
        name: "make_half4",
        signature: "INLINE half4 make_half4(half x, half y, half z, half w)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 135,
        end_line: 141,
        name: "make_half4",
        signature: "INLINE half4 make_half4(half3 xyz, half w)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 143,
        end_line: 148,
        name: "make_half4",
        signature: "INLINE half4 make_half4(half x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 150,
        end_line: 150,
        name: "make_half4",
        signature: "INLINE half4 make_half4(half4 x) { return x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 152,
        end_line: 152,
        name: "make_bool2",
        signature: "INLINE bool2 make_bool2(bool b) { return bool2(b, b); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 154,
        end_line: 161,
        name: "make_half3x3",
        signature: "INLINE half3x3 make_half3x3(half3 a, half3 b, half3 c)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 163,
        end_line: 169,
        name: "make_half2x3",
        signature: "INLINE half2x3 make_half2x3(half3 a, half3 b)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 171,
        end_line: 179,
        name: "make_half4x4",
        signature: "INLINE half4x4 make_half4x4(half4 a, half4 b, half4 c, half4 d)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 181,
        end_line: 181,
        name: "make_float2x2",
        signature: "INLINE float2x2 make_float2x2(float4 x) { return float2x2(x.xy, x.zw); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 183,
        end_line: 183,
        name: "make_uint",
        signature: "INLINE uint make_uint(ushort x) { return x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 185,
        end_line: 188,
        name: "unchecked_mix",
        signature: "INLINE float2 unchecked_mix(float2 a, float2 b, float t)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 190,
        end_line: 196,
        name: "id_bits_to_f16",
        signature: "INLINE half id_bits_to_f16(uint idBits, uint pathIDGranularity)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 198,
        end_line: 203,
        name: "atan2",
        signature: "INLINE float atan2(float2 v)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 205,
        end_line: 208,
        name: "premultiply",
        signature: "INLINE half4 premultiply(half4 color)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 210,
        end_line: 216,
        name: "unmultiply_rgb",
        signature: "INLINE half3 unmultiply_rgb(half4 premul)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 218,
        end_line: 218,
        name: "min_component",
        signature: "INLINE half min_component(half2 min2) { return min(min2.x, min2.y); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 220,
        end_line: 223,
        name: "min_component",
        signature: "INLINE half min_component(half3 min3)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 225,
        end_line: 230,
        name: "min_component",
        signature: "INLINE half min_component(half4 min4)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 232,
        end_line: 232,
        name: "max_component",
        signature: "INLINE half max_component(half2 max2) { return max(max2.x, max2.y); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 234,
        end_line: 237,
        name: "max_component",
        signature: "INLINE half max_component(half3 max3)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 239,
        end_line: 244,
        name: "max_component",
        signature: "INLINE half max_component(half4 max4)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 246,
        end_line: 246,
        name: "manhattan_width",
        signature: "INLINE float manhattan_width(float2 x) { return abs(x.x) + abs(x.y); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 250,
        end_line: 267,
        name: "safe_clamp_for_mali",
        signature: "INLINE half safe_clamp_for_mali(half x, half lo, half hi)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 269,
        end_line: 274,
        name: "interleaved_gradient_noise",
        signature: "INLINE half interleaved_gradient_noise(float2 fragCoord, half scale, half bias)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 279,
        end_line: 292,
        name: "bayer4x4f",
        signature: "INLINE half bayer4x4f(float2 fragCoord, float scale, float bias)",
        guard_path: "(0)",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 294,
        end_line: 301,
        name: "bayer2x2f",
        signature: "INLINE half bayer2x2f(float2 fragCoord, float scale, float bias)",
        guard_path: "(0)",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 305,
        end_line: 309,
        name: "get_dither",
        signature: "INLINE half get_dither(float2 fragCoord, half scale, half bias)",
        guard_path: "(defined(@ENABLE_DITHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 311,
        end_line: 324,
        name: "add_dither_if_alpha_nonzero",
        signature: "INLINE half3 add_dither_if_alpha_nonzero(half3 color, half alpha, float2 fragCoord, half scale, half bias)",
        guard_path: "(defined(@ENABLE_DITHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 326,
        end_line: 336,
        name: "add_dither_if_alpha_nonzero",
        signature: "INLINE half3 add_dither_if_alpha_nonzero(half3 color, half alpha, half precomputedDither)",
        guard_path: "(defined(@ENABLE_DITHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 339,
        end_line: 339,
        name: "get_dither",
        signature: "INLINE half get_dither(float2 fragCoord, float scale, float bias) { return 0.; }",
        guard_path: "(!((defined(@ENABLE_DITHER))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 341,
        end_line: 348,
        name: "add_dither_if_alpha_nonzero",
        signature: "INLINE half3 add_dither_if_alpha_nonzero(half3 color, half alpha, float2 fragCoord, half scale, half bias)",
        guard_path: "(!((defined(@ENABLE_DITHER))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 350,
        end_line: 355,
        name: "add_dither_if_alpha_nonzero",
        signature: "INLINE half3 add_dither_if_alpha_nonzero(half3 color, half alpha, half precomputedDither)",
        guard_path: "(!((defined(@ENABLE_DITHER))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 360,
        end_line: 368,
        name: "pixel_coord_to_clip_coord",
        signature: "INLINE float4 pixel_coord_to_clip_coord(float2 pixelCoord, float inverseViewportX, float inverseViewportY)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 376,
        end_line: 400,
        name: "find_clip_rect_coverage_distances",
        signature: "INLINE float4 find_clip_rect_coverage_distances(float2x2 clipRectInverseMatrix, float2 clipRectInverseTranslate, float2 pixelPosition)",
        guard_path: "(defined(@VERTEX)) && (!defined(@RENDER_MODE_MSAA))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 404,
        end_line: 407,
        name: "normalize_z_index",
        signature: "INLINE float normalize_z_index(uint zIndex)",
        guard_path: "(defined(@VERTEX)) && (!((!defined(@RENDER_MODE_MSAA))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 410,
        end_line: 440,
        name: "set_clip_rect_plane_distances",
        signature: "INLINE void set_clip_rect_plane_distances(float2x2 clipRectInverseMatrix, float2 clipRectInverseTranslate, float2 pixelPosition CLIP_CONTEXT_FORWARD)",
        guard_path: "(defined(@VERTEX)) && (!((!defined(@RENDER_MODE_MSAA)))) && (defined(@ENABLE_CLIP_RECT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 448,
        end_line: 452,
        name: "gamma_to_linear",
        signature: "INLINE half gamma_to_linear(half color)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 454,
        end_line: 459,
        name: "gamma_to_linear",
        signature: "INLINE half3 gamma_to_linear(half3 color)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 461,
        end_line: 464,
        name: "gamma_to_linear",
        signature: "INLINE half4 gamma_to_linear(half4 color)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 472,
        end_line: 493,
        name: "dst_color_fetch",
        signature: "INLINE half4 dst_color_fetch(half4x4 dstSamples, int sampleMask)",
        guard_path: "(defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
        inline_qualifier: "INLINE",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderInclude {
    pub upstream_file: &'static str,
    pub include_line: u16,
    pub directive: &'static str,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub correspondence_owner: &'static str,
    pub mapping_status: &'static str,
    pub translation_status: &'static str,
    pub translation_disposition: &'static str,
}

pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[
    ShaderInclude {
        upstream_file: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 10,
        directive: "include",
        include_token: "generated/shaders/common.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "prepared",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: "renderer/src/shaders/metal/color_ramp.metal",
        include_line: 10,
        directive: "include",
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "prepared",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: "renderer/src/shaders/metal/draw.metal",
        include_line: 14,
        directive: "include",
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "prepared",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: "renderer/src/shaders/metal/tessellate.metal",
        include_line: 10,
        directive: "include",
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "prepared",
        translation_disposition: "required-source-edge",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncludeDependency {
    pub including_source: &'static str,
    pub include_line: u16,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub translation_disposition: &'static str,
}

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 10,
        include_token: "generated/shaders/common.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/color_ramp.metal",
        include_line: 10,
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/draw.metal",
        include_line: 14,
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/tessellate.metal",
        include_line: 10,
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
