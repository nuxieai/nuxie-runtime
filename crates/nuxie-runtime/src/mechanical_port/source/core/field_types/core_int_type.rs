use crate::mechanical_port::source::core::binary_reader::BinaryReader;
use crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType;

pub struct CoreIntType;

impl CoreIntType {
    pub const ID: i32 = CoreUintType::ID;

    pub fn deserialize(reader: &mut BinaryReader) -> i32 {
        Self::zigzag_decode(reader.read_var_uint_as_u32())
    }

    #[cfg(feature = "rive_tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> i32 {
        Self::deserialize(reader)
    }

    /// `(n << 1) ^ (n >> 31)` maps small magnitudes of either sign onto
    /// small unsigned values so the varuint stays short.
    pub fn zigzag_encode(value: i32) -> u32 {
        (value as u32).wrapping_shl(1) ^ ((value >> 31) as u32)
    }

    pub fn zigzag_decode(value: u32) -> i32 {
        ((value >> 1) as i32) ^ -((value & 1) as i32)
    }
}
