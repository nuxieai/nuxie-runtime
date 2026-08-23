/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_msaa_object.frag.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, includes, exports, functions, and source
 * metadata as literal source-shaped data. It does not compile, evaluate,
 * simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_object.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "28ec08b53f7f32a12439d5f85481c4f5d66660f0e80e867b44fcbd35adba9d85";
pub const PINNED_SOURCE_LINE_COUNT: usize = 103;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3424;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_msaa_object_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
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
    half4 color = find_paint_color(v_paint, coverage FRAGMENT_CONTEXT_UNPACK);
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

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_MSAA_OBJECT_SOURCE: &str = PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE;
pub const DRAW_MSAA_OBJECT_FRAG_SOURCE: &str = PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_MSAA_OBJECT_FRAG_SOURCE
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
        block_id: "pp-0380",
        block_start: 5,
        block_end: 103,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0381",
        block_start: 9,
        block_end: 20,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0382",
        block_start: 12,
        block_end: 14,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0383",
        block_start: 24,
        block_end: 38,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0384",
        block_start: 27,
        block_end: 29,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0385",
        block_start: 32,
        block_end: 34,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0386",
        block_start: 35,
        block_end: 37,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0387",
        block_start: 40,
        block_end: 60,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0388",
        block_start: 48,
        block_end: 58,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0389",
        block_start: 64,
        block_end: 81,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0390",
        block_start: 66,
        block_end: 74,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0391",
        block_start: 87,
        block_end: 92,
        block_depth: 1,
        branch_count: 1,
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
        block_id: "pp-0380",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0381",
        branch_ordinal: 1,
        branch_line: 9,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0382",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0383",
        branch_ordinal: 1,
        branch_line: 24,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0383",
        branch_ordinal: 2,
        branch_line: 30,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH))))",
    },
    ConditionalBranch {
        block_id: "pp-0384",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0385",
        branch_ordinal: 1,
        branch_line: 32,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH)))) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0386",
        branch_ordinal: 1,
        branch_line: 35,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH)))) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0387",
        branch_ordinal: 1,
        branch_line: 40,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0387",
        branch_ordinal: 2,
        branch_line: 46,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH))))",
    },
    ConditionalBranch {
        block_id: "pp-0388",
        branch_ordinal: 1,
        branch_line: 48,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH)))) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0388",
        branch_ordinal: 2,
        branch_line: 56,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_IMAGE_MESH)))) && (!((defined(@FEATHER_ATLAS_BLIT))))",
    },
    ConditionalBranch {
        block_id: "pp-0389",
        branch_ordinal: 1,
        branch_line: 64,
        directive: "#if defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0390",
        branch_ordinal: 1,
        branch_line: 66,
        directive: "#ifdef @DRAW_IMAGE_MESH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@DRAW_IMAGE_MESH))",
    },
    ConditionalBranch {
        block_id: "pp-0390",
        branch_ordinal: 2,
        branch_line: 69,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (!((defined(@DRAW_IMAGE_MESH))))",
    },
    ConditionalBranch {
        block_id: "pp-0391",
        branch_ordinal: 1,
        branch_line: 87,
        directive: "#ifdef @NEEDS_GAMMA_CORRECTION",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NEEDS_GAMMA_CORRECTION))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The ten direct @-prefixed identifiers occurring in this shader source,
/// retained in first-occurrence source order.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 9,
        source_name: "@DRAW_IMAGE_MESH",
        generated_name: "LB",
        generated_header_name: "GLSL_DRAW_IMAGE_MESH",
    },
    ExportedSymbol {
        source_line: 11,
        source_name: "@imageTexture",
        generated_name: "AC",
        generated_header_name: "GLSL_imageTexture",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@dstColorTexture",
        generated_name: "LD",
        generated_header_name: "GLSL_dstColorTexture",
    },
    ExportedSymbol {
        source_line: 22,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "EB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 49,
        source_name: "@featherAtlasTexture",
        generated_name: "UC",
        generated_header_name: "GLSL_atlasTexture",
    },
    ExportedSymbol {
        source_line: 64,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 87,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 9,
        source_name: "@DRAW_IMAGE_MESH",
        generated_name: "LB",
        generated_header_name: "GLSL_DRAW_IMAGE_MESH",
    },
    ExportedSymbol {
        source_line: 12,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "EB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 64,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 87,
        source_name: "@NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
        generated_header_name: "GLSL_NEEDS_GAMMA_CORRECTION",
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

/// The macro-defined fragment entrypoint is retained as a source spelling and
/// range. Its body remains in the pinned fragment source rather than becoming
/// an executable Rust function.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[ShaderFunction {
    source_line: 22,
    end_line: 101,
    name: "drawFragmentMain",
    signature: "FRAG_DATA_MAIN(half4, @drawFragmentMain)",
    guard_path: "(defined(@FRAGMENT))",
    inline_qualifier: "",
}];

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
        source_name: "DRAW_IMAGE_MESH",
        generated_name: "LB",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "dstColorTexture",
        generated_name: "LD",
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
        source_name: "featherAtlasTexture",
        generated_name: "UC",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "NEEDS_GAMMA_CORRECTION",
        generated_name: "UB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "DRAW_IMAGE_MESH",
    "imageTexture",
    "ENABLE_ADVANCED_BLEND",
    "dstColorTexture",
    "drawFragmentMain",
    "FEATHER_ATLAS_BLIT",
    "featherAtlasTexture",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "NEEDS_GAMMA_CORRECTION",
];

/// The source spelling featherAtlasTexture uses the atlasTexture export-header alias.
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

/// draw_msaa_object.frag has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// No incoming generated-source edge is recorded for this owner in the pinned
/// include/dependency authorities; direct include inventory remains empty.
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

/// This shader source has no direct #include/#import directive.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
