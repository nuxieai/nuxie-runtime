use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub struct CoreUintType;

impl CoreUintType {
    pub const ID: i32 = 0;

    pub fn deserialize(reader: &mut BinaryReader) -> u32 {
        reader.read_var_uint_as_u32()
    }

    #[cfg(feature = "rive_tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> u32 {
        Self::deserialize(reader)
    }
}
