// Copyright 2026 Rive
//! Translation of tests/unit_tests/runtime/utf_test.cpp at
//! 8c7bb1ea1b502361e086324fc0187ad3ccfd9afd.

use nuxie_runtime::source::text::utf::Utf;

fn from_surrogate_pair(hi: u16, lo: u16) -> u32 {
    0x10000 + ((u32::from(hi) - 0xd800) << 10) + (u32::from(lo) - 0xdc00)
}

#[test]
fn bmp_code_points_encode_to_a_single_utf16_unit() {
    let mut utf16 = [0u16; 2];
    for uni in [0, u32::from('A'), 0xd7ff, 0xe000, 0xffff] {
        assert_eq!(Utf::to_utf16(uni, &mut utf16), 1);
        assert_eq!(u32::from(utf16[0]), uni);
    }
}

#[test]
fn supplementary_code_points_encode_to_a_valid_surrogate_pair() {
    // MATHEMATICAL BOLD CAPITAL M must have lead unit 0xd835, not the
    // non-surrogate 0xd7f5 produced by the former bit-or shortcut.
    let cases = [
        (0x10000, 0xd800, 0xdc00),
        (0x1d40c, 0xd835, 0xdc0c),
        (0x1f600, 0xd83d, 0xde00),
        (0x10ffff, 0xdbff, 0xdfff),
    ];
    let mut utf16 = [0u16; 2];
    for (uni, hi, lo) in cases {
        assert_eq!(Utf::to_utf16(uni, &mut utf16), 2);
        assert_eq!(utf16[0], hi);
        assert_eq!(utf16[1], lo);
    }
}

#[test]
fn every_supplementary_code_point_round_trips_through_utf16() {
    let mut utf16 = [0u16; 2];
    for uni in 0x10000..=0x10ffff {
        assert_eq!(Utf::to_utf16(uni, &mut utf16), 2);
        assert!(utf16[0] >= 0xd800);
        assert!(utf16[0] <= 0xdbff);
        assert!(utf16[1] >= 0xdc00);
        assert!(utf16[1] <= 0xdfff);
        assert_eq!(from_surrogate_pair(utf16[0], utf16[1]), uni);
    }
}
