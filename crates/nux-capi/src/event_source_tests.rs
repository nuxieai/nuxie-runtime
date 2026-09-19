use super::*;
use nuxie_render_api::RecordingFactory;

#[test]
fn nested_event_source_survives_owned_c_projection() {
    let bytes = include_bytes!("../../../fixtures/purchase-scopes/screen.riv");
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

#[test]
fn scalar_mutation_before_player_keeps_nested_event_on_bound_graph() {
    for host_write in [false, true] {
        unsafe {
            let bytes = include_bytes!("../../../fixtures/purchase-scopes/screen.riv");
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let index = (*file)
                .file
                .with_file(|file| {
                    (0..file.artboard_count()).find(|&i| file.artboard_name_at(i) == "Purchase")
                })
                .unwrap();
            let mut artboard = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, index, &mut artboard),
                NuxStatus::Ok
            );
            let mut model = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_default(artboard, &mut model),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(artboard, model),
                NuxStatus::Ok
            );
            let expected = [
                snapshot_reference(model, "first"),
                snapshot_reference(model, "second"),
            ];
            assert_ne!(expected[0], expected[1]);
            if host_write {
                let path = b"fontScale";
                let mutation = NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
                    instance: model,
                    path: NuxStringView {
                        data: path.as_ptr().cast(),
                        len: path.len(),
                    },
                    number_value: 1.0,
                    ..NuxViewModelMutation::default()
                };
                let batch = NuxViewModelMutationBatch {
                    mutations: &mutation,
                    mutation_count: 1,
                    ..NuxViewModelMutationBatch::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
                assert_eq!(nux_view_model_mutation_result_free(result), NuxStatus::Ok);
            }
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
            let mut result = ptr::null_mut();
            assert_eq!(
                nux_player_step(player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            let mut sources = Vec::new();
            for x in [80.0, 240.0] {
                for kind in [NUX_PLAYER_POINTER_KIND_DOWN, NUX_PLAYER_POINTER_KIND_UP] {
                    let pointer = NuxPlayerPointerEvent {
                        kind,
                        x,
                        y: 50.0,
                        pointer_id: 1,
                        timestamp_seconds: 0.0,
                    };
                    let step = NuxPlayerStep {
                        pointers: &pointer,
                        pointer_count: 1,
                        ..NuxPlayerStep::default()
                    };
                    assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                    for index in 0..(*result).events.len() {
                        let mut identity = 0;
                        assert_eq!(
                            nux_player_step_result_event_view_model_instance(
                                result,
                                index,
                                &mut identity
                            ),
                            NuxStatus::Ok
                        );
                        sources.push(identity);
                    }
                    assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                }
            }
            assert_eq!(sources, expected, "host_write={host_write}");
            assert_eq!(snapshot_reference(model, "first"), expected[0]);
            assert_eq!(snapshot_reference(model, "second"), expected[1]);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_view_model_instance_free(model), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }
}

unsafe fn snapshot_reference(model: *mut NuxViewModelInstance, name: &str) -> u64 {
    unsafe {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_snapshot(model, &mut snapshot),
            NuxStatus::Ok
        );
        let mut info = NuxViewModelSnapshotInfo::default();
        assert_eq!(
            nux_view_model_snapshot_info(snapshot, &mut info),
            NuxStatus::Ok
        );
        let mut found = None;
        for index in 0..info.value_count {
            let mut value = NuxViewModelSnapshotValueView::default();
            assert_eq!(
                nux_view_model_snapshot_value(snapshot, index, &mut value),
                NuxStatus::Ok
            );
            if value.owner_instance_id == info.root_instance_id
                && slice::from_raw_parts(value.name.data.cast::<u8>(), value.name.len)
                    == name.as_bytes()
            {
                found = Some(value.referenced_instance_id);
            }
        }
        assert_eq!(nux_view_model_snapshot_free(snapshot), NuxStatus::Ok);
        found.expect("authored reference in C snapshot")
    }
}
