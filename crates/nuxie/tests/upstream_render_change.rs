//! Direct port of pinned `tests/unit_tests/runtime/render_test.cpp`.

use std::path::PathBuf;

use nuxie::{
    File, PersistentFactory, RecordingFactory, RuntimeFactoryHandle, ViewModelInstanceRuntime,
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
fn file_with_only_solid_color_animating_triggers_change_on_artboard() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &pinned_fixture("solid_affects_has_changed.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine zero");
    let view_model = file
        .with_file_mut(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("artboard view model");
    state_machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    state_machine.advance_and_apply(0.1);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);

    let frames = 10;
    for _ in 0..frames {
        state_machine.advance_and_apply(0.1);
        assert!(artboard.with_artboard(|artboard| artboard.base.did_change()));
        artboard.draw(&mut renderer);
    }
}
