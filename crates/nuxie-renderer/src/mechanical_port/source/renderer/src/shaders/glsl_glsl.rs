/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/glsl.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "d7e3b795badbe6e5108f268ddea4f7c0bb5af4ad1416e41c7304beca89a15523";
pub const PINNED_SOURCE_LINE_COUNT: usize = 726;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 30330;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/glsl_glsl.rs";
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
pub const PINNED_GLSL_GLSL_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

// This header provides GLSL-specific #defines and declarations that enable our
// shaders to be compiled on MSL and GLSL both.

#define GLSL

#ifndef @GLSL_VERSION
// In "#version 320 es", Qualcomm incorrectly substitutes __VERSION__ to 300.
// @GLSL_VERSION is a workaround for this.
#define @GLSL_VERSION __VERSION__
#endif

#define float2 vec2
#define float3 vec3
#define packed_float3 vec3
#define float4 vec4

#define half mediump float
#define half2 mediump vec2
#define half3 mediump vec3
#define half4 mediump vec4
#define half3x3 mediump mat3x3
#define half2x3 mediump mat2x3
#define half4x4 mediump mat4x4

#define int2 ivec2
#define int3 ivec3
#define int4 ivec4

#define short mediump int
#define short2 mediump ivec2
#define short3 mediump ivec3
#define short4 mediump ivec4

#define uint2 uvec2
#define uint3 uvec3
#define uint4 uvec4

#define ushort mediump uint
#define ushort2 mediump uvec2
#define ushort3 mediump uvec3
#define ushort4 mediump uvec4

#define bool2 bvec2
#define bool3 bvec3
#define bool4 bvec4

#define float2x2 mat2

#define INLINE
#define OUT(ARG_TYPE) out ARG_TYPE
#define INOUT(ARG_TYPE) inout ARG_TYPE

#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin
#extension GL_ANGLE_base_vertex_base_instance_shader_builtin : require
#endif

#ifdef @ENABLE_KHR_BLEND
#extension GL_KHR_blend_equation_advanced : require
#endif

// Enable the necessary extensions for rendering the feather atlas.
// NOTE: We do this here instead of render_atlas.glsl because extensions have to
// be declared before any code.
#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
#extension GL_EXT_shader_framebuffer_fetch : require
#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)
#extension GL_EXT_shader_pixel_local_storage : require
#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
#extension GL_ANGLE_shader_pixel_local_storage : require
#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store : require
#endif
#ifdef GL_OES_shader_image_atomic
#extension GL_OES_shader_image_atomic : require
#endif
#endif

// clang-format off
#if defined(@RENDER_MODE_MSAA) && defined(@ENABLE_CLIP_RECT) && defined(GL_ES) && !defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS)
#ifdef GL_EXT_clip_cull_distance
#extension GL_EXT_clip_cull_distance : require
#elif defined(GL_ANGLE_clip_cull_distance)
#extension GL_ANGLE_clip_cull_distance : require
#endif
#endif // RENDER_MODE_MSAA && ENABLE_CLIP_RECT && GL_ES && !DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS
// clang-format on

