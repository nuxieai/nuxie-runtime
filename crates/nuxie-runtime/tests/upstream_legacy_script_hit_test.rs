//! Both hittest_test.cpp additions at upstream e54bc92a.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle, RuntimeFileHandle,
    source::{
        artboard::RuntimeArtboardInstanceHandle, assets::script_asset::ScriptAsset,
        core::CoreHandle, lua::scripting_vm::RuntimeScriptingVmHandle, math::vec2d::Vec2D,
    },
};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

fn load() -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .expect("RIVE_RUNTIME_DIR must name the pinned upstream checkout");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/legacy_script_hit_test.riv"),
    )
    .expect("pinned legacy_script_hit_test.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let vm = RuntimeScriptingVmHandle::new(Box::new(
        ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default()).unwrap(),
    ));
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        Some(vm),
    )
    .expect("legacy_script_hit_test imports")
}

fn first_scripted_drawable(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard.with_artboard(|artboard| {
        artboard
            .objects()
            .iter()
            .flatten()
            .find(|object| {
                object.with(|object| object.as_scripted_drawable().is_some()) == Some(true)
            })
            .cloned()
            .expect("first scripted drawable")
    })
}

#[test]
fn legacy_script_without_pointer_handlers_does_not_take_pointer_hits() {
    let file = load();
    let artboard = file
        .with_file(|file| file.artboard_named("NoPointerHandlers"))
        .unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.advance_and_apply(0.0);
    assert!(machine.with_instance(|machine| machine.hit_test(Vec2D::new(200.0, 200.0))));
    assert!(!machine.with_instance(|machine| machine.hit_test(Vec2D::new(20.0, 20.0))));
    assert!(!machine.with_instance(|machine| machine.hit_test(Vec2D::new(380.0, 380.0))));

    let drawable = first_scripted_drawable(&artboard);
    let asset = drawable
        .with(|object| {
            let drawable = object.as_scripted_drawable().unwrap();
            assert!(!drawable.wants_any_pointer());
            drawable.scripted.script_asset().expect("script asset")
        })
        .unwrap();
    asset
        .with_downcast::<ScriptAsset, _>(|asset| {
            assert!(!asset.optional_methods().wants_any_pointer());
            assert!(!asset.optional_methods().wants_pointer_down());
        })
        .unwrap();

    // The second occurrence reuses the corrected asset bits.
    let second = file
        .with_file(|file| file.artboard_named("NoPointerHandlers"))
        .unwrap();
    let second_machine = second.state_machine_instance_handle(0).unwrap();
    second_machine.advance_and_apply(0.0);
    assert!(second_machine.with_instance(|machine| machine.hit_test(Vec2D::new(200.0, 200.0))));
    assert!(!second_machine.with_instance(|machine| machine.hit_test(Vec2D::new(20.0, 20.0))));
}

#[test]
fn legacy_script_with_a_pointer_handler_takes_pointer_hits_everywhere() {
    let file = load();
    let artboard = file
        .with_file(|file| file.artboard_named("PointerHandler"))
        .unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.advance_and_apply(0.0);
    assert!(machine.with_instance(|machine| machine.hit_test(Vec2D::new(200.0, 200.0))));
    assert!(machine.with_instance(|machine| machine.hit_test(Vec2D::new(20.0, 20.0))));
    first_scripted_drawable(&artboard)
        .with(|object| {
            let drawable = object.as_scripted_drawable().unwrap();
            assert!(drawable.wants_any_pointer());
        })
        .unwrap();

    let second = file
        .with_file(|file| file.artboard_named("PointerHandler"))
        .unwrap();
    let second_machine = second.state_machine_instance_handle(0).unwrap();
    second_machine.advance_and_apply(0.0);
    assert!(second_machine.with_instance(|machine| machine.hit_test(Vec2D::new(20.0, 20.0))));
}
