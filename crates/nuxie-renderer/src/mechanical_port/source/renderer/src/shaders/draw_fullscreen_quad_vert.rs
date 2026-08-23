/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_fullscreen_quad.vert.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_fullscreen_quad.vert";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-vert";
pub const PINNED_SOURCE_SHA256: &str =
    "6a9842803e8472ab8f756a191c6a6d60a7c28db5587ee22c5e9bddb000c49cc2";
pub const PINNED_SOURCE_LINE_COUNT: usize = 15;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 335;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_fullscreen_quad_vert.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned vertex-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @VERTEX
void main()
{
    // Fill the entire screen. The caller will use a scissor test to control the
    // bounds being drawn.
    gl_Position.x = (gl_VertexID & 1) == 0 ? -1. : 1.;
    gl_Position.y = (gl_VertexID & 2) == 0 ? -1. : 1.;
    gl_Position.z = 0.;
    gl_Position.w = 1.;
}
#endif
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_FULLSCREEN_QUAD_SOURCE: &str = PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE;
pub const DRAW_FULLSCREEN_QUAD_VERT_SOURCE: &str = PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_FULLSCREEN_QUAD_VERT_SOURCE
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

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[ConditionalBlock {
    block_id: "pp-0327",
    block_start: 5,
    block_end: 15,
    block_depth: 0,
    branch_count: 1,
}];

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

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[ConditionalBranch {
    block_id: "pp-0327",
    branch_ordinal: 1,
    branch_line: 5,
    directive: "#ifdef @VERTEX",
    active_branch_path: "(defined(@VERTEX))",
}];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The sole @-prefixed identifier occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[ExportedSymbol {
    source_line: 5,
    source_name: "@VERTEX",
    generated_name: "CB",
    generated_header_name: "GLSL_VERTEX",
}];

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

/// The ordinary vertex entrypoint is retained as a source spelling and range.
/// Its body remains in the pinned source rather than becoming executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[ShaderFunction {
    source_line: 6,
    end_line: 14,
    name: "main",
    signature: "void main()",
    guard_path: "(defined(@VERTEX))",
    inline_qualifier: "",
}];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings without the leading @.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[ExportedIdentifier {
    source_name: "VERTEX",
    generated_name: "CB",
}];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &["VERTEX"];

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

/// draw_fullscreen_quad.vert has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// No incoming include/source edge is recorded for this owner in the pinned
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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
