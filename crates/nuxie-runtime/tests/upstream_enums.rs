//! Native enum-owner ports of pinned `tests/unit_tests/runtime/enums_test.cpp`.
//!
//! Preserve this file's unsigned fixtures and independent integer oracles, but
//! exercise the translated enum functions rather than testing Rust against
//! itself. The companion exact file preserves the pinned signed/64-bit fixtures.

use nuxie_runtime::source::enums::{self, FlagEnum, ScopedEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Flags<U>(U);

// Value declarations for arbitrary flag bit patterns, corresponding to the
// pinned C++ enum declarations. Operations under test live in source::enums.
macro_rules! flags {
    ($repr:ty) => {
        impl ScopedEnum for Flags<$repr> {
            type Repr = $repr;
            fn underlying_value(self) -> $repr {
                self.0
            }
            fn from_underlying(value: $repr) -> Self {
                Self(value)
            }
            fn increment_underlying(value: $repr) -> $repr {
                value.wrapping_add(1)
            }
            fn decrement_underlying(value: $repr) -> $repr {
                value.wrapping_sub(1)
            }
        }
        impl FlagEnum for Flags<$repr> {
            const NONE: Self = Self(0);
        }
    };
}
flags!(u32);
flags!(u64);

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
        assert_eq!(
            enums::bit_or(Flags(left), Flags(right)).0,
            (!left & right) | left
        );
        assert_eq!(
            enums::bit_or(Flags(left as u32), Flags(right as u32)).0,
            (!(left as u32) & right as u32) | left as u32
        );
    }
}

#[test]
fn flag_operator_and() {
    for (left, right) in binary_values() {
        assert_eq!(
            enums::bit_and(Flags(left), Flags(right)).0,
            !(!left | !right)
        );
        assert_eq!(
            enums::bit_and(Flags(left as u32), Flags(right as u32)).0,
            !(!(left as u32) | !(right as u32))
        );
    }
}

#[test]
fn flag_operator_xor() {
    for (left, right) in binary_values() {
        assert_eq!(
            enums::bit_xor(Flags(left), Flags(right)).0,
            (left | right) & !(left & right)
        );
        assert_eq!(
            enums::bit_xor(Flags(left as u32), Flags(right as u32)).0,
            ((left as u32) | right as u32) & !((left as u32) & right as u32)
        );
    }
}

#[test]
fn flag_operator_not() {
    for value in unary_values() {
        assert_eq!(enums::bit_not(Flags(value)).0, u64::MAX ^ value);
        assert_eq!(
            enums::bit_not(Flags(value as u32)).0,
            u32::MAX ^ value as u32
        );
    }
}

#[test]
fn flag_operator_or_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = Flags(left);
        actual64 = enums::bit_or(actual64, Flags(right));
        assert_eq!(actual64.0, left | right);
        let mut actual32 = Flags(left as u32);
        actual32 = enums::bit_or(actual32, Flags(right as u32));
        assert_eq!(actual32.0, (left as u32) | right as u32);
    }
}

#[test]
fn flag_operator_and_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = Flags(left);
        actual64 = enums::bit_and(actual64, Flags(right));
        assert_eq!(actual64.0, left & right);
        let mut actual32 = Flags(left as u32);
        actual32 = enums::bit_and(actual32, Flags(right as u32));
        assert_eq!(actual32.0, (left as u32) & right as u32);
    }
}

#[test]
fn flag_operator_xor_assign() {
    for (left, right) in binary_values() {
        let mut actual64 = Flags(left);
        actual64 = enums::bit_xor(actual64, Flags(right));
        assert_eq!(actual64.0, left ^ right);
        let mut actual32 = Flags(left as u32);
        actual32 = enums::bit_xor(actual32, Flags(right as u32));
        assert_eq!(actual32.0, (left as u32) ^ right as u32);
    }
}

#[test]
fn is_single_flag() {
    assert!(!enums::is_single_flag(Flags(0u32)));
    assert!(!enums::is_single_flag(Flags(0u64)));
    for bit in 0..u32::BITS {
        assert!(enums::is_single_flag(Flags(1u32 << bit)));
    }
    for bit in 0..u64::BITS {
        assert!(enums::is_single_flag(Flags(1u64 << bit)));
    }
    for value in unary_values() {
        assert_eq!(
            enums::is_single_flag(Flags(value)),
            value != 0 && value & value.wrapping_sub(1) == 0
        );
        let value = value as u32;
        assert_eq!(
            enums::is_single_flag(Flags(value)),
            value != 0 && value & value.wrapping_sub(1) == 0
        );
    }
}

