pub trait ScopedEnum: Copy + PartialEq {
    type Repr: Copy
        + PartialEq
        + std::ops::BitAnd<Output = Self::Repr>
        + std::ops::BitOr<Output = Self::Repr>
        + std::ops::BitXor<Output = Self::Repr>
        + std::ops::Not<Output = Self::Repr>
        + std::ops::Sub<Output = Self::Repr>
        + From<u8>;

    fn underlying_value(self) -> Self::Repr;
    fn from_underlying(value: Self::Repr) -> Self;
    fn increment_underlying(value: Self::Repr) -> Self::Repr;
    fn decrement_underlying(value: Self::Repr) -> Self::Repr;
}

pub trait FlagEnum: ScopedEnum {
    const NONE: Self;
}

pub fn underlying_value<E: ScopedEnum>(value: E) -> E::Repr {
    value.underlying_value()
}

pub fn increment<E: ScopedEnum>(value: E) -> E {
    E::from_underlying(E::increment_underlying(value.underlying_value()))
}

pub fn decrement<E: ScopedEnum>(value: E) -> E {
    E::from_underlying(E::decrement_underlying(value.underlying_value()))
}

pub fn bit_and<E: FlagEnum>(a: E, b: E) -> E {
    E::from_underlying(a.underlying_value() & b.underlying_value())
}

pub fn bit_xor<E: FlagEnum>(a: E, b: E) -> E {
    E::from_underlying(a.underlying_value() ^ b.underlying_value())
}

pub fn bit_or<E: FlagEnum>(a: E, b: E) -> E {
    E::from_underlying(a.underlying_value() | b.underlying_value())
}

pub fn bit_not<E: FlagEnum>(value: E) -> E {
    E::from_underlying(!value.underlying_value())
}

pub fn is_single_flag<E: FlagEnum>(flags: E) -> bool {
    let value = flags.underlying_value();
    let zero = E::Repr::from(0);
    let one = E::Repr::from(1);
    value != zero && (value & (value - one)) == zero
}

pub fn is_flag_set<E: FlagEnum>(flags: E, test_flag: E) -> bool {
    assert!(is_single_flag(test_flag));
    bit_and(flags, test_flag) != E::NONE
}

pub fn any_flag_set<E: FlagEnum>(flags: E) -> bool {
    flags != E::NONE
}

pub fn any_flag_in_mask<E: FlagEnum>(flags: E, mask: E) -> bool {
    any_flag_set(bit_and(flags, mask))
}

pub fn all_flags_set<E: FlagEnum>(flags: E, mask: E) -> bool {
    bit_and(flags, mask) == mask
}

pub fn no_flags_set<E: FlagEnum>(flags: E) -> bool {
    flags == E::NONE
}

pub fn no_flags_in_mask<E: FlagEnum>(flags: E, mask: E) -> bool {
    no_flags_set(bit_and(flags, mask))
}
