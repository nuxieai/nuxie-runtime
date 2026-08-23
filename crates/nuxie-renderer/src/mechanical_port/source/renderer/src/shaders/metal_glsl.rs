/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/metal.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, functions, and dependencies as literal
 * source-shaped data. It does not compile, evaluate, simplify, or generate
 * shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "4a2cc45e01b2fa4d9a5e6428cae8f721f3ce2fe0dc143a326d84f12a9cd38794";
pub const PINNED_SOURCE_LINE_COUNT: usize = 531;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 26890;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/metal_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_METAL_GLSL_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

// This header provides Metal-specific #defines and declarations that enable our
// shaders to be compiled on MSL and GLSL both.

#define METAL

// #define native metal types if their names are being rewritten.
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
#define packed_float3 $packed_float3
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

#define INLINE $inline
#define OUT(ARG_TYPE) $thread ARG_TYPE&
#define INOUT(ARG_TYPE) $thread ARG_TYPE&

#define equal(A, B) ((A) == (B))
#define notEqual(A, B) ((A) != (B))
#define lessThanEqual(A, B) ((A) <= (B))
#define lessThan(A, B) ((A) < (B))
#define greaterThan(A, B) ((A) > (B))
#define greaterThanEqual(A, B) ((A) >= (B))
#define MUL(A, B) ((A) * (B))
#define inversesqrt $rsqrt

#define UNIFORM_BLOCK_BEGIN(IDX, NAME)                                         \
    struct NAME                                                                \
    {
#define UNIFORM_BLOCK_END(NAME)                                                \
    }                                                                          \
    ;

#define ATTR_BLOCK_BEGIN(NAME)                                                 \
    struct NAME                                                                \
    {
#define ATTR(IDX, TYPE, NAME) TYPE NAME
#define ATTR_BLOCK_END                                                         \
    }                                                                          \
    ;
#define ATTR_UNPACK(ID, attrs, NAME, TYPE) TYPE NAME = attrs[ID].NAME

#define VARYING_BLOCK_BEGIN                                                    \
    struct Varyings                                                            \
    {
#define VARYING(IDX, TYPE, NAME) TYPE NAME
#define FLAT [[flat]]
#define NO_PERSPECTIVE [[$center_no_perspective]]
#ifndef @OPTIONALLY_FLAT
// Don't use no-perspective interpolation for varyings that need to be flat.
// No-persective interpolation appears to break the guarantee that a varying ==
// "x" when all barycentric values also == "x". Default (perspective-correct)
// interpolation does preserve this guarantee, and seems to be faster faster
// than flat on Apple Silicon.
#define @OPTIONALLY_FLAT
#endif
#define VARYING_BLOCK_END                                                      \
    float4 _pos [[$position]] [[$invariant]];                                  \
    }                                                                          \
    ;

#define VARYING_INIT(NAME, TYPE) $thread TYPE& NAME = _varyings.NAME
#define VARYING_PACK(NAME)
#define VARYING_UNPACK(NAME, TYPE) TYPE NAME = _varyings.NAME

#define VERTEX_STORAGE_BUFFER_BLOCK_BEGIN                                      \
    struct VertexStorageBuffers                                                \
    {
#define VERTEX_STORAGE_BUFFER_BLOCK_END                                        \
    }                                                                          \
    ;

#define FRAG_STORAGE_BUFFER_BLOCK_BEGIN                                        \
    struct FragmentStorageBuffers                                              \
    {
#define FRAG_STORAGE_BUFFER_BLOCK_END                                          \
    }                                                                          \
    ;

#define STORAGE_BUFFER_U32x2(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $constant uint2* NAME [[$buffer(METAL_BUFFER_IDX(IDX))]]
#define STORAGE_BUFFER_U32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $constant uint4* NAME [[$buffer(METAL_BUFFER_IDX(IDX))]]
#define STORAGE_BUFFER_F32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    $constant float4* NAME [[$buffer(METAL_BUFFER_IDX(IDX))]]
#define STORAGE_BUFFER_LOAD4(NAME, I) _buffers.NAME[I]
#define STORAGE_BUFFER_LOAD2(NAME, I) _buffers.NAME[I]