#[test]
fn is_flag_set() {
    assert!(!enums::is_flag_set(Flags(0u32), Flags(1u32)));
    assert!(enums::is_flag_set(Flags(1u32), Flags(1u32)));
    assert!(!enums::is_flag_set(Flags(1u32), Flags(2u32)));
    assert!(enums::is_flag_set(Flags(1u32 | 2), Flags(1u32)));
    for bit in 0..u64::BITS {
        let flag = 1u64 << bit;
        assert!(!enums::is_flag_set(Flags(0u64), Flags(flag)));
        for value in random_values().take(100) {
            assert_eq!(
                enums::is_flag_set(Flags(value), Flags(flag)),
                value / flag % 2 == 1
            );
        }
    }
    for bit in 0..u32::BITS {
        let flag = 1u32 << bit;
        assert!(!enums::is_flag_set(Flags(0u32), Flags(flag)));
        for value in random_values().take(100).map(|value| value as u32) {
            assert_eq!(
                enums::is_flag_set(Flags(value), Flags(flag)),
                value / flag % 2 == 1
            );
        }
    }
}

#[test]
fn underlying_value() {
    for value in unary_values() {
        assert_eq!(enums::underlying_value(Flags(value)), value);
        assert_eq!(enums::underlying_value(Flags(value as u32)), value as u32);
    }
}

#[test]
fn incr() {
    for value in unary_values() {
        assert_eq!(
            enums::increment(Flags(value)).0,
            value.wrapping_sub(u64::MAX)
        );
        let value = value as u32;
        assert_eq!(
            enums::increment(Flags(value)).0,
            value.wrapping_sub(u32::MAX)
        );
    }
}

#[test]
fn decr() {
    for value in unary_values() {
        assert_eq!(
            enums::decrement(Flags(value)).0,
            value.wrapping_add(u64::MAX)
        );
        let value = value as u32;
        assert_eq!(
            enums::decrement(Flags(value)).0,
            value.wrapping_add(u32::MAX)
        );
    }
}

#[test]
fn any_flag_set_unmasked() {
    for value in unary_values() {
        assert_eq!(enums::any_flag_set(Flags(value)), value.count_ones() > 0);
        // Preserve the pinned upstream test exactly: its `Flags64` branch
        // accidentally invokes `decr` instead of `any_flag_set`.
        assert_eq!(
            enums::decrement(Flags(value)).0,
            value.wrapping_add(u64::MAX)
        );
    }
}

#[test]
fn any_flag_set_masked() {
    for (flags, mask) in binary_values() {
        assert_eq!(
            enums::any_flag_in_mask(Flags(flags), Flags(mask)),
            (flags & mask).count_ones() > 0
        );
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(
            enums::any_flag_in_mask(Flags(flags), Flags(mask)),
            (flags & mask).count_ones() > 0
        );
    }
}

#[test]
fn all_flags_set() {
    for (flags, mask) in binary_values() {
        assert_eq!(
            enums::all_flags_set(Flags(flags), Flags(mask)),
            flags | mask == flags
        );
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(
            enums::all_flags_set(Flags(flags), Flags(mask)),
            flags | mask == flags
        );
    }
}

#[test]
fn no_flags_set_unmasked() {
    for value in unary_values() {
        assert_eq!(enums::no_flags_set(Flags(value)), value.count_ones() == 0);
        let value = value as u32;
        assert_eq!(enums::no_flags_set(Flags(value)), value.count_ones() == 0);
    }
}

#[test]
fn no_flags_set_masked() {
    for (flags, mask) in binary_values() {
        assert_eq!(
            enums::no_flags_in_mask(Flags(flags), Flags(mask)),
            (!flags | !mask) == u64::MAX
        );
        let (flags, mask) = (flags as u32, mask as u32);
        assert_eq!(
            enums::no_flags_in_mask(Flags(flags), Flags(mask)),
            (!flags | !mask) == u32::MAX
        );
    }
}
