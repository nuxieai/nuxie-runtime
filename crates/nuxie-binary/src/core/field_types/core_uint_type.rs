use crate::core::binary_reader::BinaryReader;
use anyhow::{Result, bail};

pub(super) fn deserialize(reader: &mut BinaryReader<'_>, label: &str) -> Result<u64> {
    let value = reader.read_var_uint()?;
    if value > u32::MAX as u64 {
        bail!("{label} {value} does not fit in C++ unsigned int");
    }
    Ok(value)
}

pub(super) fn deserialize_uint8(reader: &mut BinaryReader<'_>, label: &str) -> Result<u64> {
    deserialize(reader, label).map(|value| u64::from(value as u8))
}
