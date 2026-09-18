use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle, collection_semantics::SemanticCollectionData};

fn record(name: &str, values: &[(&str, FixtureValue)]) -> FixtureRecord {
    let definition = nuxie_schema::definition_by_name(name).unwrap();
    FixtureRecord {
        type_key: definition.type_key.int,
        properties: values
            .iter()
            .map(|(name, value)| {
                let key = std::iter::once(definition.name)
                    .chain(definition.ancestors.iter().copied())
                    .filter_map(nuxie_schema::definition_by_name)
                    .flat_map(|d| d.properties)
                    .find(|p| p.name == *name)
                    .unwrap()
                    .key
                    .int;
                FixtureProperty {
                    key,
                    value: value.clone(),
                }
            })
            .collect(),
    }
}

fn scene(role: u64, count: Option<u64>, position: Option<u64>) -> Vec<u8> {
    let mut properties = vec![
        ("parentId", FixtureValue::Uint(0)),
        ("role", FixtureValue::Uint(role)),
        ("label", FixtureValue::String("Plans".into())),
    ];
    if let Some(value) = count {
        properties.push(("itemCount", FixtureValue::Uint(value)));
    }
    if let Some(value) = position {
        properties.push(("itemPosition", FixtureValue::Uint(value)));
    }
    let records = vec![
        record("Backboard", &[]),
        record(
            "Artboard",
            &[
                ("width", FixtureValue::Double(200.0)),
                ("height", FixtureValue::Double(200.0)),
            ],
        ),
        record("SemanticCollectionData", &properties),
    ];
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

#[test]
fn collection_metadata_survives_scene_import_and_artboard_instantiation() {
    for count in [None, Some(0), Some(10)] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &scene(10, count, None),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|file| file.artboard_default()).unwrap();
        artboard.update_pass(true);
        let object = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(SemanticCollectionData::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        object
            .with_downcast::<SemanticCollectionData, _>(|data| {
                assert_eq!(data.item_count(), count.map(|v| v as u32));
                assert_eq!(data.item_position(), None);
            })
            .unwrap();
        object
            .with_semantic_data(|data| {
                assert_eq!(data.base.role(), 10);
                assert_eq!(data.base.label(), "Plans");
            })
            .unwrap();
    }
}

#[test]
fn scene_import_rejects_collection_properties_on_incompatible_roles() {
    for (role, count, position) in [(1, None, None), (10, None, Some(0)), (11, Some(1), None)] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        assert!(
            File::import(
                &scene(role, count, position),
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None
            )
            .is_none()
        );
    }
}
