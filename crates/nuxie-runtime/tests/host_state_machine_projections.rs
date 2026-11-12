use std::path::{Path, PathBuf};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    ArtboardInstance, File, RuntimeFactoryHandle, RuntimeGeometryHitOccurrence,
    RuntimeGeometryHitPathSegment, StateMachineEventContext,
};

fn fixture_path(name: &str) -> PathBuf {
    let runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime_dir)
        .join("tests/unit_tests/assets")
        .join(name)
}

fn import_host_artboard(fixture: &str) -> (PersistentFactory<RecordingFactory>, ArtboardInstance) {
    let bytes = std::fs::read(fixture_path(fixture)).expect("pinned fixture bytes");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let artboard = ArtboardInstance::from_native(file, 0).expect("default artboard instance");
    (factory, artboard)
}

#[test]
fn current_animation_is_an_owned_observation_of_the_translated_layer() {
    let (_factory, mut artboard) = import_host_artboard("rocket.riv");
    let mut machine = artboard
        .state_machine_instance_named("Button")
        .expect("Button machine");

    machine.advance_and_apply(0.0);

    assert_eq!(machine.current_animation_count(), 1);
    let animation = machine.current_animation(0).expect("current animation");
    assert_eq!(
        animation.name(),
        artboard.animation_name_at(animation.animation_index())
    );
    assert!(animation.time().is_finite());
    assert!(machine.current_animation(1).is_none());
}

#[test]
fn pointer_reports_drain_once_without_consuming_the_translated_event_queue() {
    let (_factory, mut artboard) = import_host_artboard("event_on_listener.riv");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("event state machine");
    machine.advance_and_apply(0.0);
    assert_eq!(machine.reported_event_count(), 0);

    let context = StateMachineEventContext::new(
        vec![RuntimeGeometryHitPathSegment {
            artboard_global_id: 7,
            local_id: 11,
        }],
        vec![RuntimeGeometryHitOccurrence {
            artboard_global_id: 7,
            host_local_id: 5,
            item_index: 2,
            occurrence_identity: 41,
        }],
    );
    let down = machine.pointer_down_with_event_context(343.0, 116.0, 17, &context);
    let _up = machine.pointer_up_with_event_context(343.0, 116.0, 17, &context);
    assert!(down.is_hit());
    assert_eq!(machine.reported_event_count(), 2);

    let reports = machine.take_reported_events();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].name(), Some("Footstep"));
    assert_eq!(reports[1].name(), Some("Event 3"));
    assert!(
        reports
            .iter()
            .all(|report| report.context() == Some(&context))
    );
    assert!(machine.take_reported_events().is_empty());

    // Host draining is an observation cursor. The translated queue remains
    // intact until the next new-frame advance applies it.
    assert_eq!(machine.reported_event_count(), 2);
    machine.advance(0.0, true);
    assert_eq!(machine.reported_event_count(), 0);
    assert!(machine.take_reported_events().is_empty());
}

#[test]
fn gamepad_batches_delegate_to_the_translated_machine_owner() {
    let (_factory, mut artboard) = import_host_artboard("rocket.riv");
    let mut machine = artboard
        .state_machine_instance_named("Button")
        .expect("Button machine");

    assert!(!machine.submit_gamepads_from_buffer(&[]));
    assert!(
        machine
            .submit_gamepads_from_buffer(&nuxie_runtime::GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes())
    );
}

/// A machine the runtime already runs, here a component list row's, wraps as
/// a host machine on that same instance: what the wrapper writes, the row's
/// running machine reads.
#[test]
fn a_running_list_row_machine_wraps_as_that_instance() {
    use nuxie_runtime::StateMachineInstance;
    use nuxie_runtime::source::artboard_component_list::ArtboardComponentList;

    let bytes = std::fs::read(fixture_path("component_list_default_state_machine.riv"))
        .expect("pinned fixture bytes");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let mut main = (0..)
        .map_while(|index| ArtboardInstance::from_native(file.clone(), index).ok())
        .find(|artboard| artboard.name() == "Main")
        .expect("Main artboard");
    let native = main.native_handle();
    let vm = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(native.core_handle())
        })
        .expect("default view model");
    native.bind_view_model_instance(Some(vm.clone()));
    let mut machine = main.state_machine_instance(0).expect("Main machine");
    machine
        .native_handle()
        .with_instance_mut(|machine| machine.bind_view_model_instance(vm.clone()));
    machine.advance_and_apply(0.016);

    let list = native
        .with_artboard(|board| board.find_handle::<ArtboardComponentList>("List"))
        .expect("List");
    let (row, running) = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            Some((list.artboard_instance(0)?, list.state_machine_instance(0)?))
        })
        .flatten()
        .expect("the list's first row runs a machine");
    let definition = running.with_instance(|machine| machine.state_machine());
    let index = (0..row.with_artboard(|row| row.state_machine_count()))
        .find(|&index| {
            row.with_artboard(|row| row.state_machine_handle_at(index)) == Some(definition.clone())
        })
        .expect("the row's artboard holds the running machine");

    let mut wrapped =
        StateMachineInstance::from_native_handle(file.clone(), row, index, running.clone());
    assert_eq!(wrapped.name(), "Default");
    let level = wrapped.input_index_named("level").expect("level input");
    assert!(wrapped.set_number(level, 9.0));
    assert_eq!(
        running.with_instance(|machine| machine.get_number("level").unwrap().value()),
        9.0
    );
    assert!(
        wrapped
            .native_handle()
            .downgrade()
            .ptr_eq(&running.downgrade())
    );
}
