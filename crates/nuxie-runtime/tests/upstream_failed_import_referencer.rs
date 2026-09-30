//! Both bb2903fa failed_import_referencer_test.cpp streams and assertions.
//! These are behavioral load-success tests, not a claim of ASan execution.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::generated::{
    backboard_base::BackboardBase,
    data_bind::converters::{
        data_converter_group_item_base::DataConverterGroupItemBase,
        data_converter_rounder_base::DataConverterRounderBase,
    },
    nested_artboard_base::NestedArtboardBase,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
use riv_bytes::{RivBytes, write_artboard_object};

fn assert_remaining_artboard_loads(riv: RivBytes) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut result = ImportResult::Malformed;
    let file = File::import(&riv.bytes(), factory, Some(&mut result), None, None);
    assert_eq!(result, ImportResult::Success);
    let file = file.expect("file imports despite failed object");
    let artboard = file
        .with_file(File::artboard)
        .expect("following artboard imports");
    assert_eq!(
        artboard
            .with_downcast::<Artboard, _>(|a| a.name().to_owned())
            .unwrap(),
        "A"
    );
}

#[test]
fn a_nested_artboard_that_fails_to_import_does_not_outlive_itself() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(NestedArtboardBase::TYPE_KEY);
    riv.prop_uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 0);
    riv.end();
    write_artboard_object(&mut riv);
    assert_remaining_artboard_loads(riv);
}

#[test]
fn a_converter_group_item_that_fails_to_import_does_not_outlive_itself() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(DataConverterRounderBase::TYPE_KEY);
    riv.end();
    riv.object(DataConverterGroupItemBase::TYPE_KEY);
    riv.prop_uint(DataConverterGroupItemBase::CONVERTER_ID_PROPERTY_KEY, 0);
    riv.end();
    write_artboard_object(&mut riv);
    assert_remaining_artboard_loads(riv);
}
