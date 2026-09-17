use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{ArtboardInstance, File, RuntimeFactoryHandle, RuntimeOwnedViewModelHandle};

fn purchase_events(artboard_name: &str, machine_name: Option<&str>, x: f32, y: f32) -> Vec<String> {
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
    let mut machine = match machine_name {
        Some(name) => artboard.state_machine_instance_named(name),
        None => artboard.state_machine_instance(0),
    }
    .expect("interaction machine");
    machine.bind_owned_view_model_handle(view_model);
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.0, true)
        .expect("initial frame");
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
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
        .expect("pointer frame");
    events.extend(machine.take_reported_events());
    events
        .iter()
        .map(|event| event.name().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn direct_purchase_component_reports_the_authored_event() {
    assert_eq!(
        purchase_events("Plan card", Some("Generated Nuxie Interaction"), 70.0, 40.0),
        ["Nuxie Interaction"]
    );
}

#[test]
fn nested_purchase_component_reports_the_authored_event() {
    assert_eq!(
        purchase_events("Purchase", None, 80.0, 50.0),
        ["Nuxie Interaction"]
    );
}
