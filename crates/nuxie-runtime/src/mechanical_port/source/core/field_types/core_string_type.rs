use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub struct CoreStringType;

impl CoreStringType {
    pub const ID: i32 = 1;

    pub fn deserialize(reader: &mut BinaryReader) -> String {
        reader.read_string()
    }

    #[cfg(feature = "tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> String {
        let length = reader.length_in_bytes();
        reader.read_string_length(length)
    }
}
