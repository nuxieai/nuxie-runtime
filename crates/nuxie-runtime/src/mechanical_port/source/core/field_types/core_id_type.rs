use crate::mechanical_port::source::core::{binary_reader::BinaryReader, id::Id};

pub struct CoreIdType;

impl CoreIdType {
    pub const ID: i32 = 0;

    pub fn deserialize(reader: &mut BinaryReader) -> Id {
        reader.read_var_uint_as::<Id>()
    }

    #[cfg(feature = "tools")]
    pub fn deserialize_rev(reader: &mut BinaryReader) -> Id {
        Self::deserialize(reader)
    }

    pub fn runtime_deserialize(reader: &mut BinaryReader) -> Id {
        reader.read_var_uint_as::<Id>()
    }
}
