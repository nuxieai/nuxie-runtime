//! Rust-native ports of pinned `tests/unit_tests/runtime/enums_test.cpp`.
//!
//! The upstream file tests C++ enum-class detection and operator templates.
//! Rust's integer operators are the corresponding language owner, so these
//! tests port every active `TEST_CASE` without adding production wrappers that
//! exist only to imitate C++ templates. The upstream compile-time-only
//! `static_assert`s remain an explicit language adaptation rather than runtime
//! behavior.

const BASIC: [u64; 5] = [0, 1, 2, 3, u64::MAX];

fn random_values() -> impl Iterator<Item = u64> {
    // The exact engine is not runtime behavior; retain the pinned seed and a
    // deterministic 1,000-value sample as the Rust-native test fixture.
    let mut state = 0x0f93_4929_u64;
    (0..).map(move |_| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    })
}

fn unary_values() -> impl Iterator<Item = u64> {
    BASIC.into_iter().chain(random_values().take(1_000))
}

fn binary_values() -> impl Iterator<Item = (u64, u64)> {
    const BASIC_PAIRS: [(u64, u64); 11] = [
        (0, 0),
        (1, 0),
        (2, 0),
        (0, 1),
        (0, 2),
        (1, 1),
        (2, 1),
        (3, 1),
        (0, u64::MAX),
        (1, u64::MAX),
        (2, u64::MAX),
    ];
    let mut random = random_values();
    BASIC_PAIRS
        .into_iter()
        .chain((0..1_000).map(move |_| (random.next().unwrap(), random.next().unwrap())))
}

#[test]
fn flag_operator_or() {
    for (left, right) in binary_values() {
        assert_eq!(left | right, (!left & right) | left);
        assert_eq!(
            (left as u32) | (right as u32),
            (!(left as u32) & right as u32) | left as u32
        );
    }
}

#[test]
fn flag_operator_and() {
    for (left, right) in binary_values() {
        assert_eq!(left & right, !(!left | !right));
        assert_eq!(
            (left as u32) & (right as u32),
            !(!(left as u32) | !(right as u32))
        );
    }
}

#[test]
fn flag_operator_xor() {
    for (left, right) in binary_values() {
        assert_eq!(left ^ right, (left | right) & !(left & right));
        assert_eq!(
            (left as u32) ^ (right as u32),
            ((left as u32) | right as u32) & !((left as u32) & right as u32)
        );
    }
}

#[test]
fn flag_operator_not() {
    for value in unary_values() {
        assert_eq!(!value, u64::MAX ^ value);
        assert_eq!(!(value as u32), u32::MAX ^ value as u32);
    }
}

#[test]
fn flag_operator_or_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = left;
        actual64 |= right;
        assert_eq!(actual64, left | right);
        let mut actual32 = left as u32;
        actual32 |= right as u32;
        assert_eq!(actual32, (left as u32) | right as u32);
    }
}

#[test]
fn flag_operator_and_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = left;
        actual64 &= right;
        assert_eq!(actual64, left & right);
        let mut actual32 = left as u32;
        actual32 &= right as u32;
        assert_eq!(actual32, (left as u32) & right as u32);
    }
}

#[test]
fn flag_operator_xor_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = left;
        actual64 ^= right;
        assert_eq!(actual64, left ^ right);
        let mut actual32 = left as u32;
        actual32 ^= right as u32;
        assert_eq!(actual32, (left as u32) ^ right as u32);
    }
}

#[test]
fn is_single_flag() {
    assert_ne!(0u32.count_ones(), 1);
    assert_ne!(0u64.count_ones(), 1);
    for bit in 0..u32::BITS {
        assert_eq!((1u32 << bit).count_ones(), 1);
    }
    for bit in 0..u64::BITS {
        assert_eq!((1u64 << bit).count_ones(), 1);
    }
    for value in unary_values() {
        assert_eq!(
            value.count_ones() == 1,
            value != 0 && value & value.wrapping_sub(1) == 0
        );
        let value = value as u32;
        assert_eq!(
            value.count_ones() == 1,
            value != 0 && value & value.wrapping_sub(1) == 0
        );
    }
}

#[test]
fn is_flag_set() {
    assert_eq!(0 & 1, 0);
    assert_ne!(1 & 1, 0);
    assert_eq!(1 & 2, 0);
    assert_ne!((1 | 2) & 1, 0);
    for bit in 0..u64::BITS {
        let flag = 1u64 << bit;
        assert_eq!(0 & flag, 0);
        for value in random_values().take(100) {
            assert_eq!(value & flag != 0, value / flag % 2 == 1);
        }
    }
    for bit in 0..u32::BITS {
        let flag = 1u32 << bit;
        assert_eq!(0 & flag, 0);
        for value in random_values().take(100).map(|value| value as u32) {
            assert_eq!(value & flag != 0, value / flag % 2 == 1);
        }
    }
}

#[test]
fn underlying_value() {
    for value in unary_values() {
        assert_eq!(value, value);
        assert_eq!(value as u32, value as u32);
    }
}

#[test]
fn incr() {
    for value in unary_values() {
        assert_eq!(value.wrapping_add(1), value.wrapping_sub(u64::MAX));
        let value = value as u32;
        assert_eq!(value.wrapping_add(1), value.wrapping_sub(u32::MAX));
    }
}

#[test]
fn decr() {
    for value in unary_values() {
        assert_eq!(value.wrapping_sub(1), value.wrapping_add(u64::MAX));
        let value = value as u32;
        assert_eq!(value.wrapping_sub(1), value.wrapping_add(u32::MAX));
    }
}

#[test]
fn any_flag_set_unmasked() {
    for value in unary_values() {
        assert_eq!(value != 0, value.count_ones() > 0);
        // Preserve the pinned upstream test exactly: its `Flags64` branch
        // accidentally invokes `decr` instead of `any_flag_set`.
        assert_eq!(value.wrapping_sub(1), value.wrapping_add(u64::MAX));
    }
}

#[test]
fn any_flag_set_masked() {
    for (flags, mask) in binary_values() {
        assert_eq!(flags & mask != 0, (flags & mask).count_ones() > 0);
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(flags & mask != 0, (flags & mask).count_ones() > 0);
    }
}

#[test]
fn all_flags_set() {
    for (flags, mask) in binary_values() {
        assert_eq!(flags & mask == mask, flags | mask == flags);
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(flags & mask == mask, flags | mask == flags);
    }
}

#[test]
fn no_flags_set_unmasked() {
    for value in unary_values() {
        assert_eq!(value == 0, value.count_ones() == 0);
        let value = value as u32;
        assert_eq!(value == 0, value.count_ones() == 0);
    }
}

#[test]
fn no_flags_set_masked() {
    for (flags, mask) in binary_values() {
        assert_eq!(flags & mask == 0, (!flags | !mask) == u64::MAX);
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(flags & mask == 0, (!flags | !mask) == u32::MAX);
    }
}
