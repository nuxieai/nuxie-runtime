/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/init_clockwise_atomic_workaround.frag.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/init_clockwise_atomic_workaround.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "092a1f498d0f6ff336edecdc73f96c46c1b3a51d494249839fe9934203dd53a3";
pub const PINNED_SOURCE_LINE_COUNT: usize = 33;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1082;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/init_clockwise_atomic_workaround_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

// This shader implements a seeming workaround for Qualcomm. Basically, input
// attachment reads of the clip and color buffers don't work unless we first
// draw these buffers into themselves between borrowed coverage and the main
// subpass. This draw is issued with a scissor that only allows one pixel
// through, so the fill rate impact should be negligible.
#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
PLS_BLOCK_END

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    // Draw the clip buffer onto itself.
    PLS_STORE4F(clipBuffer, make_half4(PLS_LOAD4F(clipBuffer).r, .0, .0, 1.));
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    // Draw the color buffer onto itself.
    EMIT_CLOCKWISE_ATOMIC_PLS(PLS_LOAD4F(colorBuffer));
#else
    // This render pass doesn't read the color buffer. Emit 0 (since srcOver
    // blend is enabled) to leave the color buffer unaffected.
    EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(.0));
#endif
}

#endif
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_SOURCE: &str =
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE;
pub const INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE: &str =
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_INIT_CLOCKWISE_ATOMIC_WORKAROUND_FRAG_SOURCE
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
        block_id: "pp-0550",
        block_start: 10,
        block_end: 33,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0551",
        block_start: 13,
        block_end: 15,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0552",
        block_start: 23,
        block_end: 30,
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
        block_id: "pp-0550",
        branch_ordinal: 1,
        branch_line: 10,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0551",
        branch_ordinal: 1,
        branch_line: 13,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0552",
        branch_ordinal: 1,
        branch_line: 23,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0552",
        branch_ordinal: 2,
        branch_line: 26,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((!defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The three @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 10,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 19,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 10,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 13,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
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
    source_line: 19,
    end_line: 31,
    name: "drawFragmentMain",
    signature: "CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)",
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
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "drawFragmentMain",
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

/// init_clockwise_atomic_workaround.frag has no direct #include/#import directive.
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
