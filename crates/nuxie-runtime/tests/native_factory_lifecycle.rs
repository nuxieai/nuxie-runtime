//! Native lifecycle smoke test grounded in pinned
//! `tests/unit_tests/runtime/file_test.cpp` ("dependencies are as expected" and
//! "artboards can be counted and accessed via index or name"). The fixture and
//! exact world-position expectations are upstream's; drawing additionally
//! exercises the retained factory through the new native instance boundary.
//! The instance flag assertion is also used by pinned `instancing_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::shapes::shape::Shape;
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

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
fn native_import_instance_advance_and_draw_preserves_upstream_dependencies() {
    let bytes = pinned_fixture("dependency_test.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut renderer = factory.borrow().make_renderer();
    let retained_factory = RuntimeFactoryHandle::from_factory(&mut factory)
        .expect("explicit PersistentFactory supplies retained ownership");
    drop(factory);

    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained_factory, Some(&mut result), None, None)
        .expect("upstream dependency_test.riv imports");
    assert_eq!(result, ImportResult::Success);
    file.with_file(|file| {
        assert_eq!(file.artboard_count(), 1);
        assert_eq!(file.artboard_name_at(0), "Blue");
        assert!(file.artboard_named_source("Blue").is_some());
    });

    let instance = file
        .with_file(File::artboard_default)
        .expect("default instance");
    let rectangle = instance
        .with_artboard(|instance| instance.find_handle::<Shape>("Rectangle"))
        .expect("upstream Rectangle shape");
    instance.advance_default(0.0);
    let world = rectangle
        .with_downcast::<Shape, _>(|shape| *shape.shape_world_transform())
        .expect("live Rectangle shape");
    assert_eq!(world[4], 39.203125_f32);
    assert_eq!(world[5], 29.535156_f32);

    // No factory is supplied at draw time, and the original factory variable
    // was dropped before import. All allocations use the retained occurrence.
    instance.draw(&mut renderer);
    assert_eq!(
        instance
            .core_handle()
            .with_downcast::<Artboard, _>(Artboard::is_instance),
        Some(true),
    );
}
