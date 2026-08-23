/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_clockwise_atomic_clip.frag.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_atomic_clip.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "c50f3d97fa389ef3e68ac2a20579f022ea688079ab97956d66e4c4b62310ceb4";
pub const PINNED_SOURCE_LINE_COUNT: usize = 138;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 5352;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_clip_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_CLIP_FRAG_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F_WRITEONLY(CLIP_PLANE_IDX, clipBuffer);
PLS_BLOCK_END

#ifdef @NESTED_CLIP_UPDATE_ONLY
FRAG_STORAGE_BUFFER_BLOCK_BEGIN
STORAGE_BUFFER_U32_ATOMIC(COVERAGE_BUFFER_IDX, CoverageBuffer, coverageBuffer);
FRAG_STORAGE_BUFFER_BLOCK_END
#endif

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
#define CLOCKWISE_ATOMIC_PLS_MAIN PLS_FRAG_COLOR_MAIN
#define EMIT_CLOCKWISE_ATOMIC_PLS(FRAG_COLOR)                                  \
    _fragColor = FRAG_COLOR;                                                   \
    EMIT_PLS_AND_FRAG_COLOR
#else
#define CLOCKWISE_ATOMIC_PLS_MAIN PLS_MAIN
#define EMIT_CLOCKWISE_ATOMIC_PLS(FRAG_COLOR)                                  \
    PLS_STORE4F(colorBuffer, FRAG_COLOR);                                      \
    EMIT_PLS;
#endif

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_windingWeight, half);
    half fragCoverage = v_windingWeight;
#else
    VARYING_UNPACK(v_coverages, COVERAGE_TYPE);
    half fragCoverage = v_coverages.x;
#endif //@DRAW_INTERIOR_TRIANGLES

#ifdef @NESTED_CLIP_UPDATE_ONLY
    if (@NESTED_CLIP_UPDATE_ONLY)
    {
        VARYING_UNPACK(v_coveragePlacement, uint2);
        VARYING_UNPACK(v_coverageCoord, float2);

        uint coveragePitch = v_coveragePlacement.y;
        uint coverageIndex =
            v_coveragePlacement.x +
            swizzle_image_buffer_idx(uint2(floor(v_coverageCoord)),
                                     coveragePitch);

        uint preexistingCoverageValue =
            STORAGE_BUFFER_LOAD(coverageBuffer, coverageIndex);
        half pathCoverage;
        if (fragCoverage >= 1. &&
            (preexistingCoverageValue < uniforms.coverageBufferPrefix ||
             preexistingCoverageValue >=
                 (uniforms.coverageBufferPrefix | CLOCKWISE_FILL_ZERO_VALUE)))
        {
            // The inverse path has reached a coverage of 1, meaning, the area
            // we are erasing from the clip has reached 0.
            pathCoverage = .0;
            // No need to update the coverage buffer because the blend op is
            // min() and we have bottomed out at 0 -- it doesn't matter what any
            // future fragments do anymore.
        }
        else
        {
            // clockwiseAtomic nested clip updates take the inverse path as
            // input.
            half inversePathCoverage = fragCoverage;
            half unappliedFragCoverage = fragCoverage;
            if (preexistingCoverageValue < uniforms.coverageBufferPrefix)
            {
                // There was no borrowed coverage and we *might* be the first
                // fragment of the path to touch this pixel. Attempt to write
                // out our coverage with an atomicMax.
                uint targetCoverageValue =
                    uniforms.coverageBufferPrefix |
                    (CLOCKWISE_FILL_ZERO_VALUE +
                     clockwise_atomic_coverage_delta_to_fixed(
                         abs(fragCoverage)));
                uint coverageBeforeMax =
                    STORAGE_BUFFER_ATOMIC_MAX(coverageBuffer,
                                              coverageIndex,
                                              targetCoverageValue);
                if (coverageBeforeMax <= uniforms.coverageBufferPrefix)
                {
                    // Success! We were the first fragment of the path at this
                    // pixel.
                    unappliedFragCoverage = .0; // We're done.
                }
                else if (coverageBeforeMax < targetCoverageValue)
                {
                    // We were not first fragment of the path at this pixel, AND
                    // our atomicMax had some effect, but did not fully apply
                    // our coverage.
                    unappliedFragCoverage =
                        clockwise_atomic_fixed_to_coverage(coverageBeforeMax);
                }
            }
            if (unappliedFragCoverage > .0)
            {
                // Coverage wasn't fully applied during the implicit clear
                // operations above. Apply it now.
                uint coverageBeforeAdd = STORAGE_BUFFER_ATOMIC_ADD(
                    coverageBuffer,
                    coverageIndex,
                    clockwise_atomic_coverage_delta_to_fixed(
                        abs(unappliedFragCoverage)));
                inversePathCoverage =
                    clockwise_atomic_fixed_to_coverage(coverageBeforeAdd) +
                    fragCoverage;
            }

            // clockwiseAtomic nested clip updates take the inverse path as
            // input, so take 1 - inversePathCoverageto get back to the original
            // nested path.
            pathCoverage = 1. - inversePathCoverage;
        }

        PLS_STORE4F(clipBuffer, make_half4(pathCoverage));

        // Since the blend op is min(), emitting a color of 1 is effectively a
        // no-op as long as we're using unorm targets.
        EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(1.))
    }
    else
