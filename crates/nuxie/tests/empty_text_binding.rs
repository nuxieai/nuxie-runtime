//! Empty-text binding parity evidence.
//!
//! This test emits a source-first empty-text file when
//! `EMPTY_TEXT_BINDING_EVIDENCE_RIV` is set. It verifies that an explicitly
//! empty ViewModel string remains empty through encode, import, binding,
//! shaping, and drawing without synthesizing placeholder glyphs.

use anyhow::{Context, Result};
use nuxie::{
    File, ImportResult, PersistentFactory, RecordingFactory, RuntimeFactoryHandle,
    ViewModelInstanceRuntime,
    runtime::{assets::font_asset::FontAsset, text::text_value_run::TextValueRun},
};
use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, read_runtime_file};

fn fixture_font_bytes() -> Vec<u8> {
    include_bytes!("../../../fixtures/fonts/roboto-a.ttf").to_vec()
}

fn fixture_record(type_name: &str, properties: Vec<(&str, FixtureValue)>) -> FixtureRecord {
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
            FixtureProperty {
                key: property.key.int,
                value,
            }
        })
        .collect();
    FixtureRecord {
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

fn encode_fixture_records(records: &[FixtureRecord]) -> Vec<u8> {
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
                FixtureValue::Bool(value) => bytes.push(u8::from(*value)),
                FixtureValue::Bytes(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value);
                }
                FixtureValue::Color(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                FixtureValue::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                FixtureValue::Int(value) => {
                    let encoded = ((*value as u32) << 1) ^ ((*value >> 31) as u32);
                    push_var_uint(&mut bytes, u64::from(encoded));
                }
                FixtureValue::String(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value.as_bytes());
                }
                FixtureValue::Uint(value) => push_var_uint(&mut bytes, *value),
            }
        }
        push_var_uint(&mut bytes, 0);
    }
    bytes
}

fn empty_source_first_records() -> Vec<FixtureRecord> {
    vec![
        fixture_record("Backboard", vec![]),
        fixture_record(
            "FontAsset",
            vec![
                ("name", FixtureValue::String("Evidence font".into())),
                ("assetId", FixtureValue::Uint(0)),
            ],
        ),
        fixture_record(
            "FileAssetContents",
            vec![("bytes", FixtureValue::Bytes(fixture_font_bytes()))],
        ),
        fixture_record(
            "ViewModel",
            vec![("name", FixtureValue::String("Evidence model".into()))],
        ),
        fixture_record(
            "ViewModelPropertyString",
            vec![("name", FixtureValue::String("value".into()))],
        ),
        fixture_record(
            "ViewModelInstance",
            vec![
                ("name", FixtureValue::String("Evidence defaults".into())),
                ("viewModelId", FixtureValue::Uint(0)),
            ],
        ),
        fixture_record(
            "ViewModelInstanceString",
            vec![
                ("viewModelPropertyId", FixtureValue::Uint(0)),
                ("propertyValue", FixtureValue::String(String::new())),
            ],
        ),
        fixture_record(
            "Artboard",
            vec![
                ("name", FixtureValue::String("Empty 1".into())),
                ("width", FixtureValue::Double(120.0)),
                ("height", FixtureValue::Double(40.0)),
                ("viewModelId", FixtureValue::Uint(0)),
            ],
        ),
        fixture_record(
            "Text",
            vec![
                ("name", FixtureValue::String("Empty projection".into())),
                ("x", FixtureValue::Double(0.0)),
                ("y", FixtureValue::Double(0.0)),
                ("alignValue", FixtureValue::Uint(0)),
                ("sizingValue", FixtureValue::Uint(2)),
                ("width", FixtureValue::Double(120.0)),
                ("height", FixtureValue::Double(40.0)),
                ("overflowValue", FixtureValue::Uint(0)),
                ("wrapValue", FixtureValue::Uint(1)),
            ],
        ),
        fixture_record(
            "TextStylePaint",
            vec![
                ("name", FixtureValue::String("Evidence style".into())),
                ("parentId", FixtureValue::Uint(1)),
                ("fontSize", FixtureValue::Double(18.0)),
                ("fontAssetId", FixtureValue::Uint(0)),
                ("lineHeight", FixtureValue::Double(22.0)),
                ("letterSpacing", FixtureValue::Double(0.0)),
            ],
        ),
        fixture_record(
            "Fill",
            vec![
                ("name", FixtureValue::String("Evidence fill".into())),
                ("parentId", FixtureValue::Uint(2)),
                ("fillRule", FixtureValue::Uint(0)),
            ],
        ),
        fixture_record(
            "SolidColor",
            vec![
                ("name", FixtureValue::String("Evidence color".into())),
                ("parentId", FixtureValue::Uint(3)),
                ("colorValue", FixtureValue::Color(0xffab_cdef)),
            ],
        ),
        fixture_record(
            "TextValueRun",
            vec![
                ("name", FixtureValue::String("Evidence run".into())),
                ("parentId", FixtureValue::Uint(1)),
                ("text", FixtureValue::String(String::new())),
                ("styleId", FixtureValue::Uint(2)),
            ],
        ),
        fixture_record(
            "DataBindContext",
            vec![
                // TextValueRun.text.
                ("propertyKey", FixtureValue::Uint(268)),
                // TwoWay | SourceToTargetRunsFirst.
                ("flags", FixtureValue::Uint(10)),
                ("sourcePathIds", FixtureValue::Bytes(vec![0, 0])),
            ],
        ),
    ]
}

