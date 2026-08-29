//! Serialization of explicitly decoded/local-authoring descriptors.

use crate::{BinaryWriter, FieldValue, HeaderFieldKind, RuntimeFile};
use anyhow::{Result, bail, ensure};

/// Materialize a locally authored descriptor as ordinary Rive records so it
/// follows the same import/lifecycle path as byte input. This is not a claimed
/// lossless serialization of an imported file: discarded unknown fields and
/// duplicate wire occurrences cannot be recovered from a descriptor.
pub fn encode_runtime_file(file: &RuntimeFile) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    {
        let mut writer = BinaryWriter::new(&mut bytes);
        writer.write_bytes(b"RIVE");
        writer.write_var_uint64(file.header.major_version);
        writer.write_var_uint64(file.header.minor_version);
        writer.write_var_uint64(file.header.file_id);
        for &key in file.header.property_field_ids.keys() {
            ensure!(key != 0, "property key zero is the record terminator");
            writer.write_var_uint32(key);
        }
        writer.write_var_uint32(0);
        let fields: Vec<_> = file.header.property_field_ids.values().collect();
        // RuntimeHeader::read consumes four two-bit entries per uint32, not
        // sixteen entries. Preserve that pinned wire packing exactly.
        for group in fields.chunks(4) {
            let mut packed = 0u32;
            for (index, field) in group.iter().enumerate() {
                let id = match field {
                    HeaderFieldKind::Uint => 0,
                    HeaderFieldKind::StringOrBytes => 1,
                    HeaderFieldKind::Double => 2,
                    HeaderFieldKind::Color => 3,
                };
                packed |= id << (index * 2);
            }
            writer.write_u32(packed);
        }
        for object in &file.objects {
            let Some(object) = object else {
                writer.write_var_uint32(0);
                writer.write_var_uint32(0);
                continue;
            };
            writer.write_var_uint32(u32::from(object.type_key));
            for property in &object.properties {
                write_property(&mut writer, property.key, &property.value)?;
            }
            for property in &object.skipped_properties {
                if let Some(value) = &property.value {
                    write_property(&mut writer, property.key, value)?;
                }
            }
            writer.write_var_uint32(0);
        }
    }
    Ok(bytes)
}

fn write_property(writer: &mut BinaryWriter<'_>, key: u16, value: &FieldValue) -> Result<()> {
    ensure!(key != 0, "property key zero is the record terminator");
    if matches!(value, FieldValue::Callback) {
        bail!("callback property {key} has no serializable wire value");
    }
    writer.write_var_uint32(u32::from(key));
    match value {
        FieldValue::Bool(value) => writer.write_u8(u8::from(*value)),
        FieldValue::Bytes(value) => writer.write_string(value.as_bytes()),
        FieldValue::Color(value) => writer.write_u32(*value),
        FieldValue::Double(value) => writer.write_f32(*value),
        FieldValue::Int(value) => {
            let encoded = ((*value as u32) << 1) ^ ((*value >> 31) as u32);
            writer.write_var_uint32(encoded);
        }
        FieldValue::String(value) => writer.write_string(value.as_bytes()),
        FieldValue::Uint(value) => writer.write_var_uint64(*value),
        FieldValue::Callback => unreachable!("rejected before writing the key"),
    }
    Ok(())
}
