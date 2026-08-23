/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/hlsl.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, exports, functions, and dependencies as
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/hlsl.glsl";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "ccdbdadea1add6c67088c2b36e4a25975d01150412f7d0554a8933bd91cb337d";
pub const PINNED_SOURCE_LINE_COUNT: usize = 458;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18857;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/hlsl_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_path: &'static str,
    pub source_stage: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_target: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_path: PINNED_SOURCE_PATH,
    source_stage: PINNED_SOURCE_STAGE,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_target: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Exact pinned HLSL preprocessor source, retained for provenance and audit.
pub const PINNED_HLSL_GLSL_SOURCE: &str = r###"/*
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
#define float2x2 $float2x2
#define half3x3 $half3x3
#define half2x3 $half2x3
#define half4x4 $half4x4
#endif

$typedef float3 packed_float3;

#ifdef @ENABLE_MIN_16_PRECISION

// Use #define instead of typedef because typedef generates
// "error X3093: out of memory while parsing".
#define short $min16int
#define short2 $min16int2
#define ushort $min16uint

#else

// Use #define instead of typedef because typedef generates
// "error X3093: out of memory while parsing".
#define short int
#define short2 int2
#define ushort uint

#endif

#define INLINE $inline
#define OUT(ARG_TYPE) out ARG_TYPE
#define INOUT(ARG_TYPE) inout ARG_TYPE

#define ATTR_BLOCK_BEGIN(NAME)                                                 \
    struct NAME                                                                \
    {
#define ATTR(IDX, TYPE, NAME) TYPE NAME : NAME
#define ATTR_BLOCK_END                                                         \
    }                                                                          \
    ;
#define ATTR_LOAD(T, A, N, I)
#define ATTR_UNPACK(ID, attrs, NAME, TYPE) TYPE NAME = attrs.NAME

#define UNIFORM_BUFFER_REGISTER(IDX) $register($b##IDX)

#define UNIFORM_BLOCK_BEGIN(IDX, NAME)                                         \
    $cbuffer NAME : UNIFORM_BUFFER_REGISTER(IDX)                               \
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
#define VARYING(IDX, TYPE, NAME) TYPE NAME : $TEXCOORD##IDX

#define VARYING_BLOCK_END                                                      \
    float4 _pos : $SV_Position;                                                \
    }                                                                          \
    ;

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

