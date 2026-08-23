/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/specialization.glsl.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "e510983192e84c1cc69d990a14f606cfa92af234290636593d5be3e3f4e07f72";
pub const PINNED_SOURCE_LINE_COUNT: usize = 43;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2044;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/specialization_glsl.rs";
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
pub const PINNED_SPECIALIZATION_GLSL_SOURCE: &str = r###"layout(constant_id = CLIPPING_SPECIALIZATION_IDX) const
    bool EnableClipping = true;
layout(constant_id = CLIP_RECT_SPECIALIZATION_IDX) const
    bool EnableClipRect = true;
layout(constant_id = ADVANCED_BLEND_SPECIALIZATION_IDX) const
    bool EnableAdvancedBlend = true;
layout(constant_id = FEATHER_SPECIALIZATION_IDX) const
    bool EnableFeather = true;
layout(constant_id = EVEN_ODD_SPECIALIZATION_IDX) const
    bool EnableEvenOdd = true;
layout(constant_id = NESTED_CLIPPING_SPECIALIZATION_IDX) const
    bool EnableNestedClipping = true;
layout(constant_id = HSL_BLEND_MODES_SPECIALIZATION_IDX) const
    bool EnableHSLBlendModes = true;
layout(constant_id = DITHER_SPECIALIZATION_IDX) const bool EnableDither = true;
layout(constant_id = CLOCKWISE_FILL_SPECIALIZATION_IDX) const
    bool ClockwiseFill = true;
layout(constant_id = BORROWED_COVERAGE_PASS_SPECIALIZATION_IDX) const
    bool BorrowedCoveragePrepass = false;
layout(constant_id = NESTED_CLIP_UPDATE_ONLY_IDX) const
    bool NestedClipUpdateOnly = false;
layout(constant_id = VULKAN_VENDOR_ARM_SPECIALIZATION_IDX) const
    bool VulkanVendorARM = false;
layout(constant_id = STORE_COLOR_CLEAR_SPECIALIZATION_IDX) const
    bool StoreColorClear = false;
layout(constant_id = LOAD_COLOR_FROM_DST_TEXTURE_SPECIALIZATION_IDX) const
    bool LoadColorFromDstTexture = false;

#define @ENABLE_CLIPPING EnableClipping
#define @ENABLE_CLIP_RECT EnableClipRect
#define @ENABLE_ADVANCED_BLEND EnableAdvancedBlend
#define @DISABLE_ADVANCED_BLEND DisableAdvancedBlend
#define @ENABLE_FEATHER EnableFeather
#define @ENABLE_EVEN_ODD EnableEvenOdd
#define @ENABLE_NESTED_CLIPPING EnableNestedClipping
#define @ENABLE_HSL_BLEND_MODES EnableHSLBlendModes
#define @ENABLE_DITHER EnableDither
#define @CLOCKWISE_FILL ClockwiseFill
#define @BORROWED_COVERAGE_PASS BorrowedCoveragePrepass
#define @NESTED_CLIP_UPDATE_ONLY NestedClipUpdateOnly
#define @VULKAN_VENDOR_ARM VulkanVendorARM
#define @STORE_COLOR_CLEAR StoreColorClear
#define @LOAD_COLOR_FROM_DST_TEXTURE LoadColorFromDstTexture
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_SPECIALIZATION_SOURCE: &str = PINNED_SPECIALIZATION_GLSL_SOURCE;
pub const SPECIALIZATION_GLSL_SOURCE: &str = PINNED_SPECIALIZATION_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_SPECIALIZATION_GLSL_SOURCE
}

/// Every semantic preprocessor block in the pinned source remains literal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[];

