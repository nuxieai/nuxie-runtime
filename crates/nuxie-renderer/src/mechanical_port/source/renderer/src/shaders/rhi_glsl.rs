/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/rhi.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, exports, functions, and source metadata as
 * literal source-shaped data. It does not compile, evaluate, simplify, or
 * generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "b08bc3a81cd7e88eb82ffba447fd073630aaa51f996641e8f7cd367678617f96";
pub const PINNED_SOURCE_LINE_COUNT: usize = 560;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22901;
pub const PINNED_SOURCE_STAGE: &str = "minify-input-glsl";
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/rhi_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_stage: &'static str,
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
    source_stage: PINNED_SOURCE_STAGE,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_RHI_GLSL_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

// This header provides GLSL-specific #defines and declarations that enable our
// shaders to be compiled on MSL and GLSL both.

// HLSL warns that it will unroll the loops through r,g,b values in
// advanced_blend.glsl, but unrolling these loops is exactly what we want.
#pragma $warning($disable : 3550)

// Don't warn about uninitialized variables. If we leave one uninitialized it's
// because we know what we're doing and don't want to pay the cost of
// initializing it.
#pragma $warning($disable : 4000)

// #define native hlsl types if their names are being rewritten.
#define _ARE_TOKEN_NAMES_PRESERVED
#ifndef $_ARE_TOKEN_NAMES_PRESERVED
#define half $half
#define half2 $half2
#define half3 $half3
#define half4 $half4
#define short $short
#define short2 $short2
#define short3 $short3
#define short4 $short4
#define ushort $ushort
#define ushort2 $ushort2
#define ushort3 $ushort3
#define ushort4 $ushort4
#define float2 $float2
#define float3 $float3
#define float4 $float4
#define bool2 $bool2
#define bool3 $bool3
#define bool4 $bool4
#define uint2 $uint2
#define uint3 $uint3
#define uint4 $uint4
#define int2 $int2
#define int3 $int3
#define int4 $int4
#define float4x2 $float4x2
#define ushort $ushort
#define float2x2 $float2x2
#define half3x3 $half3x3
#define half2x3 $half2x3
#define half4x4 $half4x4
#endif

$typedef float3 packed_float3;

#ifdef @ENABLE_MIN_16_PRECISION

#if NEEDS_USHORT_DEFINE

$typedef $min16uint ushort;

#endif // NEEDS_USHORT_DEFINE

#else

#if NEEDS_USHORT_DEFINE

$typedef $uint ushort;

#endif // NEEDS_USHORT_DEFINE

#endif // ENABLE_MIN_16_PRECISION

#define CONCAT(A, B) A##B

#define INLINE $inline
#define OUT(ARG_TYPE) out ARG_TYPE
#define INOUT(ARG_TYPE) inout ARG_TYPE

#define ATTR_BLOCK_BEGIN(NAME)                                                 \
    struct NAME                                                                \
    {
#define ATTR(IDX, TYPE, NAME) TYPE NAME : CONCAT(ATTRIBUTE, IDX)
#define ATTR_BLOCK_END                                                         \
    }                                                                          \
    ;
#define ATTR_LOAD(T, A, N, I)
#define ATTR_UNPACK(ID, attrs, NAME, TYPE) TYPE NAME = attrs.NAME

#define UNIFORM_BLOCK_BEGIN(IDX, NAME)                                         \
    $cbuffer NAME                                                              \
    {                                                                          \
        struct                                                                 \
        {

#define UNIFORM_BLOCK_END(NAME)                                                \
    }                                                                          \
    NAME;                                                                      \
    }

#define VARYING_BLOCK_BEGIN                                                    \
    struct Varyings                                                            \
    {

#define NO_PERSPECTIVE $noperspective
#define @OPTIONALLY_FLAT $nointerpolation
#define FLAT $nointerpolation
#define VARYING(IDX, TYPE, NAME) TYPE NAME : CONCAT($TEXCOORD, IDX)

#ifdef @NEEDS_CLIP_DISTANCE
#define VARYING_BLOCK_END                                                      \
    float4 _pos : $SV_Position;                                                \
    float4 _clip : $SV_ClipDistance;                                           \
    }                                                                          \
    ;
#else // !@NEEDS_CLIP_DISTANCE
#define VARYING_BLOCK_END                                                      \
    float4 _pos : $SV_Position;                                                \
    }                                                                          \
    ;
#endif // @NEEDS_CLIP_DISTANCE

#define VARYING_INIT(NAME, TYPE) TYPE NAME
#define VARYING_PACK(NAME) _varyings.NAME = NAME
#define VARYING_UNPACK(NAME, TYPE) TYPE NAME = _varyings.NAME

#ifdef @VERTEX
#define VERTEX_TEXTURE_BLOCK_BEGIN
#define VERTEX_TEXTURE_BLOCK_END
#endif

#ifdef @FRAGMENT
#define FRAG_TEXTURE_BLOCK_BEGIN
#define FRAG_TEXTURE_BLOCK_END
#endif

#define DYNAMIC_SAMPLER_BLOCK_BEGIN
#define DYNAMIC_SAMPLER_BLOCK_END

