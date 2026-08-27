use core::ops::BitAnd;

pub fn clz32(value: u32) -> i32 {
    assert_ne!(value, 0);
    value.leading_zeros() as i32
}
pub fn clz64(value: u64) -> i32 {
    assert_ne!(value, 0);
    value.leading_zeros() as i32
}
pub fn most_significant_bit(value: u32) -> u32 {
    if value != 0 {
        32 - value.leading_zeros()
    } else {
        0
    }
}
pub fn rotate_left32(value: u32, amount: i32) -> u32 {
    value.rotate_left(amount as u32)
}
pub const fn count_set_bits_fallback(mut value: u64) -> u32 {
    let mut count = 0;
    while value != 0 {
        count += 1;
        value &= value - 1;
    }
    count
}
pub fn count_set_bits<T: Into<u64>>(value: T) -> u32 {
    value.into().count_ones()
}
pub const fn compact_bitmask_value(value: u32, mask: u32) -> u32 {
    let mut compacted = 0;
    let mut index = 31;
    loop {
        let bit = 1u32 << index;
        if mask & bit != 0 {
            compacted <<= 1;
            compacted |= if value & bit != 0 { 1 } else { 0 };
        }
        if index == 0 {
            break;
        }
        index -= 1;
    }
    compacted
}
pub const fn expand_compacted_bitmask_value(mut compacted: u32, mask: u32) -> u32 {
    let mut expanded = 0;
    let mut index = 0;
    while index < 32 {
        let bit = 1u32 << index;
        if mask & bit != 0 {
            if compacted & 1 != 0 {
                expanded |= bit;
            }
            compacted >>= 1;
        }
        index += 1;
    }
    expanded
}

pub trait BitMask: Copy + Eq + BitAnd<Output = Self> {
    fn to_u128(self) -> u128;
    fn from_u128(value: u128) -> Self;
}
macro_rules! bit_mask { ($($ty:ty),* $(,)?) => {$(impl BitMask for $ty { fn to_u128(self)->u128{self as u128} fn from_u128(value:u128)->Self{value as Self} })*}; }
bit_mask!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

pub struct BitCombinationIterable<T: BitMask> {
    mask: T,
}
pub struct BitCombinationIterator<T: BitMask> {
    current: T,
    mask: T,
    was_advanced: bool,
    end: bool,
}
impl<T: BitMask> Iterator for BitCombinationIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.end {
            return None;
        }
        let result = self.current;
        if self.was_advanced && self.current.to_u128() == 0 {
            self.end = true;
        } else {
            self.was_advanced = true;
            self.current = T::from_u128(self.current.to_u128().wrapping_sub(1)) & self.mask;
        }
        Some(result)
    }
}
impl<T: BitMask> IntoIterator for BitCombinationIterable<T> {
    type Item = T;
    type IntoIter = BitCombinationIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        BitCombinationIterator {
            current: self.mask,
            mask: self.mask,
            was_advanced: false,
            end: false,
        }
    }
}
pub fn iterate_bit_combinations_in_mask<T: BitMask>(mask: T) -> BitCombinationIterable<T> {
    BitCombinationIterable { mask }
}
pub fn add_bits_to_key<Key: BitMask, Bits: BitMask>(key: Key, bits: Bits, bit_count: u32) -> Key {
    let key_value = key.to_u128();
    let bits_value = bits.to_u128();
    assert_eq!((key_value << bit_count) >> bit_count, key_value);
    assert_eq!(bits_value & ((1u128 << bit_count) - 1), bits_value);
    Key::from_u128((key_value << bit_count) | bits_value)
}