/// Every branch entry remains literal, in authority/source order. This source
/// contains no conditional preprocessor directives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The fifteen @-prefixed identifiers exported by the pinned batch minifier.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 29,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 30,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@DISABLE_ADVANCED_BLEND",
        generated_name: "NF",
        generated_header_name: "GLSL_DISABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@ENABLE_FEATHER",
        generated_name: "HB",
        generated_header_name: "GLSL_ENABLE_FEATHER",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@ENABLE_EVEN_ODD",
        generated_name: "PC",
        generated_header_name: "GLSL_ENABLE_EVEN_ODD",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
        generated_header_name: "GLSL_ENABLE_NESTED_CLIPPING",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@ENABLE_HSL_BLEND_MODES",
        generated_name: "XB",
        generated_header_name: "GLSL_ENABLE_HSL_BLEND_MODES",
    },
    ExportedSymbol {
        source_line: 37,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
    },
    ExportedSymbol {
        source_line: 38,
        source_name: "@CLOCKWISE_FILL",
        generated_name: "UD",
        generated_header_name: "GLSL_CLOCKWISE_FILL",
    },
    ExportedSymbol {
        source_line: 39,
        source_name: "@BORROWED_COVERAGE_PASS",
        generated_name: "WB",
        generated_header_name: "GLSL_BORROWED_COVERAGE_PASS",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
        generated_header_name: "GLSL_NESTED_CLIP_UPDATE_ONLY",
    },
    ExportedSymbol {
        source_line: 41,
        source_name: "@VULKAN_VENDOR_ARM",
        generated_name: "WC",
        generated_header_name: "GLSL_VULKAN_VENDOR_ARM",
    },
    ExportedSymbol {
        source_line: 42,
        source_name: "@STORE_COLOR_CLEAR",
        generated_name: "ED",
        generated_header_name: "GLSL_STORE_COLOR_CLEAR",
    },
    ExportedSymbol {
        source_line: 43,
        source_name: "@LOAD_COLOR_FROM_DST_TEXTURE",
        generated_name: "FD",
        generated_header_name: "GLSL_LOAD_COLOR_FROM_DST_TEXTURE",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "BORROWED_COVERAGE_PASS",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "CLOCKWISE_FILL",
        generated_name: "UD",
    },
    ExportedIdentifier {
        source_name: "DISABLE_ADVANCED_BLEND",
        generated_name: "NF",
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
        source_name: "ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
    },
    ExportedIdentifier {
        source_name: "LOAD_COLOR_FROM_DST_TEXTURE",
        generated_name: "FD",
    },
    ExportedIdentifier {
        source_name: "NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
    },
    ExportedIdentifier {
        source_name: "STORE_COLOR_CLEAR",
        generated_name: "ED",
    },
    ExportedIdentifier {
        source_name: "VULKAN_VENDOR_ARM",
        generated_name: "WC",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "ENABLE_ADVANCED_BLEND",
    "DISABLE_ADVANCED_BLEND",
    "ENABLE_FEATHER",
    "ENABLE_EVEN_ODD",
    "ENABLE_NESTED_CLIPPING",
    "ENABLE_HSL_BLEND_MODES",
    "ENABLE_DITHER",
    "CLOCKWISE_FILL",
    "BORROWED_COVERAGE_PASS",
    "NESTED_CLIP_UPDATE_ONLY",
    "VULKAN_VENDOR_ARM",
    "STORE_COLOR_CLEAR",
    "LOAD_COLOR_FROM_DST_TEXTURE",
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

/// This specialization source has no function declarations.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[];
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

/// specialization.glsl has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[IncludeDependency] = &[];

/// Incoming generated-source edges retained from the pinned shader consumers.
pub const CONSUMER_INCLUDE_AUTHORITY: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/atomic_base.glsl",
        include_line: 12,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/clear_clockwise_atomic_clip.main",
        include_line: 13,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_atlas_blit.main",
        include_line: 8,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atlas_blit.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_atlas_blit.main",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_borrowed_coverage.frag",
        include_line: 11,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_borrowed_coverage_interior_triangles.frag",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_clip.frag",
        include_line: 11,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_clip_interior_triangles.frag",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_image_mesh.main",
        include_line: 11,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_interior_triangles.main",
        include_line: 13,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_atomic_path.main",
        include_line: 14,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_clip.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_clip_interior_triangles.main",
        include_line: 8,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_image_mesh.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_interior_triangles.main",
        include_line: 8,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_clockwise_path.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_image_mesh.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_interior_triangles.main",
        include_line: 8,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_msaa_atlas_blit.main",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_msaa_image_mesh.main",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_msaa_path.main",
        include_line: 10,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_msaa_stencil.main",
        include_line: 13,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/spirv/draw_path.main",
        include_line: 9,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "spirv-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/unreal/draw_clockwise_atlas_blit.usf",
        include_line: 15,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "unreal-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/unreal/draw_clockwise_image_mesh.usf",
        include_line: 15,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "unreal-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/unreal/draw_clockwise_interior_triangles.usf",
        include_line: 14,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "unreal-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/unreal/draw_clockwise_path.usf",
        include_line: 15,
        include_token: "specialization.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/specialization.glsl",
        source_unit: "unreal-shader-source",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "required-source-edge",
    },
];

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = CONSUMER_INCLUDE_AUTHORITY;
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