#define TEXTURE_RGBA32UI(SET, IDX, NAME) uniform $Texture2D<uint4> NAME
#define TEXTURE_RGBA32F(SET, IDX, NAME) uniform $Texture2D<float4> NAME
#ifdef @SOURCE_TEXTURE_MSAA
#define TEXTURE_RGBA8_MS(SET, IDX, NAME) uniform $Texture2DMS<half4> NAME
#endif
#define TEXTURE_RGBA8(SET, IDX, NAME) uniform $Texture2D<half4> NAME
#define TEXTURE_R16F(SET, IDX, NAME) uniform $Texture2D<half> NAME
#define TEXTURE_R16F_1D_ARRAY(SET, IDX, NAME) uniform $Texture2DArray<half> NAME
#define SAMPLED_R16F_REF(NAME, SAMPLER_NAME)                                   \
    $Texture2D<half> NAME, $SamplerState SAMPLER_NAME
#define SAMPLED_R16F(NAME, SAMPLER_NAME) NAME, SAMPLER_NAME

// SAMPLER_LINEAR is the same as SAMPLER because in d3d11, sampler
// parameters are defined at the API level.
#define SAMPLER(IDX, NAME) $SamplerState NAME;
#define SAMPLER_LINEAR SAMPLER
#define SAMPLER_DYNAMIC(SET, IDX, NAME) SAMPLER(IDX, NAME)
#define SAMPLER_DYNAMIC_IMAGE(NAME) SAMPLER(IMAGE_TEXTURE_IDX, NAME)

#ifdef SOURCE_TEXTURE_MSAA
#define TEXEL_FETCH_MS(NAME, LEVEL, COORD) NAME.Load(COORD, LEVEL)
#endif
#define TEXEL_FETCH(NAME, COORD) NAME[COORD]
#define TEXTURE_SAMPLE(NAME, SAMPLER_NAME, COORD)                              \
    NAME.$Sample(SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_LOD(NAME, SAMPLER_NAME, COORD, LOD)                     \
    NAME.$SampleLevel(SAMPLER_NAME, COORD, LOD)
#define TEXTURE_SAMPLE_LODBIAS(NAME, SAMPLER_NAME, COORD, LODBIAS)             \
    NAME.$SampleBias(SAMPLER_NAME, COORD, LODBIAS)
#define TEXTURE_REF_SAMPLE_LOD TEXTURE_SAMPLE_LOD
#define TEXTURE_SAMPLE_GRAD(NAME, SAMPLER_NAME, COORD, DDX, DDY)               \
    NAME.$SampleGrad(SAMPLER_NAME, COORD, DDX, DDY)
#define TEXTURE_GATHER(NAME, SAMPLER_NAME, COORD, TEXTURE_INVERSE_SIZE)        \
    NAME.$Gather(SAMPLER_NAME, (COORD) * (TEXTURE_INVERSE_SIZE))
#define TEXTURE_SAMPLE_LOD_1D_ARRAY(NAME,                                      \
                                    SAMPLER_NAME,                              \
                                    X,                                         \
                                    ARRAY_INDEX,                               \
                                    ARRAY_INDEX_NORMALIZED,                    \
                                    LOD)                                       \
    NAME.$SampleLevel(SAMPLER_NAME, float3(X, 0.5, ARRAY_INDEX), LOD)

#define TEXTURE_SAMPLE_DYNAMIC(TEXTURE, SAMPLER_NAME, COORD)                   \
    TEXTURE_SAMPLE(TEXTURE, SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_DYNAMIC_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)          \
    TEXTURE_SAMPLE_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)
#define TEXTURE_SAMPLE_DYNAMIC_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)  \
    TEXTURE_SAMPLE_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#ifdef @ENABLE_RASTERIZER_ORDERED_VIEWS
#define PLS_TEX2D $RasterizerOrderedTexture2D
#else
#define PLS_TEX2D $RWTexture2D
#endif

#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)

#ifdef @SUPPORTS_SUBPASS_LOAD
#define DST_COLOR_TEXTURE(NAME)                                                \
    [[vk::input_attachment_index(COLOR_PLANE_IDX)]] $SubpassInputMS<half4> NAME

#define DST_COLOR_FETCH(NAME)                                                  \
    dst_color_fetch(half4x4(NAME.SubpassLoad(0),                               \
                            NAME.SubpassLoad(1),                               \
                            NAME.SubpassLoad(2),                               \
                            NAME.SubpassLoad(3)),                              \
                    _sampleMask)
#else
#define DST_COLOR_TEXTURE(NAME) $Texture2D NAME

#define DST_COLOR_FETCH(NAME) NAME[_plsCoord]
#endif
#endif // @FRAGMENT && @RENDER_MODE_MSAA

#define PLS_BLOCK_BEGIN
#define PLS_BLOCK_END

#ifdef @ENABLE_TYPED_UAV_LOAD_STORE
#define PLS_DECL4F(IDX, NAME) uniform PLS_TEX2D<UNORM half4> NAME
#else
#define PLS_DECL4F(IDX, NAME) uniform PLS_TEX2D<uint> NAME
#endif
#define PLS_DECL4F_READONLY PLS_DECL4F
#define PLS_DECLUI(IDX, NAME) uniform PLS_TEX2D<uint> NAME

