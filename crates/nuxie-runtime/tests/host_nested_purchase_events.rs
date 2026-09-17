use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    ArtboardInstance, File, RuntimeFactoryHandle, RuntimeOwnedViewModelHandle,
    StateMachineReportedEvent,
};

fn purchase_scene(artboard_name: &str) -> (ArtboardInstance, RuntimeOwnedViewModelHandle) {
    let bytes = include_bytes!("fixtures/purchase-scopes/screen.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("factory");
    let file = File::import(bytes, retained, None, None, None).expect("purchase fixture imports");
    let index = file
        .with_file(|file| {
            (0..file.artboard_count()).find(|&index| file.artboard_name_at(index) == artboard_name)
        })
        .expect("authored artboard");
    let mut artboard = ArtboardInstance::from_native(file, index).expect("artboard");
    let file = artboard.native_file();
    let source = file
        .with_file(|file| file.artboard_at_source(index))
        .expect("source");
    let view_model = file
        .with_file_mut(|file| file.create_default_view_model_instance_for_artboard(source))
        .and_then(|native| RuntimeOwnedViewModelHandle::from_native(file.clone(), native))
        .expect("default view model");
    artboard.bind_owned_view_model_handle(view_model.clone());
    (artboard, view_model)
}

fn purchase_events(
    artboard_name: &str,
    machine_name: Option<&str>,
    taps: &[(f32, f32)],
    host_write: bool,
) -> Vec<StateMachineReportedEvent> {
    let (mut artboard, view_model) = purchase_scene(artboard_name);
    if host_write {
        let mut transaction = nuxie_runtime::RuntimeOwnedViewModelTransaction::begin().unwrap();
        assert!(
            transaction
                .try_set_number(&view_model, "fontScale", 2.0)
                .is_some()
        );
        transaction.commit();
    }
    let mut machine = match machine_name {
        Some(name) => artboard.state_machine_instance_named(name),
        None => artboard.state_machine_instance(0),
    }
    .expect("interaction machine");
    machine.bind_owned_view_model_handle(view_model.clone());
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.0, true)
        .expect("initial frame");
    for _ in 0..20 {
        machine.bind_owned_view_model_handle(view_model.clone());
        artboard
            .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
            .unwrap();
    }
    let mut all_events = Vec::new();
    for &(x, y) in taps {
        assert!(machine.pointer_down(x, y, 1).is_hit());
        machine.pointer_up(x, y, 1);
        let mut nested_reports = 0;
        artboard.native_handle().with_artboard(|root| {
        for handle in root.base.objects().iter().flatten() {
            handle.with(|object| {
                if let Some(nested) = object.as_nested_artboard() {
                    for animation in nested.nested_animations() {
                        animation.with_downcast::<nuxie_runtime::mechanical_port::source::animation::nested_state_machine::NestedStateMachine, _>(|animation| {
                            if let Some(machine) = animation.state_machine_instance() {
                                nested_reports += machine.with_instance(|machine| machine.reported_event_count());
                            }
                        });
                    }
                }
            });
        }
    });
        if artboard_name == "Purchase" {
            assert_eq!(
                nested_reports, 1,
                "the tapped child must queue exactly one event before host draining"
            );
        }
        let mut events = machine.take_reported_events();
        assert!(
            machine.take_reported_events().is_empty(),
            "repeat drain must not replay pointer events"
        );
        artboard
            .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
            .expect("pointer frame");
        events.extend(machine.take_reported_events());
        assert!(
            machine.take_reported_events().is_empty(),
            "repeat drain after advancing must stay empty"
        );
        assert_eq!(events.len(), 1, "one authored purchase event per tap");
        assert_eq!(events[0].name(), Some("Nuxie Interaction"));
        if artboard_name == "Purchase" {
            let property = if x < 160.0 { "first" } else { "second" };
            let expected = view_model
                .linked_view_model_by_property_name_path(property)
                .expect("published component reference")
                .instance_identity();
            assert_eq!(
                events[0]
                    .context()
                    .and_then(|context| context.view_model_instance_id()),
                Some(expected),
                "event identity must match the live view-model graph, not just differ between buttons"
            );
        }
        all_events.extend(events);
    }
    all_events
}

#[test]
fn direct_purchase_component_reports_the_authored_event() {
    let events = purchase_events(
        "Plan card",
        Some("Generated Nuxie Interaction"),
        &[(70.0, 40.0)],
        false,
    );
    assert_eq!(events.len(), 1);
}

#[test]
fn nested_purchase_component_reports_the_authored_event() {
    let events = purchase_events(
        "Purchase",
        None,
        &[(80.0, 50.0), (240.0, 50.0), (80.0, 50.0)],
        false,
    );
    let sources: Vec<_> = events
        .iter()
        .map(|event| event.context().expect("nested source"))
        .collect();
    assert!(!sources[0].path().is_empty());
    assert_ne!(
        sources[0].path(),
        sources[1].path(),
        "repeated definitions retain distinct occurrence paths"
    );
    assert_eq!(
        sources[0].path(),
        sources[2].path(),
        "the same occurrence keeps its identity"
    );
}

