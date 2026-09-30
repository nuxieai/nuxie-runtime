//! Translation of tests/unit_tests/renderer/pack_normalized_depth_test.cpp at
//! 57dddb3727306e284773ec20c653cf686c45abee. These call the source owner's CPU
//! translation, not a test-private packing implementation or a GPU shader.

use super::packNormalizedDepth;
use crate::mechanical_port::source::renderer::src::shaders::constants_glsl::{
    DEPTH_COVERAGE_BIT_COUNT, DEPTH_Z_INDEX_BIT_COUNT, PINNED_CONSTANTS_GLSL_SOURCE,
};

const PAYLOAD_BIT_COUNT: u32 = DEPTH_Z_INDEX_BIT_COUNT + DEPTH_COVERAGE_BIT_COUNT;
const _: () = assert!(PAYLOAD_BIT_COUNT == 23);
const PAYLOAD_COUNT: u32 = 1 << PAYLOAD_BIT_COUNT;
const Z_INDEX_COUNT: u32 = 1 << DEPTH_Z_INDEX_BIT_COUNT;
const COVERAGE_MASK: u32 = (1 << DEPTH_COVERAGE_BIT_COUNT) - 1;

fn hardware_stores(depth: f32) -> u32 {
    // std::lround rounds halfway cases away from zero; Rust round does too.
    (depth * 0xffffffu32 as f32).round() as u32
}

fn ldexp(value: f32, exponent: i32) -> f32 {
    // Every operand here is a small positive normal f32. Widening first and
    // scaling by an exact power of two implements ldexp without an intermediate
    // f32 rounding. No subnormal/overflow cases occur in these source tests.
    (f64::from(value) * 2.0f64.powi(exponent)) as f32
}

#[test]
fn pack_normalized_depth_exhaustive_0_to_1() {
    let mut prev = -1.0f32;
    for payload in 0..PAYLOAD_COUNT {
        let depth = packNormalizedDepth(
            payload >> DEPTH_COVERAGE_BIT_COUNT,
            payload & COVERAGE_MASK,
            false,
        );
        assert!(depth > 0.0);
        assert!(depth < 0.5);
        assert_eq!(hardware_stores(depth), payload);
        let d = payload as f32;
        assert_eq!(depth, (d + 0.5) * (1.0 / 16777216.0));
        assert_eq!(depth, ldexp(d + 0.5, -24));
        assert!(depth > prev);
        prev = depth;
    }
}

#[test]
fn pack_normalized_depth_exhaustive_minus1_to_1() {
    let mut prev = -2.0f32;
    for payload in 0..PAYLOAD_COUNT {
        let depth = packNormalizedDepth(
            payload >> DEPTH_COVERAGE_BIT_COUNT,
            payload & COVERAGE_MASK,
            true,
        );
        assert!(depth > -1.0);
        assert!(depth < 0.0);
        assert_eq!(hardware_stores((depth + 1.0) * 0.5), payload);
        let d = payload as f32;
        assert_eq!(depth, (d + 0.5) * (1.0 / 8388608.0) - 1.0);
        assert_eq!(depth, ldexp(d + 0.5, -23) - 1.0);
        assert!(depth > prev);
        prev = depth;
    }
}

#[test]
fn pack_normalized_depth_scale_and_bias() {
    assert_eq!(f32::from_bits(0x33800000), 1.0 / 16777216.0);
    assert_eq!(f32::from_bits(0x33000000), 0.5 / 16777216.0);
    assert_eq!(f32::from_bits(0x34000000), 1.0 / 8388608.0);
    assert_eq!(f32::from_bits(0xbf7fffff), 0.5 / 8388608.0 - 1.0);

    // Keep the host constants and arithmetic tied to the retained executable
    // GLSL owner. This is a source guard, not a claim of GPU execution.
    assert!(PINNED_CONSTANTS_GLSL_SOURCE.contains(&format!(
        "#define DEPTH_Z_INDEX_BIT_COUNT {DEPTH_Z_INDEX_BIT_COUNT}u"
    )));
    assert!(PINNED_CONSTANTS_GLSL_SOURCE.contains(&format!(
        "#define DEPTH_COVERAGE_BIT_COUNT {DEPTH_COVERAGE_BIT_COUNT}u"
    )));
    let source = super::PINNED_COMMON_GLSL_SOURCE;
    assert!(
        source.contains("float depth = float((zIndex15 << DEPTH_COVERAGE_BIT_COUNT) | coverage8);")
    );
    assert!(source.contains("#if defined(GLSL) && !defined(@TARGET_SPIRV)"));
    assert!(source
        .contains("return depth * uintBitsToFloat(0x34000000u) + uintBitsToFloat(0xbf7fffffu);"));
    assert!(source
        .contains("return depth * uintBitsToFloat(0x33800000u) + uintBitsToFloat(0x33000000u);"));
}

#[test]
fn pack_normalized_depth_24_bits_does_not_pack() {
    let mut half_survived = 0u32;
    let mut round_tripped = 0u32;
    for payload in (1u32 << 23)..(1u32 << 24) {
        let d = payload as f32;
        assert_eq!(d as u32, payload);
        let biased = d + 0.5;
        if f64::from(biased) == f64::from(payload) + 0.5 {
            half_survived += 1;
        }
        assert_eq!(biased, biased.floor());
        if hardware_stores(biased * (1.0 / 16777216.0)) == payload {
            round_tripped += 1;
        }
    }
    assert_eq!(half_survived, 0);
    assert!(round_tripped < (1 << 23));
    assert!(round_tripped > 0);
    let last_good = PAYLOAD_COUNT - 1;
    let d = last_good as f32;
    assert_eq!(f64::from(d + 0.5), f64::from(last_good) + 0.5);
    assert_eq!(hardware_stores((d + 0.5) * (1.0 / 16777216.0)), last_good);
}

#[test]
fn pack_normalized_depth_24th_bit_unused() {
    let max_depth = packNormalizedDepth(Z_INDEX_COUNT - 1, COVERAGE_MASK, false);
    assert!(max_depth < 0.5);
    assert_eq!(hardware_stores(max_depth), PAYLOAD_COUNT - 1);
}
