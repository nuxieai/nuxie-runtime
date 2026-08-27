use crate::mechanical_port::source::core::binary_reader::BinaryReader;
use crate::mechanical_port::source::span::Span;

pub struct CoreBytesType;

impl CoreBytesType {
    pub const ID: i32 = 1;

    pub fn deserialize<'a>(reader: &'a mut BinaryReader) -> Span<'a, u8> {
        reader.read_bytes()
    }

    #[cfg(feature = "rive_tools")]
    pub fn deserialize_rev<'a>(reader: &'a mut BinaryReader) -> Span<'a, u8> {
        let length = reader.length_in_bytes();
        reader.read_bytes_with_length(length)
    }
}
