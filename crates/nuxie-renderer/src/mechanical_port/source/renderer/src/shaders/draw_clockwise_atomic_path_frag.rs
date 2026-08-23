/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_clockwise_atomic_path.frag.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_atomic_path.frag";
pub const PINNED_SOURCE_STAGE: &str = "minify-input-frag";
pub const PINNED_SOURCE_SHA256: &str =
    "4ea83385144a73678293d4313b53fad6cc4204551b41800c2c8b7c8821ea1287";
pub const PINNED_SOURCE_LINE_COUNT: usize = 377;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 14325;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_path_frag.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned fragment-shader source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_PATH_FRAG_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

#ifdef @FRAGMENT

PLS_BLOCK_BEGIN
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F(COLOR_PLANE_IDX, colorBuffer);
#endif
PLS_DECL4F(CLIP_PLANE_IDX, clipBuffer);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
PLS_DECL4F_RGB10_A2_UAV(SCRATCH_COLOR_PLANE_IDX, blendColorBuffer);
#endif
PLS_BLOCK_END

FRAG_STORAGE_BUFFER_BLOCK_BEGIN
STORAGE_BUFFER_U32_ATOMIC(COVERAGE_BUFFER_IDX, CoverageBuffer, coverageBuffer);
FRAG_STORAGE_BUFFER_BLOCK_END

INLINE void apply_stroke_coverage(INOUT(float) paintAlpha,
                                  half fragCoverage,
                                  uint coverageIndex,
                                  OUT(uint) preexistingCoverageValue,
                                  OUT(half) newCoverage)
{
#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    if (min(paintAlpha, fragCoverage) >= 1.)
    {
        // Solid stroke pixels don't need to work out coverage at all. We can
        // just blast them out without ever touching the coverage buffer, even
        // if another fragment from the path will get drawn on top. This is
        // because any fragment drawn on top will be the same color, and any
        // color blended onto a fully opaque version of itself is a no-op.
        return;
    }
#endif

    half X;
    uint fragCoverageFixed =
        clockwise_atomic_coverage_delta_to_fixed(abs(fragCoverage));
    preexistingCoverageValue = STORAGE_BUFFER_ATOMIC_MAX(
        coverageBuffer,
        coverageIndex,
        uniforms.coverageBufferPrefix | fragCoverageFixed);
    if (preexistingCoverageValue < uniforms.coverageBufferPrefix)
    {
        // This is the first fragment of the stroke to touch this pixel. Just
        // multiply in our coverage and write it out.
        X = fragCoverage;
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        newCoverage = fragCoverage;
#endif
    }
    else
    {
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        if ((preexistingCoverageValue & BLEND_COLOR_VALID_BIT) != 0u)
        {
            // The BLEND_COLOR_VALID_BIT had already been set at this fragment.
            // Redo the atomic max with that bit set.
            preexistingCoverageValue = STORAGE_BUFFER_ATOMIC_MAX(
                coverageBuffer,
                coverageIndex,
                uniforms.coverageBufferPrefix | BLEND_COLOR_VALID_BIT |
                    fragCoverageFixed);
        }
#endif
        // This pixel has been touched previously by a fragment in the stroke.
        // Multiply in an incremental coverage value that mixes with what's
        // already in the framebuffer.
        half c0 = cast_uint_to_half(preexistingCoverageValue &
                                    CLOCKWISE_COVERAGE_MASK) *
                  CLOCKWISE_COVERAGE_INVERSE_PRECISION;
        half c1 = max(c0, fragCoverage);
        X = incremental_clockwise_coverage(c0, c1, paintAlpha);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        newCoverage = c1;
#endif
    }

    paintAlpha *= X;
}