#define TEXTURE_RGBA32UI(SET, IDX, NAME)                                       \
    uniform $Texture2D<uint4> NAME : $register($t##IDX)
#define TEXTURE_RGBA32F(SET, IDX, NAME)                                        \
    uniform $Texture2D<float4> NAME : $register($t##IDX)
#define TEXTURE_RGBA8(SET, IDX, NAME)                                          \
    uniform $Texture2D<$unorm float4> NAME : $register($t##IDX)
#define TEXTURE_R16F(SET, IDX, NAME)                                           \
    uniform $Texture2D<half> NAME : $register($t##IDX)
#define TEXTURE_R16F_1D_ARRAY(SET, IDX, NAME)                                  \
    uniform $Texture1DArray<half> NAME : $register($t##IDX)

// SAMPLER_LINEAR is the same as SAMPLER because in d3d11, sampler
// parameters are defined at the API level.
#define SAMPLER(IDX, NAME) $SamplerState NAME : $register($s##IDX);
#define SAMPLER_LINEAR SAMPLER
#define SAMPLER_DYNAMIC(SET, IDX, NAME) SAMPLER(IDX, NAME)
#define SAMPLER_DYNAMIC_IMAGE(NAME) SAMPLER(IMAGE_TEXTURE_IDX, NAME)

#define TEXEL_FETCH(NAME, COORD) NAME[COORD]
#define TEXTURE_SAMPLE(NAME, SAMPLER_NAME, COORD)                              \
    NAME.$Sample(SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_LOD(NAME, SAMPLER_NAME, COORD, LOD)                     \
    NAME.$SampleLevel(SAMPLER_NAME, COORD, LOD)
#define TEXTURE_SAMPLE_LODBIAS(NAME, SAMPLER_NAME, COORD, LODBIAS)             \
    NAME.$SampleBias(SAMPLER_NAME, COORD, LODBIAS)
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
    NAME.$SampleLevel(SAMPLER_NAME, float2(X, ARRAY_INDEX), LOD)

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

#define PLS_BLOCK_BEGIN
#ifdef @ENABLE_TYPED_UAV_LOAD_STORE
#define PLS_DECL4F(IDX, NAME)                                                  \
    uniform PLS_TEX2D<$unorm half4> NAME : $register($u##IDX)
#else
#define PLS_DECL4F(IDX, NAME) uniform PLS_TEX2D<uint> NAME : $register($u##IDX)
#endif
#define PLS_DECL4F_READONLY PLS_DECL4F
#define PLS_DECLUI(IDX, NAME) uniform PLS_TEX2D<uint> NAME : $register($u##IDX)
#define PLS_DECLUI_UAV PLS_DECLUI
#define PLS_LOADUI_UAV PLS_LOADUI
#define PLS_STOREUI_UAV PLS_STOREUI
#define PLS_BLOCK_END

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

#define PLS_PRESERVE_4F(PLANE)
#define PLS_PRESERVE_UI(PLANE)

#define VERTEX_CONTEXT_DECL
#define VERTEX_CONTEXT_UNPACK

#define TEXTURE_CONTEXT_DECL
#define TEXTURE_CONTEXT_FORWARD

#define CLIP_CONTEXT_FORWARD
#define CLIP_CONTEXT_UNPACK

#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
    $cbuffer DrawUniforms                                                      \
        : UNIFORM_BUFFER_REGISTER(PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX)       \
    {                                                                          \
        uint baseInstance;                                                     \
        uint NAME##_pad0;                                                      \
        uint NAME##_pad1;                                                      \
        uint NAME##_pad2;                                                      \
    }                                                                          \
    Varyings main(Attrs attrs,                                                 \
                  uint _vertexID : $SV_VertexID,                               \
                  uint _instanceIDWithoutBase : $SV_InstanceID)                \
    {                                                                          \
        uint _instanceID = _instanceIDWithoutBase + baseInstance;              \
        Varyings _varyings;

#define IMAGE_RECT_VERTEX_MAIN(NAME,                                           \
                               Attrs,                                          \
                               attrs,                                          \
                               ImageDrawAttrs,                                 \
                               imageDrawAttrs,                                 \
                               _vertexID,                                      \
                               _instanceID)                                    \
    Varyings main(Attrs attrs,                                                 \
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
    Varyings main(PositionAttr position,                                       \
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

#define FRAG_DATA_MAIN(DATA_TYPE, NAME)                                        \
    DATA_TYPE main(Varyings _varyings) : $SV_Target                            \
    {

#define FRAG_DATA_MAIN_WITH_CLOCKWISE(DATA_TYPE, NAME)                         \
    DATA_TYPE main(Varyings _varyings, bool _clockwise : $SV_IsFrontFace) :    \
        $SV_Target                                                             \
    {

#define EMIT_FRAG_DATA(VALUE)                                                  \
    return VALUE;                                                              \
    }

#define FRAGMENT_CONTEXT_DECL , float2 _fragCoord
#define FRAGMENT_CONTEXT_UNPACK , _fragCoord

#define PLS_CONTEXT_DECL , int2 _plsCoord
#define PLS_CONTEXT_UNPACK , _plsCoord

#define PLS_MAIN(NAME) [$earlydepthstencil] void main(Varyings _varyings) { \
        float2 _fragCoord = _varyings._pos.xy;\
        int2 _plsCoord = int2(floor(_fragCoord));

#define EMIT_PLS }

#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    [$earlydepthstencil] half4 main(Varyings _varyings) : $SV_Target           \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        int2 _plsCoord = int2(floor(_fragCoord));                              \
        half4 _fragColor;

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
    $StructuredBuffer<uint2> NAME : $register($t##IDX)
#define STORAGE_BUFFER_U32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $StructuredBuffer<uint4> NAME : $register($t##IDX)
#define STORAGE_BUFFER_F32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $StructuredBuffer<float4> NAME : $register($t##IDX)

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

// mix() with a boolean type in glsl maps to the hlsl ternary operator

INLINE float mix(float x, float y, bool s) { return s ? y : x; }
INLINE float2 mix(float2 x, float2 y, bool2 s) { return s ? y : x; }
INLINE float3 mix(float3 x, float3 y, bool3 s) { return s ? y : x; }
INLINE float4 mix(float4 x, float4 y, bool4 s) { return s ? y : x; }

INLINE half mix(half x, half y, bool s) { return s ? y : x; }
INLINE half2 mix(half2 x, half2 y, bool2 s) { return s ? y : x; }
INLINE half3 mix(half3 x, half3 y, bool3 s) { return s ? y : x; }
INLINE half4 mix(half4 x, half4 y, bool4 s) { return s ? y : x; }

// Redirects for intrinsics that have different names in HLSL

INLINE float mix(float x, float y, float s) { return $lerp(x, y, s); }
INLINE float2 mix(float2 x, float2 y, float2 s) { return $lerp(x, y, s); }
INLINE float3 mix(float3 x, float3 y, float3 s) { return $lerp(x, y, s); }
INLINE float4 mix(float4 x, float4 y, float4 s) { return $lerp(x, y, s); }

INLINE half mix(half x, half y, half s) { return $lerp(x, y, s); }
INLINE half2 mix(half2 x, half2 y, half2 s) { return $lerp(x, y, s); }
INLINE half3 mix(half3 x, half3 y, half3 s) { return $lerp(x, y, s); }
INLINE half4 mix(half4 x, half4 y, half4 s) { return $lerp(x, y, s); }

INLINE float fract(float x) { return $frac(x); }
INLINE float2 fract(float2 x) { return $frac(x); }
INLINE float3 fract(float3 x) { return $frac(x); }
INLINE float4 fract(float4 x) { return $frac(x); }

INLINE half fract(half x) { return $frac(x); }
INLINE half2 fract(half2 x) { return half2($frac(x)); }
INLINE half3 fract(half3 x) { return half3($frac(x)); }
INLINE half4 fract(half4 x) { return half4($frac(x)); }

INLINE float mod(float x, float y) { return $fmod(x, y); }

// Reimplement intrinsics for half types.
// This shadows the intrinsic function for floats, so we also have to declare
// that overload.

INLINE half rive_sign(half x) { return sign(x); }
INLINE half2 rive_sign(half2 x) { return half2(sign(x)); }
INLINE half3 rive_sign(half3 x) { return half3(sign(x)); }
INLINE half4 rive_sign(half4 x) { return half4(sign(x)); }

INLINE float rive_sign(float x) { return sign(x); }
INLINE float2 rive_sign(float2 x) { return sign(x); }
INLINE float3 rive_sign(float3 x) { return sign(x); }
INLINE float4 rive_sign(float4 x) { return sign(x); }

#define sign rive_sign

INLINE half rive_abs(half x) { return abs(x); }
INLINE half2 rive_abs(half2 x) { return half2(abs(x)); }
INLINE half3 rive_abs(half3 x) { return half3(abs(x)); }
INLINE half4 rive_abs(half4 x) { return half4(abs(x)); }

INLINE float rive_abs(float x) { return abs(x); }
INLINE float2 rive_abs(float2 x) { return abs(x); }
INLINE float3 rive_abs(float3 x) { return abs(x); }
INLINE float4 rive_abs(float4 x) { return abs(x); }

#define abs rive_abs

INLINE half rive_sqrt(half x) { return sqrt(x); }
INLINE half2 rive_sqrt(half2 x) { return half2(sqrt(x)); }
INLINE half3 rive_sqrt(half3 x) { return half3(sqrt(x)); }
INLINE half4 rive_sqrt(half4 x) { return half4(sqrt(x)); }

INLINE float rive_sqrt(float x) { return sqrt(x); }
INLINE float2 rive_sqrt(float2 x) { return sqrt(x); }
INLINE float3 rive_sqrt(float3 x) { return sqrt(x); }
INLINE float4 rive_sqrt(float4 x) { return sqrt(x); }

#define sqrt rive_sqrt
"###;

pub const PINNED_HLSL_SOURCE: &str = PINNED_HLSL_GLSL_SOURCE;
pub const HLSL_GLSL_SOURCE: &str = PINNED_HLSL_GLSL_SOURCE;
pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_HLSL_GLSL_SOURCE
}

/// Every semantic preprocessor block in the pinned source, in source order.
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
        block_id: "pp-0542",
        block_start: 19,
        block_end: 41,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0543",
        block_start: 45,
        block_end: 61,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0544",
        block_start: 108,
        block_end: 111,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0545",
        block_start: 113,
        block_end: 116,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0546",
        block_start: 167,
        block_end: 171,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0547",
        block_start: 174,
        block_end: 179,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0548",
        block_start: 187,
        block_end: 191,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0549",
        block_start: 193,
        block_end: 197,
        block_depth: 0,
        branch_count: 2,
    },
];

/// Every branch entry remains literal, in authority/source order.
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
        block_id: "pp-0542",
        branch_ordinal: 1,
        branch_line: 19,
        directive: "#ifndef $_ARE_TOKEN_NAMES_PRESERVED",
        active_branch_path: "(!defined($_ARE_TOKEN_NAMES_PRESERVED))",
    },
    ConditionalBranch {
        block_id: "pp-0543",
        branch_ordinal: 1,
        branch_line: 45,
        directive: "#ifdef @ENABLE_MIN_16_PRECISION",
        active_branch_path: "(defined(@ENABLE_MIN_16_PRECISION))",
    },
    ConditionalBranch {
        block_id: "pp-0543",
        branch_ordinal: 2,
        branch_line: 53,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_MIN_16_PRECISION))))",
    },
    ConditionalBranch {
        block_id: "pp-0544",
        branch_ordinal: 1,
        branch_line: 108,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0545",
        branch_ordinal: 1,
        branch_line: 113,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0546",
        branch_ordinal: 1,
        branch_line: 167,
        directive: "#ifdef @ENABLE_RASTERIZER_ORDERED_VIEWS",
        active_branch_path: "(defined(@ENABLE_RASTERIZER_ORDERED_VIEWS))",
    },
    ConditionalBranch {
        block_id: "pp-0546",
        branch_ordinal: 2,
        branch_line: 169,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_RASTERIZER_ORDERED_VIEWS))))",
    },
    ConditionalBranch {
        block_id: "pp-0547",
        branch_ordinal: 1,
        branch_line: 174,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0547",
        branch_ordinal: 2,
        branch_line: 177,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
    ConditionalBranch {
        block_id: "pp-0548",
        branch_ordinal: 1,
        branch_line: 187,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0548",
        branch_ordinal: 2,
        branch_line: 189,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
    ConditionalBranch {
        block_id: "pp-0549",
        branch_ordinal: 1,
        branch_line: 193,
        directive: "#ifdef @ENABLE_TYPED_UAV_LOAD_STORE",
        active_branch_path: "(defined(@ENABLE_TYPED_UAV_LOAD_STORE))",
    },
    ConditionalBranch {
        block_id: "pp-0549",
        branch_ordinal: 2,
        branch_line: 195,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_TYPED_UAV_LOAD_STORE))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The @-prefixed identifiers occurring directly in hlsl.glsl.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 45,
        source_name: "@ENABLE_MIN_16_PRECISION",
        generated_name: "HE",
        generated_header_name: "GLSL_ENABLE_MIN_16_PRECISION",
    },
    ExportedSymbol {
        source_line: 95,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "OB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 108,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 113,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 167,
        source_name: "@ENABLE_RASTERIZER_ORDERED_VIEWS",
        generated_name: "IE",
        generated_header_name: "GLSL_ENABLE_RASTERIZER_ORDERED_VIEWS",
    },
    ExportedSymbol {
        source_line: 174,
        source_name: "@ENABLE_TYPED_UAV_LOAD_STORE",
        generated_name: "KC",
        generated_header_name: "GLSL_ENABLE_TYPED_UAV_LOAD_STORE",
    },
];
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;
pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "ENABLE_MIN_16_PRECISION",
    "OPTIONALLY_FLAT",
    "VERTEX",
    "FRAGMENT",
    "ENABLE_RASTERIZER_ORDERED_VIEWS",
    "ENABLE_TYPED_UAV_LOAD_STORE",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Full shared export inventory from the pinned hlsl.glsl.exports.h.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "ATLAS_BLIT",
        generated_name: "EB",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_FILL",
        generated_name: "FC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_STROKE",
        generated_name: "MC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "ME",
    },
    ExportedIdentifier {
        source_name: "BASE_INSTANCE_UNIFORM_NAME",
        generated_name: "RD",
    },
    ExportedIdentifier {
        source_name: "BORROWED_COVERAGE_PASS",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "CLEAR_CLIP",
        generated_name: "JF",
    },
    ExportedIdentifier {
        source_name: "CLEAR_COLOR",
        generated_name: "JE",
    },
    ExportedIdentifier {
        source_name: "CLEAR_COVERAGE",
        generated_name: "TD",
    },
    ExportedIdentifier {
        source_name: "CLOCKWISE_FILL",
        generated_name: "UD",
    },
    ExportedIdentifier {
        source_name: "COALESCED_PLS_RESOLVE_AND_TRANSFER",
        generated_name: "SC",
    },
    ExportedIdentifier {
        source_name: "COLOR_PLANE_IDX_OVERRIDE",
        generated_name: "CE",
    },
    ExportedIdentifier {
        source_name: "DISABLE_ADVANCED_BLEND",
        generated_name: "NF",
    },
    ExportedIdentifier {
        source_name: "DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        generated_name: "EE",
    },
    ExportedIdentifier {
        source_name: "DISABLE_SHADER_STORAGE_BUFFERS",
        generated_name: "CF",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE",
        generated_name: "AE",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_MESH",
        generated_name: "LB",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_RECT",
        generated_name: "CD",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "DRAW_PATH",
        generated_name: "BD",
    },
    ExportedIdentifier {
        source_name: "DRAW_RENDER_TARGET_UPDATE_BOUNDS",
        generated_name: "TE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIPPING",
        generated_name: "O",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIP_RECT",
        generated_name: "AB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_DITHER",
        generated_name: "JB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_EVEN_ODD",
        generated_name: "PC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_FEATHER",
        generated_name: "HB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_HSL_BLEND_MODES",
        generated_name: "XB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_INSTANCE_INDEX",
        generated_name: "GE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_KHR_BLEND",
        generated_name: "ZD",
    },
    ExportedIdentifier {
        source_name: "ENABLE_MIN_16_PRECISION",
        generated_name: "HE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_RASTERIZER_ORDERED_VIEWS",
        generated_name: "IE",
    },
    ExportedIdentifier {
        source_name: "ENABLE_TYPED_UAV_LOAD_STORE",
        generated_name: "KC",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "FRAMEBUFFER_BOTTOM_UP",
        generated_name: "SF",
    },
    ExportedIdentifier {
        source_name: "FlushUniforms",
        generated_name: "NB",
    },
    ExportedIdentifier {
        source_name: "GLSL_VERSION",
        generated_name: "EC",
    },
    ExportedIdentifier {
        source_name: "GL_RENDERER_MALI",
        generated_name: "AF",
    },
    ExportedIdentifier {
        source_name: "INITIALIZE_PLS",
        generated_name: "BE",
    },
    ExportedIdentifier {
        source_name: "INPUT_ATTACHMENT_BINDING",
        generated_name: "SE",
    },
    ExportedIdentifier {
        source_name: "ImageDrawUniforms",
        generated_name: "LC",
    },
    ExportedIdentifier {
        source_name: "LOAD_COLOR",
        generated_name: "LE",
    },
    ExportedIdentifier {
        source_name: "LOAD_COLOR_FROM_DST_TEXTURE",
        generated_name: "FD",
    },
    ExportedIdentifier {
        source_name: "NEEDS_CLIP_DISTANCE",
        generated_name: "QE",
    },
    ExportedIdentifier {
        source_name: "NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
    },
    ExportedIdentifier {
        source_name: "NEEDS_PATH_ID_CLAMP_WORKAROUND",
        generated_name: "UE",
    },
    ExportedIdentifier {
        source_name: "NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
    },
    ExportedIdentifier {
        source_name: "NEVER_GENERATE_PREMULTIPLIED_PAINT_COLORS",
        generated_name: "RF",
    },
    ExportedIdentifier {
        source_name: "NO_VARYING",
        generated_name: "RE",
    },
    ExportedIdentifier {
        source_name: "OPTIONALLY_FLAT",
        generated_name: "OB",
    },
    ExportedIdentifier {
        source_name: "PLS_BLEND_SRC_OVER",
        generated_name: "OC",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_ANGLE",
        generated_name: "EXPORTED_PLS_IMPL_ANGLE",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_DEVICE_BUFFER",
        generated_name: "HF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        generated_name: "IF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_EXT_NATIVE",
        generated_name: "EF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_NONE",
        generated_name: "GF",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_STORAGE_BUFFER",
        generated_name: "PD",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_STORAGE_TEXTURE",
        generated_name: "QD",
    },
    ExportedIdentifier {
        source_name: "PLS_IMPL_SUBPASS_LOAD",
        generated_name: "FF",
    },
    ExportedIdentifier {
        source_name: "POST_INVERT_Y",
        generated_name: "JC",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "RESOLVE_PLS",
        generated_name: "IC",
    },
    ExportedIdentifier {
        source_name: "SOURCE_TEXTURE_MSAA",
        generated_name: "GD",
    },
    ExportedIdentifier {
        source_name: "STORE_COLOR",
        generated_name: "SD",
    },
    ExportedIdentifier {
        source_name: "STORE_COLOR_CLEAR",
        generated_name: "ED",
    },
    ExportedIdentifier {
        source_name: "SUPPORTS_SUBPASS_LOAD",
        generated_name: "MF",
    },
    ExportedIdentifier {
        source_name: "SWIZZLE_COLOR_BGRA_TO_RGBA",
        generated_name: "VE",
    },
    ExportedIdentifier {
        source_name: "TARGET_SPIRV",
        generated_name: "VB",
    },
    ExportedIdentifier {
        source_name: "TARGET_WGSL",
        generated_name: "FE",
    },
    ExportedIdentifier {
        source_name: "TESS_TEXTURE_FLOATING_POINT",
        generated_name: "ZE",
    },
    ExportedIdentifier {
        source_name: "USE_FILTERING",
        generated_name: "VC",
    },
    ExportedIdentifier {
        source_name: "USE_WEBGPU_SAMPLERS",
        generated_name: "BF",
    },
    ExportedIdentifier {
        source_name: "USING_PLS_STORAGE_TEXTURES",
        generated_name: "DF",
    },
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "VULKAN_VENDOR_ARM",
        generated_name: "WC",
    },
    ExportedIdentifier {
        source_name: "a_args",
        generated_name: "RB",
    },
    ExportedIdentifier {
        source_name: "a_color0",
        generated_name: "JD",
    },
    ExportedIdentifier {
        source_name: "a_color1",
        generated_name: "KD",
    },
    ExportedIdentifier {
        source_name: "a_contourIDWithFlags",
        generated_name: "YD",
    },
    ExportedIdentifier {
        source_name: "a_imageRectVertex",
        generated_name: "ZB",
    },
    ExportedIdentifier {
        source_name: "a_joinTan_and_ys",
        generated_name: "NC",
    },
    ExportedIdentifier {
        source_name: "a_mirroredVertexData",
        generated_name: "TB",
    },
    ExportedIdentifier {
        source_name: "a_p0p1_",
        generated_name: "ZC",
    },
    ExportedIdentifier {
        source_name: "a_p2p3_",
        generated_name: "AD",
    },
    ExportedIdentifier {
        source_name: "a_patchVertexData",
        generated_name: "SB",
    },
    ExportedIdentifier {
        source_name: "a_position",
        generated_name: "GC",
    },
    ExportedIdentifier {
        source_name: "a_reflectionX0X1",
        generated_name: "WD",
    },
    ExportedIdentifier {
        source_name: "a_segmentCounts",
        generated_name: "XD",
    },
    ExportedIdentifier {
        source_name: "a_span",
        generated_name: "CC",
    },
    ExportedIdentifier {
        source_name: "a_spanX",
        generated_name: "HD",
    },
    ExportedIdentifier {
        source_name: "a_texCoord",
        generated_name: "HC",
    },
    ExportedIdentifier {
        source_name: "a_triangleVertex",
        generated_name: "KB",
    },
    ExportedIdentifier {
        source_name: "a_x0x1",
        generated_name: "VD",
    },
    ExportedIdentifier {
        source_name: "a_yWithFlags",
        generated_name: "ID",
    },
    ExportedIdentifier {
        source_name: "atlasFillFragmentMain",
        generated_name: "NE",
    },
    ExportedIdentifier {
        source_name: "atlasRenderTexture",
        generated_name: "PE",
    },
    ExportedIdentifier {
        source_name: "atlasResolveVertexMain",
        generated_name: "LF",
    },
    ExportedIdentifier {
        source_name: "atlasStrokeFragmentMain",
        generated_name: "OE",
    },
    ExportedIdentifier {
        source_name: "atlasTexture",
        generated_name: "UC",
    },
    ExportedIdentifier {
        source_name: "atlasVertexMain",
        generated_name: "KF",
    },
    ExportedIdentifier {
        source_name: "blitFragmentMain",
        generated_name: "DE",
    },
    ExportedIdentifier {
        source_name: "blitVertexMain",
        generated_name: "WE",
    },
    ExportedIdentifier {
        source_name: "clearColor",
        generated_name: "KE",
    },
    ExportedIdentifier {
        source_name: "colorRampFragmentMain",
        generated_name: "YE",
    },
    ExportedIdentifier {
        source_name: "colorRampVertexMain",
        generated_name: "XE",
    },
    ExportedIdentifier {
        source_name: "contourBuffer",
        generated_name: "XC",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "drawVertexMain",
        generated_name: "YB",
    },
    ExportedIdentifier {
        source_name: "dstColorTexture",
        generated_name: "LD",
    },
    ExportedIdentifier {
        source_name: "featherTexture",
        generated_name: "QC",
    },
    ExportedIdentifier {
        source_name: "gradTexture",
        generated_name: "DD",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "paintAuxBuffer",
        generated_name: "PB",
    },
    ExportedIdentifier {
        source_name: "paintBuffer",
        generated_name: "TC",
    },
    ExportedIdentifier {
        source_name: "pathBuffer",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "sourceTexture",
        generated_name: "BC",
    },
    ExportedIdentifier {
        source_name: "stencilVertexMain",
        generated_name: "OF",
    },
    ExportedIdentifier {
        source_name: "tessVertexTexture",
        generated_name: "DC",
    },
    ExportedIdentifier {
        source_name: "tessellateFragmentMain",
        generated_name: "QF",
    },
    ExportedIdentifier {
        source_name: "tessellateVertexMain",
        generated_name: "PF",
    },
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

