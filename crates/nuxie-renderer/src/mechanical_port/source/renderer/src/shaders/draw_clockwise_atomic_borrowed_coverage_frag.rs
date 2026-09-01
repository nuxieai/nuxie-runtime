/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_clockwise_atomic_borrowed_coverage.frag.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str =
    "renderer/src/shaders/draw_clockwise_atomic_borrowed_coverage.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "ddb9ef796b76b9d6bd2e743a0766aaf6b66732f124d1ddfcc80d2951db970a92";
pub const PINNED_SOURCE_LINE_COUNT: usize = 55;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1862;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_FRAG_SOURCE: &str = r###"/*
 * Copyright 2026 Rive
 */

#ifdef @FRAGMENT

FRAG_STORAGE_BUFFER_BLOCK_BEGIN
STORAGE_BUFFER_U32_ATOMIC(COVERAGE_BUFFER_IDX, CoverageBuffer, coverageBuffer);
FRAG_STORAGE_BUFFER_BLOCK_END

void main()
{
#ifdef @DRAW_INTERIOR_TRIANGLES
    VARYING_INIT(v_windingWeight, half);
#else
    VARYING_INIT(v_coverages, COVERAGE_TYPE);
#endif //@DRAW_INTERIOR_TRIANGLES
    VARYING_UNPACK(v_coveragePlacement, uint2);
    VARYING_UNPACK(v_coverageCoord, float2);

    half fragCoverage =
#ifdef @DRAW_INTERIOR_TRIANGLES
        v_windingWeight;
#else
        find_frag_coverage(v_coverages);
#endif

    uint2 coverageCoord = uint2(floor(v_coverageCoord));
    uint coveragePitch = v_coveragePlacement.y;
    uint coverageIndex = v_coveragePlacement.x +
                         swizzle_image_buffer_idx(coverageCoord, coveragePitch);

    // Try to apply borrowedCoverage, assuming the existing coverage value
    // is zero.
    uint borrowedCoverageFixed =
        clockwise_atomic_coverage_delta_to_fixed(abs(fragCoverage));
    uint targetCoverageValue =
        uniforms.coverageBufferPrefix |
        (CLOCKWISE_FILL_ZERO_VALUE - borrowedCoverageFixed);
    uint coverageBeforeMax = STORAGE_BUFFER_ATOMIC_MAX(coverageBuffer,
                                                       coverageIndex,
                                                       targetCoverageValue);
    if (coverageBeforeMax >= uniforms.coverageBufferPrefix)
    {
        // Coverage was not zero. Undo the atomicMax and then subtract
        // borrowedCoverageFixed this time.
        uint undoAtomicMax =
            coverageBeforeMax - max(coverageBeforeMax, targetCoverageValue);
        STORAGE_BUFFER_ATOMIC_ADD(coverageBuffer,
                                  coverageIndex,
                                  undoAtomicMax - borrowedCoverageFixed);
    }
}

#endif // FRAGMENT
"###;

/// Stable source aliases.
pub const PINNED_DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_FRAG_SOURCE;
pub const DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_FRAG_SOURCE: &str =
    PINNED_DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_CLOCKWISE_ATOMIC_BORROWED_COVERAGE_FRAG_SOURCE
}