#if @GLSL_VERSION >= 310
#define UNIFORM_BLOCK_BEGIN(IDX, NAME)                                         \
    layout(binding = IDX, std140) uniform NAME                                 \
    {
#else
#define UNIFORM_BLOCK_BEGIN(IDX, NAME)                                         \
    layout(std140) uniform NAME                                                \
    {
#endif
// clang-format barrier... Otherwise it tries to merge this #define into the
// above macro...
#define UNIFORM_BLOCK_END(NAME)                                                \
    }                                                                          \
    NAME;

#define ATTR_BLOCK_BEGIN(NAME)
#define ATTR(IDX, TYPE, NAME) layout(location = IDX) in TYPE NAME
#define ATTR_BLOCK_END
#define ATTR_LOAD(A, B, C, D)
#define ATTR_UNPACK(ID, attrs, NAME, TYPE)

#ifdef @VERTEX
#if @GLSL_VERSION >= 310
#define VARYING(IDX, TYPE, NAME) layout(location = IDX) out TYPE NAME
#else
#define VARYING(IDX, TYPE, NAME) out TYPE NAME
#endif
#else
#if @GLSL_VERSION >= 310
#define VARYING(IDX, TYPE, NAME) layout(location = IDX) in TYPE NAME
#else
#define VARYING(IDX, TYPE, NAME) in TYPE NAME
#endif
#endif
#define FLAT flat
#define VARYING_BLOCK_BEGIN
#define VARYING_BLOCK_END

// clang-format off
#ifdef @TARGET_SPIRV
   // Since Vulkan is compiled offline and not all platforms support noperspective, don't use it.
#  define NO_PERSPECTIVE
#else
#  ifdef GL_NV_shader_noperspective_interpolation
#    extension GL_NV_shader_noperspective_interpolation : require
#    define NO_PERSPECTIVE noperspective
#  else
#    define NO_PERSPECTIVE
#  endif
#endif
// clang-format on

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

#ifdef @TARGET_SPIRV
#define TEXTURE_RGBA32UI(SET, IDX, NAME)                                       \
    layout(set = SET, binding = IDX) uniform highp utexture2D NAME
#define TEXTURE_RGBA32F(SET, IDX, NAME)                                        \
    layout(set = SET, binding = IDX) uniform highp texture2D NAME
#define TEXTURE_RGBA8(SET, IDX, NAME)                                          \
    layout(set = SET, binding = IDX) uniform mediump texture2D NAME
#define TEXTURE_R16F(SET, IDX, NAME)                                           \
    layout(binding = IDX) uniform mediump texture2D NAME
#define TEXTURE_R32I(SET, IDX, NAME)                                           \
    layout(binding = IDX) uniform highp itexture2D NAME
#define TEXTURE_R32UI(SET, IDX, NAME)                                          \
    layout(binding = IDX) uniform highp utexture2D NAME
#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)
#endif // @FRAGMENT && @RENDER_MODE_MSAA
#elif @GLSL_VERSION >= 310
#define TEXTURE_RGBA32UI(SET, IDX, NAME)                                       \
    layout(binding = IDX) uniform highp usampler2D NAME
#define TEXTURE_RGBA32F(SET, IDX, NAME)                                        \
    layout(binding = IDX) uniform highp sampler2D NAME
#define TEXTURE_RGBA8(SET, IDX, NAME)                                          \
    layout(binding = IDX) uniform mediump sampler2D NAME
#define TEXTURE_R16F(SET, IDX, NAME)                                           \
    layout(binding = IDX) uniform mediump sampler2D NAME
#define TEXTURE_R32I(SET, IDX, NAME)                                           \
    layout(binding = IDX) uniform highp isampler2D NAME
#define TEXTURE_R32UI(SET, IDX, NAME)                                          \
    layout(binding = IDX) uniform highp usampler2D NAME
#else
#define TEXTURE_RGBA32UI(SET, IDX, NAME) uniform highp usampler2D NAME
#define TEXTURE_RGBA32F(SET, IDX, NAME) uniform highp sampler2D NAME
#define TEXTURE_RGBA8(SET, IDX, NAME) uniform mediump sampler2D NAME
#define TEXTURE_R16F(SET, IDX, NAME) uniform mediump sampler2D NAME
#define TEXTURE_R32I(SET, IDX, NAME) uniform highp isampler2D NAME
#define TEXTURE_R32UI(SET, IDX, NAME) uniform highp usampler2D NAME
#endif

#ifdef @TARGET_SPIRV

#define SAMPLER_DYNAMIC(SET, IDX, NAME)                                        \
    layout(set = SET, binding = IDX) uniform mediump sampler NAME;

#ifdef @USE_WEBGPU_SAMPLERS
#define SAMPLER_LINEAR(TEXTURE_IDX, NAME)                                      \
    layout(set = WEBGPU_SAMPLER_BINDINGS_SET, binding = TEXTURE_IDX)           \
        uniform mediump sampler NAME;
#define SAMPLER_DYNAMIC_IMAGE(NAME)                                            \
    SAMPLER_DYNAMIC(PER_DRAW_BINDINGS_SET, WEBGPU_IMAGE_SAMPLER_IDX, NAME)
#else
#define SAMPLER_LINEAR(TEXTURE_IDX, NAME)                                      \
    layout(set = PER_FLUSH_BINDINGS_SET, binding = TEXTURE_IDX)                \
        uniform mediump sampler NAME;
#define SAMPLER_DYNAMIC_IMAGE(NAME)                                            \
    SAMPLER_DYNAMIC(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, NAME)
#endif
#define TEXTURE_SAMPLE(NAME, SAMPLER_NAME, COORD)                              \
    texture(sampler2D(NAME, SAMPLER_NAME), COORD)
#define TEXTURE_SAMPLE_LOD(NAME, SAMPLER_NAME, COORD, LOD)                     \
    textureLod(sampler2D(NAME, SAMPLER_NAME), COORD, LOD)
#define TEXTURE_SAMPLE_LODBIAS(NAME, SAMPLER_NAME, COORD, LODBIAS)             \
    texture(sampler2D(NAME, SAMPLER_NAME), COORD, LODBIAS)
#define TEXTURE_SAMPLE_GRAD(NAME, SAMPLER_NAME, COORD, DDX, DDY)               \
    textureGrad(sampler2D(NAME, SAMPLER_NAME), COORD, DDX, DDY)
#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)
#extension GL_OES_sample_variables : require
#endif // @FRAGMENT && @RENDER_MODE_MSAA

#else // @TARGET_SPIRV -> !@TARGET_SPIRV

// SAMPLER_LINEAR is a no-op because in GL, sampling parameters are API-level
// state tied to the texture.
#define SAMPLER_LINEAR(TEXTURE_IDX, NAME)
#define SAMPLER_DYNAMIC(SET, IDX, NAME)
#define SAMPLER_DYNAMIC_IMAGE(NAME)
#define TEXTURE_SAMPLE(NAME, SAMPLER_NAME, COORD) texture(NAME, COORD)
#define TEXTURE_SAMPLE_LOD(NAME, SAMPLER_NAME, COORD, LOD)                     \
    textureLod(NAME, COORD, LOD)
#define TEXTURE_SAMPLE_LODBIAS(NAME, SAMPLER_NAME, COORD, LODBIAS)             \
    texture(NAME, COORD, LODBIAS)
#define TEXTURE_SAMPLE_GRAD(NAME, SAMPLER_NAME, COORD, DDX, DDY)               \
    textureGrad(NAME, COORD, DDX, DDY)
#endif // !@TARGET_SPIRV

#define TEXTURE_SAMPLE_DYNAMIC(TEXTURE, SAMPLER_NAME, COORD)                   \
    TEXTURE_SAMPLE(TEXTURE, SAMPLER_NAME, COORD)
#define TEXTURE_SAMPLE_DYNAMIC_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)          \
    TEXTURE_SAMPLE_LOD(TEXTURE, SAMPLER_NAME, COORD, LOD)
#define TEXTURE_SAMPLE_DYNAMIC_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)  \
    TEXTURE_SAMPLE_LODBIAS(TEXTURE, SAMPLER_NAME, COORD, LODBIAS)

// Polyfill the gaussian integral texture as a sampler2D since ES doesn't
// support sampler1DArray. This is why the macro needs "ARRAY_INDEX_NORMALIZED":
// when polyfilled as a 2D texture, the "array index" needs to be a 0..1
// normalized y coordinate instead of the literal array index.
#define TEXTURE_R16F_1D_ARRAY(SET, IDX, NAME) TEXTURE_R16F(SET, IDX, NAME)
// clang-format off
// Clang formatting on this line trips up the Qualcomm compiler.
#define TEXTURE_SAMPLE_LOD_1D_ARRAY(NAME, SAMPLER_NAME, X, ARRAY_INDEX, ARRAY_INDEX_NORMALIZED, LOD)                                       \
    TEXTURE_SAMPLE_LOD(NAME, SAMPLER_NAME, float2(X, ARRAY_INDEX_NORMALIZED), LOD)
// clang-format on

#define TEXTURE_RG32UI(SET, IDX, NAME) TEXTURE_RGBA32UI(SET, IDX, NAME)

#define TEXTURE_CONTEXT_DECL

#define TEXTURE_CONTEXT_FORWARD
#define TEXEL_FETCH(NAME, COORD) texelFetch(NAME, COORD, 0)

#ifdef @TARGET_SPIRV
#define TEXTURE_GATHER(NAME, SAMPLER_NAME, COORD, TEXTURE_INVERSE_SIZE)        \
    textureGather(sampler2D(NAME, SAMPLER_NAME),                               \
                  (COORD) * (TEXTURE_INVERSE_SIZE))
#elif @GLSL_VERSION >= 310
#define TEXTURE_GATHER(NAME, SAMPLER_NAME, COORD, TEXTURE_INVERSE_SIZE)        \
    textureGather(NAME, (COORD) * (TEXTURE_INVERSE_SIZE))
#else
#define TEXTURE_GATHER(NAME, SAMPLER_NAME, COORD, TEXTURE_INVERSE_SIZE)        \
    TEXTURE_GATHER_MATRIX(NAME, COORD, .r)
#endif

#define VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
#define VERTEX_STORAGE_BUFFER_BLOCK_END

#define FRAG_STORAGE_BUFFER_BLOCK_BEGIN
#define FRAG_STORAGE_BUFFER_BLOCK_END

#ifdef @DISABLE_SHADER_STORAGE_BUFFERS

#define STORAGE_BUFFER_U32x2(IDX, GLSL_STRUCT_NAME, NAME)                      \
    TEXTURE_RGBA32UI(PER_FLUSH_BINDINGS_SET, IDX, NAME)
#define STORAGE_BUFFER_U32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    TEXTURE_RG32UI(PER_FLUSH_BINDINGS_SET, IDX, NAME)
#define STORAGE_BUFFER_F32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    TEXTURE_RGBA32F(PER_FLUSH_BINDINGS_SET, IDX, NAME)
#define STORAGE_BUFFER_LOAD4(NAME, I)                                          \
    TEXEL_FETCH(                                                               \
        NAME,                                                                  \
        int2((I) & STORAGE_TEXTURE_MASK_X, (I) >> STORAGE_TEXTURE_SHIFT_Y))
#define STORAGE_BUFFER_LOAD2(NAME, I)                                          \
    TEXEL_FETCH(                                                               \
        NAME,                                                                  \
        int2((I) & STORAGE_TEXTURE_MASK_X, (I) >> STORAGE_TEXTURE_SHIFT_Y))    \
        .xy

#else

#ifdef GL_ARB_shader_storage_buffer_object
#extension GL_ARB_shader_storage_buffer_object : require
#endif
#define STORAGE_BUFFER_U32x2(IDX, GLSL_STRUCT_NAME, NAME)                      \
    layout(std430, binding = IDX) readonly buffer GLSL_STRUCT_NAME             \
    {                                                                          \
        uint2 _values[];                                                       \
    }                                                                          \
    NAME
#define STORAGE_BUFFER_U32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    layout(std430, binding = IDX) readonly buffer GLSL_STRUCT_NAME             \
    {                                                                          \
        uint4 _values[];                                                       \
    }                                                                          \
    NAME
#define STORAGE_BUFFER_F32x4(IDX, GLSL_STRUCT_NAME, NAME)                      \
    layout(std430, binding = IDX) readonly buffer GLSL_STRUCT_NAME             \
    {                                                                          \
        float4 _values[];                                                      \
    }                                                                          \
    NAME
#define STORAGE_BUFFER_U32_ATOMIC(IDX, GLSL_STRUCT_NAME, NAME)                 \
    layout(std430, binding = IDX) buffer GLSL_STRUCT_NAME { uint _values[]; }  \
    NAME
#define STORAGE_BUFFER_LOAD4(NAME, I) NAME._values[I]
#define STORAGE_BUFFER_LOAD2(NAME, I) NAME._values[I]
#define STORAGE_BUFFER_LOAD(NAME, I) NAME._values[I]
#define STORAGE_BUFFER_ATOMIC_MAX(NAME, I, X) atomicMax(NAME._values[I], X)
#define STORAGE_BUFFER_ATOMIC_ADD(NAME, I, X) atomicAdd(NAME._values[I], X)
#define STORAGE_BUFFER_ATOMIC_OR(NAME, I, X) atomicOr(NAME._values[I], X)

#endif // DISABLE_SHADER_STORAGE_BUFFERS

#ifdef @PLS_IMPL_STORAGE_BUFFER

#define PLS_MAIN(NAME)                                                         \
    void main()                                                                \
    {                                                                          \
        int2 _plsCoord = ivec2(floor(_fragCoord));                             \
        int _plsIdx = int(swizzle_image_buffer_idx(                            \
            uvec2(_plsCoord),                                                  \
            (uniforms.renderTargetWidth + (BUFFER_IMAGE_TILE_SIZE - 1u)) &     \
                ~(BUFFER_IMAGE_TILE_SIZE - 1u)));

#define EMIT_PLS }

#define PLS_CONTEXT_DECL , int _plsIdx
#define PLS_CONTEXT_UNPACK , _plsIdx

#ifdef @TARGET_WGSL
// WGSL has no `coherent` qualifier — naga would propagate it as an invalid
// `@coherent` attribute that Tint rejects. WGSL's storage memory model
// already guarantees the visibility we need across the atomic ops below.
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    layout(std430, set = PLS_TEXTURE_BINDINGS_SET, binding = IDX)              \
        buffer NAME##_struct                                                   \
    {                                                                          \
        uint _values[];                                                        \
    }                                                                          \
    NAME
#elif defined(@TARGET_SPIRV)
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    layout(std430, set = PLS_TEXTURE_BINDINGS_SET, binding = IDX)              \
        coherent buffer NAME##_struct                                          \
    {                                                                          \
        uint _values[];                                                        \
    }                                                                          \
    NAME
#else
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    layout(std430, binding = IDX) coherent buffer NAME##_struct                \
    {                                                                          \
        uint _values[];                                                        \
    }                                                                          \
    NAME
#endif
#define PLS_DECL4F_UAV PLS_DECLUI_UAV

#define PLS_LOADUI_UAV(PLANE) PLANE._values[_plsIdx]
#define PLS_STOREUI_UAV(PLANE, VALUE) PLANE._values[_plsIdx] = VALUE
#define PLS_LOAD4F_UAV(PLANE) unpackUnorm4x8(PLS_LOADUI_UAV(PLANE))
#define PLS_STORE4F_UAV(PLANE, VALUE)                                          \
    PLS_STOREUI_UAV(PLANE, packUnorm4x8(VALUE))
#define PLS_ATOMIC_MAX(PLANE, X) atomicMax(PLANE._values[_plsIdx], X)
#define PLS_ATOMIC_ADD(PLANE, X) atomicAdd(PLANE._values[_plsIdx], X)

#elif defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES)

#ifdef GL_ARB_shader_image_load_store
#extension GL_ARB_shader_image_load_store : require
#endif

#define PLS_MAIN(NAME)                                                         \
    void main()                                                                \
    {                                                                          \
        int2 _plsCoord = ivec2(floor(_fragCoord));

#define EMIT_PLS }

#define PLS_CONTEXT_DECL , int2 _plsCoord
#define PLS_CONTEXT_UNPACK , _plsCoord

#ifdef @TARGET_SPIRV
#define PLS_DECL4F_UAV(IDX, NAME)                                              \
    layout(set = PLS_TEXTURE_BINDINGS_SET, binding = IDX, rgba8)               \
        uniform mediump coherent image2D NAME
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    layout(set = PLS_TEXTURE_BINDINGS_SET, binding = IDX, r32ui)               \
        uniform highp coherent uimage2D NAME
#define PLS_DECL4F_RGB10_A2_UAV(IDX, NAME)                                     \
    layout(set = PLS_TEXTURE_BINDINGS_SET, binding = IDX, rgb10_a2)            \
        uniform mediump coherent image2D NAME
#else
#define PLS_DECL4F_UAV(IDX, NAME)                                              \
    layout(binding = IDX, rgba8) uniform mediump coherent image2D NAME
#define PLS_DECLUI_UAV(IDX, NAME)                                              \
    layout(binding = IDX, r32ui) uniform highp coherent uimage2D NAME
#define PLS_DECL4F_RGB10_A2_UAV(IDX, NAME)                                     \
    layout(binding = IDX, rgb10_a2) uniform mediump coherent image2D NAME;
#endif

#define PLS_LOADUI_UAV(PLANE) imageLoad(PLANE, _plsCoord).r
#define PLS_STOREUI_UAV(PLANE, VALUE) imageStore(PLANE, _plsCoord, uvec4(VALUE))
#define PLS_LOAD4F_UAV(PLANE) imageLoad(PLANE, _plsCoord)
#define PLS_STORE4F_UAV(PLANE, VALUE) imageStore(PLANE, _plsCoord, VALUE)
#define PLS_ATOMIC_MAX(PLANE, X) imageAtomicMax(PLANE, _plsCoord, X)
#define PLS_ATOMIC_ADD(PLANE, X) imageAtomicAdd(PLANE, _plsCoord, X)

#else

#define PLS_MAIN(NAME) void main()
#define EMIT_PLS

#define PLS_CONTEXT_DECL
#define PLS_CONTEXT_UNPACK

#endif

// Define macros for implementing pixel local storage based on available
// extensions.
#ifdef @PLS_IMPL_ANGLE

#extension GL_ANGLE_shader_pixel_local_storage : require

#define PLS_BLOCK_BEGIN
#define PLS_DECL4F(IDX, NAME)                                                  \
    layout(binding = IDX, rgba8) uniform mediump pixelLocalANGLE NAME
#define PLS_DECLUI(IDX, NAME)                                                  \
    layout(binding = IDX, r32ui) uniform highp upixelLocalANGLE NAME
#define PLS_BLOCK_END

#define PLS_LOAD4F(PLANE) pixelLocalLoadANGLE(PLANE)
#define PLS_LOADUI(PLANE) pixelLocalLoadANGLE(PLANE).r
#define PLS_STORE4F(PLANE, VALUE) pixelLocalStoreANGLE(PLANE, VALUE)
#define PLS_STOREUI(PLANE, VALUE) pixelLocalStoreANGLE(PLANE, uvec4(VALUE))

#define PLS_PRESERVE_4F(PLANE)
#define PLS_PRESERVE_UI(PLANE)

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#endif // PLS_IMPL_ANGLE

#ifdef @PLS_IMPL_EXT_NATIVE

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
// fixedFunctionColorOutput renders directly to the framebuffer, which requires
// EXT_shader_pixel_local_storage2.
#extension GL_EXT_shader_pixel_local_storage2 : require
#else
#extension GL_EXT_shader_pixel_local_storage : require
#endif

#define PLS_BLOCK_BEGIN                                                        \
    __pixel_localEXT PLS                                                       \
    {
#define PLS_DECL4F(IDX, NAME) layout(rgba8) mediump vec4 NAME
#define PLS_DECL4F_RGB10_A2(IDX, NAME) layout(rgb10_a2) mediump vec4 NAME
#define PLS_DECLUI(IDX, NAME) layout(r32ui) highp uint NAME
#define PLS_BLOCK_END                                                          \
    }                                                                          \
    ;

#define PLS_LOAD4F(PLANE) PLANE
#define PLS_LOADUI(PLANE) PLANE
#define PLS_STORE4F(PLANE, VALUE) PLANE = (VALUE)
#define PLS_STOREUI(PLANE, VALUE) PLANE = (VALUE)

#define PLS_PRESERVE_4F(PLANE) PLANE = PLANE
#define PLS_PRESERVE_UI(PLANE) PLANE = PLANE

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
// EXT_shader_pixel_local_storage2 requires explicit output format qualifiers
// on fragment shader outputs.
#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    layout(location = 0, rgba8) out half4 _fragColor;                          \
    PLS_MAIN(NAME)
#endif
#endif

#if defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)

#define PLS_BLOCK_BEGIN
#define PLS_BLOCK_END

#define PLS_DECL4F PLS_DECL4F_UAV
#define PLS_DECLUI PLS_DECLUI_UAV
#define PLS_DECL4F_RGB10_A2 PLS_DECL4F_RGB10_A2_UAV

#define PLS_LOAD4F PLS_LOAD4F_UAV
#define PLS_STORE4F PLS_STORE4F_UAV
#define PLS_LOADUI PLS_LOADUI_UAV
#define PLS_STOREUI PLS_STOREUI_UAV

#define PLS_PRESERVE_4F(PLANE)
#define PLS_PRESERVE_UI(PLANE)

#if defined(GL_ARB_fragment_shader_interlock)
#extension GL_ARB_fragment_shader_interlock : require
#define PLS_INTERLOCK_BEGIN beginInvocationInterlockARB()
#define PLS_INTERLOCK_END endInvocationInterlockARB()
#elif defined(GL_INTEL_fragment_shader_ordering)
#extension GL_INTEL_fragment_shader_ordering : require
#define PLS_INTERLOCK_BEGIN beginFragmentShaderOrderingINTEL()
#define PLS_INTERLOCK_END
#else
#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END
#endif

#endif // PLS_IMPL_STORAGE_TEXTURE || PLS_IMPL_STORAGE_BUFFER

#ifdef @PLS_IMPL_SUBPASS_LOAD

#define PLS_BLOCK_BEGIN
#define PLS_DECL4F_READONLY(IDX, NAME)                                         \
    layout(input_attachment_index = IDX,                                       \
           binding = IDX,                                                      \
           set = PLS_TEXTURE_BINDINGS_SET)                                     \
        uniform mediump subpassInput _in_##NAME
#define PLS_DECL4F_WRITEONLY(IDX, NAME)                                        \
    layout(location = IDX) out mediump vec4 NAME
#define PLS_DECL4F(IDX, NAME)                                                  \
    PLS_DECL4F_READONLY(IDX, NAME);                                            \
    PLS_DECL4F_WRITEONLY(IDX, NAME)
#define PLS_DECLUI(IDX, NAME)                                                  \
    layout(input_attachment_index = IDX,                                       \
           binding = IDX,                                                      \
           set = PLS_TEXTURE_BINDINGS_SET)                                     \
        uniform highp usubpassInput _in_##NAME;                                \
    layout(location = IDX) out highp uvec4 NAME
#define PLS_BLOCK_END

#define PLS_LOAD4F(PLANE) subpassLoad(_in_##PLANE)
#define PLS_LOADUI(PLANE) subpassLoad(_in_##PLANE).r
#define PLS_STORE4F(PLANE, VALUE) PLANE = (VALUE)
#define PLS_STOREUI(PLANE, VALUE) PLANE.r = (VALUE)

#define PLS_PRESERVE_4F(PLANE) PLS_STORE4F(PLANE, subpassLoad(_in_##PLANE))
#define PLS_PRESERVE_UI(PLANE) PLS_STOREUI(PLANE, subpassLoad(_in_##PLANE).r)

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#endif

#ifdef @PLS_IMPL_NONE

#define PLS_BLOCK_BEGIN
#define PLS_DECL4F(IDX, NAME) layout(location = IDX) out mediump vec4 NAME
#define PLS_DECLUI(IDX, NAME) layout(location = IDX) out highp uvec4 NAME
#define PLS_BLOCK_END

#define PLS_LOAD4F(PLANE) vec4(0)
#define PLS_LOADUI(PLANE) 0u
#define PLS_STORE4F(PLANE, VALUE) PLANE = (VALUE)
#define PLS_STOREUI(PLANE, VALUE) PLANE.r = (VALUE)

#define PLS_PRESERVE_4F(PLANE) PLANE = vec4(0)
#define PLS_PRESERVE_UI(PLANE) PLANE.r = 0u

#define PLS_INTERLOCK_BEGIN
#define PLS_INTERLOCK_END

#endif

#ifndef PLS_DECL4F_READONLY
#define PLS_DECL4F_READONLY PLS_DECL4F
#endif

#ifdef @TARGET_SPIRV
#define gl_VertexID gl_VertexIndex
#endif

// clang-format off
#ifdef @ENABLE_INSTANCE_INDEX
#  ifdef @TARGET_SPIRV
#    define INSTANCE_INDEX gl_InstanceIndex
#  else
#    ifdef @BASE_INSTANCE_UNIFORM_NAME
       // gl_BaseInstance isn't supported on this platform. The rendering
       // backend will set this uniform for us instead.
       uniform highp int @BASE_INSTANCE_UNIFORM_NAME;
#      define INSTANCE_INDEX (gl_InstanceID + @BASE_INSTANCE_UNIFORM_NAME)
#    else
#        define INSTANCE_INDEX (gl_InstanceID + gl_BaseInstance)
#    endif
#  endif
#else
#  define INSTANCE_INDEX 0
#endif
// clang-format on

#define VERTEX_CONTEXT_DECL
#define VERTEX_CONTEXT_UNPACK

#define CLIP_CONTEXT_FORWARD
#define CLIP_CONTEXT_UNPACK

#define VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)                \
    void main()                                                                \
    {                                                                          \
        int _vertexID = gl_VertexID;                                           \
        int _instanceID = INSTANCE_INDEX;

// clang-format off
#define IMAGE_RECT_VERTEX_MAIN(NAME, Attrs, attrs, ImageDrawAttrs, imageDrawAttrs, _vertexID, _instanceID)                                    \
    VERTEX_MAIN(NAME, Attrs, attrs, _vertexID, _instanceID)
#define IMAGE_MESH_VERTEX_MAIN(NAME, PositionAttr, position, UVAttr, uv, ImageDrawAttrs,  imageDrawAttrs, _vertexID)                                      \
    VERTEX_MAIN(NAME, PositionAttr, position, _vertexID, _instanceID)
// clang-format on

#define VARYING_INIT(NAME, TYPE)
#define VARYING_PACK(NAME)
#define VARYING_UNPACK(NAME, TYPE)

#define EMIT_VERTEX(_pos)                                                      \
    gl_Position = _pos;                                                        \
    }

#define FRAG_DATA_MAIN(DATA_TYPE, NAME)                                        \
    layout(location = 0) out DATA_TYPE _fd;                                    \
    void main()

#define FRAG_DATA_MAIN_WITH_CLOCKWISE FRAG_DATA_MAIN

#define _clockwise gl_FrontFacing

#define EMIT_FRAG_DATA(VALUE) _fd = VALUE

#define _fragCoord gl_FragCoord.xy

#define FRAGMENT_CONTEXT_DECL
#define FRAGMENT_CONTEXT_UNPACK

#if defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)
// Global storage is expensive to update. It's faster to conditionally update
// them when possible.
#define PLS_STORE4F_OPTIONAL_IF(CONDITION, PLANE, VALUE)                       \
    if (!(CONDITION))                                                          \
    {                                                                          \
        PLS_STORE4F(PLANE, VALUE);                                             \
    }
#define PLS_STOREUI_OPTIONAL_IF(CONDITION, PLANE, VALUE)                       \
    if (!(CONDITION))                                                          \
    {                                                                          \
        PLS_STOREUI(PLANE, VALUE);                                             \
    }
#else
// Cheap forms of PLS do better to update unconditionally, even if it might be a
// no-op. (Especially since we otherwise would have had to preserve anyway.)
#define PLS_STORE4F_OPTIONAL_IF(CONDITION, PLANE, VALUE)                       \
    PLS_STORE4F(PLANE, VALUE);
#define PLS_STOREUI_OPTIONAL_IF(CONDITION, PLANE, VALUE)                       \
    PLS_STOREUI(PLANE, VALUE);
#endif

#ifndef PLS_FRAG_COLOR_MAIN
#define PLS_FRAG_COLOR_MAIN(NAME)                                              \
    layout(location = 0) out half4 _fragColor;                                 \
    PLS_MAIN(NAME)
#endif

#define EMIT_PLS_AND_FRAG_COLOR EMIT_PLS

#if defined(@TARGET_SPIRV) && !defined(@TARGET_WGSL)
#define DST_COLOR_TEXTURE(NAME)                                                \
    layout(input_attachment_index = 0,                                         \
           binding = COLOR_PLANE_IDX,                                          \
           set = PLS_TEXTURE_BINDINGS_SET) uniform mediump subpassInputMS NAME
#define DST_COLOR_FETCH(NAME)                                                  \
    dst_color_fetch(mat4(subpassLoad(NAME, 0),                                 \
                         subpassLoad(NAME, 1),                                 \
                         subpassLoad(NAME, 2),                                 \
                         subpassLoad(NAME, 3)),                                \
                    gl_SampleMaskIn[0])
#else
#define DST_COLOR_TEXTURE(NAME)                                                \
    TEXTURE_RGBA8(PER_FLUSH_BINDINGS_SET, DST_COLOR_TEXTURE_IDX, NAME)
#define DST_COLOR_FETCH(NAME) texelFetch(NAME, ivec2(floor(_fragCoord.xy)), 0)
#endif

#define MUL(A, B) ((A) * (B))

precision highp float;
precision highp int;

#if @GLSL_VERSION < 310
// Polyfill ES 3.1+ methods.
INLINE half4 polyfill_unpackUnorm4x8(uint u)
{
    uint4 vals = uint4(u & 0xffu, (u >> 8) & 0xffu, (u >> 16) & 0xffu, u >> 24);
    return float4(vals) * (1. / 255.);
}
// Use #define for unpackUnorm4x8 because some drivers (e.g., Adreno 308)
// incorrectly declare this builtin on ES 3.0, leading to compiler errors if we
// just declare it as a normal function.
#define unpackUnorm4x8 polyfill_unpackUnorm4x8
#endif
"###;

pub const PINNED_GLSL_SOURCE: &str = PINNED_GLSL_GLSL_SOURCE;
pub const GLSL_GLSL_SOURCE: &str = PINNED_GLSL_GLSL_SOURCE;
pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_GLSL_GLSL_SOURCE
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
        block_id: "pp-0497",
        block_start: 10,
        block_end: 14,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0498",
        block_start: 57,
        block_end: 59,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0499",
        block_start: 61,
        block_end: 63,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0500",
        block_start: 68,
        block_end: 81,
        block_depth: 0,
        branch_count: 4,
    },
    ConditionalBlock {
        block_id: "pp-0501",
        block_start: 75,
        block_end: 77,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0502",
        block_start: 78,
        block_end: 80,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0503",
        block_start: 84,
        block_end: 90,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0504",
        block_start: 85,
        block_end: 89,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0505",
        block_start: 93,
        block_end: 101,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0506",
        block_start: 114,
        block_end: 126,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0507",
        block_start: 115,
        block_end: 119,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0508",
        block_start: 121,
        block_end: 125,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0509",
        block_start: 132,
        block_end: 142,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0510",
        block_start: 136,
        block_end: 141,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0511",
        block_start: 145,
        block_end: 148,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0512",
        block_start: 150,
        block_end: 153,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0513",
        block_start: 158,
        block_end: 193,
        block_depth: 0,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0514",
        block_start: 171,
        block_end: 172,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0515",
        block_start: 195,
        block_end: 239,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0516",
        block_start: 200,
        block_end: 212,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0517",
        block_start: 221,
        block_end: 223,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0518",
        block_start: 266,
        block_end: 276,
        block_depth: 0,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0519",
        block_start: 284,
        block_end: 335,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0520",
        block_start: 304,
        block_end: 306,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0521",
        block_start: 337,
        block_end: 440,
        block_depth: 0,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0522",
        block_start: 353,
        block_end: 379,
        block_depth: 1,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0523",
        block_start: 392,
        block_end: 394,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0524",
        block_start: 406,
        block_end: 423,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0525",
        block_start: 444,
        block_end: 466,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0526",
        block_start: 468,
        block_end: 506,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0527",
        block_start: 470,
        block_end: 476,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0528",
        block_start: 499,
        block_end: 505,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0529",
        block_start: 508,
        block_end: 538,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0530",
        block_start: 525,
        block_end: 536,
        block_depth: 1,
        branch_count: 3,
    },
    ConditionalBlock {
        block_id: "pp-0531",
        block_start: 540,
        block_end: 572,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0532",
        block_start: 574,
        block_end: 592,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0533",
        block_start: 594,
        block_end: 596,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0534",
        block_start: 598,
        block_end: 600,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0535",
        block_start: 603,
        block_end: 618,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0536",
        block_start: 604,
        block_end: 615,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0537",
        block_start: 607,
        block_end: 614,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0538",
        block_start: 663,
        block_end: 683,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0539",
        block_start: 685,
        block_end: 689,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0540",
        block_start: 693,
        block_end: 708,
        block_depth: 0,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0541",
        block_start: 715,
        block_end: 726,
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
        block_id: "pp-0497",
        branch_ordinal: 1,
        branch_line: 10,
        directive: "#ifndef @GLSL_VERSION",
        active_branch_path: "(!defined(@GLSL_VERSION))",
    },
    ConditionalBranch {
        block_id: "pp-0498",
        branch_ordinal: 1,
        branch_line: 57,
        directive: "#ifdef GL_ANGLE_base_vertex_base_instance_shader_builtin",
        active_branch_path: "(defined(GL_ANGLE_base_vertex_base_instance_shader_builtin))",
    },
    ConditionalBranch {
        block_id: "pp-0499",
        branch_ordinal: 1,
        branch_line: 61,
        directive: "#ifdef @ENABLE_KHR_BLEND",
        active_branch_path: "(defined(@ENABLE_KHR_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0500",
        branch_ordinal: 1,
        branch_line: 68,
        directive: "#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        active_branch_path: "(defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))",
    },
    ConditionalBranch {
        block_id: "pp-0500",
        branch_ordinal: 2,
        branch_line: 70,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)",
        active_branch_path: "(!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)))",
    },
    ConditionalBranch {
        block_id: "pp-0500",
        branch_ordinal: 3,
        branch_line: 72,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)",
        active_branch_path: "(!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)))",
    },
    ConditionalBranch {
        block_id: "pp-0500",
        branch_ordinal: 4,
        branch_line: 74,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)",
        active_branch_path: "(!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)))",
    },
    ConditionalBranch {
        block_id: "pp-0501",
        branch_ordinal: 1,
        branch_line: 75,
        directive: "#ifdef GL_ARB_shader_image_load_store",
        active_branch_path: "(!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(GL_ARB_shader_image_load_store))",
    },
    ConditionalBranch {
        block_id: "pp-0502",
        branch_ordinal: 1,
        branch_line: 78,
        directive: "#ifdef GL_OES_shader_image_atomic",
        active_branch_path: "(!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(GL_OES_shader_image_atomic))",
    },
    ConditionalBranch {
        block_id: "pp-0503",
        branch_ordinal: 1,
        branch_line: 84,
        directive: "#if defined(@RENDER_MODE_MSAA) && defined(@ENABLE_CLIP_RECT) && defined(GL_ES) && !defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS)",
        active_branch_path: "(defined(@RENDER_MODE_MSAA) && defined(@ENABLE_CLIP_RECT) && defined(GL_ES) && !defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))",
    },
    ConditionalBranch {
        block_id: "pp-0504",
        branch_ordinal: 1,
        branch_line: 85,
        directive: "#ifdef GL_EXT_clip_cull_distance",
        active_branch_path: "(defined(@RENDER_MODE_MSAA) && defined(@ENABLE_CLIP_RECT) && defined(GL_ES) && !defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS)) && (defined(GL_EXT_clip_cull_distance))",
    },
    ConditionalBranch {
        block_id: "pp-0504",
        branch_ordinal: 2,
        branch_line: 87,
        directive: "#elif defined(GL_ANGLE_clip_cull_distance)",
        active_branch_path: "(defined(@RENDER_MODE_MSAA) && defined(@ENABLE_CLIP_RECT) && defined(GL_ES) && !defined(@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS)) && (!((defined(GL_EXT_clip_cull_distance))) && (defined(GL_ANGLE_clip_cull_distance)))",
    },
    ConditionalBranch {
        block_id: "pp-0505",
        branch_ordinal: 1,
        branch_line: 93,
        directive: "#if @GLSL_VERSION >= 310",
        active_branch_path: "(@GLSL_VERSION >= 310)",
    },
    ConditionalBranch {
        block_id: "pp-0505",
        branch_ordinal: 2,
        branch_line: 97,
        directive: "#else",
        active_branch_path: "(!((@GLSL_VERSION >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0506",
        branch_ordinal: 1,
        branch_line: 114,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0506",
        branch_ordinal: 2,
        branch_line: 120,
        directive: "#else",
        active_branch_path: "(!((defined(@VERTEX))))",
    },
    ConditionalBranch {
        block_id: "pp-0507",
        branch_ordinal: 1,
        branch_line: 115,
        directive: "#if @GLSL_VERSION >= 310",
        active_branch_path: "(defined(@VERTEX)) && (@GLSL_VERSION >= 310)",
    },
    ConditionalBranch {
        block_id: "pp-0507",
        branch_ordinal: 2,
        branch_line: 117,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX)) && (!((@GLSL_VERSION >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0508",
        branch_ordinal: 1,
        branch_line: 121,
        directive: "#if @GLSL_VERSION >= 310",
        active_branch_path: "(!((defined(@VERTEX)))) && (@GLSL_VERSION >= 310)",
    },
    ConditionalBranch {
        block_id: "pp-0508",
        branch_ordinal: 2,
        branch_line: 123,
        directive: "#else",
        active_branch_path: "(!((defined(@VERTEX)))) && (!((@GLSL_VERSION >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0509",
        branch_ordinal: 1,
        branch_line: 132,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0509",
        branch_ordinal: 2,
        branch_line: 135,
        directive: "#else",
        active_branch_path: "(!((defined(@TARGET_SPIRV))))",
    },
    ConditionalBranch {
        block_id: "pp-0510",
        branch_ordinal: 1,
        branch_line: 136,
        directive: "# ifdef GL_NV_shader_noperspective_interpolation",
        active_branch_path: "(!((defined(@TARGET_SPIRV)))) && (defined(GL_NV_shader_noperspective_interpolation))",
    },
    ConditionalBranch {
        block_id: "pp-0510",
        branch_ordinal: 2,
        branch_line: 139,
        directive: "# else",
        active_branch_path: "(!((defined(@TARGET_SPIRV)))) && (!((defined(GL_NV_shader_noperspective_interpolation))))",
    },
    ConditionalBranch {
        block_id: "pp-0511",
        branch_ordinal: 1,
        branch_line: 145,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0512",
        branch_ordinal: 1,
        branch_line: 150,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0513",
        branch_ordinal: 1,
        branch_line: 158,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0513",
        branch_ordinal: 2,
        branch_line: 173,
        directive: "#elif @GLSL_VERSION >= 310",
        active_branch_path: "(!((defined(@TARGET_SPIRV))) && (@GLSL_VERSION >= 310))",
    },
    ConditionalBranch {
        block_id: "pp-0513",
        branch_ordinal: 3,
        branch_line: 186,
        directive: "#else",
        active_branch_path: "(!((defined(@TARGET_SPIRV)) || (@GLSL_VERSION >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0514",
        branch_ordinal: 1,
        branch_line: 171,
        directive: "#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@TARGET_SPIRV)) && (defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0515",
        branch_ordinal: 1,
        branch_line: 195,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0515",
        branch_ordinal: 2,
        branch_line: 225,
        directive: "#else // @TARGET_SPIRV -> !@TARGET_SPIRV",
        active_branch_path: "(!((defined(@TARGET_SPIRV))))",
    },
    ConditionalBranch {
        block_id: "pp-0516",
        branch_ordinal: 1,
        branch_line: 200,
        directive: "#ifdef @USE_WEBGPU_SAMPLERS",
        active_branch_path: "(defined(@TARGET_SPIRV)) && (defined(@USE_WEBGPU_SAMPLERS))",
    },
    ConditionalBranch {
        block_id: "pp-0516",
        branch_ordinal: 2,
        branch_line: 206,
        directive: "#else",
        active_branch_path: "(defined(@TARGET_SPIRV)) && (!((defined(@USE_WEBGPU_SAMPLERS))))",
    },
    ConditionalBranch {
        block_id: "pp-0517",
        branch_ordinal: 1,
        branch_line: 221,
        directive: "#if defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA)",
        active_branch_path: "(defined(@TARGET_SPIRV)) && (defined(@FRAGMENT) && defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0518",
        branch_ordinal: 1,
        branch_line: 266,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0518",
        branch_ordinal: 2,
        branch_line: 270,
        directive: "#elif @GLSL_VERSION >= 310",
        active_branch_path: "(!((defined(@TARGET_SPIRV))) && (@GLSL_VERSION >= 310))",
    },
    ConditionalBranch {
        block_id: "pp-0518",
        branch_ordinal: 3,
        branch_line: 273,
        directive: "#else",
        active_branch_path: "(!((defined(@TARGET_SPIRV)) || (@GLSL_VERSION >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0519",
        branch_ordinal: 1,
        branch_line: 284,
        directive: "#ifdef @DISABLE_SHADER_STORAGE_BUFFERS",
        active_branch_path: "(defined(@DISABLE_SHADER_STORAGE_BUFFERS))",
    },
    ConditionalBranch {
        block_id: "pp-0519",
        branch_ordinal: 2,
        branch_line: 302,
        directive: "#else",
        active_branch_path: "(!((defined(@DISABLE_SHADER_STORAGE_BUFFERS))))",
    },
    ConditionalBranch {
        block_id: "pp-0520",
        branch_ordinal: 1,
        branch_line: 304,
        directive: "#ifdef GL_ARB_shader_storage_buffer_object",
        active_branch_path: "(!((defined(@DISABLE_SHADER_STORAGE_BUFFERS)))) && (defined(GL_ARB_shader_storage_buffer_object))",
    },
    ConditionalBranch {
        block_id: "pp-0521",
        branch_ordinal: 1,
        branch_line: 337,
        directive: "#ifdef @PLS_IMPL_STORAGE_BUFFER",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_BUFFER))",
    },
    ConditionalBranch {
        block_id: "pp-0521",
        branch_ordinal: 2,
        branch_line: 390,
        directive: "#elif defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES)",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_BUFFER))) && (defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES)))",
    },
    ConditionalBranch {
        block_id: "pp-0521",
        branch_ordinal: 3,
        branch_line: 432,
        directive: "#else",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_BUFFER)) || (defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES))))",
    },
    ConditionalBranch {
        block_id: "pp-0522",
        branch_ordinal: 1,
        branch_line: 353,
        directive: "#ifdef @TARGET_WGSL",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_BUFFER)) && (defined(@TARGET_WGSL))",
    },
    ConditionalBranch {
        block_id: "pp-0522",
        branch_ordinal: 2,
        branch_line: 364,
        directive: "#elif defined(@TARGET_SPIRV)",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_BUFFER)) && (!((defined(@TARGET_WGSL))) && (defined(@TARGET_SPIRV)))",
    },
    ConditionalBranch {
        block_id: "pp-0522",
        branch_ordinal: 3,
        branch_line: 372,
        directive: "#else",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_BUFFER)) && (!((defined(@TARGET_WGSL)) || (defined(@TARGET_SPIRV))))",
    },
    ConditionalBranch {
        block_id: "pp-0523",
        branch_ordinal: 1,
        branch_line: 392,
        directive: "#ifdef GL_ARB_shader_image_load_store",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_BUFFER))) && (defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES))) && (defined(GL_ARB_shader_image_load_store))",
    },
    ConditionalBranch {
        block_id: "pp-0524",
        branch_ordinal: 1,
        branch_line: 406,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_BUFFER))) && (defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES))) && (defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0524",
        branch_ordinal: 2,
        branch_line: 416,
        directive: "#else",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_BUFFER))) && (defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@USING_PLS_STORAGE_TEXTURES))) && (!((defined(@TARGET_SPIRV))))",
    },
    ConditionalBranch {
        block_id: "pp-0525",
        branch_ordinal: 1,
        branch_line: 444,
        directive: "#ifdef @PLS_IMPL_ANGLE",
        active_branch_path: "(defined(@PLS_IMPL_ANGLE))",
    },
    ConditionalBranch {
        block_id: "pp-0526",
        branch_ordinal: 1,
        branch_line: 468,
        directive: "#ifdef @PLS_IMPL_EXT_NATIVE",
        active_branch_path: "(defined(@PLS_IMPL_EXT_NATIVE))",
    },
    ConditionalBranch {
        block_id: "pp-0527",
        branch_ordinal: 1,
        branch_line: 470,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@PLS_IMPL_EXT_NATIVE)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0527",
        branch_ordinal: 2,
        branch_line: 474,
        directive: "#else",
        active_branch_path: "(defined(@PLS_IMPL_EXT_NATIVE)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0528",
        branch_ordinal: 1,
        branch_line: 499,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@PLS_IMPL_EXT_NATIVE)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0529",
        branch_ordinal: 1,
        branch_line: 508,
        directive: "#if defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER))",
    },
    ConditionalBranch {
        block_id: "pp-0530",
        branch_ordinal: 1,
        branch_line: 525,
        directive: "#if defined(GL_ARB_fragment_shader_interlock)",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)) && (defined(GL_ARB_fragment_shader_interlock))",
    },
    ConditionalBranch {
        block_id: "pp-0530",
        branch_ordinal: 2,
        branch_line: 529,
        directive: "#elif defined(GL_INTEL_fragment_shader_ordering)",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)) && (!((defined(GL_ARB_fragment_shader_interlock))) && (defined(GL_INTEL_fragment_shader_ordering)))",
    },
    ConditionalBranch {
        block_id: "pp-0530",
        branch_ordinal: 3,
        branch_line: 533,
        directive: "#else",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)) && (!((defined(GL_ARB_fragment_shader_interlock)) || (defined(GL_INTEL_fragment_shader_ordering))))",
    },
    ConditionalBranch {
        block_id: "pp-0531",
        branch_ordinal: 1,
        branch_line: 540,
        directive: "#ifdef @PLS_IMPL_SUBPASS_LOAD",
        active_branch_path: "(defined(@PLS_IMPL_SUBPASS_LOAD))",
    },
    ConditionalBranch {
        block_id: "pp-0532",
        branch_ordinal: 1,
        branch_line: 574,
        directive: "#ifdef @PLS_IMPL_NONE",
        active_branch_path: "(defined(@PLS_IMPL_NONE))",
    },
    ConditionalBranch {
        block_id: "pp-0533",
        branch_ordinal: 1,
        branch_line: 594,
        directive: "#ifndef PLS_DECL4F_READONLY",
        active_branch_path: "(!defined(PLS_DECL4F_READONLY))",
    },
    ConditionalBranch {
        block_id: "pp-0534",
        branch_ordinal: 1,
        branch_line: 598,
        directive: "#ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0535",
        branch_ordinal: 1,
        branch_line: 603,
        directive: "#ifdef @ENABLE_INSTANCE_INDEX",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX))",
    },
    ConditionalBranch {
        block_id: "pp-0535",
        branch_ordinal: 2,
        branch_line: 616,
        directive: "#else",
        active_branch_path: "(!((defined(@ENABLE_INSTANCE_INDEX))))",
    },
    ConditionalBranch {
        block_id: "pp-0536",
        branch_ordinal: 1,
        branch_line: 604,
        directive: "# ifdef @TARGET_SPIRV",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX)) && (defined(@TARGET_SPIRV))",
    },
    ConditionalBranch {
        block_id: "pp-0536",
        branch_ordinal: 2,
        branch_line: 606,
        directive: "# else",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX)) && (!((defined(@TARGET_SPIRV))))",
    },
    ConditionalBranch {
        block_id: "pp-0537",
        branch_ordinal: 1,
        branch_line: 607,
        directive: "# ifdef @BASE_INSTANCE_UNIFORM_NAME",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX)) && (!((defined(@TARGET_SPIRV)))) && (defined(@BASE_INSTANCE_UNIFORM_NAME))",
    },
    ConditionalBranch {
        block_id: "pp-0537",
        branch_ordinal: 2,
        branch_line: 612,
        directive: "# else",
        active_branch_path: "(defined(@ENABLE_INSTANCE_INDEX)) && (!((defined(@TARGET_SPIRV)))) && (!((defined(@BASE_INSTANCE_UNIFORM_NAME))))",
    },
    ConditionalBranch {
        block_id: "pp-0538",
        branch_ordinal: 1,
        branch_line: 663,
        directive: "#if defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER)",
        active_branch_path: "(defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER))",
    },
    ConditionalBranch {
        block_id: "pp-0538",
        branch_ordinal: 2,
        branch_line: 676,
        directive: "#else",
        active_branch_path: "(!((defined(@PLS_IMPL_STORAGE_TEXTURE) || defined(@PLS_IMPL_STORAGE_BUFFER))))",
    },
    ConditionalBranch {
        block_id: "pp-0539",
        branch_ordinal: 1,
        branch_line: 685,
        directive: "#ifndef PLS_FRAG_COLOR_MAIN",
        active_branch_path: "(!defined(PLS_FRAG_COLOR_MAIN))",
    },
    ConditionalBranch {
        block_id: "pp-0540",
        branch_ordinal: 1,
        branch_line: 693,
        directive: "#if defined(@TARGET_SPIRV) && !defined(@TARGET_WGSL)",
        active_branch_path: "(defined(@TARGET_SPIRV) && !defined(@TARGET_WGSL))",
    },
    ConditionalBranch {
        block_id: "pp-0540",
        branch_ordinal: 2,
        branch_line: 704,
        directive: "#else",
        active_branch_path: "(!((defined(@TARGET_SPIRV) && !defined(@TARGET_WGSL))))",
    },
    ConditionalBranch {
        block_id: "pp-0541",
        branch_ordinal: 1,
        branch_line: 715,
        directive: "#if @GLSL_VERSION < 310",
        active_branch_path: "(@GLSL_VERSION < 310)",
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
        source_line: 10,
        source_name: "@GLSL_VERSION",
        generated_name: "EC",
        generated_header_name: "GLSL_GLSL_VERSION",
    },
    ExportedSymbol {
        source_line: 61,
        source_name: "@ENABLE_KHR_BLEND",
        generated_name: "ZD",
        generated_header_name: "GLSL_ENABLE_KHR_BLEND",
    },
    ExportedSymbol {
        source_line: 68,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    },
    ExportedSymbol {
        source_line: 70,
        source_name: "@ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT",
    },
    ExportedSymbol {
        source_line: 72,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedSymbol {
        source_line: 74,
        source_name: "@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    },
    ExportedSymbol {
        source_line: 84,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 84,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 84,
        source_name: "@DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
        generated_name: "EE",
        generated_header_name: "GLSL_DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
    },
    ExportedSymbol {
        source_line: 114,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 132,
        source_name: "@TARGET_SPIRV",
        generated_name: "VB",
        generated_header_name: "GLSL_TARGET_SPIRV",
    },
    ExportedSymbol {
        source_line: 150,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 200,
        source_name: "@USE_WEBGPU_SAMPLERS",
        generated_name: "BF",
        generated_header_name: "GLSL_USE_WEBGPU_SAMPLERS",
    },
    ExportedSymbol {
        source_line: 284,
        source_name: "@DISABLE_SHADER_STORAGE_BUFFERS",
        generated_name: "CF",
        generated_header_name: "GLSL_DISABLE_SHADER_STORAGE_BUFFERS",
    },
    ExportedSymbol {
        source_line: 337,
        source_name: "@PLS_IMPL_STORAGE_BUFFER",
        generated_name: "PD",
        generated_header_name: "GLSL_PLS_IMPL_STORAGE_BUFFER",
    },
    ExportedSymbol {
        source_line: 353,
        source_name: "@TARGET_WGSL",
        generated_name: "FE",
        generated_header_name: "GLSL_TARGET_WGSL",
    },
    ExportedSymbol {
        source_line: 390,
        source_name: "@PLS_IMPL_STORAGE_TEXTURE",
        generated_name: "QD",
        generated_header_name: "GLSL_PLS_IMPL_STORAGE_TEXTURE",
    },
    ExportedSymbol {
        source_line: 390,
        source_name: "@USING_PLS_STORAGE_TEXTURES",
        generated_name: "DF",
        generated_header_name: "GLSL_USING_PLS_STORAGE_TEXTURES",
    },
    ExportedSymbol {
        source_line: 444,
        source_name: "@PLS_IMPL_ANGLE",
        generated_name: "EXPORTED_PLS_IMPL_ANGLE",
        generated_header_name: "GLSL_PLS_IMPL_ANGLE",
    },
    ExportedSymbol {
        source_line: 468,
        source_name: "@PLS_IMPL_EXT_NATIVE",
        generated_name: "EF",
        generated_header_name: "GLSL_PLS_IMPL_EXT_NATIVE",
    },
    ExportedSymbol {
        source_line: 470,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 540,
        source_name: "@PLS_IMPL_SUBPASS_LOAD",
        generated_name: "FF",
        generated_header_name: "GLSL_PLS_IMPL_SUBPASS_LOAD",
    },
    ExportedSymbol {
        source_line: 574,
        source_name: "@PLS_IMPL_NONE",
        generated_name: "GF",
        generated_header_name: "GLSL_PLS_IMPL_NONE",
    },
    ExportedSymbol {
        source_line: 603,
        source_name: "@ENABLE_INSTANCE_INDEX",
        generated_name: "GE",
        generated_header_name: "GLSL_ENABLE_INSTANCE_INDEX",
    },
    ExportedSymbol {
        source_line: 607,
        source_name: "@BASE_INSTANCE_UNIFORM_NAME",
        generated_name: "RD",
        generated_header_name: "GLSL_BASE_INSTANCE_UNIFORM_NAME",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "GLSL_VERSION",
    "ENABLE_KHR_BLEND",
    "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    "ATLAS_RENDER_TARGET_R8_PLS_EXT",
    "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    "RENDER_MODE_MSAA",
    "ENABLE_CLIP_RECT",
    "DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS",
    "VERTEX",
    "TARGET_SPIRV",
    "FRAGMENT",
    "USE_WEBGPU_SAMPLERS",
    "DISABLE_SHADER_STORAGE_BUFFERS",
    "PLS_IMPL_STORAGE_BUFFER",
    "TARGET_WGSL",
    "PLS_IMPL_STORAGE_TEXTURE",
    "USING_PLS_STORAGE_TEXTURES",
    "PLS_IMPL_ANGLE",
    "PLS_IMPL_EXT_NATIVE",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "PLS_IMPL_SUBPASS_LOAD",
    "PLS_IMPL_NONE",
    "ENABLE_INSTANCE_INDEX",
    "BASE_INSTANCE_UNIFORM_NAME",
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

pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[ShaderFunction {
    source_line: 717,
    end_line: 721,
    name: "polyfill_unpackUnorm4x8",
    signature: "INLINE half4 polyfill_unpackUnorm4x8(uint u)",
    guard_path: "(@GLSL_VERSION < 310)",
    inline_qualifier: "INLINE",
}];

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

/// This shader source has no direct #include/#import directive or incoming
/// include/source dependency authority entry.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
