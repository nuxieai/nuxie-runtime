//! Exact Rust-language ports of pinned `runtime/enums_test.cpp`.

use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

const SEED: u64 = 0x0f93_4929;

struct Mt19937_64 {
    state: [u64; 312],
    index: usize,
}

impl Mt19937_64 {
    fn new(seed: u64) -> Self {
        let mut state = [0; 312];
        state[0] = seed;
        for index in 1..312 {
            state[index] = 6_364_136_223_846_793_005_u64
                .wrapping_mul(state[index - 1] ^ (state[index - 1] >> 62))
                .wrapping_add(index as u64);
        }
        Self { state, index: 312 }
    }

    fn twist(&mut self) {
        const A: u64 = 0xb502_6f5a_a966_19e9;
        const LOWER: u64 = 0x7fff_ffff;
        const UPPER: u64 = 0xffff_ffff_8000_0000;
        for index in 0..156 {
            let bits = (self.state[index] & UPPER) | (self.state[index + 1] & LOWER);
            self.state[index] =
                self.state[index + 156] ^ (bits >> 1) ^ if bits & 1 == 0 { 0 } else { A };
        }
        for index in 156..311 {
            let bits = (self.state[index] & UPPER) | (self.state[index + 1] & LOWER);
            self.state[index] =
                self.state[index - 156] ^ (bits >> 1) ^ if bits & 1 == 0 { 0 } else { A };
        }
        let bits = (self.state[311] & UPPER) | (self.state[0] & LOWER);
        self.state[311] = self.state[155] ^ (bits >> 1) ^ if bits & 1 == 0 { 0 } else { A };
        self.index = 0;
    }

    fn next(&mut self) -> u64 {
        if self.index == 312 {
            self.twist();
        }
        let mut value = self.state[self.index];
        self.index += 1;
        value ^= (value >> 29) & 0x5555_5555_5555_5555;
        value ^= (value << 17) & 0x71d6_7fff_eda6_0000;
        value ^= (value << 37) & 0xfff7_eee0_0000_0000;
        value ^ (value >> 43)
    }
}

trait Underlying:
    BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Copy
    + Debug
    + Eq
    + Not<Output = Self>
{
    fn from_random(value: u64) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self;
    fn bit_width() -> u32;
    fn single_bit(index: u32) -> Self;
    fn count_ones(self) -> u32;
    fn wrapping_add_one(self) -> Self;
    fn wrapping_sub_one(self) -> Self;
    fn unary_basics() -> Vec<Self>;
    fn binary_basics() -> Vec<(Self, Self)>;
}

macro_rules! impl_underlying {
    ($ty:ty, $minus_one:expr) => {
        impl Underlying for $ty {
            fn from_random(value: u64) -> Self {
                value as Self
            }
            fn zero() -> Self {
                0
            }
            fn one() -> Self {
                1
            }
            fn two() -> Self {
                2
            }
            fn bit_width() -> u32 {
                Self::BITS
            }
            fn single_bit(index: u32) -> Self {
                ((1_u64 << index) as Self)
            }
            fn count_ones(self) -> u32 {
                self.count_ones()
            }
            fn wrapping_add_one(self) -> Self {
                self.wrapping_add(1)
            }
            fn wrapping_sub_one(self) -> Self {
                self.wrapping_sub(1)
            }
            fn unary_basics() -> Vec<Self> {
                vec![0, 1, 2, 3, $minus_one]
            }
            fn binary_basics() -> Vec<(Self, Self)> {
                vec![
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (0, 1),
                    (0, 2),
                    (1, 1),
                    (2, 1),
                    (3, 1),
                    (0, $minus_one),
                    (1, $minus_one),
                    (2, $minus_one),
                ]
            }
        }
    };
}

impl_underlying!(i32, -1);
impl_underlying!(u64, u64::MAX);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Flag<U>(U);

