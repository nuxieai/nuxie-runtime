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
