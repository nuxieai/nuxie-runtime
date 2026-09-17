use super::*;
use nuxie_render_api::RecordingFactory;

#[test]
fn nested_event_source_survives_owned_c_projection() {
    let bytes = include_bytes!("../../nuxie-runtime/tests/fixtures/purchase-scopes/screen.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = nuxie_runtime::File::import(bytes, retained, None, None, None).unwrap();
    let index = file
        .with_file(|file| {
            (0..file.artboard_count()).find(|&index| file.artboard_name_at(index) == "Purchase")
        })
        .unwrap();
    let mut artboard = ArtboardInstance::from_native(file, index).unwrap();
    let file = artboard.native_file();
    let source = file
        .with_file(|file| file.artboard_at_source(index))
        .unwrap();
    let native = file
        .with_file_mut(|file| file.create_default_view_model_instance_for_artboard(source))
        .unwrap();
    let model =
        nuxie_runtime::RuntimeOwnedViewModelHandle::from_native(file.clone(), native).unwrap();
    let expected = model
        .linked_view_model_by_property_name_path("first")
        .unwrap()
        .instance_identity();
    artboard.bind_owned_view_model_handle(model.clone());
    let mut machine = artboard.state_machine_instance(0).unwrap();
    machine.bind_owned_view_model_handle(model);
    artboard
        .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.0, true)
        .unwrap();
    machine.pointer_down(80.0, 50.0, 1);
    machine.pointer_up(80.0, 50.0, 1);
    let events = own_reported_events(machine.take_reported_events(), 0, 0).unwrap();
    assert_eq!(events.len(), 1);
    let mut owned = player_step_failure(NuxStatus::Ok, "");
    owned.events = events;
    let mut result = std::ptr::null_mut();
    publish_player_step_result(&mut result, owned);
    // Event source identity is owned by the step, not borrowed from a live scene.
    drop(machine);
    drop(artboard);
    unsafe {
        let mut identity = u64::MAX;
        assert_eq!(
            nux_player_step_result_event_view_model_instance(result, 0, &mut identity),
            NuxStatus::Ok
        );
        assert_eq!(identity, expected);
        let mut event = NuxPlayerEventView::default();
        assert_eq!(
            nux_player_step_result_event(result, 0, &mut event),
            NuxStatus::Ok
        );
        assert!(event.property_count > 0);
        assert_eq!(
            nux_player_step_result_event_view_model_instance(result, 1, &mut identity),
            NuxStatus::NotFound
        );
        assert_eq!(identity, expected);
        assert_eq!(
            nux_player_step_result_event_view_model_instance(result, 0, std::ptr::null_mut()),
            NuxStatus::NullArgument
        );
        assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
    }
}

#[test]
fn unscoped_events_do_not_invent_a_source_or_overwrite_output() {
    let mut owned = player_step_failure(NuxStatus::Ok, "");
    owned.events.push(OwnedPlayerEvent {
        event_local_index: 0,
        event_core_type: 0,
        name: None,
        url: None,
        target: None,
        seconds_delay: 0.0,
        properties: Vec::new(),
        view_model_instance_id: None,
    });
    let mut result = std::ptr::null_mut();
    publish_player_step_result(&mut result, owned);
    unsafe {
        let mut identity = 91;
        assert_eq!(
            nux_player_step_result_event_view_model_instance(result, 0, &mut identity),
            NuxStatus::NotFound
        );
        assert_eq!(identity, 91);
        assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
    }
}