#define PLS_LOADUI_UAV PLS_LOADUI
#define PLS_STOREUI_UAV PLS_STOREUI

#if $COMPILER_METAL || $FORCE_ATOMIC_BUFFER
#define PLS_DECLUI_UAV(IDX, NAME) uniform $RWBuffer<uint> NAME
#define PLS_LOADUI_UAV(PLANE) PLANE[_plsIdx]
#define PLS_STOREUI_UAV(PLANE, VALUE) PLANE[_plsIdx] = VALUE
#else
#define PLS_DECLUI_UAV PLS_DECLUI
#define PLS_LOADUI_UAV PLS_LOADUI
#define PLS_STOREUI_UAV PLS_STOREUI
#endif // COMPILER_METAL

#ifdef @ENABLE_TYPED_UAV_LOAD_STORE
#define PLS_LOAD4F(PLANE) PLANE[_plsCoord]
#else
#define PLS_LOAD4F(PLANE) unpackUnorm4x8(PLANE[_plsCoord])
#endif
#define PLS_LOADUI(PLANE) PLANE[_plsCoord]
#ifdef @ENABLE_TYPED_UAV_LOAD_STORE
#define PLS_STORE4F(PLANE, VALUE) PLANE[_plsCoord] = (VALUE)
#else
#define PLS_STORE4F(PLANE, VALUE) PLANE[_plsCoord] = packUnorm4x8(VALUE)
#endif
#define PLS_STOREUI(PLANE, VALUE) PLANE[_plsCoord] = (VALUE)

#if $COMPILER_METAL || $FORCE_ATOMIC_BUFFER
INLINE uint pls_atomic_max($RWBuffer<uint> plane, uint _plsIdx, uint x)
{
    uint originalValue;
    $InterlockedMax(plane[_plsIdx], x, originalValue);
    return originalValue;
}

#define PLS_ATOMIC_MAX(PLANE, X) pls_atomic_max(PLANE, _plsIdx, X)

INLINE uint pls_atomic_add($RWBuffer<uint> plane, uint _plsIdx, uint x)
{
    uint originalValue;
    $InterlockedAdd(plane[_plsIdx], x, originalValue);
    return originalValue;
}

#define PLS_ATOMIC_ADD(PLANE, X) pls_atomic_add(PLANE, _plsIdx, X)
#else
INLINE uint pls_atomic_max(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)
{
    uint originalValue;
    $InterlockedMax(plane[_plsCoord], x, originalValue);
    return originalValue;
}

#define PLS_ATOMIC_MAX(PLANE, X) pls_atomic_max(PLANE, _plsCoord, X)

INLINE uint pls_atomic_add(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)
{
    uint originalValue;
    $InterlockedAdd(plane[_plsCoord], x, originalValue);
    return originalValue;
}

#define PLS_ATOMIC_ADD(PLANE, X) pls_atomic_add(PLANE, _plsCoord, X)
#endif

#define PLS_PRESERVE_4F(PLANE)
#define PLS_PRESERVE_UI(PLANE)

#define VERTEX_CONTEXT_DECL
#define VERTEX_CONTEXT_UNPACK

#define TEXTURE_CONTEXT_DECL
#define TEXTURE_CONTEXT_FORWARD

#ifdef @NO_VARYING

#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
                                                                               \
    uint $baseInstance;                                                        \
                                                                               \
    float4 NAME(Attrs attrs,                                                   \
                uint _vertexID : $SV_VertexID,                                 \
                uint _instanceIDWithoutBase : $SV_InstanceID) :                \
        $SV_Position                                                           \
    {                                                                          \
        uint _instanceID = _instanceIDWithoutBase + $baseInstance;

#define EMIT_VERTEX(POSITION)                                                  \
    return POSITION;                                                           \
    }

#else // !@NO_VARYING

#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
                                                                               \
    uint $baseInstance;                                                        \
                                                                               \
    Varyings NAME(Attrs attrs,                                                 \
                  uint _vertexID : $SV_VertexID,                               \
                  uint _instanceIDWithoutBase : $SV_InstanceID)                \
    {                                                                          \
        uint _instanceID = _instanceIDWithoutBase + $baseInstance;             \
        Varyings _varyings;

#define IMAGE_RECT_VERTEX_MAIN(NAME,                                           \
                               Attrs,                                          \
                               attrs,                                          \
                               ImageDrawAttrs,                                 \
                               imageDrawAttrs,                                 \
                               _vertexID,                                      \
                               _instanceID)                                    \
    Varyings NAME(Attrs attrs,                                                 \
                  ImageDrawAttrs imageDrawAttrs,                               \
                  uint _vertexID : $SV_VertexID)                               \
    {                                                                          \
        Varyings _varyings;                                                    \
        float4 _pos;

#define IMAGE_MESH_VERTEX_MAIN(NAME,                                           \
                               PositionAttr,                                   \
                               position,                                       \
                               UVAttr,                                         \
                               uv,                                             \
                               ImageDrawAttrs,                                 \
                               imageDrawAttrs,                                 \
                               _vertexID)                                      \
    Varyings NAME(PositionAttr position,                                       \
                  UVAttr uv,                                                   \
                  ImageDrawAttrs imageDrawAttrs,                               \
                  uint _vertexID : $SV_VertexID)                               \
    {                                                                          \
        Varyings _varyings;                                                    \
        float4 _pos;

#define EMIT_VERTEX(POSITION)                                                  \
    _varyings._pos = POSITION;                                                 \
    }                                                                          \
    return _varyings;