#[test]
fn nested_purchase_events_retain_distinct_view_model_identity() {
    let events = purchase_events(
        "Purchase",
        None,
        &[(80.0, 50.0), (240.0, 50.0), (80.0, 50.0)],
        false,
    );
    let ids: Vec<_> = events
        .iter()
        .map(|event| {
            event
                .context()
                .expect("nested source")
                .view_model_instance_id()
                .expect("live component view model")
        })
        .collect();
    assert_ne!(ids[0], ids[1]);
    assert_eq!(ids[0], ids[2]);
}

#[test]
fn host_scalar_write_preserves_nested_purchase_identity() {
    purchase_events("Purchase", None, &[(80.0, 50.0), (240.0, 50.0)], true);
}

#[test]
fn players_sharing_an_occurrence_do_not_republish_nested_taps() {
    let (mut artboard, model) = purchase_scene("Purchase");
    let mut primary = artboard.state_machine_instance(0).unwrap();
    primary.bind_owned_view_model_handle(model.clone());
    let mut auxiliary = artboard.state_machine_instance(0).unwrap();
    auxiliary.bind_owned_view_model_handle(model.clone());
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut primary), 0.0, true)
        .unwrap();
    primary.pointer_down(80.0, 50.0, 1);
    primary.pointer_up(80.0, 50.0, 1);
    let events = primary.take_reported_events();
    assert_eq!(events.len(), 1);
    assert!(
        auxiliary.take_reported_events().is_empty(),
        "another player must not republish the same occurrence event"
    );
    drop(primary);
    let mut replacement = artboard.state_machine_instance(0).unwrap();
    replacement.bind_owned_view_model_handle(model);
    assert!(
        replacement.take_reported_events().is_empty(),
        "replacing a player must not replay an already observed tap"
    );
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut replacement), 0.016, true)
        .unwrap();
    replacement.pointer_down(240.0, 50.0, 1);
    replacement.pointer_up(240.0, 50.0, 1);
    let next = replacement.take_reported_events();
    assert_eq!(next.len(), 1, "a genuinely new tap must still publish");
    assert_ne!(
        events[0].context().unwrap().view_model_instance_id(),
        next[0].context().unwrap().view_model_instance_id()
    );
    assert!(auxiliary.take_reported_events().is_empty());
}

#[test]
fn batched_nested_taps_preserve_pointer_order_across_occurrences() {
    let (mut artboard, model) = purchase_scene("Purchase");
    let first = model
        .linked_view_model_by_property_name_path("first")
        .unwrap()
        .instance_identity();
    let second = model
        .linked_view_model_by_property_name_path("second")
        .unwrap()
        .instance_identity();
    let mut machine = artboard.state_machine_instance(0).unwrap();
    machine.bind_owned_view_model_handle(model);
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.0, true)
        .unwrap();
    for x in [80.0, 240.0, 80.0] {
        machine.pointer_down(x, 50.0, 1);
        machine.pointer_up(x, 50.0, 1);
    }
    let sources: Vec<_> = machine
        .take_reported_events()
        .iter()
        .map(|event| event.context().unwrap().view_model_instance_id().unwrap())
        .collect();
    assert_eq!(sources, [first, second, first]);
    assert!(machine.take_reported_events().is_empty());
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
        .unwrap();
    assert!(
        machine.take_reported_events().is_empty(),
        "native queue delivery must not republish observed taps"
    );
}

#[test]
fn queued_tap_keeps_its_source_when_the_authored_reference_is_replaced() {
    let (mut artboard, model) = purchase_scene("Purchase");
    let first = model
        .linked_view_model_by_property_name_path("first")
        .unwrap();
    let second = model
        .linked_view_model_by_property_name_path("second")
        .unwrap();
    let mut machine = artboard.state_machine_instance(0).unwrap();
    machine.bind_owned_view_model_handle(model.clone());
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.0, true)
        .unwrap();
    machine.pointer_down(80.0, 50.0, 1);
    machine.pointer_up(80.0, 50.0, 1);
    let mut transaction = nuxie_runtime::RuntimeOwnedViewModelTransaction::begin().unwrap();
    assert!(
        transaction
            .link_view_model(&model, "first", &second)
            .unwrap()
    );
    transaction.commit();
    artboard.bind_owned_view_model_handle(model.clone());
    machine.bind_owned_view_model_handle(model.clone());
    assert_eq!(
        model
            .linked_view_model_by_property_name_path("first")
            .unwrap()
            .instance_identity(),
        second.instance_identity()
    );
    let events = machine.take_reported_events();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].context().unwrap().view_model_instance_id(),
        Some(first.instance_identity()),
        "a queued tap must not be attributed to a replacement model"
    );
}