INLINE void apply_fill_coverage(INOUT(float) paintAlpha,
                                half fragCoverageRemaining,
                                uint coverageIndex,
                                OUT(uint) preexistingCoverageValue,
                                OUT(half) newCoverage)
{
    half X = .0; // Amount by which to multiply paintAlpha.
    uint fragCoverageRemainingFixed =
        clockwise_atomic_coverage_delta_to_fixed(abs(fragCoverageRemaining));

    preexistingCoverageValue =
        STORAGE_BUFFER_LOAD(coverageBuffer, coverageIndex);

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
    if (min(paintAlpha, fragCoverageRemaining) >= 1. &&
        (preexistingCoverageValue < uniforms.coverageBufferPrefix ||
         preexistingCoverageValue >=
             (uniforms.coverageBufferPrefix | CLOCKWISE_FILL_ZERO_VALUE)))
    {
        // If we're solid, AND the current coverage at this pixel is >= 0, then
        // we can just write out our color without working out coverage any
        // further, even if another fragment from the path will get drawn on
        // top. This is because any fragment drawn on top will be the same
        // color, and any color blended onto a fully opaque version of itself is
        // a no-op.
        return;
    }
#endif

    if (preexistingCoverageValue < uniforms.coverageBufferPrefix)
    {
        // The initial coverage value does not belong to this path. We *might*
        // be the first fragment of the path to touch this pixel. Attempt to
        // write out our coverage with an atomicMax.
        uint targetCoverage =
            uniforms.coverageBufferPrefix |
            (CLOCKWISE_FILL_ZERO_VALUE + fragCoverageRemainingFixed);
        uint coverageBeforeMax = STORAGE_BUFFER_ATOMIC_MAX(coverageBuffer,
                                                           coverageIndex,
                                                           targetCoverage);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        preexistingCoverageValue = coverageBeforeMax;
#endif
        if (coverageBeforeMax <= uniforms.coverageBufferPrefix)
        {
            // Success! We were the first fragment of the path at this pixel.
            X = fragCoverageRemaining; // Just multiply paintAlpha by coverage.
#ifdef @DRAW_INTERIOR_TRIANGLES
            X = min(X, 1.);
#endif
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
            newCoverage = X;
#endif
            fragCoverageRemaining = .0; // We're done.
        }
        else if (coverageBeforeMax < targetCoverage)
        {
            // We were not first fragment of the path at this pixel, AND our
            // atomicMax had an effect that we now have to account for in
            // paintAlpha. Coverage increased from "coverageBeforeMax" to
            // "fragCoverageRemaining".
            //
            // NOTE: because we know coverage was initially zero, and because
            // coverage is always positive in this pass, we know
            // coverageBeforeMax >= 0.
            uint c0Fixed = (coverageBeforeMax & CLOCKWISE_COVERAGE_MASK) -
                           CLOCKWISE_FILL_ZERO_VALUE;
            half c0 = cast_uint_to_half(c0Fixed) *
                      CLOCKWISE_COVERAGE_INVERSE_PRECISION;
            half c1 = fragCoverageRemaining;
#ifdef @DRAW_INTERIOR_TRIANGLES
            c1 = min(c1, 1.);
#endif
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
            newCoverage = c1;
#endif
            // Apply the coverage increase from the atomicMax here. The next
            // step will apply the remaining increase.
            X = incremental_clockwise_coverage(c0, c1, paintAlpha);

            // We increased coverage by an amount of "fragCoverageRemaining" -
            // "coverageBeforeMax". However, we wanted to increase coverage by
            // "fragCoverageRemaining". So the remaining amount we still need to
            // increase by is "coverageBeforeMax".
            fragCoverageRemainingFixed = c0Fixed;
            fragCoverageRemaining = c0;
        }
    }

    if (fragCoverageRemaining > .0)
    {
        // At this point we know the value in the coverage buffer belongs to
        // this path, so we can do a simple atomicAdd.
        uint coverageBeforeAdd =
            STORAGE_BUFFER_ATOMIC_ADD(coverageBuffer,
                                      coverageIndex,
                                      fragCoverageRemainingFixed);
        half c0 = clockwise_atomic_fixed_to_coverage(coverageBeforeAdd);
        half c1 = c0 + fragCoverageRemaining;
        c0 = clamp(c0, .0, 1.);
        c1 = clamp(c1, .0, 1.);
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
        newCoverage = c1;
#endif
        // Apply the coverage increase from c0 -> c1 that we just did, in
        // addition to any coverage that had been applied previously.
        X += (1. - X * paintAlpha) *
             incremental_clockwise_coverage(c0, c1, paintAlpha);
    }

    paintAlpha *= X;
}

CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)
{
    VARYING_UNPACK(v_paint, float4);
#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_INIT(v_windingWeight, half);
#else
    VARYING_INIT(v_coverages, COVERAGE_TYPE);
#endif //@DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_pathID, half);
#ifdef @ENABLE_CLIPPING
    VARYING_UNPACK(v_clipIDs, half2);
#endif
#ifdef @ENABLE_CLIP_RECT
    VARYING_UNPACK(v_clipRect, float4);
#endif
#ifdef @ENABLE_ADVANCED_BLEND
    VARYING_UNPACK(v_blendMode, half);
#endif
    VARYING_UNPACK(v_coveragePlacement, uint2);
    VARYING_UNPACK(v_coverageCoord, float2);

    half4 paintColor = find_paint_color(v_paint, 1. FRAGMENT_CONTEXT_UNPACK);

#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    // Fetch the framebuffer BEFORE any atomic operations on the coverage
    // buffer. In order for advanced blend to work, we have to fetch the
    // framebuffer value before checking if it's still valid.
    half4 dstColor = PLS_LOAD4F(colorBuffer);
#endif

    half fragCoverage =
#ifdef @DRAW_INTERIOR_TRIANGLES
        v_windingWeight;
#else
        find_frag_coverage(v_coverages);
#endif

    float2 coverageCoord = v_coverageCoord;
#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    // This little trick forces the shader to fetch the framebuffer BEFORE any
    // atomic operations on the coverage buffer. (i.e., not to reorder the above
    // fetch past this point). In order for advanced blend to work, we have to
    // fetch the framebuffer value before operating on coverage.
    //
    // NOTE: Since v_coverageCoord is pixel-grid aligned, it will always have a
    // fractional value of ~.5 (because varyings are sampled at at pixel
    // center). So as long as colorBuffer is a standard unorm in the range 0..1,
    // this will have literally no effect on the final outcome. If we ever
    // support rendering to full floating point targets outside the range 0..1,
    // we may need to put some more thought into this.
    coverageCoord +=
        (dstColor.rg + dstColor.ba) * uniforms.epsilonForPseudoMemoryBarrier;
#endif
    coverageCoord = floor(coverageCoord);
    uint coveragePitch = v_coveragePlacement.y;
    uint coverageIndex =
        v_coveragePlacement.x +
        swizzle_image_buffer_idx(uint2(coverageCoord), coveragePitch);

    half maxCoverage = 1.;

#ifdef @ENABLE_CLIP_RECT
    if (@ENABLE_CLIP_RECT)
    {
        half clipRectMin = min_component(cast_float4_to_half4(v_clipRect));
        maxCoverage = min(clipRectMin, maxCoverage);
    }
#endif

#ifdef @ENABLE_CLIPPING
    if (@ENABLE_CLIPPING && v_clipIDs.x != .0)
    {
        half clip = PLS_LOAD4F(clipBuffer).r;
        maxCoverage = min(clip, maxCoverage);
    }
#endif

    maxCoverage = max(maxCoverage, .0);
    fragCoverage = clamp(fragCoverage, .0, maxCoverage);

    uint preexistingCoverageValue;
    float newCoverage;
#ifndef @DRAW_INTERIOR_TRIANGLES
    if (is_stroke(v_coverages))
    {
        apply_stroke_coverage(paintColor.a,
                              fragCoverage,
                              coverageIndex,
                              preexistingCoverageValue,
                              newCoverage);
    }
    else // It's a fill.
#endif   // !DRAW_INTERIOR_TRIANGLES
    {
        apply_fill_coverage(paintColor.a,
                            fragCoverage,
                            coverageIndex,
                            preexistingCoverageValue,
                            newCoverage);
    }

#ifdef @ENABLE_DITHER
    half dither;
    if (@ENABLE_DITHER)
    {
        dither = get_dither(_fragCoord.xy,
                            uniforms.ditherScale,
                            uniforms.ditherBias);
    }
#endif

#ifndef @FIXED_FUNCTION_COLOR_OUTPUT
    if (paintColor.a > .0)
    {
        bool wasBlendColorValid =
            preexistingCoverageValue >= uniforms.coverageBufferPrefix &&
            (preexistingCoverageValue & BLEND_COLOR_VALID_BIT) != 0u;
        if (!wasBlendColorValid)
        {
            // If the saved blend color was not yet valid after we fetched
            // dstColor, we are guaranteed that dstColor is valid because the
            // BLEND_COLOR_VALID_BIT gets set before any color outputs that
            // might overwrite the framebuffer.
            // Calculate a blendColor based on dstColor.
            paintColor.rgb =
                advanced_color_blend(paintColor.rgb,
                                     dstColor,
                                     cast_half_to_ushort(v_blendMode));

            // Anybody who updated, or will update, the coverage buffer before
            // we overwrite the framebuffer is guaranteed to have a dstColor
            // that is unaffected by our color output. They already have it.
            // But if 0 < coverage < 1 after our fragment, we have to save out
            // the blend color we just found for any future fragments that may
            // need to blend, before we overwrite the contents of the
            // framebuffer.
            if (newCoverage < 1.)
            {
                half3 blendRGBToSave = paintColor.rgb;
#ifdef @ENABLE_DITHER
                if (@ENABLE_DITHER)
                {
                    blendRGBToSave += dither * uniforms.ditherConversionToRGB10;
                }
#endif
                PLS_STORE4F_UAV(blendColorBuffer,
                                make_half4(blendRGBToSave, .0));

                // Mark this pixel as having a valid blendColor, AFTER writing
                // out the blendColor, but BEFORE updating the framebuffer.
                memoryBarrier();
                STORAGE_BUFFER_ATOMIC_OR(coverageBuffer,
                                         coverageIndex,
                                         BLEND_COLOR_VALID_BIT);
            }
        }
        else
        {
            // Use the saved blendColor whenever it's valid, because shortly
            // after that point the framebuffer can be overwritten, invalidating
            // the dstColor.
            paintColor.rgb = PLS_LOAD4F_UAV(blendColorBuffer).rgb;
        }
    }
#endif

    paintColor.rgb *= paintColor.a;

#ifdef @ENABLE_DITHER
    paintColor.rgb =
        add_dither_if_alpha_nonzero(paintColor.rgb, paintColor.a, dither);
#endif

    // Since blend is enabled, storing 0 to the clip will ensure it remains
    // unchanged.
    PLS_STORE4F(clipBuffer, make_half4(.0));
    EMIT_CLOCKWISE_ATOMIC_PLS(paintColor);
}

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_PATH_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_PATH_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_ATOMIC_PATH_FRAG_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_PATH_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_ATOMIC_PATH_FRAG_SOURCE
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
        block_id: "pp-0266",
        block_start: 5,
        block_end: 377,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0267",
        block_start: 8,
        block_end: 10,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0268",
        block_start: 12,
        block_end: 14,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0269",
        block_start: 27,
        block_end: 37,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0270",
        block_start: 51,
        block_end: 53,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0271",
        block_start: 57,
        block_end: 68,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0272",
        block_start: 77,
        block_end: 79,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0273",
        block_start: 98,
        block_end: 112,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0274",
        block_start: 125,
        block_end: 127,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0275",
        block_start: 132,
        block_end: 134,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0276",
        block_start: 135,
        block_end: 137,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0277",
        block_start: 155,
        block_end: 157,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0278",
        block_start: 158,
        block_end: 160,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0279",
        block_start: 186,
        block_end: 188,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0280",
        block_start: 201,
        block_end: 205,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0281",
        block_start: 207,
        block_end: 209,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0282",
        block_start: 210,
        block_end: 212,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0283",
        block_start: 213,
        block_end: 215,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0284",
        block_start: 221,
        block_end: 226,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0285",
        block_start: 229,
        block_end: 233,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0286",
        block_start: 236,
        block_end: 250,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0287",
        block_start: 259,
        block_end: 265,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0288",
        block_start: 267,
        block_end: 273,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0289",
        block_start: 280,
        block_end: 290,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0290",
        block_start: 299,
        block_end: 307,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0291",
        block_start: 309,
        block_end: 362,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0292",
        block_start: 337,
        block_end: 342,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0293",
        block_start: 366,
        block_end: 369,
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
        block_id: "pp-0266",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0267",
        branch_ordinal: 1,
        branch_line: 8,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0268",
        branch_ordinal: 1,
        branch_line: 12,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0269",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0270",
        branch_ordinal: 1,
        branch_line: 51,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0271",
        branch_ordinal: 1,
        branch_line: 57,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0272",
        branch_ordinal: 1,
        branch_line: 77,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0273",
        branch_ordinal: 1,
        branch_line: 98,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0274",
        branch_ordinal: 1,
        branch_line: 125,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0275",
        branch_ordinal: 1,
        branch_line: 132,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0276",
        branch_ordinal: 1,
        branch_line: 135,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0277",
        branch_ordinal: 1,
        branch_line: 155,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0278",
        branch_ordinal: 1,
        branch_line: 158,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0279",
        branch_ordinal: 1,
        branch_line: 186,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0280",
        branch_ordinal: 1,
        branch_line: 201,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0280",
        branch_ordinal: 2,
        branch_line: 203,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0281",
        branch_ordinal: 1,
        branch_line: 207,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0282",
        branch_ordinal: 1,
        branch_line: 210,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0283",
        branch_ordinal: 1,
        branch_line: 213,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0284",
        branch_ordinal: 1,
        branch_line: 221,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0285",
        branch_ordinal: 1,
        branch_line: 229,
        directive: "#ifdef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0285",
        branch_ordinal: 2,
        branch_line: 231,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@DRAW_INTERIOR_TRIANGLES))))",
    },
    ConditionalBranch {
        block_id: "pp-0286",
        branch_ordinal: 1,
        branch_line: 236,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0287",
        branch_ordinal: 1,
        branch_line: 259,
        directive: "#ifdef @ENABLE_CLIP_RECT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIP_RECT))",
    },
    ConditionalBranch {
        block_id: "pp-0288",
        branch_ordinal: 1,
        branch_line: 267,
        directive: "#ifdef @ENABLE_CLIPPING",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_CLIPPING))",
    },
    ConditionalBranch {
        block_id: "pp-0289",
        branch_ordinal: 1,
        branch_line: 280,
        directive: "#ifndef @DRAW_INTERIOR_TRIANGLES",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0290",
        branch_ordinal: 1,
        branch_line: 299,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0291",
        branch_ordinal: 1,
        branch_line: 309,
        directive: "#ifndef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0292",
        branch_ordinal: 1,
        branch_line: 337,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (!defined(@FIXED_FUNCTION_COLOR_OUTPUT)) && (defined(@ENABLE_DITHER))",
    },
    ConditionalBranch {
        block_id: "pp-0293",
        branch_ordinal: 1,
        branch_line: 366,
        directive: "#ifdef @ENABLE_DITHER",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_DITHER))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The eight @-prefixed identifiers occurring directly in this shader source.
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
        source_line: 132,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 198,
        source_name: "@drawFragmentMain",
        generated_name: "IB",
        generated_header_name: "GLSL_drawFragmentMain",
    },
    ExportedSymbol {
        source_line: 207,
        source_name: "@ENABLE_CLIPPING",
        generated_name: "O",
        generated_header_name: "GLSL_ENABLE_CLIPPING",
    },
    ExportedSymbol {
        source_line: 210,
        source_name: "@ENABLE_CLIP_RECT",
        generated_name: "AB",
        generated_header_name: "GLSL_ENABLE_CLIP_RECT",
    },
    ExportedSymbol {
        source_line: 213,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 299,
        source_name: "@ENABLE_DITHER",
        generated_name: "JB",
        generated_header_name: "GLSL_ENABLE_DITHER",
    },
];

