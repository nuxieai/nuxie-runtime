//! One case per pinned `listener_align_target_test.cpp` owner flow.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::StateMachineInstance, math::vec2d::Vec2D,
    shapes::shape::Shape,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

fn run_case(artboard_name: &str, expected_y: f32) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let path = root.join("tests/unit_tests/assets/align_target.riv");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("align_target.riv imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let source = file
        .with_file(|file| file.artboard_named_source(artboard_name))
        .unwrap_or_else(|| panic!("missing artboard {artboard_name}"));
    let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.state_machine_count()),
        1
    );
    let definition = source
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.state_machine_named("align-state-machine")
        })
        .flatten()
        .expect("align-state-machine definition");
    let state_machine = StateMachineInstance::new(definition, artboard.downgrade());
    assert_eq!(
        state_machine.with_instance(StateMachineInstance::name),
        "align-state-machine"
    );

    artboard.advance_default(0.0);
    state_machine.advance_and_apply(0.0);
    state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    let circle = artboard
        .with_artboard(|artboard| artboard.find_handle::<Shape>("circle"))
        .expect("circle Shape");
    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(100.0, 50.0), 0.0, 0);
        machine.pointer_move(Vec2D::new(100.0, 51.0), 0.0, 0);
    });
    state_machine.advance_and_apply(1.0);
    state_machine.with_instance_mut(|machine| machine.advance_seconds(0.0));

    circle
        .with_downcast::<Shape, _>(|circle| {
            // The pinned test checks authored x/y, not only the world transform.
            assert_eq!(circle.base.x(), 100.0);
            assert_eq!(circle.base.y(), expected_y);
            // Keep the prior Rust port's additional world-space assertions too.
            assert_eq!(circle.world_transform()[4], 100.0);
            assert_eq!(circle.world_transform()[5], expected_y);
        })
        .expect("live circle Shape");
}

#[test]
fn wave_c2_listener_align_001_preserve_offset_off() {
    run_case("preserve-inactive", 51.0);
}

#[test]
fn wave_c2_listener_align_002_preserve_offset_on() {
    run_case("preserve-active", 101.0);
}