#define VERTEX_TEXTURE_BLOCK_BEGIN                                             \
    struct VertexTextures                                                      \
    {
#define VERTEX_TEXTURE_BLOCK_END                                               \
    }                                                                          \
    ;

#define FRAG_TEXTURE_BLOCK_BEGIN                                               \
    struct FragmentTextures                                                    \
    {
#define FRAG_TEXTURE_BLOCK_END                                                 \
    }                                                                          \
    ;

#define DYNAMIC_SAMPLER_BLOCK_BEGIN                                            \
    struct DynamicSamplers                                                     \
    {
#define DYNAMIC_SAMPLER_BLOCK_END                                              \
    }                                                                          \
    ;

#define TEXTURE_RGBA32UI(SET, IDX, NAME) [[$texture(IDX)]] $texture2d<uint> NAME
#define TEXTURE_RGBA32F(SET, IDX, NAME) [[$texture(IDX)]] $texture2d<float> NAME
#define TEXTURE_RGBA8(SET, IDX, NAME) [[$texture(IDX)]] $texture2d<half> NAME
#define TEXTURE_R16F(SET, IDX, NAME) [[$texture(IDX)]] $texture2d<half> NAME
#define TEXTURE_R16F_1D_ARRAY(SET, IDX, NAME)                                  \
    [[$texture(IDX)]] $texture1d_array<half> NAME

#define SAMPLER_LINEAR(TEXTURE_IDX, NAME)                                      \
    $constexpr $sampler NAME($filter::$linear, $mip_filter::$none);
#define SAMPLER_DYNAMIC(SET, IDX, NAME) [[$sampler(IDX)]] $sampler NAME;
#define SAMPLER_DYNAMIC_IMAGE(NAME)                                            \
    [[$sampler(IMAGE_TEXTURE_IDX)]] $sampler NAME;
#define TEXEL_FETCH(TEXTURE, COORD) _textures.TEXTURE.$read(uint2(COORD))
#define TEXTURE_SAMPLE(TEXTURE, SAMPLER_NAME, COORD)                           \
    _textures.TEXTURE.$sample(SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)                  \
    _textures.TEXTURE.$sample(SAMPLER_NAME, COORD, $level(LOD))
#define TEXTURE_SAMPLE_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)          \
    _textures.TEXTURE.$sample(SAMPLER_NAME, COORD, $bias(LODBIAS))
#define TEXTURE_SAMPLE_GRAD(TEXTURE, SAMPLER_NAME, COORD, DDX, DDY)            \
    _textures.TEXTURE.$sample(SAMPLER_NAME, COORD, $gradient2d(DDX, DDY))
#define TEXTURE_GATHER(TEXTURE, SAMPLER_NAME, COORD, TEXTURE_INVERSE_SIZE)     \
    _textures.TEXTURE.$gather(SAMPLER_NAME, (COORD) * (TEXTURE_INVERSE_SIZE))
#define TEXTURE_SAMPLE_DYNAMIC(TEXTURE, SAMPLER_NAME, COORD)                   \
    _textures.TEXTURE.$sample(_dynamicSampler.SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_DYNAMIC_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)          \
    _textures.TEXTURE.$sample(_dynamicSampler.SAMPLER_NAME, COORD, $level(LOD))
#define TEXTURE_SAMPLE_DYNAMIC_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)  \
    _textures.TEXTURE.$sample(_dynamicSampler.SAMPLER_NAME,                    \
                              COORD,                                           \
                              $bias(LODBIAS))