#endif
    {
        PLS_STORE4F(clipBuffer, make_half4(fragCoverage));
        EMIT_CLOCKWISE_ATOMIC_PLS(make_half4(.0))
    }
}

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_CLIP_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_CLIP_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_ATOMIC_CLIP_FRAG_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_CLIP_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_ATOMIC_CLIP_FRAG_SOURCE
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
        block_id: "pp-0260",
        block_start: 5,
        block_end: 138,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0261",
        block_start: 8,
        block_end: 10,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0262",
        block_start: 14,
        block_end: 18,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0263",
        block_start: 20,
        block_end: 30,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0264",
        block_start: 34,
        block_end: 40,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0265",
        block_start: 42,
        block_end: 131,
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
        block_id: "pp-0260",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0261",
        branch_ordinal: 1,
        branch_line: 8,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0262",
        branch_ordinal: 1,
        branch_line: 14,
        directive: "#ifdef @NESTED_CLIP_UPDATE_ONLY",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NESTED_CLIP_UPDATE_ONLY))",
    },
    ConditionalBranch {
        block_id: "pp-0263",
        branch_ordinal: 1,
        branch_line: 20,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0263",
        branch_ordinal: 2,
        branch_line: 25,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
    ConditionalBranch {
        block_id: "pp-0264",
        branch_ordinal: 1,
        branch_line: 34,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0264",
        branch_ordinal: 2,
        branch_line: 37,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0265",
        branch_ordinal: 1,
        branch_line: 42,
        directive: "#ifdef @NESTED_CLIP_UPDATE_ONLY",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@NESTED_CLIP_UPDATE_ONLY))",
    },
    ConditionalBranch {
        block_id: "pp-0265",
        branch_ordinal: 2,
        branch_line: 130,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@NESTED_CLIP_UPDATE_ONLY))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The five @-prefixed identifiers occurring directly in this shader source.
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
        source_line: 14,
        source_name: "@NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
        generated_header_name: "GLSL_NESTED_CLIP_UPDATE_ONLY",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
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
        source_line: 14,
        source_name: "@NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
        generated_header_name: "GLSL_NESTED_CLIP_UPDATE_ONLY",
    },
    ExportedSymbol {
        source_line: 34,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
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
    source_line: 32,
    end_line: 136,
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
        source_name: "NESTED_CLIP_UPDATE_ONLY",
        generated_name: "YC",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "NESTED_CLIP_UPDATE_ONLY",
    "drawFragmentMain",
    "DRAW_INTERIOR_TRIANGLES",
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

/// draw_clockwise_atomic_clip.frag has no direct #include/#import directive.
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
