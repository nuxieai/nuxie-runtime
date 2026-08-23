/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_mesh.frag.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, function entrypoints, direct include
 * inventory, and source metadata as literal source-shaped data. It does not
 * compile, evaluate, simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "d3a060c05d66e187ca2a0edab03788e7d885364ba833237e2e89f39a2b5e9c1f";
pub const PINNED_SOURCE_LINE_COUNT: usize = 227;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6989;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_mesh_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_MESH_FRAG_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

#ifdef @FRAGMENT

// This is a basic fragment shader for non-msaa, non-path objects, e.g., image
// meshes, atlas blits.
// These objects are simple in that they can write their fragments out directly,
// without having to cooperate with overlapping fragments to work out coverage.

#if (defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@ENABLE_CLIPPING)) ||   \
    defined(@RENDER_MODE_CLOCKWISE_ATOMIC)
// @FIXED_FUNCTION_COLOR_OUTPUT without clipping can skip the interlock.
#undef NEEDS_INTERLOCK
#else
#define NEEDS_INTERLOCK
#endif

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(SCRATCH_COLOR_PLANE_IDX, scratchColorBuffer);
#endif
PLS_DECLUI(COVERAGE_PLANE_IDX, coverageBuffer);
#else // @RENDER_MODE_CLOCKWISE_ATOMIC
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
#endif
PLS_BLOCK_END

// FEATHER_ATLAS_BLIT includes draw_path_common.glsl, which declares the
// textures & samplers, so we only need to declare these for image meshes.
#ifdef @DRAW_IMAGE_MESH
FRAG_TEXTURE_BLOCK_BEGIN
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @imageTexture);
FRAG_TEXTURE_BLOCK_END

DYNAMIC_SAMPLER_BLOCK_BEGIN
SAMPLER_DYNAMIC_IMAGE(imageSampler)
DYNAMIC_SAMPLER_BLOCK_END

FRAG_STORAGE_BUFFER_BLOCK_BEGIN
FRAG_STORAGE_BUFFER_BLOCK_END
#endif // @DRAW_IMAGE_MESH

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
#ifdef @DRAW_IMAGE_MESH
PLS_FRAG_COLOR_MAIN(@drawFragmentMain)
#else
PLS_FRAG_COLOR_MAIN(@drawFragmentMain)
#endif
#else
#ifdef @DRAW_IMAGE_MESH
PLS_MAIN(@drawFragmentMain)
#else
PLS_MAIN(@drawFragmentMain)
#endif
#endif
{
#ifdef @FEATHER_ATLAS_BLIT
    VARYING_UNPACK(v_paint, float4);
    VARYING_UNPACK(v_atlasCoord, float2);
#endif
#ifdef @ENABLE_CLIPPING
    VARYING_UNPACK(v_clipID, half);
#endif
#ifdef @ENABLE_CLIP_RECT
    VARYING_UNPACK(v_clipRect, float4);
#endif
#if defined(@FEATHER_ATLAS_BLIT) && defined(@ENABLE_ADVANCED_BLEND)
    VARYING_UNPACK(v_blendMode, half);
#endif
#ifdef @DRAW_IMAGE_MESH
    VARYING_UNPACK(v_imageTexCoord, float2);
    VARYING_UNPACK(v_imageOpacity, half);
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_imageBlendMode, ushort);
#endif
#endif

#ifdef @FEATHER_ATLAS_BLIT
    half4 color = find_paint_color(v_paint, 1. FRAGMENT_CONTEXT_UNPACK);
    half coverage = clamp(TEXTURE_SAMPLE_LOD(@featherAtlasTexture,
                                             featherAtlasSampler,
                                             v_atlasCoord,
                                             .0)
                              .r,
                          make_half(.0),
                          make_half(1.));
#endif

#ifdef @DRAW_IMAGE_MESH
    half4 color = TEXTURE_SAMPLE_DYNAMIC_LODBIAS(@imageTexture,
                                                 imageSampler,
                                                 v_imageTexCoord,
                                                 uniforms.mipMapLODBias);
    half coverage = 1.;
#endif

#ifdef @ENABLE_CLIP_RECT
    // Calculate the clip rect before entering the interlock.
    if (@ENABLE_CLIP_RECT)
    {
        half clipRectCoverage =
            max(min_component(cast_float4_to_half4(v_clipRect)), make_half(.0));
        coverage = min(clipRectCoverage, coverage);
    }
#endif

#ifdef NEEDS_INTERLOCK
    PLS_INTERLOCK_BEGIN;
#endif