#define TEXTURE_SAMPLE_LOD_1D_ARRAY(TEXTURE,                                   \
                                    SAMPLER_NAME,                              \
                                    X,                                         \
                                    ARRAY_INDEX,                               \
                                    ARRAY_INDEX_NORMALIZED,                    \
                                    LOD)                                       \
    _textures.TEXTURE.$sample(SAMPLER_NAME, X, ARRAY_INDEX)

#define VERTEX_CONTEXT_DECL                                                    \
    , $constant @FlushUniforms &uniforms, VertexTextures _textures,            \
        VertexStorageBuffers _buffers
#define VERTEX_CONTEXT_UNPACK , uniforms, _textures, _buffers

#ifdef @ENABLE_INSTANCE_INDEX
#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
    $__attribute__(($visibility("default"))) Varyings $vertex NAME(            \
        uint _vertexID [[$vertex_id]],                                         \
        uint _instanceID [[$instance_id]],                                     \
        $constant uint& _baseInstance                                          \
        [[$buffer(METAL_BUFFER_IDX(PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX))]],  \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        $constant Attrs* attrs [[$buffer(0)]],                                 \
        VertexTextures _textures,                                              \
        VertexStorageBuffers _buffers)                                         \
    {                                                                          \
        _instanceID += _baseInstance;                                          \
        Varyings _varyings;
#else
#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
    $__attribute__(($visibility("default"))) Varyings $vertex NAME(            \
        uint _vertexID [[$vertex_id]],                                         \
        uint _instanceID [[$instance_id]],                                     \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        $constant Attrs* attrs [[$buffer(0)]],                                 \
        VertexTextures _textures,                                              \
        VertexStorageBuffers _buffers)                                         \
    {                                                                          \
        Varyings _varyings;
#endif

#define IMAGE_RECT_VERTEX_MAIN(NAME,                                           \
                               Attrs,                                          \
                               attrs,                                          \
                               ImageDrawAttrs,                                 \
                               imageDrawAttrs,                                 \
                               _vertexID,                                      \
                               _instanceID)                                    \
    $__attribute__(($visibility("default"))) Varyings $vertex NAME(            \
        uint _vertexID [[$vertex_id]],                                         \
        uint _instanceID [[$instance_id]],                                     \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        $constant Attrs* attrs [[$buffer(0)]],                                 \
        $constant ImageDrawAttrs* imageDrawAttrs [[$buffer(2)]],               \
        VertexTextures _textures,                                              \
        VertexStorageBuffers _buffers)                                         \
    {                                                                          \
        Varyings _varyings;

#define IMAGE_MESH_VERTEX_MAIN(NAME,                                           \
                               PositionAttr,                                   \
                               position,                                       \
                               UVAttr,                                         \
                               uv,                                             \
                               ImageDrawAttrs,                                 \
                               imageDrawAttrs,                                 \
                               _vertexID)                                      \
    $__attribute__(($visibility("default"))) Varyings $vertex NAME(            \
        uint _vertexID [[$vertex_id]],                                         \
        uint _instanceID [[$instance_id]],                                     \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        $constant PositionAttr* position [[$buffer(0)]],                       \
        $constant UVAttr* uv [[$buffer(1)]],                                   \
        $constant ImageDrawAttrs* imageDrawAttrs [[$buffer(2)]])               \
    {                                                                          \
        Varyings _varyings;

#define EMIT_VERTEX(POSITION)                                                  \
    _varyings._pos = POSITION;                                                 \
    }                                                                          \
    return _varyings;

#define FRAG_DATA_MAIN(DATA_TYPE, NAME)                                        \
    DATA_TYPE $__attribute__(($visibility("default"))) $fragment NAME(         \
        Varyings _varyings [[$stage_in]],                                      \
        FragmentTextures _textures)                                            \
    {

#define FRAG_DATA_MAIN_WITH_CLOCKWISE(DATA_TYPE, NAME)                         \
    DATA_TYPE $__attribute__(($visibility("default"))) $fragment NAME(         \
        Varyings _varyings [[$stage_in]],                                      \
        FragmentTextures _textures,                                            \
        bool _clockwise [[$front_facing]])                                     \
    {

#define EMIT_FRAG_DATA(VALUE)                                                  \
    return VALUE;                                                              \
    }

#define FRAGMENT_CONTEXT_DECL                                                  \
    , float2 _fragCoord, FragmentTextures _textures,                           \
        FragmentStorageBuffers _buffers, DynamicSamplers _dynamicSampler
#define FRAGMENT_CONTEXT_UNPACK                                                \
    , _fragCoord, _textures, _buffers, _dynamicSampler

#define TEXTURE_CONTEXT_DECL , FragmentTextures _textures
#define TEXTURE_CONTEXT_FORWARD , _textures

#define CLIP_CONTEXT_FORWARD
#define CLIP_CONTEXT_UNPACK

#ifdef @PLS_IMPL_DEVICE_BUFFER

#define PLS_BLOCK_BEGIN                                                        \
    struct PLS                                                                 \
    {
#ifdef @PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
// Apple Silicon doesn't support fragment-fragment memory barriers, so on this
// hardware we use raster order groups instead. Since the PLS plane indices
// collide with other buffer bindings, offset the binding indices of these
// buffers by DEFAULT_BINDINGS_SET_SIZE.
#define PLS_DECL4F(IDX, NAME)                                                  \
    $device uint* NAME                                                         \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE)),          \
          $raster_order_group(0)]]
