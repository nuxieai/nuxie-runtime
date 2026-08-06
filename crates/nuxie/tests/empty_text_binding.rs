//! Empty-text binding parity evidence.
//!
//! This test emits a source-first empty-text file when
//! `EMPTY_TEXT_BINDING_EVIDENCE_RIV` is set. It verifies that an explicitly
//! empty ViewModel string remains empty through encode, import, binding,
//! shaping, and drawing without synthesizing placeholder glyphs.

use std::sync::Arc;

use anyhow::{Context, Result};
use nuxie::{File, OwnedArtboardInstance, RecordingFactory};
use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, read_runtime_file};

#[allow(clippy::arithmetic_side_effects)]
fn fixture_font_bytes() -> Vec<u8> {
    let mut accumulator = 0u32;
    let mut bit_count = 0u8;
    let mut decoded = Vec::new();
    for byte in include_bytes!("../../nuxie-product/tests/fixtures/roboto-a.ttf.base64")
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 font fixture"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            decoded.push((accumulator >> bit_count) as u8);
            accumulator &= (1u32 << bit_count) - 1;
        }
    }
    decoded
}

fn fixture_authoring_record(
    type_name: &str,
    properties: Vec<(&str, AuthoringValue)>,
) -> AuthoringRecord {
    let definition =
        nuxie_schema::definition_by_name(type_name).expect("fixture record type exists");
    let properties = properties
        .into_iter()
        .map(|(property_name, value)| {
            let property = std::iter::once(definition.name)
                .chain(definition.ancestors.iter().copied())
                .filter_map(nuxie_schema::definition_by_name)
                .flat_map(|owner| owner.properties)
                .find(|property| property.name == property_name)
                .unwrap_or_else(|| panic!("fixture property exists: {type_name}.{property_name}"));
            AuthoringProperty {
                key: property.key.int,
                value,
            }
        })
        .collect();
    AuthoringRecord {
        type_key: definition.type_key.int,
        properties,
    }
}

fn fixture_property_key(type_name: &str, property_name: &str) -> u16 {
    let definition =
        nuxie_schema::definition_by_name(type_name).expect("fixture record type exists");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("fixture property exists: {type_name}.{property_name}"))
        .key
        .int
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_authoring_records(records: &[AuthoringRecord]) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 0x4e55_5849);
    push_var_uint(&mut bytes, 0);
    for record in records {
        push_var_uint(&mut bytes, u64::from(record.type_key));
        for property in &record.properties {
            push_var_uint(&mut bytes, u64::from(property.key));
            match &property.value {
                AuthoringValue::Bool(value) => bytes.push(u8::from(*value)),
                AuthoringValue::Bytes(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value);
                }
                AuthoringValue::Color(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                AuthoringValue::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                AuthoringValue::Int(value) => {
                    let encoded = ((*value as u32) << 1) ^ ((*value >> 31) as u32);
                    push_var_uint(&mut bytes, u64::from(encoded));
                }
                AuthoringValue::String(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value.as_bytes());
                }
                AuthoringValue::Uint(value) => push_var_uint(&mut bytes, *value),
            }
        }
        push_var_uint(&mut bytes, 0);
    }
    bytes
}

fn empty_source_first_records() -> Vec<AuthoringRecord> {
    vec![
        fixture_authoring_record("Backboard", vec![]),
        fixture_authoring_record(
            "FontAsset",
            vec![
                ("name", AuthoringValue::String("Evidence font".into())),
                ("assetId", AuthoringValue::Uint(0)),
            ],
        ),
        fixture_authoring_record(
            "FileAssetContents",
            vec![("bytes", AuthoringValue::Bytes(fixture_font_bytes()))],
        ),
        fixture_authoring_record(
            "ViewModel",
            vec![("name", AuthoringValue::String("Evidence model".into()))],
        ),
        fixture_authoring_record(
            "ViewModelPropertyString",
            vec![("name", AuthoringValue::String("value".into()))],
        ),
        fixture_authoring_record(
            "ViewModelInstance",
            vec![
                ("name", AuthoringValue::String("Evidence defaults".into())),
                ("viewModelId", AuthoringValue::Uint(0)),
            ],
        ),
        fixture_authoring_record(
            "ViewModelInstanceString",
            vec![
                ("viewModelPropertyId", AuthoringValue::Uint(0)),
                ("propertyValue", AuthoringValue::String(String::new())),
            ],
        ),
        fixture_authoring_record(
            "Artboard",
            vec![
                ("name", AuthoringValue::String("Empty 1".into())),
                ("width", AuthoringValue::Double(120.0)),
                ("height", AuthoringValue::Double(40.0)),
                ("viewModelId", AuthoringValue::Uint(0)),
            ],
        ),
        fixture_authoring_record(
            "Text",
            vec![
                ("name", AuthoringValue::String("Empty projection".into())),
                ("x", AuthoringValue::Double(0.0)),
                ("y", AuthoringValue::Double(0.0)),
                ("alignValue", AuthoringValue::Uint(0)),
                ("sizingValue", AuthoringValue::Uint(2)),
                ("width", AuthoringValue::Double(120.0)),
                ("height", AuthoringValue::Double(40.0)),
                ("overflowValue", AuthoringValue::Uint(0)),
                ("wrapValue", AuthoringValue::Uint(1)),
            ],
        ),
        fixture_authoring_record(
            "TextStylePaint",
            vec![
                ("name", AuthoringValue::String("Evidence style".into())),
                ("parentId", AuthoringValue::Uint(1)),
                ("fontSize", AuthoringValue::Double(18.0)),
                ("fontAssetId", AuthoringValue::Uint(0)),
                ("lineHeight", AuthoringValue::Double(22.0)),
                ("letterSpacing", AuthoringValue::Double(0.0)),
            ],
        ),
        fixture_authoring_record(
            "Fill",
            vec![
                ("name", AuthoringValue::String("Evidence fill".into())),
                ("parentId", AuthoringValue::Uint(2)),
                ("fillRule", AuthoringValue::Uint(0)),
            ],
        ),
        fixture_authoring_record(
            "SolidColor",
            vec![
                ("name", AuthoringValue::String("Evidence color".into())),
                ("parentId", AuthoringValue::Uint(3)),
                ("colorValue", AuthoringValue::Color(0xffab_cdef)),
            ],
        ),
        fixture_authoring_record(
            "TextValueRun",
            vec![
                ("name", AuthoringValue::String("Evidence run".into())),
                ("parentId", AuthoringValue::Uint(1)),
                ("text", AuthoringValue::String(String::new())),
                ("styleId", AuthoringValue::Uint(2)),
            ],
        ),
        fixture_authoring_record(
            "DataBindContext",
            vec![
                // TextValueRun.text.
                ("propertyKey", AuthoringValue::Uint(268)),
                // TwoWay | SourceToTargetRunsFirst.
                ("flags", AuthoringValue::Uint(10)),
                ("sourcePathIds", AuthoringValue::Bytes(vec![0, 0])),
            ],
        ),
    ]
}

