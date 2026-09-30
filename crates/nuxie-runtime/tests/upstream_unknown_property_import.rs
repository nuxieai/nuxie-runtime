//! tests/unit_tests/runtime/unknown_property_import_test.cpp at edddc609.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        artboard_base::ArtboardBase, backboard_base::BackboardBase, component_base::ComponentBase,
        layout_component_base::LayoutComponentBase, semantic::semantic_data_base::SemanticDataBase,
    },
    semantic::semantic_data::SemanticData,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
use riv_bytes::{RivBytes, write_artboard};

fn import(
    bytes: &[u8],
    result: &mut ImportResult,
) -> Option<nuxie_runtime::source::file::RuntimeFileHandle> {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(bytes, factory, Some(result), None, None)
}

#[test]
fn an_unknown_bool_property_is_skipped_not_misread_as_a_key() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    riv.object(SemanticDataBase::TYPE_KEY);
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
    riv.prop_bool(SemanticDataBase::IS_HIDDEN_PROPERTY_KEY, true);
    riv.prop_string(SemanticDataBase::LABEL_PROPERTY_KEY, "after");
    riv.end();
    let bytes = riv.bytes();
    let mut result = ImportResult::Malformed;
    let file = import(&bytes, &mut result).expect("file imports");
    assert_eq!(result, ImportResult::Success);
    let artboard = file.with_file(File::artboard).unwrap();
    let semantic = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.objects().iter().flatten().find_map(|object| {
                object.with_downcast::<SemanticData, _>(|semantic| {
                    (
                        semantic.base.label().to_owned(),
                        semantic.base.state_flags(),
                    )
                })
            })
        })
        .flatten()
        .expect("semantic data");
    assert_eq!(semantic, ("after".to_owned(), 0));
    // The live binary decoder follows the same rejected-property path.
    nuxie_binary::read_runtime_file(&bytes).expect("binary importer skips bool");
}

#[test]
fn a_property_missing_from_the_toc_fails_the_import() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(ArtboardBase::TYPE_KEY);
    riv.prop_string(ComponentBase::NAME_PROPERTY_KEY, "A");
    riv.prop_uint_outside_toc(65000, 1);
    riv.prop_float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0);
    riv.end();
    let bytes = riv.bytes();
    let mut result = ImportResult::Success;
    assert!(import(&bytes, &mut result).is_none());
    assert_eq!(result, ImportResult::Malformed);
    assert!(nuxie_binary::read_runtime_file(&bytes).is_err());
}

#[cfg(feature = "tools")]
#[test]
fn stripping_assets_from_a_file_with_an_untyped_property_fails() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(ArtboardBase::TYPE_KEY);
    riv.prop_string(ComponentBase::NAME_PROPERTY_KEY, "A");
    riv.prop_uint_outside_toc(65000, 1);
    riv.end();
    let mut result = ImportResult::Success;
    let stripped = File::strip_assets(&riv.bytes(), &Default::default(), Some(&mut result));
    assert_eq!(result, ImportResult::Malformed);
    assert!(stripped.is_empty());
}

#[test]
fn an_unknown_object_type_is_skipped_and_the_file_still_loads() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    riv.object(65001);
    riv.prop_uint(65002, 7);
    riv.end();
    let bytes = riv.bytes();
    let mut result = ImportResult::Malformed;
    let file = import(&bytes, &mut result).expect("forward compatible import");
    assert_eq!(result, ImportResult::Success);
    assert_eq!(file.with_file(|file| file.artboard_name_at(0)), "A");
    nuxie_binary::read_runtime_file(&bytes).expect("unknown object remains skippable");
}
