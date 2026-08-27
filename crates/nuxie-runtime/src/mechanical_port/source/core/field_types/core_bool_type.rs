use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub struct CoreBoolType;

impl CoreBoolType {
    pub const ID: i32 = 4;

    pub fn deserialize(reader: &mut BinaryReader) -> bool {
        reader.read_byte() == 1
    }

    #[cfg(feature = "rive_tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> bool {
        Self::deserialize(reader)
    }
}