#endif // End !@NO_VARYING

// Unreal flips the front face for direct x but not vulkan. We should test this
// in other platforms and make sure it comes out the correct direction.
#if $COMPILER_DXC && ($COMPILER_VULKAN || $COMPILER_GLSL_ES3_1)
#define CLOCKWISE_FROM_FRONT_FACE(_ff) (_ff)
#else
#define CLOCKWISE_FROM_FRONT_FACE(_ff) (!(_ff))
#endif

#ifdef @NO_VARYING
#define FRAG_DATA_MAIN(DATA_TYPE, NAME)                                        \
    $EARLYDEPTHSTENCIL DATA_TYPE NAME(float4 _pos : $SV_Position) : $SV_Target \
    {                                                                          \
        float2 _fragCoord = _pos.xy;

#define FRAG_DATA_MAIN_WITH_CLOCKWISE(DATA_TYPE, NAME)                         \
    EARLYDEPTHSTENCIL DATA_TYPE NAME(float4 _pos : $SV_Position,               \
                                     uint _sampleMask : $SV_Coverage,          \
                                     bool _isFrontFace : $SV_IsFrontFace) :    \
        $SV_Target                                                             \
    {                                                                          \
        float2 _fragCoord = _pos.xy;                                           \
        bool _clockwise = CLOCKWISE_FROM_FRONT_FACE(_isFrontFace);
#else
#define FRAG_DATA_MAIN(DATA_TYPE, NAME)                                        \
    $EARLYDEPTHSTENCIL DATA_TYPE NAME(Varyings _varyings,                      \
                                      uint _sampleMask : $SV_Coverage) :       \
        $SV_Target                                                             \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        int2 _plsCoord = int2(floor(_fragCoord));                              \
        uint _plsIdx = _plsCoord.y * uniforms.renderTargetWidth + _plsCoord.x;

#define FRAG_DATA_MAIN_WITH_CLOCKWISE(DATA_TYPE, NAME)                         \
    DATA_TYPE NAME(Varyings _varyings,                                         \
                   uint _sampleMask : $SV_Coverage,                            \
                   bool _isFrontFace : $SV_IsFrontFace) :                      \
        $SV_Target                                                             \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        int2 _plsCoord = int2(floor(_fragCoord));                              \
        uint _plsIdx = _plsCoord.y * uniforms.renderTargetWidth + _plsCoord.x; \
        bool _clockwise = CLOCKWISE_FROM_FRONT_FACE(_isFrontFace);

#endif

#define EMIT_FRAG_DATA(VALUE)                                                  \
    return VALUE;                                                              \
    }
#ifdef @NEEDS_CLIP_DISTANCE
#define CLIP_CONTEXT_FORWARD , out float4 gl_ClipDistance
#define CLIP_CONTEXT_UNPACK , _varyings._clip
#else
#define CLIP_CONTEXT_FORWARD
#define CLIP_CONTEXT_UNPACK
#endif

#define FRAGMENT_CONTEXT_DECL , float2 _fragCoord
#define FRAGMENT_CONTEXT_UNPACK , _fragCoord

#define PLS_CONTEXT_DECL , int2 _plsCoord
#define PLS_CONTEXT_UNPACK , _plsCoord

#define PLS_MAIN(NAME)                                                         \
    $EARLYDEPTHSTENCIL void NAME(Varyings _varyings)                           \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        int2 _plsCoord = int2(floor(_fragCoord));                              \
        uint _plsIdx = _plsCoord.y * uniforms.renderTargetWidth + _plsCoord.x;

#define PLS_MAIN_WITH_IMAGE_UNIFORMS(NAME) PLS_MAIN(NAME)

#if defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@DRAW_IMAGE_MESH)
#define EMIT_PLS EMIT_PLS_AND_FRAG_COLOR
#else
#define EMIT_PLS }
#endif

#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    $EARLYDEPTHSTENCIL half4 NAME(Varyings _varyings) : $SV_Target             \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        int2 _plsCoord = int2(floor(_fragCoord));                              \
        uint _plsIdx = _plsCoord.y * uniforms.renderTargetWidth + _plsCoord.x; \
        half4 _fragColor;

#define PLS_FRAG_COLOR_MAIN_WITH_IMAGE_UNIFORMS(NAME) PLS_FRAG_COLOR_MAIN(NAME)

#define EMIT_PLS_AND_FRAG_COLOR                                                \
    }                                                                          \
    return _fragColor;

#define uintBitsToFloat $asfloat
#define intBitsToFloat $asfloat
#define floatBitsToInt $asint
#define floatBitsToUint $asuint
#define inversesqrt $rsqrt
#define equal(A, B) ((A) == (B))
#define notEqual(A, B) ((A) != (B))
#define lessThanEqual(A, B) ((A) <= (B))
#define lessThan(A, B) ((A) < (B))
#define greaterThan(A, B) ((A) > (B))
#define greaterThanEqual(A, B) ((A) >= (B))