#define PLS_DECLUI(IDX, NAME)                                                  \
    $device uint* NAME                                                         \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE)),          \
          $raster_order_group(0)]]
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    $device $atomic_uint* NAME                                                 \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE)),          \
          $raster_order_group(0)]]
#else
// Since the PLS plane indices collide with other buffer bindings, offset the
// binding indices of these buffers by DEFAULT_BINDINGS_SET_SIZE.
#define PLS_DECL4F(IDX, NAME)                                                  \
    $device uint* NAME                                                         \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE))]]
#define PLS_DECLUI(IDX, NAME)                                                  \
    $device uint* NAME                                                         \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE))]]
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    $device $atomic_uint* NAME                                                 \
        [[$buffer(METAL_BUFFER_IDX(IDX + DEFAULT_BINDINGS_SET_SIZE))]]
#endif // @PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED
#define PLS_BLOCK_END                                                          \
    }                                                                          \
    ;
#define PLS_CONTEXT_DECL , PLS _pls, uint _plsIdx
#define PLS_CONTEXT_UNPACK , _pls, _plsIdx

#define PLS_LOAD4F(PLANE) unpackUnorm4x8(_pls.PLANE[_plsIdx])
#define PLS_LOADUI(PLANE) _pls.PLANE[_plsIdx]
#define PLS_LOADUI_UAV(PLANE)                                                  \
    $atomic_load_explicit(&_pls.PLANE[_plsIdx],                                \
                          $memory_order::$memory_order_relaxed)
#define PLS_STORE4F(PLANE, VALUE) _pls.PLANE[_plsIdx] = packUnorm4x8(VALUE)
#define PLS_STOREUI(PLANE, VALUE) _pls.PLANE[_plsIdx] = (VALUE)
#define PLS_STOREUI_UAV(PLANE, VALUE)                                          \
    $atomic_store_explicit(&_pls.PLANE[_plsIdx],                               \
                           VALUE,                                              \
                           $memory_order::$memory_order_relaxed)
#define PLS_PRESERVE_4F(PLANE)
#define PLS_PRESERVE_UI(PLANE)

#define PLS_ATOMIC_MAX(PLANE, X)                                               \
    $atomic_fetch_max_explicit(&_pls.PLANE[_plsIdx],                           \
                               X,                                              \
                               $memory_order::$memory_order_relaxed)