#if defined(@ENABLE_CLIPPING)
    if (@ENABLE_CLIPPING && v_clipID != .0)
    {
        half clipCoverage;
#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC
        half2 clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
        half clipContentID = clipData.g;
        clipCoverage =
            max(clipContentID == v_clipID ? clipData.r : make_half(.0),
                make_half(.0));
#else
        clipCoverage = PLS_LOAD4F(clipBuffer).r;
#endif
        clipCoverage = max(clipCoverage, make_half(.0));
        coverage = min(coverage, clipCoverage);
    }
#endif

#ifdef @DRAW_IMAGE_MESH
    // Apply opacity after clipping.
    coverage *= v_imageOpacity;
#endif

#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
    half4 dstColorPremul = PLS_LOAD4F(colorBuffer);
#ifdef @ENABLE_ADVANCED_BLEND
    if (@ENABLE_ADVANCED_BLEND)
    {
#ifdef @FEATHER_ATLAS_BLIT
        // GENERATE_PREMULTIPLIED_PAINT_COLORS is false in this case for
        // find_paint_color() because advanced blend needs unmultiplied colors.
        ushort blendMode = cast_half_to_ushort(v_blendMode);
#endif

#ifdef @DRAW_IMAGE_MESH
        // Unmultiply the image for advanced blend. Images are always
        // premultiplied so that the filtering works correctly.
        // TODO: This unmultiply technically isn't necessary with srcOver blend.
        // We may want to experiment with dynamically not premultiplying here
        // and in find_paint_color() when the blend mode is srcOver.
        color.rgb = unmultiply_rgb(color);
        ushort blendMode = v_imageBlendMode;
#endif

        if (blendMode != BLEND_SRC_OVER)
        {
            color.rgb =
                advanced_color_blend(color.rgb, dstColorPremul, blendMode);
        }
        // Premultiply alpha now.
        color.a *= coverage;
        color.rgb *= color.a;
    }
    else
#endif // @ENABLE_ADVANCED_BLEND
    {
        color *= coverage;
    }

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

#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC
    color = dstColorPremul * (1. - color.a) + color;
#endif

    PLS_STORE4F(colorBuffer, color);
#endif // !@FIXED_FUNCTION_COLOR_OUTPUT

#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC
    PLS_PRESERVE_UI(clipBuffer);
    PLS_PRESERVE_UI(coverageBuffer);
#else
    // Since blend is enabled, storing 0 to the clip will ensure it remains
    // unchanged.
    PLS_STORE4F(clipBuffer, make_half4(.0));
#endif
#ifdef NEEDS_INTERLOCK
    PLS_INTERLOCK_END;
#endif

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    color = (color * coverage);
    color.rgb = add_dither_if_alpha_nonzero(color.rgb,
                                            color.a,
                                            _fragCoord.xy,
                                            uniforms.ditherScale,
                                            uniforms.ditherBias);
    _fragColor = color;
    EMIT_PLS_AND_FRAG_COLOR
#else
    EMIT_PLS;
#endif
}

