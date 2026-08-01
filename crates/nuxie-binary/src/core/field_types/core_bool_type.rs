use crate::core::binary_reader::BinaryReader;
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<bool> {
    Ok(reader.read_byte()? == 1)
}