/// Function declarations and source macro entry points are retained as source ranges.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 200,
        end_line: 205,
        name: "pls_atomic_max",
        signature: "INLINE uint pls_atomic_max(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 209,
        end_line: 214,
        name: "pls_atomic_add",
        signature: "INLINE uint pls_atomic_add(PLS_TEX2D<uint> plane, int2 _plsCoord, uint x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 350,
        end_line: 355,
        name: "unpackHalf2x16",
        signature: "INLINE half2 unpackHalf2x16(uint u)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 357,
        end_line: 362,
        name: "packHalf2x16",
        signature: "INLINE uint packHalf2x16(float2 v)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 364,
        end_line: 368,
        name: "unpackUnorm4x8",
        signature: "INLINE half4 unpackUnorm4x8(uint u)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 370,
        end_line: 376,
        name: "packUnorm4x8",
        signature: "INLINE uint packUnorm4x8(half4 color)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 378,
        end_line: 382,
        name: "inverse",
        signature: "INLINE float2x2 inverse(float2x2 m)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 386,
        end_line: 386,
        name: "mix",
        signature: "INLINE float mix(float x, float y, bool s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 387,
        end_line: 387,
        name: "mix",
        signature: "INLINE float2 mix(float2 x, float2 y, bool2 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 388,
        end_line: 388,
        name: "mix",
        signature: "INLINE float3 mix(float3 x, float3 y, bool3 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 389,
        end_line: 389,
        name: "mix",
        signature: "INLINE float4 mix(float4 x, float4 y, bool4 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 391,
        end_line: 391,
        name: "mix",
        signature: "INLINE half mix(half x, half y, bool s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 392,
        end_line: 392,
        name: "mix",
        signature: "INLINE half2 mix(half2 x, half2 y, bool2 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 393,
        end_line: 393,
        name: "mix",
        signature: "INLINE half3 mix(half3 x, half3 y, bool3 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 394,
        end_line: 394,
        name: "mix",
        signature: "INLINE half4 mix(half4 x, half4 y, bool4 s) { return s ? y : x; }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 398,
        end_line: 398,
        name: "mix",
        signature: "INLINE float mix(float x, float y, float s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 399,
        end_line: 399,
        name: "mix",
        signature: "INLINE float2 mix(float2 x, float2 y, float2 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 400,
        end_line: 400,
        name: "mix",
        signature: "INLINE float3 mix(float3 x, float3 y, float3 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 401,
        end_line: 401,
        name: "mix",
        signature: "INLINE float4 mix(float4 x, float4 y, float4 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 403,
        end_line: 403,
        name: "mix",
        signature: "INLINE half mix(half x, half y, half s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 404,
        end_line: 404,
        name: "mix",
        signature: "INLINE half2 mix(half2 x, half2 y, half2 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 405,
        end_line: 405,
        name: "mix",
        signature: "INLINE half3 mix(half3 x, half3 y, half3 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 406,
        end_line: 406,
        name: "mix",
        signature: "INLINE half4 mix(half4 x, half4 y, half4 s) { return $lerp(x, y, s); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 408,
        end_line: 408,
        name: "fract",
        signature: "INLINE float fract(float x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 409,
        end_line: 409,
        name: "fract",
        signature: "INLINE float2 fract(float2 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 410,
        end_line: 410,
        name: "fract",
        signature: "INLINE float3 fract(float3 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 411,
        end_line: 411,
        name: "fract",
        signature: "INLINE float4 fract(float4 x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 413,
        end_line: 413,
        name: "fract",
        signature: "INLINE half fract(half x) { return $frac(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 414,
        end_line: 414,
        name: "fract",
        signature: "INLINE half2 fract(half2 x) { return half2($frac(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 415,
        end_line: 415,
        name: "fract",
        signature: "INLINE half3 fract(half3 x) { return half3($frac(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 416,
        end_line: 416,
        name: "fract",
        signature: "INLINE half4 fract(half4 x) { return half4($frac(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 418,
        end_line: 418,
        name: "mod",
        signature: "INLINE float mod(float x, float y) { return $fmod(x, y); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 424,
        end_line: 424,
        name: "rive_sign",
        signature: "INLINE half rive_sign(half x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 425,
        end_line: 425,
        name: "rive_sign",
        signature: "INLINE half2 rive_sign(half2 x) { return half2(sign(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 426,
        end_line: 426,
        name: "rive_sign",
        signature: "INLINE half3 rive_sign(half3 x) { return half3(sign(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 427,
        end_line: 427,
        name: "rive_sign",
        signature: "INLINE half4 rive_sign(half4 x) { return half4(sign(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 429,
        end_line: 429,
        name: "rive_sign",
        signature: "INLINE float rive_sign(float x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 430,
        end_line: 430,
        name: "rive_sign",
        signature: "INLINE float2 rive_sign(float2 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 431,
        end_line: 431,
        name: "rive_sign",
        signature: "INLINE float3 rive_sign(float3 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 432,
        end_line: 432,
        name: "rive_sign",
        signature: "INLINE float4 rive_sign(float4 x) { return sign(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 436,
        end_line: 436,
        name: "rive_abs",
        signature: "INLINE half rive_abs(half x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 437,
        end_line: 437,
        name: "rive_abs",
        signature: "INLINE half2 rive_abs(half2 x) { return half2(abs(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 438,
        end_line: 438,
        name: "rive_abs",
        signature: "INLINE half3 rive_abs(half3 x) { return half3(abs(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 439,
        end_line: 439,
        name: "rive_abs",
        signature: "INLINE half4 rive_abs(half4 x) { return half4(abs(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 441,
        end_line: 441,
        name: "rive_abs",
        signature: "INLINE float rive_abs(float x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 442,
        end_line: 442,
        name: "rive_abs",
        signature: "INLINE float2 rive_abs(float2 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 443,
        end_line: 443,
        name: "rive_abs",
        signature: "INLINE float3 rive_abs(float3 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 444,
        end_line: 444,
        name: "rive_abs",
        signature: "INLINE float4 rive_abs(float4 x) { return abs(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 448,
        end_line: 448,
        name: "rive_sqrt",
        signature: "INLINE half rive_sqrt(half x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 449,
        end_line: 449,
        name: "rive_sqrt",
        signature: "INLINE half2 rive_sqrt(half2 x) { return half2(sqrt(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 450,
        end_line: 450,
        name: "rive_sqrt",
        signature: "INLINE half3 rive_sqrt(half3 x) { return half3(sqrt(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 451,
        end_line: 451,
        name: "rive_sqrt",
        signature: "INLINE half4 rive_sqrt(half4 x) { return half4(sqrt(x)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 453,
        end_line: 453,
        name: "rive_sqrt",
        signature: "INLINE float rive_sqrt(float x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 454,
        end_line: 454,
        name: "rive_sqrt",
        signature: "INLINE float2 rive_sqrt(float2 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 455,
        end_line: 455,
        name: "rive_sqrt",
        signature: "INLINE float3 rive_sqrt(float3 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 456,
        end_line: 456,
        name: "rive_sqrt",
        signature: "INLINE float4 rive_sqrt(float4 x) { return sqrt(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
];
pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

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

/// hlsl.glsl has no direct #include/#import directive or pinned source dependency edge.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;

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