#endif // @FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_MESH_SOURCE: &str = PINNED_DRAW_MESH_FRAG_SOURCE;
pub const DRAW_MESH_FRAG_SOURCE: &str = PINNED_DRAW_MESH_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_MESH_FRAG_SOURCE
}

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
        block_id: "pp-0349",
        block_start: 5,
        block_end: 227,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0350",
        block_start: 12,
        block_end: 18,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0351",
        block_start: 21,
        block_end: 23,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0352",
        block_start: 24,
        block_end: 32,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0353",
        block_start: 26,
        block_end: 28,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0354",
        block_start: 37,
        block_end: 48,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0355",
        block_start: 50,
        block_end: 62,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0356",
        block_start: 51,
        block_end: 55,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0357",
        block_start: 57,
        block_end: 61,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0358",
        block_start: 64,
        block_end: 67,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0359",
        block_start: 68,
        block_end: 70,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0360",
        block_start: 71,
        block_end: 73,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0361",
        block_start: 74,
        block_end: 76,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0362",
        block_start: 77,
        block_end: 83,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0363",
        block_start: 80,
        block_end: 82,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0364",
        block_start: 85,
        block_end: 94,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0365",
        block_start: 96,
        block_end: 102,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0366",
        block_start: 104,
        block_end: 112,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0367",
        block_start: 114,
        block_end: 116,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0368",
        block_start: 118,
        block_end: 134,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0369",
        block_start: 122,
        block_end: 130,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0370",
        block_start: 136,
        block_end: 139,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0371",
        block_start: 141,
        block_end: 199,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0372",
        block_start: 143,
        block_end: 172,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0373",
        block_start: 146,
        block_end: 150,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0374",
        block_start: 152,
        block_end: 160,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0375",
        block_start: 181,
        block_end: 186,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0376",
        block_start: 194,
        block_end: 196,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0377",
        block_start: 201,
        block_end: 208,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0378",
        block_start: 209,
        block_end: 211,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0379",
        block_start: 213,
        block_end: 224,
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
        block_id: "pp-0349",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0350",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#if (defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@ENABLE_CLIPPING)) || defined(@RENDER_MODE_CLOCKWISE_ATOMIC)",
        active_branch_path: "(defined(@FRAGMENT)) && ((defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@ENABLE_CLIPPING)) || defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0350",
        branch_ordinal: 2,
        branch_line: 16,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!(((defined(@FIXED_FUNCTION_COLOR_OUTPUT) && !defined(@ENABLE_CLIPPING)) || defined(@RENDER_MODE_CLOCKWISE_ATOMIC))))",
    },
    ConditionalBranch {
        block_id: "pp-0351",
        branch_ordinal: 1,
        branch_line: 21,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0352",
        branch_ordinal: 1,
        branch_line: 24,
        directive: "#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0352",
        branch_ordinal: 2,
        branch_line: 30,
        directive: "#else // @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@FRAGMENT)) && (!((!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))))",
    },
    ConditionalBranch {
        block_id: "pp-0353",
        branch_ordinal: 1,
        branch_line: 26,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@RENDER_MODE_CLOCKWISE_ATOMIC)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0354",
        branch_ordinal: 1,
        branch_line: 37,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0355",
        branch_ordinal: 1,
        branch_line: 50,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0355",
        branch_ordinal: 2,
        branch_line: 56,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0356",
        branch_ordinal: 1,
        branch_line: 51,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0356",
        branch_ordinal: 2,
        branch_line: 53,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!((defined(@DRAW_IMAGE_MESH))))",
    },
    ConditionalBranch {
        block_id: "pp-0357",
        branch_ordinal: 1,
        branch_line: 57,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT)))) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0357",
        branch_ordinal: 2,
        branch_line: 59,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT)))) && (!((defined(@DRAW_IMAGE_MESH))))",
    },
    ConditionalBranch {
        block_id: "pp-0358",
        branch_ordinal: 1,
        branch_line: 64,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0359",
        branch_ordinal: 1,
        branch_line: 68,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0360",
        branch_ordinal: 1,
        branch_line: 71,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0361",
        branch_ordinal: 1,
        branch_line: 74,
        directive: "#if defined(@FEATHER_ATLAS_BLIT) && defined(@ENABLE_ADVANCED_BLEND)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FEATHER_ATLAS_BLIT) && defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0362",
        branch_ordinal: 1,
        branch_line: 77,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0363",
        branch_ordinal: 1,
        branch_line: 80,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0364",
        branch_ordinal: 1,
        branch_line: 85,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0365",
        branch_ordinal: 1,
        branch_line: 96,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0366",
        branch_ordinal: 1,
        branch_line: 104,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0367",
        branch_ordinal: 1,
        branch_line: 114,
        directive: "#ifdef NEEDS_INTERLOCK",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(NEEDS_INTERLOCK))",
    },
    ConditionalBranch {
        block_id: "pp-0368",
        branch_ordinal: 1,
        branch_line: 118,
        directive: "#if defined(@ENABLE_CLIPPING)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0369",
        branch_ordinal: 1,
        branch_line: 122,
        directive: "#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0369",
        branch_ordinal: 2,
        branch_line: 128,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING)) && (!((!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))))",
    },
    ConditionalBranch {
        block_id: "pp-0370",
        branch_ordinal: 1,
        branch_line: 136,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0371",
        branch_ordinal: 1,
        branch_line: 141,
        directive: "#if !defined(@FIXED_FUNCTION_COLOR_OUTPUT)",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0372",
        branch_ordinal: 1,
        branch_line: 143,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0373",
        branch_ordinal: 1,
        branch_line: 146,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0374",
        branch_ordinal: 1,
        branch_line: 152,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0375",
        branch_ordinal: 1,
        branch_line: 181,
        directive: "#ifdef @NEEDS_GAMMA_CORRECTION",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
    },
    ConditionalBranch {
        block_id: "pp-0376",
        branch_ordinal: 1,
        branch_line: 194,
        directive: "#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0377",
        branch_ordinal: 1,
        branch_line: 201,
        directive: "#ifndef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0377",
        branch_ordinal: 2,
        branch_line: 204,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((!defined(@RENDER_MODE_CLOCKWISE_ATOMIC))))",
    },
    ConditionalBranch {
        block_id: "pp-0378",
        branch_ordinal: 1,
        branch_line: 209,
        directive: "#ifdef NEEDS_INTERLOCK",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(NEEDS_INTERLOCK))",
    },
    ConditionalBranch {
        block_id: "pp-0379",
        branch_ordinal: 1,
        branch_line: 213,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0379",
        branch_ordinal: 2,
        branch_line: 222,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The twelve @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
        generated_header_name: "GLSL_RENDER_MODE_CLOCKWISE_ATOMIC",
    },
    ExportedSymbol {
        source_line: 37,
        source_name: "@DRAW_IMAGE_MESH",
        generated_name: "LB",
        generated_header_name: "GLSL_DRAW_IMAGE_MESH",
    },
    ExportedSymbol {
        source_line: 39,
        source_name: "@imageTexture",
        generated_name: "AC",
        generated_header_name: "GLSL_imageTexture",
    },
    ExportedSymbol {
        source_line: 52,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 64,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "EB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 71,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 74,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 87,
        source_name: "@featherAtlasTexture",
        generated_name: "UC",
        generated_header_name: "GLSL_atlasTexture",
    },
    ExportedSymbol {
        source_line: 181,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    EXPORTED_SYMBOLS[0],
    EXPORTED_SYMBOLS[1],
    EXPORTED_SYMBOLS[2],
    EXPORTED_SYMBOLS[3],
    EXPORTED_SYMBOLS[4],
    EXPORTED_SYMBOLS[7],
    EXPORTED_SYMBOLS[8],
    EXPORTED_SYMBOLS[9],
    EXPORTED_SYMBOLS[11],
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

/// All four conditional macro-defined fragment entrypoint declarations are
/// retained as source spellings and ranges. Their shared body remains in the
/// pinned source rather than becoming an executable Rust function.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 52,
        end_line: 225,
        name: "drawFragmentMain",
        signature: "PLS_FRAG_COLOR_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@DRAW_IMAGE_MESH))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 54,
        end_line: 225,
        name: "drawFragmentMain",
        signature: "PLS_FRAG_COLOR_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!((defined(@DRAW_IMAGE_MESH))))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 58,
        end_line: 225,
        name: "drawFragmentMain",
        signature: "PLS_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT)))) && (defined(@DRAW_IMAGE_MESH))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 60,
        end_line: 225,
        name: "drawFragmentMain",
        signature: "PLS_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT)))) && (!((defined(@DRAW_IMAGE_MESH))))",
        inline_qualifier: "",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings without the leading @ and
/// the generated names assigned by the pinned batch minifier.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIPPING",
        generated_name: "O",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
    },
    ExportedIdentifier {
        source_name: "DRAW_IMAGE_MESH",
        generated_name: "LB",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "FEATHER_ATLAS_BLIT",
        generated_name: "EB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_CLIP_RECT",
        generated_name: "AB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "featherAtlasTexture",
        generated_name: "UC",
    },
    ExportedIdentifier {
        source_name: "NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "ENABLE_CLIPPING",
    "RENDER_MODE_CLOCKWISE_ATOMIC",
    "DRAW_IMAGE_MESH",
    "imageTexture",
    "drawFragmentMain",
    "FEATHER_ATLAS_BLIT",
    "ENABLE_CLIP_RECT",
    "ENABLE_ADVANCED_BLEND",
    "featherAtlasTexture",
    "NEEDS_GAMMA_CORRECTION",
];

/// These source spellings share a generated name with a differently named
/// global export-header identifier in the pinned generated shader batch.
pub const EXPORT_MAPPING_AMBIGUITIES: &[(&str, &str, &str)] =
    &[("featherAtlasTexture", "atlasTexture", "UC")];

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

/// draw_mesh.frag has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source include edge retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[ShaderInclude {
    upstream_file: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 16,
    directive: "include",
    include_token: "generated/shaders/draw_mesh.frag.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_mesh.frag",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    correspondence_owner: "-",
    mapping_status: "prepared",
    translation_status: "pending",
    translation_disposition: "required-source-edge",
}];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[IncludeDependency {
    including_source: "renderer/src/metal/background_shader_compiler.mm",
    include_line: 16,
    include_token: "generated/shaders/draw_mesh.frag.hpp",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/draw_mesh.frag",
    source_unit: "metal-background-shader-compiler",
    dependency_unit: "metal-shader-source-batch",
    translation_disposition: "preserve-source-dependency",
}];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
