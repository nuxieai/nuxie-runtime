/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/metal/draw.metal.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal/draw.metal";
pub const PINNED_SOURCE_STAGE: &str = "metal-input";
pub const PINNED_SOURCE_SHA256: &str =
    "1111713584059e5d2b6469d45200b5c11949de17d7dcb7ffe62529c96c6269bd";
pub const PINNED_SOURCE_LINE_COUNT: usize = 42;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 947;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/metal/draw_metal.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned Metal source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_METAL_SOURCE: &str = r###"#include <metal_stdlib>

// Add baseInstance to the instanceID for path draws.
#define ENABLE_INSTANCE_INDEX

#define FRAGMENT

#define VERTEX
#include "metal.minified.glsl"
#include "constants.minified.glsl"
#define DRAW_IMAGE

#include "flush_uniforms.minified.glsl"
#include "common.minified.glsl"
#undef DRAW_IMAGE
#define DRAW_PATH
#define DRAW_INTERIOR_TRIANGLES
#define ENABLE_FEATHER
#define FEATHER_ATLAS_BLIT
#include "draw_path_common.minified.glsl"

#define ATLAS_FEATHERED_FILL
#define ATLAS_FEATHERED_STROKE
#include "render_atlas.minified.glsl"
#undef ATLAS_FEATHERED_FILL
#undef ATLAS_FEATHERED_STROKE

#undef FEATHER_ATLAS_BLIT
#undef ENABLE_FEATHER
#undef DRAW_INTERIOR_TRIANGLES
#undef DRAW_PATH
#undef VERTEX

#define ENABLE_ADVANCED_BLEND 1
#define ENABLE_HSL_BLEND_MODES 1
#include "advanced_blend.minified.glsl"
#undef ENABLE_HSL_BLEND_MODES
#undef ENABLE_ADVANCED_BLEND

#undef FRAGMENT

#include "draw_combinations.metal"
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_SOURCE: &str = PINNED_DRAW_METAL_SOURCE;
pub const DRAW_METAL_SOURCE: &str = PINNED_DRAW_METAL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_METAL_SOURCE
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

/// The pinned wrapper has no #if/#ifdef/#ifndef conditional block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[];

/// The pinned wrapper has no conditional branch directive.
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

/// draw.metal has no direct @-prefixed generated-shader exports.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[];
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Entry points and function declarations are provided by included shader
/// units; the pinned wrapper itself declares none directly.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[];
pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;
pub const EXPORTED_ENTRYPOINTS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[];
pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[];

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

/// Direct #include inventory, retained in source order.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 1,
        directive: "include",
        include_token: "metal_stdlib",
        include_syntax: "angle",
        active_branch_path: "all",
        resolution_kind: "toolchain-header",
        resolved_source: "toolchain:metal_stdlib",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: "-",
        correspondence_owner: "toolchain:metal_stdlib",
        mapping_status: "prepared",
        translation_status: "pending",
        translation_disposition: "toolchain-provided",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 9,
        directive: "include",
        include_token: "metal.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 10,
        directive: "include",
        include_token: "constants.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/constants.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 13,
        directive: "include",
        include_token: "flush_uniforms.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 14,
        directive: "include",
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 20,
        directive: "include",
        include_token: "draw_path_common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 24,
        directive: "include",
        include_token: "render_atlas.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/render_atlas.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 36,
        directive: "include",
        include_token: "advanced_blend.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/advanced_blend.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: PINNED_SOURCE_PATH,
        include_line: 42,
        directive: "include",
        include_token: "draw_combinations.metal",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal/generate_draw_combinations.py",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
];

/// No source outside this wrapper includes draw.metal directly in the pinned
/// include authority.
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

/// Direct source dependency edges, retained in authority order.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 1,
        include_token: "metal_stdlib",
        include_syntax: "angle",
        active_branch_path: "all",
        resolution_kind: "toolchain-header",
        resolved_source: "toolchain:metal_stdlib",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: "external",
        translation_disposition: "provided-by-toolchain",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 9,
        include_token: "metal.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 10,
        include_token: "constants.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/constants.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 13,
        include_token: "flush_uniforms.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/flush_uniforms.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 14,
        include_token: "common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/common.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 20,
        include_token: "draw_path_common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 24,
        include_token: "render_atlas.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/render_atlas.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 36,
        include_token: "advanced_blend.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/advanced_blend.glsl",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: PINNED_SOURCE_PATH,
        include_line: 42,
        include_token: "draw_combinations.metal",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/metal/generate_draw_combinations.py",
        source_unit: TRANSLATION_UNIT,
        dependency_unit: TRANSLATION_UNIT,
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[
    "metal_stdlib",
    "metal.minified.glsl",
    "constants.minified.glsl",
    "flush_uniforms.minified.glsl",
    "common.minified.glsl",
    "draw_path_common.minified.glsl",
    "render_atlas.minified.glsl",
    "advanced_blend.minified.glsl",
    "draw_combinations.metal",
];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
