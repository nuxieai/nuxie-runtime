//! Upstream 86ea900c state_machine_event_test.cpp lifetime regressions, plus
//! the duplicate-link and copy/assignment contracts in nested_animation.hpp.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        nested_animation::NestedEventNotifier,
        nested_state_machine::NestedStateMachine,
        state_machine_instance::{
            RuntimeStateMachineInstanceHandle, RuntimeStateMachineInstanceWeakHandle,
        },
    },
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    pointer_button::PointerButton,
};
use nuxie_runtime::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};

fn scene() -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/nested_event_test.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    (file, artboard)
}
fn listeners(
    machine: &RuntimeStateMachineInstanceHandle,
) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
    machine.with_instance(|machine| machine.nested_event_listeners())
}
fn assert_only_listener(
    listeners: &[RuntimeStateMachineInstanceWeakHandle],
    expected: &RuntimeStateMachineInstanceHandle,
) {
    assert_eq!(listeners.len(), 1);
    assert!(listeners[0].ptr_eq(&expected.downgrade()));
}
fn notifier_count(listener: &RuntimeStateMachineInstanceHandle) -> usize {
    listener.with_instance(|listener| listener.nested_event_notifier_count())
}

#[test]
fn a_deleted_state_machine_stops_listening_to_nested_artboard_events() {
    let (_file, artboard) = scene();
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    assert_eq!(nested.len(), 1);
    let animation = nested[0]
        .with_downcast::<NestedArtboard, _>(|nested| nested.nested_animations()[0].clone())
        .unwrap();
    let nested_machine = animation
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
        .unwrap();

    let first = artboard.state_machine_at(0).unwrap();
    assert_only_listener(&listeners(&nested_machine), &first);
    // Do not call dispose, notify, or a pruning accessor before this check.
    drop(first);
    assert!(listeners(&nested_machine).is_empty());

    let second = artboard.state_machine_at(0).unwrap();
    assert_only_listener(&listeners(&nested_machine), &second);
    assert!(second.with_instance(|machine| machine.get_bool("Boolean 1").is_some()));
    artboard.advance_default(0.0);
    second.with_instance_mut(|machine| machine.advance_seconds(0.0));
    second.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(250.0, 100.0), 0, PointerButton::Primary)
    });
    assert_eq!(
        nested_machine.with_instance(|machine| machine.reported_event_count()),
        1
    );
    artboard.advance_default(0.0);
    assert!(second.with_instance(|machine| machine.get_bool("Boolean 1").unwrap().value()));
}

#[test]
fn a_nested_event_notifier_deleted_first_unlinks_from_its_listener() {
    let (_file, artboard) = scene();
    // RuntimeStateMachineInstance is the existing Rust concrete callback
    // boundary for NestedEventListener; use it for upstream's no-op Listener.
    let listener = artboard.state_machine_at(0).unwrap();
    let initial_links = notifier_count(&listener);
    let mut notifier = NestedEventNotifier::default();
    notifier.add_nested_event_listener(listener.downgrade());
    assert_eq!(notifier_count(&listener), initial_links + 1);
    drop(notifier);
    assert_eq!(notifier_count(&listener), initial_links);
    drop(listener);
}

#[test]
fn duplicate_registrations_are_retained_and_removal_erases_every_link() {
    let (_file, artboard) = scene();
    let listener = artboard.state_machine_at(0).unwrap();
    let initial_links = notifier_count(&listener);
    let mut notifier = NestedEventNotifier::default();
    notifier.add_nested_event_listener(listener.downgrade());
    notifier.add_nested_event_listener(listener.downgrade());
    let entries = notifier.nested_event_listeners();
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|entry| entry.ptr_eq(&listener.downgrade()))
    );
    assert_eq!(notifier_count(&listener), initial_links + 2);
    notifier.remove_nested_event_listener(listener.downgrade());
    assert!(notifier.nested_event_listeners().is_empty());
    assert_eq!(notifier_count(&listener), initial_links);

    notifier.add_nested_event_listener(listener.downgrade());
    notifier.add_nested_event_listener(listener.downgrade());
    drop(notifier);
    assert_eq!(notifier_count(&listener), initial_links);
}

#[test]
fn notifier_copy_starts_empty_and_assignment_preserves_destination_links() {
    let (_file, artboard) = scene();
    let source_listener = artboard.state_machine_at(0).unwrap();
    let destination_listener = artboard.state_machine_at(0).unwrap();
    let source_baseline = notifier_count(&source_listener);
    let destination_baseline = notifier_count(&destination_listener);
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    assert_eq!(nested.len(), 1);
    let mut source = NestedEventNotifier::default();
    source.set_nested_artboard(nested[0].clone());
    source.add_nested_event_listener(source_listener.downgrade());
    let copy = source.clone();
    assert!(copy.nested_event_listeners().is_empty());
    assert!(copy.nested_artboard().is_none());
    assert_only_listener(&source.nested_event_listeners(), &source_listener);
    assert_eq!(notifier_count(&source_listener), source_baseline + 1);

    let mut destination = NestedEventNotifier::default();
    destination.add_nested_event_listener(destination_listener.downgrade());
    destination.clone_from(&source);
    assert!(destination.nested_artboard().is_none());
    assert_only_listener(&destination.nested_event_listeners(), &destination_listener);
    assert_only_listener(&source.nested_event_listeners(), &source_listener);
    assert_eq!(
        notifier_count(&destination_listener),
        destination_baseline + 1
    );
    drop(destination);
    assert_eq!(notifier_count(&destination_listener), destination_baseline);
    assert_eq!(notifier_count(&source_listener), source_baseline + 1);
    drop(source);
    assert_eq!(notifier_count(&source_listener), source_baseline);
}

#[test]
fn dispose_unlinks_recorded_notifiers_without_reading_a_dead_artboard() {
    let (_file, artboard) = scene();
    let listener = artboard.state_machine_at(0).unwrap();
    let mut notifier = NestedEventNotifier::default();
    notifier.add_nested_event_listener(listener.downgrade());
    drop(artboard);
    assert!(listener.with_instance(|listener| listener.artboard().upgrade().is_none()));
    listener.with_instance_mut(|listener| listener.dispose());
    assert!(notifier.nested_event_listeners().is_empty());
    assert_eq!(notifier_count(&listener), 0);
}