impl<U: BitOr<Output = U>> BitOr for Flag<U> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl<U: BitAnd<Output = U>> BitAnd for Flag<U> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl<U: BitXor<Output = U>> BitXor for Flag<U> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl<U: Not<Output = U>> Not for Flag<U> {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl<U: BitOrAssign> BitOrAssign for Flag<U> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl<U: BitAndAssign> BitAndAssign for Flag<U> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl<U: BitXorAssign> BitXorAssign for Flag<U> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

fn unary_values<U: Underlying>() -> Vec<U> {
    let mut values = U::unary_basics();
    let mut random = Mt19937_64::new(SEED);
    values.extend((0..1_000).map(|_| U::from_random(random.next())));
    values
}

fn binary_values<U: Underlying>() -> Vec<(U, U)> {
    let mut values = U::binary_basics();
    let mut random = Mt19937_64::new(SEED);
    values
        .extend((0..1_000).map(|_| (U::from_random(random.next()), U::from_random(random.next()))));
    values
}

fn unary_enum<U: Underlying>(typed: impl Fn(Flag<U>) -> Flag<U>, integral: impl Fn(U) -> U) {
    for value in unary_values::<U>() {
        assert_eq!(typed(Flag(value)).0, integral(value));
    }
}

fn binary_enum<U: Underlying>(
    typed: impl Fn(Flag<U>, Flag<U>) -> Flag<U>,
    integral: impl Fn(U, U) -> U,
) {
    for (left, right) in binary_values::<U>() {
        assert_eq!(typed(Flag(left), Flag(right)).0, integral(left, right));
    }
}

fn unary_scalar<U: Underlying, R: Debug + Eq>(
    typed: impl Fn(Flag<U>) -> R,
    integral: impl Fn(U) -> R,
) {
    for value in unary_values::<U>() {
        assert_eq!(typed(Flag(value)), integral(value));
    }
}

fn binary_scalar<U: Underlying, R: Debug + Eq>(
    typed: impl Fn(Flag<U>, Flag<U>) -> R,
    integral: impl Fn(U, U) -> R,
) {
    for (left, right) in binary_values::<U>() {
        assert_eq!(typed(Flag(left), Flag(right)), integral(left, right));
    }
}

macro_rules! both {
    ($helper:ident, $typed:expr, $integral:expr) => {{
        $helper::<i32>($typed, $integral);
        $helper::<u64>($typed, $integral);
    }};
}

#[test]
fn wave_b2_flag_operator_or() {
    let mut fixture = Mt19937_64::new(SEED);
    assert_eq!(
        (0..5).map(|_| fixture.next()).collect::<Vec<_>>(),
        vec![
            0x92b8_649e_ef07_5640,
            0x266a_f799_3e09_c384,
            0x10a1_3963_78a1_6c29,
            0xebee_2081_e2c5_d884,
            0x6916_cc9a_901f_b4d6,
        ],
        "std::mt19937_64(0xf934929) fixture prefix",
    );
    let mut fixture = Mt19937_64::new(SEED);
    let fixture_fingerprint = (0..1_000).fold(1_469_598_103_934_665_603_u64, |hash, _| {
        (hash ^ fixture.next()).wrapping_mul(1_099_511_628_211)
    });
    assert_eq!(
        fixture_fingerprint, 0xbd69_e87b_0d18_76de,
        "1,000-value std::mt19937_64 fixture fingerprint from pinned C++",
    );
    binary_enum::<i32>(|a, b| a | b, |a, b| a | b);
}

#[test]
fn wave_b2_flag_operator_and() {
    binary_enum::<i32>(|a, b| a & b, |a, b| a & b);
}

#[test]
fn wave_b2_flag_operator_xor() {
    binary_enum::<i32>(|a, b| a ^ b, |a, b| a ^ b);
}

#[test]
fn wave_b2_flag_operator_not() {
    unary_enum::<i32>(|a| !a, |a| !a);
}

#[test]
fn wave_b2_flag_operator_or_assign() {
    binary_enum::<i32>(
        |mut a, b| {
            a |= b;
            a
        },
        |mut a, b| {
            a |= b;
            a
        },
    );
}

#[test]
fn wave_b2_flag_operator_and_assign() {
    binary_enum::<i32>(
        |mut a, b| {
            a &= b;
            a
        },
        |mut a, b| {
            a &= b;
            a
        },
    );
}

#[test]
fn wave_b2_flag_operator_xor_assign() {
    binary_enum::<i32>(
        |mut a, b| {
            a ^= b;
            a
        },
        |mut a, b| {
            a ^= b;
            a
        },
    );
}

fn single<U: Underlying>(flag: Flag<U>) -> bool {
    flag.0.count_ones() == 1
}

#[test]
fn wave_b2_is_single_flag() {
    assert!(!single(Flag(0_i32)));
    assert!(!single(Flag(0_u64)));
    for bit in 0..i32::BITS {
        assert!(single(Flag(i32::single_bit(bit))));
    }
    for bit in 0..u64::BITS {
        assert!(single(Flag(u64::single_bit(bit))));
    }
    unary_scalar::<i32, bool>(single, |value| value.count_ones() == 1);
    unary_scalar::<u64, bool>(single, |value| value.count_ones() == 1);
}

fn flag_set<U: Underlying>(flags: Flag<U>, mask: Flag<U>) -> bool {
    flags.0 & mask.0 != U::zero()
}

fn check_flag_set<U: Underlying>() {
    assert!(!flag_set(Flag(U::zero()), Flag(U::one())));
    assert!(flag_set(Flag(U::one()), Flag(U::one())));
    assert!(!flag_set(Flag(U::one()), Flag(U::two())));
    assert!(flag_set(Flag(U::one() | U::two()), Flag(U::one())));
    let mut random = Mt19937_64::new(SEED);
    for bit in 0..U::bit_width() {
        let integral_flag = U::single_bit(bit);
        assert!(!flag_set(Flag(U::zero()), Flag(integral_flag)));
        for _ in 0..100 {
            let value = U::from_random(random.next());
            assert_eq!(
                flag_set(Flag(value), Flag(integral_flag)),
                value & integral_flag != U::zero()
            );
        }
    }
}

#[test]
fn wave_b2_is_flag_set() {
    check_flag_set::<i32>();
    check_flag_set::<u64>();
}

#[test]
fn wave_b2_underlying_value() {
    unary_scalar::<i32, i32>(|flag| flag.0, |value| value);
    unary_scalar::<u64, u64>(|flag| flag.0, |value| value);
}

#[test]
fn wave_b2_incr() {
    both!(
        unary_enum,
        |flag: Flag<_>| Flag(flag.0.wrapping_add_one()),
        |value: _| value.wrapping_add_one()
    );
}

#[test]
fn wave_b2_decr() {
    both!(
        unary_enum,
        |flag: Flag<_>| Flag(flag.0.wrapping_sub_one()),
        |value: _| value.wrapping_sub_one()
    );
}

#[test]
fn wave_b2_any_flag_set_unmasked() {
    unary_scalar::<i32, bool>(|flag| flag.0 != 0, |value| value != 0);
    // Preserve the pinned Flags64 branch's accidental call to decr.
    unary_enum::<u64>(
        |flag| Flag(flag.0.wrapping_sub_one()),
        |value| value.wrapping_sub_one(),
    );
}

#[test]
fn wave_b2_any_flag_set_masked() {
    binary_scalar::<i32, bool>(flag_set, |flags, mask| flags & mask != 0);
    binary_scalar::<u64, bool>(flag_set, |flags, mask| flags & mask != 0);
}

#[test]
fn wave_b2_all_flags_set() {
    binary_scalar::<i32, bool>(|f, m| f.0 & m.0 == m.0, |f, m| f & m == m);
    binary_scalar::<u64, bool>(|f, m| f.0 & m.0 == m.0, |f, m| f & m == m);
}

#[test]
fn wave_b2_no_flag_set_unmasked() {
    unary_scalar::<i32, bool>(|flag| flag.0 == 0, |value| value == 0);
    unary_scalar::<u64, bool>(|flag| flag.0 == 0, |value| value == 0);
}

#[test]
fn wave_b2_no_flags_set_masked() {
    binary_scalar::<i32, bool>(|f, m| f.0 & m.0 == 0, |f, m| f & m == 0);
    binary_scalar::<u64, bool>(|f, m| f.0 & m.0 == 0, |f, m| f & m == 0);
}
