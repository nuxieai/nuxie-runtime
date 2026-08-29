use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub struct CoreColorType;

impl CoreColorType {
    pub const ID: i32 = 3;

    pub fn deserialize(reader: &mut BinaryReader) -> i32 {
        reader.read_uint32() as i32
    }

    #[cfg(feature = "tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> i32 {
        reader.read_var_uint64() as i32
    }
}
