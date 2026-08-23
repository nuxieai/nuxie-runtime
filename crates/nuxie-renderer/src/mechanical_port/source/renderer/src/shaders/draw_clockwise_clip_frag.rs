/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_clockwise_clip.frag.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_clip.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "85c10fcfd0c66df4796ade0f0c3b33f1d633138cfd2548462e545e804e18307e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 101;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3061;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_clockwise_clip_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECLUI(CLIP_PLANE_IDX, clipBuffer);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F_RGB10_A2(SCRATCH_COLOR_PLANE_IDX, scratchColorBuffer);
#endif
PLS_DECLUI(COVERAGE_PLANE_IDX, coverageBuffer);
PLS_BLOCK_END

PLS_MAIN(@drawFragmentMain)
{
    VARYING_UNPACK(v_clipIDs, half2);
    half clipID = -v_clipIDs.x;

#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_windingWeight, half);
    half fragCoverage = v_windingWeight;
#else
    VARYING_UNPACK(v_coverages, COVERAGE_TYPE);
    half fragCoverage = v_coverages.x;
#endif //@DRAW_INTERIOR_TRIANGLES

    PLS_INTERLOCK_BEGIN;

    half2 clipData;
    half clipBufferID, clipCoverage;
#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
    if (@BORROWED_COVERAGE_PASS)
    {
        // Interior triangles with borrowed coverage are always the first
        // fragment of the path at their pixel, so we don't need to check the
        // current coverage value.
        clipCoverage = fragCoverage;
    }
    else
#endif
    {
        clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
        clipBufferID = clipData.g;
        half initialCoverage =
            clipBufferID == clipID ? clipData.r : make_half(.0);
        clipCoverage = initialCoverage + fragCoverage;
    }

#ifdef @ENABLE_NESTED_CLIPPING
    half outerClipID = v_clipIDs.y;
    if (@ENABLE_NESTED_CLIPPING && outerClipID != .0)
    {
        half outerClipCoverage = .0;
#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)
        if (@BORROWED_COVERAGE_PASS)
        {
            // Interior triangles with borrowed coverage did not load the clip
            // buffer already, so do that now.
            clipData = unpackHalf2x16(PLS_LOADUI(clipBuffer));
            clipBufferID = clipData.g;
        }
#endif
        if (clipBufferID != clipID)
        {
            outerClipCoverage = clipBufferID == outerClipID ? clipData.r : .0;
            // The coverage buffer is a free resource when rendering clip
            // because we don't use it to track coverage. Use it instead to
            // temporarily save the outer clip for nested clips. But make sure
            // to write an invalid pathID so we don't corrupt any future paths
            // that will be drawn.
            PLS_STOREUI(
                coverageBuffer,
                packHalf2x16(make_half2(outerClipCoverage, INVALID_PATH_ID)));
        }
        else
        {
            outerClipCoverage = unpackHalf2x16(PLS_LOADUI(coverageBuffer)).r;
            PLS_PRESERVE_UI(coverageBuffer);
        }
        clipCoverage = min(clipCoverage, outerClipCoverage);
    }
    else
#endif
    {
        PLS_PRESERVE_UI(coverageBuffer);
    }

    PLS_STOREUI(clipBuffer, packHalf2x16(make_half2(clipCoverage, clipID)));
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    PLS_PRESERVE_4F(colorBuffer);
#endif
    PLS_INTERLOCK_END;

    EMIT_PLS;
}

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_CLOCKWISE_CLIP_SOURCE: &str = PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_CLIP_FRAG_SOURCE: &str = PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_CLIP_FRAG_SOURCE
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
        block_id: "pp-0294",
        block_start: 5,
        block_end: 101,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0295",
        block_start: 8,
        block_end: 10,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0296",
        block_start: 12,
        block_end: 14,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0297",
        block_start: 23,
        block_end: 29,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0298",
        block_start: 35,
        block_end: 44,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0299",
        block_start: 53,
        block_end: 87,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0300",
        block_start: 58,
        block_end: 66,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0301",
        block_start: 93,
        block_end: 95,
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
        block_id: "pp-0294",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0295",
        branch_ordinal: 1,
        branch_line: 8,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0296",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0297",
        branch_ordinal: 1,
        branch_line: 23,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0297",
        branch_ordinal: 2,
        branch_line: 26,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0298",
        branch_ordinal: 1,
        branch_line: 35,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS))",
    },
    ConditionalBranch {
        block_id: "pp-0299",
        branch_ordinal: 1,
        branch_line: 53,
        directive: "#ifdef @ENABLE_NESTED_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_NESTED_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0300",
        branch_ordinal: 1,
        branch_line: 58,
        directive: "#if defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_NESTED_CLIPPING)) && (defined(@DRAW_INTERIOR_TRIANGLES) && defined(@BORROWED_COVERAGE_PASS))",
    },
    ConditionalBranch {
        block_id: "pp-0301",
        branch_ordinal: 1,
        branch_line: 93,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The six @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 8,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 23,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@BORROWED_COVERAGE_PASS",
        generated_name: "WB",
        generated_header_name: "GLSL_BORROWED_COVERAGE_PASS",
    },
    ExportedSymbol {
        source_line: 53,
        source_name: "@ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
        generated_header_name: "GLSL_ENABLE_NESTED_CLIPPING",
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
        source_line: 8,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 23,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@BORROWED_COVERAGE_PASS",
        generated_name: "WB",
        generated_header_name: "GLSL_BORROWED_COVERAGE_PASS",
    },
    ExportedSymbol {
        source_line: 53,
        source_name: "@ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
        generated_header_name: "GLSL_ENABLE_NESTED_CLIPPING",
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
    source_line: 18,
    end_line: 99,
    name: "drawFragmentMain",
    signature: "PLS_MAIN(@drawFragmentMain)",
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
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "BORROWED_COVERAGE_PASS",
        generated_name: "WB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_NESTED_CLIPPING",
        generated_name: "RC",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "drawFragmentMain",
    "DRAW_INTERIOR_TRIANGLES",
    "BORROWED_COVERAGE_PASS",
    "ENABLE_NESTED_CLIPPING",
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

/// draw_clockwise_clip.frag has no direct #include/#import directive.
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