#define PLS_ATOMIC_ADD(PLANE, X)                                               \
    $atomic_fetch_add_explicit(&_pls.PLANE[_plsIdx],                           \
                               X,                                              \
                               $memory_order::$memory_order_relaxed)

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#define PLS_METAL_MAIN(NAME)                                                   \
    $__attribute__(($visibility("default"))) $fragment NAME(                   \
        PLS _pls,                                                              \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        Varyings _varyings [[$stage_in]],                                      \
        FragmentTextures _textures,                                            \
        DynamicSamplers _dynamicSampler,                                       \
        FragmentStorageBuffers _buffers)                                       \
    {                                                                          \
        float2 _fragCoord = _varyings._pos.xy;                                 \
        uint2 _plsCoord = uint2($metal::floor(_fragCoord));                    \
        uint _plsIdx = _plsCoord.y * uniforms.renderTargetWidth + _plsCoord.x;

#define PLS_MAIN(NAME) void PLS_METAL_MAIN(NAME)
#define EMIT_PLS }

#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    half4 PLS_METAL_MAIN(NAME)                                                 \
    {                                                                          \
        half4 _fragColor;

#define EMIT_PLS_AND_FRAG_COLOR                                                \
    }                                                                          \
    return _fragColor;                                                         \
    EMIT_PLS

#else // Default implementation -- framebuffer reads.

#define PLS_BLOCK_BEGIN                                                        \
    struct PLS                                                                 \
    {
#define PLS_DECL4F(IDX, NAME) [[$color(IDX)]] half4 NAME
#define PLS_DECLUI(IDX, NAME) [[$color(IDX)]] uint NAME
#define PLS_DECLUI_UAV PLS_DECLUI
#define PLS_BLOCK_END                                                          \
    }                                                                          \
    ;
#define PLS_CONTEXT_DECL , $thread PLS &_inpls, $thread PLS &_pls
#define PLS_CONTEXT_UNPACK , _inpls, _pls

#define PLS_LOAD4F(PLANE) _inpls.PLANE
#define PLS_LOADUI(PLANE) _inpls.PLANE
#define PLS_LOADUI_UAV(PLANE) PLS_LOADUI
#define PLS_STORE4F(PLANE, VALUE) _pls.PLANE = (VALUE)
#define PLS_STOREUI(PLANE, VALUE) _pls.PLANE = (VALUE)
#define PLS_STOREUI_UAV(PLANE) PLS_STOREUI
#define PLS_PRESERVE_4F(PLANE) _pls.PLANE = _inpls.PLANE
#define PLS_PRESERVE_UI(PLANE) _pls.PLANE = _inpls.PLANE

INLINE uint pls_atomic_max($thread uint& dst, uint x)
{
    uint originalValue = dst;
    dst = $metal::max(originalValue, x);
    return originalValue;
}

#define PLS_ATOMIC_MAX(PLANE, X) pls_atomic_max(_pls.PLANE, X)

INLINE uint pls_atomic_add($thread uint& dst, uint x)
{
    uint originalValue = dst;
    dst = originalValue + x;
    return originalValue;
}

#define PLS_ATOMIC_ADD(PLANE, X) pls_atomic_add(_pls.PLANE, X)

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#define PLS_METAL_MAIN(NAME, ...)                                              \
    PLS $__attribute__(($visibility("default"))) $fragment NAME($__VA_ARGS__)  \
    {                                                                          \
        float2 _fragCoord [[$maybe_unused]] = _varyings._pos.xy;               \
        PLS _pls;

#define PLS_MAIN(NAME, ...)                                                    \
    PLS_METAL_MAIN(NAME,                                                       \
                   PLS _inpls,                                                 \
                   $constant @FlushUniforms& uniforms                          \
                   [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],    \
                   Varyings _varyings [[$stage_in]],                           \
                   DynamicSamplers _dynamicSampler,                            \
                   FragmentTextures _textures,                                 \
                   FragmentStorageBuffers _buffers)

#define EMIT_PLS                                                               \
    }                                                                          \
    return _pls;

#define PLS_FRAG_COLOR_METAL_MAIN(NAME, ...)                                   \
    struct FragmentOut                                                         \
    {                                                                          \
        half4 _color [[color(0)]];                                             \
        PLS _pls;                                                              \
    };                                                                         \
    FragmentOut $__attribute__(($visibility("default"))) $fragment NAME(       \
        $__VA_ARGS__)                                                          \
    {                                                                          \
        float2 _fragCoord [[$maybe_unused]] = _varyings._pos.xy;               \
        half4 _fragColor;                                                      \
        PLS _pls;

#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    PLS_FRAG_COLOR_METAL_MAIN(                                                 \
        NAME,                                                                  \
        PLS _inpls,                                                            \
        $constant @FlushUniforms& uniforms                                     \
        [[$buffer(METAL_BUFFER_IDX(FLUSH_UNIFORM_BUFFER_IDX))]],               \
        Varyings _varyings [[$stage_in]],                                      \
        FragmentTextures _textures,                                            \
        FragmentStorageBuffers _buffers)

#define EMIT_PLS_AND_FRAG_COLOR                                                \
    }                                                                          \
    return {._color = _fragColor, ._pls = _pls};