// HLSL matrices are stored in row-major order, and therefore transposed from
// their counterparts in GLSL and Metal. We can work around this entirely by
// reversing the arguments to mul().
#define MUL(A, B) $mul(B, A)

#define VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
#define VERTEX_STORAGE_BUFFER_BLOCK_END

#define FRAG_STORAGE_BUFFER_BLOCK_BEGIN
#define FRAG_STORAGE_BUFFER_BLOCK_END

#define STORAGE_BUFFER_U32x2(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $StructuredBuffer<uint2> NAME
#define STORAGE_BUFFER_U32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $StructuredBuffer<uint4> NAME
#define STORAGE_BUFFER_F32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $StructuredBuffer<float4> NAME

#define STORAGE_BUFFER_LOAD4(NAME, I) NAME[I]
#define STORAGE_BUFFER_LOAD2(NAME, I) NAME[I]

INLINE half2 unpackHalf2x16(uint u)
{
    uint y = (u >> 16);
    uint x = u & 0xffffu;
    return half2($f16tof32(x), $f16tof32(y));
}

INLINE uint packHalf2x16(float2 v)
{
    uint x = $f32tof16(v.x);
    uint y = $f32tof16(v.y);
    return (y << 16) | x;
}

INLINE half4 unpackUnorm4x8(uint u)
{
    uint4 vals = uint4(u & 0xffu, (u >> 8) & 0xffu, (u >> 16) & 0xffu, u >> 24);
    return half4(vals) * (1. / 255.);
}

INLINE uint packUnorm4x8(half4 color)
{
    uint4 vals = (uint4(color * 255.) & 0xff) << uint4(0, 8, 16, 24);
    vals.rg |= vals.ba;
    vals.r |= vals.g;
    return vals.r;
}

INLINE float2x2 inverse(float2x2 m)
{
    float2x2 adjoint = float2x2(m[1][1], -m[0][1], -m[1][0], m[0][0]);
    return adjoint * (1. / determinant(m));
}

// Redirects for intrinsics that have different names in HLSL

INLINE float mix(float x, float y, float s) { return $lerp(x, y, s); }
INLINE float2 mix(float2 x, float2 y, float2 s) { return $lerp(x, y, s); }
INLINE float3 mix(float3 x, float3 y, float3 s) { return $lerp(x, y, s); }
INLINE float4 mix(float4 x, float4 y, float4 s) { return $lerp(x, y, s); }

INLINE float fract(float x) { return $frac(x); }
INLINE float2 fract(float2 x) { return $frac(x); }
INLINE float3 fract(float3 x) { return $frac(x); }
INLINE float4 fract(float4 x) { return $frac(x); }

INLINE float mod(float x, float y) { return $fmod(x, y); }

// Reimplement intrinsics for half types.
// This shadows the intrinsic function for floats, so we also have to declare
// that overload.

INLINE float rive_sign(float x) { return sign(x); }
INLINE float2 rive_sign(float2 x) { return sign(x); }
INLINE float3 rive_sign(float3 x) { return sign(x); }
INLINE float4 rive_sign(float4 x) { return sign(x); }

#define sign rive_sign

INLINE float rive_abs(float x) { return abs(x); }
INLINE float2 rive_abs(float2 x) { return abs(x); }
INLINE float3 rive_abs(float3 x) { return abs(x); }
INLINE float4 rive_abs(float4 x) { return abs(x); }

#define abs rive_abs

INLINE float rive_sqrt(float x) { return sqrt(x); }
INLINE float2 rive_sqrt(float2 x) { return sqrt(x); }
INLINE float3 rive_sqrt(float3 x) { return sqrt(x); }
INLINE float4 rive_sqrt(float4 x) { return sqrt(x); }

#define sqrt rive_sqrt
"###;

pub const PINNED_RHI_SOURCE: &str = PINNED_RHI_GLSL_SOURCE;
pub const RHI_GLSL_SOURCE: &str = PINNED_RHI_GLSL_SOURCE;
pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_RHI_GLSL_SOURCE
}

/// Every semantic preprocessor block in the pinned source remains literal,
/// including nested and mutually exclusive branch alternatives.
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
        block_id: "pp-0599",
        block_start: 19,
        block_end: 50,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0600",
        block_start: 54,
        block_end: 70,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0601",
        block_start: 56,
        block_end: 60,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0602",
        block_start: 64,
        block_end: 68,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0603",
        block_start: 108,
        block_end: 119,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0604",
        block_start: 125,
        block_end: 128,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0605",
        block_start: 130,
        block_end: 133,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0606",
        block_start: 140,
        block_end: 142,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0607",
        block_start: 157,
        block_end: 159,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0608",
        block_start: 190,
        block_end: 194,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0609",
        block_start: 196,
        block_end: 213,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0610",
        block_start: 198,
        block_end: 212,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0611",
        block_start: 218,
        block_end: 222,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0612",
        block_start: 229,
        block_end: 237,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0613",
        block_start: 239,
        block_end: 243,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0614",
        block_start: 245,
        block_end: 249,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0615",
        block_start: 252,
        block_end: 288,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0616",
        block_start: 299,
        block_end: 363,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0617",
        block_start: 367,
        block_end: 371,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0618",
        block_start: 373,
        block_end: 408,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0619",
        block_start: 413,
        block_end: 419,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0620",
        block_start: 436,
        block_end: 440,
        block_depth: 0,
        branch_count: 2,
    },
];