/// The preprocessor-switch subset of the direct exports.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    EXPORTED_SYMBOLS[0],
    EXPORTED_SYMBOLS[1],
    EXPORTED_SYMBOLS[2],
    EXPORTED_SYMBOLS[4],
    EXPORTED_SYMBOLS[5],
    EXPORTED_SYMBOLS[6],
    EXPORTED_SYMBOLS[7],
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
/// bodies remain in the pinned source rather than becoming executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 21,
        end_line: 83,
        name: "apply_stroke_coverage",
        signature: "INLINE void apply_stroke_coverage(INOUT(float) paintAlpha, half fragCoverage, uint coverageIndex, OUT(uint) preexistingCoverageValue, OUT(half) newCoverage)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 85,
        end_line: 196,
        name: "apply_fill_coverage",
        signature: "INLINE void apply_fill_coverage(INOUT(float) paintAlpha, half fragCoverageRemaining, uint coverageIndex, OUT(uint) preexistingCoverageValue, OUT(half) newCoverage)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 198,
        end_line: 375,
        name: "drawFragmentMain",
        signature: "CLOCKWISE_ATOMIC_PLS_MAIN(@drawFragmentMain)",
        guard_path: "(defined(@FRAGMENT))",
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
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "drawFragmentMain",
        generated_name: "IB",
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
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "ENABLE_DITHER",
        generated_name: "JB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "FRAGMENT",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "DRAW_INTERIOR_TRIANGLES",
    "drawFragmentMain",
    "ENABLE_CLIPPING",
    "ENABLE_CLIP_RECT",
    "ENABLE_ADVANCED_BLEND",
    "ENABLE_DITHER",
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

/// draw_clockwise_atomic_path.frag has no direct #include/#import directive.
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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
