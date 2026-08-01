use crate::core::binary_reader::BinaryReader;
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<u32> {
    reader.read_u32()
}
