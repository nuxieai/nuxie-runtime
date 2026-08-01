use crate::{BytesValue, core::binary_reader::BinaryReader};
use anyhow::Result;

pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<BytesValue> {
    let bytes = reader.read_length_prefixed_bytes()?;
    Ok(BytesValue::new(bytes.to_vec()))
}
