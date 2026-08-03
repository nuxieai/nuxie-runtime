use crate::core::binary_reader::BinaryReader;
use anyhow::Result;

// Retained only while the product/correspondence pins remain pre-S4. No S4
// runtime schema field dispatches here; remove with the integrator pin advance.
#[allow(dead_code)]
pub(super) fn deserialize(reader: &mut BinaryReader<'_>) -> Result<u64> {
    reader.read_var_uint()
}
