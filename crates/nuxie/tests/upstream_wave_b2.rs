//! Executable owner-flow ports needed to close Wave B2 of the pinned runtime
//! unit-test denominator.

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
fn wave_b2_draw_rules_load_and_sort_correctly() {
    let file = File::import(&pinned_fixture("draw_rule_cycle.riv"))
        .expect("draw-rule cycle fixture imports");
    let artboard = file.default_artboard().expect("default artboard");
    let blue = artboard.graph().component_named("Blue").expect("Blue node");
    assert_eq!(blue.type_name, "Shape");

    let mut instance = artboard.instantiate().expect("artboard instance");
    instance.advance(0.0);
    assert_eq!(artboard.animation_count(), 1);
    let mut animation = instance
        .linear_animation_instance(0)
        .expect("ping-pong animation");
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();

    for _ in 0..10 {
        instance
            .raw()
            .advance_linear_animation_instance(&mut animation, 1.0);
        instance
            .raw_mut()
            .apply_linear_animation_instance(&animation, 1.0);
        instance
            .draw(&mut factory, &mut renderer)
            .expect("draw sorted artboard");
    }
}
