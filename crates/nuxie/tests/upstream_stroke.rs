//! Direct port of pinned `tests/unit_tests/runtime/stroke_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, RuntimeFactoryHandle};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    generated::shapes::paint::solid_color_base::SolidColorBase,
    shapes::paint::{solid_color::SolidColor, stroke::Stroke},
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn stroke_can_be_looked_up_at_runtime() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("stroke_name_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let stroke = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Stroke>("white_stroke"))
        .expect("named stroke");
    let paint = stroke
        .with_downcast::<Stroke, _>(|stroke| stroke.base.paint())
        .flatten()
        .expect("stroke paint");
    assert!(paint.is_type_of(SolidColorBase::TYPE_KEY));
    paint
        .with_downcast_mut::<SolidColor, _>(|paint| paint.set_color_value(0xff00_ffffu32 as i32))
        .expect("solid-color owner");
}
