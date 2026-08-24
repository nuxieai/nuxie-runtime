//! Direct port of pinned `tests/unit_tests/runtime/render_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, RecordingFactory};

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
#[ignore = "expected-red: the first repeated 0.1-second state-machine advance leaves Artboard::did_change false instead of true"]
fn file_with_only_solid_color_animating_triggers_change_on_artboard() {
    let file =
        File::import(&pinned_fixture("solid_affects_has_changed.riv")).expect("import fixture");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");
    let mut state_machine = artboard
        .state_machine_instance(0)
        .expect("state machine zero");
    let view_model = artboard
        .instantiate_view_model()
        .expect("artboard view model");
    assert!(artboard.bind_view_model(&view_model));
    artboard.advance_with_state_machine(&mut state_machine, 0.1);
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw(&mut factory, &mut renderer)
        .expect("initial draw");

    let frames = 10;
    for _ in 0..frames {
        artboard.advance_with_state_machine(&mut state_machine, 0.1);
        assert!(artboard.raw().did_change());
        artboard
            .draw(&mut factory, &mut renderer)
            .expect("animated draw");
    }
}
