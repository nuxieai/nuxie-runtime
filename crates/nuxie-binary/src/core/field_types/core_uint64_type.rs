use crate::core::binary_reader::BinaryReader;
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<u64> {
    reader.read_var_uint()
}
