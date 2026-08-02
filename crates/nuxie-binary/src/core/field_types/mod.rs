mod core_bool_type;
mod core_bytes_type;
mod core_color_type;
mod core_double_type;
mod core_int_type;
mod core_string_type;
mod core_uint64_type;
mod core_uint_type;

use crate::{FieldValue, core::binary_reader::BinaryReader};
use anyhow::{Context, Result, bail};
use nuxie_schema::{FieldKind, Property, UintStorage};

pub(crate) fn read_string_or_bytes_value(
    reader: &mut BinaryReader<'_>,
    property: &Property,
) -> Result<FieldValue> {
    if property.runtime_type == FieldKind::Bytes {
        Ok(FieldValue::Bytes(core_bytes_type::deserialize(reader)?))
    } else {
        Ok(FieldValue::String(core_string_type::deserialize(reader)?))
    }
}

pub(crate) fn read_field_value(
    reader: &mut BinaryReader<'_>,
    property: &Property,
) -> Result<FieldValue> {
    Ok(match property.runtime_type {
        FieldKind::Bool => FieldValue::Bool(core_bool_type::deserialize(reader)?),
        FieldKind::Bytes => FieldValue::Bytes(core_bytes_type::deserialize(reader)?),
        FieldKind::Callback => FieldValue::Callback,
        FieldKind::Color => FieldValue::Color(core_color_type::deserialize(reader)?),
        FieldKind::Double => FieldValue::Double(core_double_type::deserialize(reader)?),
        FieldKind::Int => FieldValue::Int(read_known_int_field(reader, property, "int field")?),
        FieldKind::String => FieldValue::String(core_string_type::deserialize(reader)?),
        FieldKind::Uint => FieldValue::Uint(read_known_uint_field(reader, property, "uint field")?),
    })
}

pub(crate) fn read_known_int_field(
    reader: &mut BinaryReader<'_>,
    property: &Property,
    label: &str,
) -> Result<i32> {
    let storage = property
        .int_storage()
        .with_context(|| format!("{label} schema property is not int-like"))?;
    core_int_type::deserialize(reader, storage, label)
}

pub(crate) fn read_known_uint_field(
    reader: &mut BinaryReader<'_>,
    property: &Property,
    label: &str,
) -> Result<u64> {
    match property.uint_storage() {
        Some(UintStorage::Uint64) => core_uint64_type::deserialize(reader),
        Some(UintStorage::Uint8) => core_uint_type::deserialize_uint8(reader, label),
        Some(UintStorage::Uint32) => core_uint_type::deserialize(reader, label),
        None => bail!("{label} schema property is not uint-like"),
    }
}
