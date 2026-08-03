use std::{rc::Rc, sync::Arc};

use nuxie_runtime::{ScriptViewModelProperty, ViewModelRuntime, script_view_models};

fn fixture_file() -> nuxie_binary::RuntimeFile {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/data_bind_blob_test.riv");
    let bytes = std::fs::read(path).expect("vendored S4-42 blob fixture");
    nuxie_binary::read_runtime_file(&bytes).expect("blob fixture imports")
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
    let file = Rc::new(fixture_file());
    let (view_model_index, property_name) = file
        .view_models()
        .iter()
        .enumerate()
        .find_map(|(view_model_index, view_model)| {
            view_model.properties.iter().find_map(|property| {
                (property.type_name == "ViewModelPropertyAssetBlob").then(|| {
                    (
                        view_model_index,
                        property.string_property("name").unwrap().to_owned(),
                    )
                })
            })
        })
        .expect("fixture exposes a blob view-model property");
    let runtime = ViewModelRuntime::new(file, view_model_index).expect("blob view model runtime");
    let instance = runtime.create_instance().expect("blob instance");
    let property = instance
        .property_blob(&property_name)
        .expect("typed blob runtime property");

    assert!(property.value().is_none());
    let first: Arc<[u8]> = Arc::from(&b"first"[..]);
    assert!(property.set_value(Some(Arc::clone(&first))));
    assert_eq!(property.value().as_deref(), Some(&b"first"[..]));
    assert!(!property.set_value(Some(Arc::clone(&first))));

    let second: Arc<[u8]> = Arc::from(&b"second"[..]);
    assert!(property.set_value(Some(Arc::clone(&second))));
    assert_eq!(property.value().as_deref(), Some(&b"second"[..]));

    let empty: Arc<[u8]> = Arc::from(&b""[..]);
    assert!(property.set_value(Some(empty)));
    assert_eq!(property.value().as_deref(), Some(&b""[..]));
    assert!(property.set_value(None));
    assert!(property.value().is_none());
}