#[test]
fn empty_source_first_text_stays_empty_through_encode_import_bind_shape_and_draw() -> Result<()> {
    let records = empty_source_first_records();
    let text_definition =
        nuxie_schema::definition_by_name("TextValueRun").context("TextValueRun definition")?;
    let text_record = records
        .iter()
        .find(|record| record.type_key == text_definition.type_key.int)
        .context("authored TextValueRun record")?;
    let text_key = fixture_property_key("TextValueRun", "text");
    assert!(text_record.properties.iter().any(|property| {
        property.key == text_key && property.value == AuthoringValue::String(String::new())
    }));

    let bytes = encode_authoring_records(&records);
    assert_eq!(&bytes[..4], b"RIVE");
    let encoded_file = read_runtime_file(&bytes)?;
    let encoded_text_run = (0..encoded_file.object_count())
        .filter_map(|id| encoded_file.object(id))
        .find(|object| object.type_name == "TextValueRun")
        .context("encoded TextValueRun")?;
    assert_eq!(
        encoded_text_run.string_property("text"),
        Some(""),
        "the encoded binary must retain the authored empty run"
    );
    let encoded_bind = (0..encoded_file.object_count())
        .filter_map(|id| encoded_file.object(id))
        .find(|object| object.type_name == "DataBindContext")
        .context("encoded source-first data bind")?;
    assert_eq!(encoded_bind.uint_property("propertyKey"), Some(268));
    assert_eq!(
        encoded_bind.uint_property("flags"),
        Some(10),
        "the encoded bind must be TwoWay | SourceToTargetRunsFirst"
    );
    assert_eq!(
        encoded_bind.bytes_property("sourcePathIds"),
        Some(&[0, 0][..])
    );
    if let Some(path) = std::env::var_os("EMPTY_TEXT_BINDING_EVIDENCE_RIV") {
        std::fs::write(path, &bytes)?;
    }

    let file = Arc::new(File::import(&bytes)?);
    let mut instance = OwnedArtboardInstance::instantiate(file, 0)?;
    let view_model = instance
        .instantiate_view_model_instance(0)
        .context("imported default ViewModel instance")?;
    assert_eq!(
        view_model
            .raw()
            .string_value_by_property_name("value")
            .as_deref(),
        Some(&[][..]),
        "the imported ViewModel source must retain explicit empty"
    );
    let _ = instance.bind_view_model(&view_model);
    assert!(
        instance.owned_view_model_context().is_some(),
        "the imported source-first bind must retain the ViewModel context"
    );
    instance.advance(0.0);

    let resolved = instance.raw_mut().semantic_text_with_bounds();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].value, "");
    // Runtime local 1 is the Text record. Empty shaping retains only the
    // insertion boundary; offset 1 would prove a synthesized character.
    assert!(instance.raw_mut().text_caret(1, 0).is_some());
    assert!(instance.raw_mut().text_caret(1, 1).is_none());

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance.draw(&mut factory, &mut renderer)?;
    let draw = factory.canonical_recording();
    assert!(
        !draw.stream().contains("drawPath "),
        "an explicitly empty source-first text run emitted glyph paths:\n{}",
        draw.stream()
    );
    Ok(())
}
