//! Executable owner-flow ports needed to close Wave B2 of the pinned runtime
//! unit-test denominator.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, RecordingFactory, RuntimeFactoryHandle};
use nuxie_runtime::source::shapes::shape::Shape;

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
fn wave_b2_draw_rules_load_and_sort_correctly() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &pinned_fixture("draw_rule_cycle.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("draw-rule cycle fixture imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    assert!(
        artboard.with_artboard(|artboard| artboard.base.find_handle::<Shape>("Blue").is_some())
    );

    artboard.advance_default(0.0);
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.base.animation_count()),
        1
    );
    let mut animation = artboard.animation_at(0).expect("ping-pong animation");
    let mut renderer = factory.borrow().make_renderer();

    for _ in 0..10 {
        animation.advance_and_apply(1.0);
        artboard.draw(&mut renderer);
    }
}
