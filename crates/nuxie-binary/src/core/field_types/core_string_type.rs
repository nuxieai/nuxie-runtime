use crate::{StringValue, core::binary_reader::BinaryReader};
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<StringValue> {
    reader.read_string()
}