/// Every branch entry remains literal, in authority/source order. Active
/// paths are ledger spellings; they are not evaluated as Rust cfg expressions.
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
        block_id: "pp-0599",
        branch_ordinal: 1,
        branch_line: 19,
        directive: "#ifndef $_ARE_TOKEN_NAMES_PRESERVED",
        active_branch_path: "(!defined($_ARE_TOKEN_NAMES_PRESERVED))",
    },
    ConditionalBranch {
        block_id: "pp-0600",
        branch_ordinal: 1,
        branch_line: 54,
        directive: "#ifdef @ENABLE_MIN_16_PRECISION",
        active_branch_path: "(defined(@ENABLE_MIN_16_PRECISION))",
    },
    ConditionalBranch {
        block_id: "pp-0600",
        branch_ordinal: 2,
        branch_line: 62,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_MIN_16_PRECISION))))",
    },
    ConditionalBranch {
        block_id: "pp-0601",
        branch_ordinal: 1,
        branch_line: 56,
        directive: "#if NEEDS_USHORT_DEFINE",
        active_branch_path: "(defined(@ENABLE_MIN_16_PRECISION)) && (NEEDS_USHORT_DEFINE)",
    },
    ConditionalBranch {
        block_id: "pp-0602",
        branch_ordinal: 1,
        branch_line: 64,
        directive: "#if NEEDS_USHORT_DEFINE",
        active_branch_path: "(!((defined(@ENABLE_MIN_16_PRECISION)))) && (NEEDS_USHORT_DEFINE)",
    },
    ConditionalBranch {
        block_id: "pp-0603",
        branch_ordinal: 1,
        branch_line: 108,
        directive: "#ifdef @NEEDS_CLIP_DISTANCE",
        active_branch_path: "(defined(@NEEDS_CLIP_DISTANCE))",
    },
    ConditionalBranch {
        block_id: "pp-0603",
        branch_ordinal: 2,
        branch_line: 114,
        directive: "#else // !@NEEDS_CLIP_DISTANCE",
        active_branch_path: "(!((defined(@NEEDS_CLIP_DISTANCE))))",
    },
    ConditionalBranch {
        block_id: "pp-0604",
        branch_ordinal: 1,
        branch_line: 125,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0605",
        branch_ordinal: 1,
        branch_line: 130,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0606",
        branch_ordinal: 1,
        branch_line: 140,
        directive: "#ifdef @SOURCE_TEXTURE_MSAA",
        active_branch_path: "(defined(@SOURCE_TEXTURE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0607",
        branch_ordinal: 1,
        branch_line: 157,
        directive: "#ifdef SOURCE_TEXTURE_MSAA",
        active_branch_path: "(defined(SOURCE_TEXTURE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0608",
        branch_ordinal: 1,
        branch_line: 190,
        directive: "#ifdef @ENABLE_RASTERIZER_ORDERED_VIEWS",
        active_branch_path: "(defined(@ENABLE_RASTERIZER_ORDERED_VIEWS))",
    },
    ConditionalBranch {
        block_id: "pp-0608",
        branch_ordinal: 2,
        branch_line: 192,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_RASTERIZER_ORDERED_VIEWS))))",
    },
    ConditionalBranch {
        block_id: "pp-0609",
        branch_ordinal: 1,
        branch_line: 196,
        directive: "#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0610",
        branch_ordinal: 1,
        branch_line: 198,
        directive: "#ifdef @SUPPORTS_SUBPASS_LOAD",
        active_branch_path: "(defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)) && (defined(@SUPPORTS_SUBPASS_LOAD))",
    },
    ConditionalBranch {
        block_id: "pp-0610",
        branch_ordinal: 2,
        branch_line: 208,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)) && (!((defined(@SUPPORTS_SUBPASS_LOAD))))",
    },
    ConditionalBranch {
        block_id: "pp-0611",
        branch_ordinal: 1,
        branch_line: 218,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0611",
        branch_ordinal: 2,
        branch_line: 220,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
    ConditionalBranch {
        block_id: "pp-0612",
        branch_ordinal: 1,
        branch_line: 229,
        directive: "#if $COMPILER_METAL || $FORCE_ATOMIC_BUFFER",
        active_branch_path: "($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)",
    },
    ConditionalBranch {
        block_id: "pp-0612",
        branch_ordinal: 2,
        branch_line: 233,
        directive: "#else",
        active_branch_path: "(!(($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)))",
    },
    ConditionalBranch {
        block_id: "pp-0613",
        branch_ordinal: 1,
        branch_line: 239,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0613",
        branch_ordinal: 2,
        branch_line: 241,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
    ConditionalBranch {
        block_id: "pp-0614",
        branch_ordinal: 1,
        branch_line: 245,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0614",
        branch_ordinal: 2,
        branch_line: 247,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
    ConditionalBranch {
        block_id: "pp-0615",
        branch_ordinal: 1,
        branch_line: 252,
        directive: "#if $COMPILER_METAL || $FORCE_ATOMIC_BUFFER",
        active_branch_path: "($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)",
    },
    ConditionalBranch {
        block_id: "pp-0615",
        branch_ordinal: 2,
        branch_line: 270,
        directive: "#else",
        active_branch_path: "(!(($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)))",
    },
    ConditionalBranch {
        block_id: "pp-0616",
        branch_ordinal: 1,
        branch_line: 299,
        directive: "#ifdef @NO_VARYING",
        active_branch_path: "(defined(@NO_VARYING))",
    },
    ConditionalBranch {
        block_id: "pp-0616",
        branch_ordinal: 2,
        branch_line: 316,
        directive: "#else // !@NO_VARYING",
        active_branch_path: "(!((defined(@NO_VARYING))))",
    },
    ConditionalBranch {
        block_id: "pp-0617",
        branch_ordinal: 1,
        branch_line: 367,
        directive: "#if $COMPILER_DXC && ($COMPILER_VULKAN || $COMPILER_GLSL_ES3_1)",
        active_branch_path: "($COMPILER_DXC && ($COMPILER_VULKAN || $COMPILER_GLSL_ES3_1))",
    },
    ConditionalBranch {
        block_id: "pp-0617",
        branch_ordinal: 2,
        branch_line: 369,
        directive: "#else",
        active_branch_path: "(!(($COMPILER_DXC && ($COMPILER_VULKAN || $COMPILER_GLSL_ES3_1))))",
    },
    ConditionalBranch {
        block_id: "pp-0618",
        branch_ordinal: 1,
        branch_line: 373,
        directive: "#ifdef @NO_VARYING",
        active_branch_path: "(defined(@NO_VARYING))",
    },
    ConditionalBranch {
        block_id: "pp-0618",
        branch_ordinal: 2,
        branch_line: 387,
        directive: "#else",
        active_branch_path: "(!((defined(@NO_VARYING))))",
    },
    ConditionalBranch {
        block_id: "pp-0619",
        branch_ordinal: 1,
        branch_line: 413,
        directive: "#ifdef @NEEDS_CLIP_DISTANCE",
        active_branch_path: "(defined(@NEEDS_CLIP_DISTANCE))",
    },
    ConditionalBranch {
        block_id: "pp-0619",
        branch_ordinal: 2,
        branch_line: 416,
        directive: "#else",
        active_branch_path: "(!((defined(@NEEDS_CLIP_DISTANCE))))",
    },
    ConditionalBranch {
        block_id: "pp-0620",
        branch_ordinal: 1,
        branch_line: 436,
        directive: "#if defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@DRAW_IMAGE_MESH)",
        active_branch_path: "(defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0620",
        branch_ordinal: 2,
        branch_line: 438,
        directive: "#else",
        active_branch_path: "(!((defined(@FIXED_FUNCTION_COLOR_OUTPUT) && defined(@DRAW_IMAGE_MESH))))",
    },
];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The direct @-prefixed identifiers occurring in rhi.glsl, retained in
/// first-occurrence source order. Generated names are pinned minifier outputs.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 54,
        source_name: "@ENABLE_MIN_16_PRECISION",
        generated_name: "HE",
        generated_header_name: "GLSL_ENABLE_MIN_16_PRECISION",
    },
    ExportedSymbol {
        source_line: 104,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "OB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 108,
        source_name: "@NEEDS_CLIP_DISTANCE",
        generated_name: "QE",
        generated_header_name: "GLSL_NEEDS_CLIP_DISTANCE",
    },
    ExportedSymbol {
        source_line: 125,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 130,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 140,
        source_name: "@SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
        generated_header_name: "GLSL_SOURCE_TEXTURE_MSAA",
    },
    ExportedSymbol {
        source_line: 190,
        source_name: "@ENABLE_RASTERIZER_ORDERED_VIEWS",
        generated_name: "IE",
        generated_header_name: "GLSL_ENABLE_RASTERIZER_ORDERED_VIEWS",
    },
    ExportedSymbol {
        source_line: 196,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 198,
        source_name: "@SUPPORTS_SUBPASS_LOAD",
        generated_name: "MF",
        generated_header_name: "GLSL_SUPPORTS_SUBPASS_LOAD",
    },
    ExportedSymbol {
        source_line: 218,
        source_name: "@ENABLE_TYPED_UAV_LOAD_STORE",
        generated_name: "KC",
        generated_header_name: "GLSL_ENABLE_TYPED_UAV_LOAD_STORE",
    },
    ExportedSymbol {
        source_line: 299,
        source_name: "@NO_VARYING",
        generated_name: "RE",
        generated_header_name: "GLSL_NO_VARYING",
    },
    ExportedSymbol {
        source_line: 436,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 436,
        source_name: "@DRAW_IMAGE_MESH",
        generated_name: "LB",
        generated_header_name: "GLSL_DRAW_IMAGE_MESH",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "ENABLE_MIN_16_PRECISION",
    "OPTIONALLY_FLAT",
    "NEEDS_CLIP_DISTANCE",
    "VERTEX",
    "FRAGMENT",
    "SOURCE_TEXTURE_MSAA",
    "ENABLE_RASTERIZER_ORDERED_VIEWS",
    "RENDER_MODE_MSAA",
    "SUPPORTS_SUBPASS_LOAD",
    "ENABLE_TYPED_UAV_LOAD_STORE",
    "NO_VARYING",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "DRAW_IMAGE_MESH",
];

pub const EXPORT_MAPPING_AMBIGUITIES: &[(&str, &str, &str)] = &[];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Inline function declarations are retained as source spellings and ranges;
/// their bodies remain in the pinned source rather than becoming Rust code.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 253,
        end_line: 258,
        name: "pls_atomic_max",
        signature: "INLINE uint pls_atomic_max($RWBuffer<uint> plane, uint _plsIdx, uint x)",
        guard_path: "($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 262,
        end_line: 267,
        name: "pls_atomic_add",
        signature: "INLINE uint pls_atomic_add($RWBuffer<uint> plane, uint _plsIdx, uint x)",
        guard_path: "($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 271,
        end_line: 276,
        name: "pls_atomic_max",
        signature: "INLINE uint pls_atomic_max(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)",
        guard_path: "(!(($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 280,
        end_line: 285,
        name: "pls_atomic_add",
        signature: "INLINE uint pls_atomic_add(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)",
        guard_path: "(!(($COMPILER_METAL || $FORCE_ATOMIC_BUFFER)))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 489,
        end_line: 494,
        name: "unpackHalf2x16",
        signature: "INLINE half2 unpackHalf2x16(uint u)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 496,
        end_line: 501,
        name: "packHalf2x16",
        signature: "INLINE uint packHalf2x16(float2 v)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 503,
        end_line: 507,
        name: "unpackUnorm4x8",
        signature: "INLINE half4 unpackUnorm4x8(uint u)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 509,
        end_line: 515,
        name: "packUnorm4x8",
        signature: "INLINE uint packUnorm4x8(half4 color)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 517,
        end_line: 521,
        name: "inverse",
        signature: "INLINE float2x2 inverse(float2x2 m)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 525,
        end_line: 525,
        name: "mix",
        signature: "INLINE float mix(float x, float y, float s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 526,
        end_line: 526,
        name: "mix",
        signature: "INLINE float2 mix(float2 x, float2 y, float2 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 527,
        end_line: 527,
        name: "mix",
        signature: "INLINE float3 mix(float3 x, float3 y, float3 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 528,
        end_line: 528,
        name: "mix",
        signature: "INLINE float4 mix(float4 x, float4 y, float4 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 530,
        end_line: 530,
        name: "fract",
        signature: "INLINE float fract(float x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 531,
        end_line: 531,
        name: "fract",
        signature: "INLINE float2 fract(float2 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 532,
        end_line: 532,
        name: "fract",
        signature: "INLINE float3 fract(float3 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 533,
        end_line: 533,
        name: "fract",
        signature: "INLINE float4 fract(float4 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 535,
        end_line: 535,
        name: "mod",
        signature: "INLINE float mod(float x, float y) { return $fmod(x, y); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 541,
        end_line: 541,
        name: "rive_sign",
        signature: "INLINE float rive_sign(float x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 542,
        end_line: 542,
        name: "rive_sign",
        signature: "INLINE float2 rive_sign(float2 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 543,
        end_line: 543,
        name: "rive_sign",
        signature: "INLINE float3 rive_sign(float3 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 544,
        end_line: 544,
        name: "rive_sign",
        signature: "INLINE float4 rive_sign(float4 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 548,
        end_line: 548,
        name: "rive_abs",
        signature: "INLINE float rive_abs(float x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 549,
        end_line: 549,
        name: "rive_abs",
        signature: "INLINE float2 rive_abs(float2 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 550,
        end_line: 550,
        name: "rive_abs",
        signature: "INLINE float3 rive_abs(float3 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 551,
        end_line: 551,
        name: "rive_abs",
        signature: "INLINE float4 rive_abs(float4 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 555,
        end_line: 555,
        name: "rive_sqrt",
        signature: "INLINE float rive_sqrt(float x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 556,
        end_line: 556,
        name: "rive_sqrt",
        signature: "INLINE float2 rive_sqrt(float2 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 557,
        end_line: 557,
        name: "rive_sqrt",
        signature: "INLINE float3 rive_sqrt(float3 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 558,
        end_line: 558,
        name: "rive_sqrt",
        signature: "INLINE float4 rive_sqrt(float4 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "ENABLE_MIN_16_PRECISION",
        generated_name: "HE",
    },
    ExportedIdentifier {
        source_name: "OPTIONALLY_FLAT",
        generated_name: "OB",
    },
    ExportedIdentifier {
        source_name: "NEEDS_CLIP_DISTANCE",
        generated_name: "QE",
    },
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
    },
    ExportedIdentifier {
        source_name: "ENABLE_RASTERIZER_ORDERED_VIEWS",
        generated_name: "IE",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "SUPPORTS_SUBPASS_LOAD",
        generated_name: "MF",
    },
    ExportedIdentifier {
        source_name: "ENABLE_TYPED_UAV_LOAD_STORE",
        generated_name: "KC",
    },
    ExportedIdentifier {
        source_name: "NO_VARYING",
        generated_name: "RE",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_MESH",
        generated_name: "LB",
    },
];

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
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
