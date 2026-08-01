use crate::core::binary_reader::BinaryReader;
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<f32> {
    reader.read_f32()
}