#endif // PLS_IMPL_DEVICE_BUFFER

#define PLS_DECL4F_READONLY PLS_DECL4F

#define discard $discard_fragment()

$using $namespace $metal;

$template<int N> INLINE $vec<uint, N> floatBitsToUint($vec<float, N> x)
{
    return $as_type<$vec<uint, N>>(x);
}

$template<int N> INLINE $vec<int, N> floatBitsToInt($vec<float, N> x)
{
    return $as_type<$vec<int, N>>(x);
}

INLINE uint floatBitsToUint(float x) { return $as_type<uint>(x); }

INLINE int floatBitsToInt(float x) { return $as_type<int>(x); }

$template<int N> INLINE $vec<float, N> uintBitsToFloat($vec<uint, N> x)
{
    return $as_type<$vec<float, N>>(x);
}

INLINE float uintBitsToFloat(uint x) { return $as_type<float>(x); }
INLINE half2 unpackHalf2x16(uint x) { return $as_type<half2>(x); }
INLINE uint packHalf2x16(half2 x) { return $as_type<uint>(x); }
INLINE half4 unpackUnorm4x8(uint x) { return $unpack_unorm4x8_to_half(x); }
INLINE uint packUnorm4x8(half4 x) { return $pack_half_to_unorm4x8(x); }

INLINE float2x2 inverse(float2x2 m)
{
    float2x2 m_ = float2x2(m[1][1], -m[0][1], -m[1][0], m[0][0]);
    float det = (m_[0][0] * m[0][0]) + (m_[0][1] * m[1][0]);
    return m_ * (1 / det);
}

INLINE half3 mix(half3 a, half3 b, bool3 c)
{
    half3 result;
    for (int i = 0; i < 3; ++i)
        result[i] = c[i] ? b[i] : a[i];
    return result;
}

INLINE float2 mix(float2 a, float2 b, bool2 c)
{
    float2 result;
    for (int i = 0; i < 2; ++i)
        result[i] = c[i] ? b[i] : a[i];
    return result;
}

INLINE float2 mix(float2 a, float2 b, float t) { return mix(a, b, float2(t)); }

INLINE float mod(float x, float y) { return $fmod(x, y); }
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_METAL_SOURCE: &str = PINNED_METAL_GLSL_SOURCE;
pub const METAL_GLSL_SOURCE: &str = PINNED_METAL_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_METAL_GLSL_SOURCE
}

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

