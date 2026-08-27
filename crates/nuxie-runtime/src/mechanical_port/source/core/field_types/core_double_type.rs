use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub struct CoreDoubleType;

impl CoreDoubleType {
    pub const ID: i32 = 2;

    pub fn deserialize(reader: &mut BinaryReader) -> f32 {
        reader.read_float32()
    }

    #[cfg(feature = "rive_tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> f32 {
        let length = reader.length_in_bytes();
        if length == 4 {
            reader.read_float32()
        } else if length == 8 {
            reader.read_float64() as f32
        } else {
            0.0
        }
    }
}