fn embedded_font_records(bytes: Vec<u8>) -> Vec<FixtureRecord> {
    vec![
        fixture_record("Backboard", vec![]),
        fixture_record(
            "FontAsset",
            vec![
                ("name", FixtureValue::String("Evidence font".into())),
                ("assetId", FixtureValue::Uint(0)),
            ],
        ),
        fixture_record(
            "FileAssetContents",
            vec![("bytes", FixtureValue::Bytes(bytes))],
        ),
    ]
}

#[test]
fn embedded_font_import_uses_the_decoded_owner_as_validation_authority() {
    let valid = encode_fixture_records(&embedded_font_records(fixture_font_bytes()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    assert!(File::import(&valid, retained, None, None, None).is_some());

    let invalid = encode_fixture_records(&embedded_font_records(b"not a font".to_vec()));
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let invalid = File::import(&invalid, retained, Some(&mut result), None, None)
        .expect("pinned importer permits unresolved assets");
    assert_eq!(result, ImportResult::Success);
    let invalid_font = invalid
        .with_file(|file| file.asset(0))
        .expect("invalid FontAsset owner remains observable");
    assert!(
        invalid_font
            .with_downcast::<FontAsset, _>(FontAsset::font)
            .flatten()
            .is_none(),
        "invalid embedded bytes must not produce a live Font"
    );
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
        property.key == text_key && property.value == FixtureValue::String(String::new())
    }));

    let bytes = encode_fixture_records(&records);
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

    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).context("retained factory")?;
    let file = File::import(&bytes, retained, None, None, None).context("fixture imports")?;
    let instance = file
        .with_file(File::artboard)
        .context("authored artboard")?;
    let instance = nuxie::Artboard::instance_from_handle(&instance).context("artboard instance")?;
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(instance.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(instance.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .context("imported default ViewModel instance")?;
    assert_eq!(
        view_model
            .property_string("value")
            .context("value property")?
            .value(),
        "",
        "the imported ViewModel source must retain explicit empty"
    );
    instance.bind_view_model_instance(Some(view_model.instance()));
    assert!(instance.data_context().is_some());
    instance.advance_default(0.0);

    let run = instance
        .with_artboard(|artboard| artboard.find_handle::<TextValueRun>("Evidence run"))
        .context("live TextValueRun")?;
    assert_eq!(
        run.with_downcast::<TextValueRun, _>(|run| run.text().to_owned())
            .as_deref(),
        Some("")
    );
    assert_eq!(
        run.with_downcast_mut::<TextValueRun, _>(TextValueRun::length),
        Some(0)
    );

    let mut renderer = factory.borrow().make_renderer();
    instance.draw(&mut renderer);
    let draw = factory.borrow().canonical_recording();
    assert!(
        !draw.stream().contains("drawPath "),
        "an explicitly empty source-first text run emitted glyph paths:\n{}",
        draw.stream()
    );
    Ok(())
}
