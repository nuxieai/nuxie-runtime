use nuxie_render_api::{PersistentFactory, RecordingFactory};
use std::sync::Arc;

use nuxie_runtime::{
    File, RuntimeBlobAsset, RuntimeFactoryHandle, RuntimeFileHandle, ScriptViewModelProperty,
    script_view_models,
};

use nuxie_runtime::source::viewmodel::runtime::viewmodel_instance_value_runtime::DataType;

fn fixture_file() -> RuntimeFileHandle {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/data_bind_blob_test.riv");
    let bytes = std::fs::read(path).expect("vendored S4-42 blob fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("blob fixture imports")
}

#[test]
fn blob_property_imports_and_preserves_live_empty_values() {
    let file = fixture_file();
    let model = script_view_models(&file)
        .into_values()
        .find_map(|definition| {
            definition
                .properties()
                .iter()
                .find(|(_, kind)| **kind == ScriptViewModelProperty::Blob)?;
            definition.named_instance(Some("Instance"))
        })
        .expect("fixture exposes a blob view model");
    let property = model
        .properties()
        .iter()
        .find_map(|(name, kind)| (*kind == ScriptViewModelProperty::Blob).then(|| name.clone()))
        .expect("blob property name");

    let imported = model
        .blob_asset(&property)
        .expect("authored id-bound blob resolves through the file asset catalog");
    let imported_again = model
        .blob_asset(&property)
        .expect("authored id-bound blob remains available");
    assert!(!imported.bytes().is_empty());
    assert!(Arc::ptr_eq(&imported, &imported_again));
    assert!(model.set_blob_asset(&property, Some(Arc::clone(&imported))));
    assert!(!model.set_blob_asset(&property, Some(imported_again)));

    let first: Arc<[u8]> = Arc::from(&b"first"[..]);
    assert!(model.set_blob(&property, Some(Arc::clone(&first))));
    assert_eq!(model.blob(&property).as_deref(), Some(&b"first"[..]));
    assert!(!model.set_blob(&property, Some(Arc::clone(&first))));

    let empty: Arc<[u8]> = Arc::from(&b""[..]);
    assert!(model.set_blob(&property, Some(Arc::clone(&empty))));
    assert_eq!(model.blob(&property).as_deref(), Some(&b""[..]));
    assert!(model.set_blob(&property, None));
    assert!(model.blob(&property).is_none());
}

#[test]
fn blob_runtime_wrapper_stores_swaps_and_clears_live_values() {
    let file = fixture_file();
    let (runtime, property_name) = file
        .with_file(|file| {
            (0..file.view_model_count()).find_map(|index| {
                let runtime = file.view_model_by_index(index)?;
                let name = runtime
                    .properties()
                    .into_iter()
                    .find(|property| property.data_type == DataType::AssetBlob)?
                    .name;
                Some((runtime, name))
            })
        })
        .expect("fixture exposes a blob view-model property");
    assert!(runtime.properties().iter().any(|property| {
        property.name == property_name && property.data_type == DataType::AssetBlob
    }));
    let instance = runtime.create_instance();
    let property = instance
        .property_blob(&property_name)
        .expect("typed blob runtime property");
    let value = || {
        property
            .value_runtime()
            .handle()
            .with(|owner| owner.as_view_model_instance_asset_blob().unwrap().asset())
            .flatten()
    };
    let write = |asset| {
        property.value_runtime().clear_changes();
        property.set_value(asset);
        property.value_runtime().has_changed()
    };

    // The pinned typed runtime setter returns void. Its real retained value
    // delegate preserves the old test's changed/no-op observations.
    assert!(value().is_none());
    let first = Arc::new(RuntimeBlobAsset::new("first", Arc::from(&b"first"[..])));
    assert!(write(Some(Arc::clone(&first))));
    assert_eq!(value().unwrap().bytes(), b"first");
    assert!(!write(Some(Arc::clone(&first))));

    let second = Arc::new(RuntimeBlobAsset::new("second", Arc::from(&b"second"[..])));
    assert!(write(Some(Arc::clone(&second))));
    assert_eq!(value().unwrap().bytes(), b"second");

    let empty = Arc::new(RuntimeBlobAsset::new("empty", Arc::from(&b""[..])));
    assert!(write(Some(empty)));
    assert_eq!(value().unwrap().bytes(), b"");
    assert!(write(None));
    assert!(value().is_none());
}