/// Every semantic preprocessor block in the pinned source remains literal,
/// including nested device-buffer and raster-order branches.
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
        block_id: "pp-0553",
        block_start: 12,
        block_end: 44,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0554",
        block_start: 81,
        block_end: 88,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0555",
        block_start: 186,
        block_end: 213,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0556",
        block_start: 287,
        block_end: 473,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0557",
        block_start: 292,
        block_end: 321,
        block_depth: 1,
        branch_count: 2,
    },
];

/// Every branch entry remains literal, in authority/source order. The active
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
        block_id: "pp-0553",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifndef $_ARE_TOKEN_NAMES_PRESERVED",
        active_branch_path: "(!defined($_ARE_TOKEN_NAMES_PRESERVED))",
    },
    ConditionalBranch {
        block_id: "pp-0554",
        branch_ordinal: 1,
        branch_line: 81,
        directive: "#ifndef @OPTIONALLY_FLAT",
        active_branch_path: "(!defined(@OPTIONALLY_FLAT))",
    },
    ConditionalBranch {
        block_id: "pp-0555",
        branch_ordinal: 1,
        branch_line: 186,
        directive: "#ifdef @ENABLE_INSTANCE_INDEX",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX))",
    },
    ConditionalBranch {
        block_id: "pp-0555",
        branch_ordinal: 2,
        branch_line: 201,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_INSTANCE_INDEX))))",
    },
    ConditionalBranch {
        block_id: "pp-0556",
        branch_ordinal: 1,
        branch_line: 287,
        directive: "#ifdef @PLS_IMPL_DEVICE_BUFFER",
        active_branch_path: "(defined(@PLS_IMPL_DEVICE_BUFFER))",
    },
    ConditionalBranch {
        block_id: "pp-0556",
        branch_ordinal: 2,
        branch_line: 382,
        directive: "#else // Default implementation -- framebuffer reads.",
        active_branch_path: "(!((defined(@PLS_IMPL_DEVICE_BUFFER))))",
    },
    ConditionalBranch {
        block_id: "pp-0557",
        branch_ordinal: 1,
        branch_line: 292,
        directive: "#ifdef @PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        active_branch_path: "(defined(@PLS_IMPL_DEVICE_BUFFER)) && (defined(@PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED))",
    },
    ConditionalBranch {
        block_id: "pp-0557",
        branch_ordinal: 2,
        branch_line: 309,
        directive: "#else",
        active_branch_path: "(defined(@PLS_IMPL_DEVICE_BUFFER)) && (!((defined(@PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The direct @-prefixed identifiers occurring in metal.glsl, retained in
/// first-occurrence source order. Generated names are pinned minifier outputs.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 81,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "OB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 183,
        source_name: "@FlushUniforms",
        generated_name: "NB",
        generated_header_name: "GLSL_FlushUniforms",
    },
    ExportedSymbol {
        source_line: 186,
        source_name: "@ENABLE_INSTANCE_INDEX",
        generated_name: "GE",
        generated_header_name: "GLSL_ENABLE_INSTANCE_INDEX",
    },
    ExportedSymbol {
        source_line: 287,
        source_name: "@PLS_IMPL_DEVICE_BUFFER",
        generated_name: "HF",
        generated_header_name: "GLSL_PLS_IMPL_DEVICE_BUFFER",
    },
    ExportedSymbol {
        source_line: 292,
        source_name: "@PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        generated_name: "IF",
        generated_header_name: "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 81,
        source_name: "@OPTIONALLY_FLAT",
        generated_name: "OB",
        generated_header_name: "GLSL_OPTIONALLY_FLAT",
    },
    ExportedSymbol {
        source_line: 186,
        source_name: "@ENABLE_INSTANCE_INDEX",
        generated_name: "GE",
        generated_header_name: "GLSL_ENABLE_INSTANCE_INDEX",
    },
    ExportedSymbol {
        source_line: 287,
        source_name: "@PLS_IMPL_DEVICE_BUFFER",
        generated_name: "HF",
        generated_header_name: "GLSL_PLS_IMPL_DEVICE_BUFFER",
    },
    ExportedSymbol {
        source_line: 292,
        source_name: "@PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        generated_name: "IF",
        generated_header_name: "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "OPTIONALLY_FLAT",
    "FlushUniforms",
    "ENABLE_INSTANCE_INDEX",
    "PLS_IMPL_DEVICE_BUFFER",
    "PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
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

/// Function declarations are retained as source spellings and ranges. Their
/// bodies remain in PINNED_METAL_GLSL_SOURCE rather than being translated into
/// executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 405,
        end_line: 410,
        name: "pls_atomic_max",
        signature: "INLINE uint pls_atomic_max($thread uint& dst, uint x)",
        guard_path: "(!((defined(@PLS_IMPL_DEVICE_BUFFER))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 414,
        end_line: 419,
        name: "pls_atomic_add",
        signature: "INLINE uint pls_atomic_add($thread uint& dst, uint x)",
        guard_path: "(!((defined(@PLS_IMPL_DEVICE_BUFFER))))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 481,
        end_line: 484,
        name: "floatBitsToUint",
        signature: "$template<int N> INLINE $vec<uint, N> floatBitsToUint($vec<float, N> x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 486,
        end_line: 489,
        name: "floatBitsToInt",
        signature: "$template<int N> INLINE $vec<int, N> floatBitsToInt($vec<float, N> x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 491,
        end_line: 491,
        name: "floatBitsToUint",
        signature: "INLINE uint floatBitsToUint(float x) { return $as_type<uint>(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 493,
        end_line: 493,
        name: "floatBitsToInt",
        signature: "INLINE int floatBitsToInt(float x) { return $as_type<int>(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 495,
        end_line: 498,
        name: "uintBitsToFloat",
        signature: "$template<int N> INLINE $vec<float, N> uintBitsToFloat($vec<uint, N> x)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 500,
        end_line: 500,
        name: "uintBitsToFloat",
        signature: "INLINE float uintBitsToFloat(uint x) { return $as_type<float>(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 501,
        end_line: 501,
        name: "unpackHalf2x16",
        signature: "INLINE half2 unpackHalf2x16(uint x) { return $as_type<half2>(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 502,
        end_line: 502,
        name: "packHalf2x16",
        signature: "INLINE uint packHalf2x16(half2 x) { return $as_type<uint>(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 503,
        end_line: 503,
        name: "unpackUnorm4x8",
        signature: "INLINE half4 unpackUnorm4x8(uint x) { return $unpack_unorm4x8_to_half(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 504,
        end_line: 504,
        name: "packUnorm4x8",
        signature: "INLINE uint packUnorm4x8(half4 x) { return $pack_half_to_unorm4x8(x); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 506,
        end_line: 511,
        name: "inverse",
        signature: "INLINE float2x2 inverse(float2x2 m)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 513,
        end_line: 519,
        name: "mix",
        signature: "INLINE half3 mix(half3 a, half3 b, bool3 c)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 521,
        end_line: 527,
        name: "mix",
        signature: "INLINE float2 mix(float2 a, float2 b, bool2 c)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 529,
        end_line: 529,
        name: "mix",
        signature:
            "INLINE float2 mix(float2 a, float2 b, float t) { return mix(a, b, float2(t)); }",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 531,
        end_line: 531,
        name: "mod",
        signature: "INLINE float mod(float x, float y) { return $fmod(x, y); }",
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

/// The metal.glsl owner has no direct #include/#import directive. These
/// incoming generated-source edges are retained from the include/source
/// dependency authorities because they determine its artifact consumers.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 7,
        include_token: "generated/shaders/metal.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/color_ramp.metal",
        include_line: 6,
        include_token: "metal.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/draw.metal",
        include_line: 9,
        include_token: "metal.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/tessellate.metal",
        include_line: 6,
        include_token: "metal.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
