use super::*;
#[path = "../tests/support/value_markers.rs"]
mod fixture;

#[test]
fn failed_step_after_marker_write_restores_values_and_poisons_player() {
    unsafe {
        let bytes = fixture::fixture(None, &[fixture::Action::Number(5.0)], false);
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
        let marker = NuxValueMarker {
            model: NuxStringView::from_static("Values"),
            value: NuxStringView::from_static("n"),
            marker: NuxStringView::from_static("n_set"),
        };
        assert_eq!(nux_file_set_value_markers(file, &marker, 1), NuxStatus::Ok);
        let mut value = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_authored(file, 0, 0, &mut value),
            NuxStatus::Ok
        );
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, value),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 10.0,
            y: 5.0,
            pointer_id: 1,
            timestamp_seconds: 0.0,
        };
        let step = NuxPlayerStep {
            pointers: &pointer,
            pointer_count: 1,
            ..Default::default()
        };
        PANIC_BEFORE_STEP_RESULT_REGISTRATION.with(|armed| armed.set(true));
        assert_eq!(
            nux_player_step(player, &step, &mut result),
            NuxStatus::RuntimeError
        );
        assert_eq!(
            (*value)
                .instance
                .borrow()
                .number_value_by_property_name("n"),
            Some(0.0)
        );
        assert_eq!(
            (*value)
                .instance
                .borrow()
                .boolean_value_by_property_name("n_set"),
            Some(false)
        );
        nux_player_step_result_free(result);
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::RuntimeError
        );
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_view_model_instance_free(value);
        nux_file_free(file);
    }
}

#[test]
fn marker_install_respects_thread_and_operation_borrow() {
    unsafe {
        let bytes = fixture::fixture(None, &[], false);
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
        let address = file as usize;
        assert_eq!(
            std::thread::spawn(move || nux_file_set_value_markers(
                address as *mut NuxFile,
                ptr::null(),
                0
            ))
            .join()
            .unwrap(),
            NuxStatus::WrongThread
        );
        let catalog = &(*file).view_model_catalog;
        let operation = catalog.value_policy.borrow();
        assert_eq!(
            nux_file_set_value_markers(file, ptr::null(), 0),
            NuxStatus::ReentrantCall
        );
        drop(operation);
        assert_eq!(
            nux_file_set_value_markers(file, ptr::null(), 0),
            NuxStatus::Ok
        );
        nux_file_free(file);
    }
}

#[test]
fn rule_and_group_install_respect_thread_and_operation_borrow() {
    unsafe {
        let bytes = fixture::fixture(None, &[], false);
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
        let address = file as usize;
        let statuses = std::thread::spawn(move || {
            (
                nux_file_set_value_rules(address as *mut NuxFile, ptr::null(), 0),
                nux_file_set_rule_groups(address as *mut NuxFile, ptr::null(), 0),
            )
        })
        .join()
        .unwrap();
        assert_eq!(statuses, (NuxStatus::WrongThread, NuxStatus::WrongThread));
        let catalog = &(*file).view_model_catalog;
        let operation = catalog.value_policy.borrow();
        assert_eq!(
            nux_file_set_value_rules(file, ptr::null(), 0),
            NuxStatus::ReentrantCall
        );
        assert_eq!(
            nux_file_set_rule_groups(file, ptr::null(), 0),
            NuxStatus::ReentrantCall
        );
        drop(operation);
        assert_eq!(
            nux_file_set_value_rules(file, ptr::null(), 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_file_set_rule_groups(file, ptr::null(), 0),
            NuxStatus::Ok
        );
        nux_file_free(file);
    }
}

#[test]
fn rule_report_getters_preserve_failure_status_and_reject_reentry() {
    unsafe {
        let mut step = ptr::null_mut();
        assert_eq!(
            publish_player_step_failure(&mut step, NuxStatus::LimitExceeded, "bounded result"),
            NuxStatus::LimitExceeded
        );
        let mut mutation = ptr::null_mut();
        let invalid_batch = NuxViewModelMutationBatch {
            mutation_count: usize::MAX,
            ..Default::default()
        };
        assert_eq!(
            nux_view_model_mutate(&invalid_batch, &mut mutation),
            NuxStatus::LimitExceeded
        );
        let mut row = NuxValueRuleReportView::default();
        let mut identity = 99;
        assert_eq!(
            nux_player_step_result_rule_report(step, 0, &mut row),
            NuxStatus::LimitExceeded
        );
        assert_eq!(
            nux_player_step_result_rule_report_list_item(step, 0, 0, &mut identity),
            NuxStatus::LimitExceeded
        );
        assert_eq!(identity, 0);
        assert_eq!(
            nux_view_model_mutation_result_rule_report(mutation, 0, &mut row),
            NuxStatus::LimitExceeded
        );
        assert_eq!(
            nux_view_model_mutation_result_rule_report_list_item(mutation, 0, 0, &mut identity),
            NuxStatus::LimitExceeded
        );
        let step_call = enter_handle(step, HandleKind::PlayerStepResult).unwrap();
        let mutation_call = enter_handle(mutation, HandleKind::ViewModelMutationResult).unwrap();
        assert_eq!(
            nux_player_step_result_rule_report(step, 0, &mut row),
            NuxStatus::ReentrantCall
        );
        assert_eq!(
            nux_player_step_result_rule_report_list_item(step, 0, 0, &mut identity),
            NuxStatus::ReentrantCall
        );
        assert_eq!(
            nux_view_model_mutation_result_rule_report(mutation, 0, &mut row),
            NuxStatus::ReentrantCall
        );
        assert_eq!(
            nux_view_model_mutation_result_rule_report_list_item(mutation, 0, 0, &mut identity),
            NuxStatus::ReentrantCall
        );
        drop(step_call);
        drop(mutation_call);
        nux_player_step_result_free(step);
        nux_view_model_mutation_result_free(mutation);
    }
}

#[test]
fn empty_attempted_step_list_has_no_member_identity() {
    // Exercise the result-view contract directly with an owned empty list.
    let mut result = player_step_failure(NuxStatus::Ok, "");
    result.rule_reports.push(nuxie::RuntimeValueRuleReport {
        owner_instance_identity: 1,
        property_index: 0,
        rule_index: 0,
        rule_owner_instance_identity: 1,
        rule_property_index: 0,
        refused: true,
        attempted: RuntimeViewModelChangeValue::List(Vec::new()),
    });
    let result = Box::into_raw(Box::new(result));
    register_handle(result, HandleKind::PlayerStepResult, thread::current().id());
    unsafe {
        for index in [0, usize::MAX] {
            let mut identity = 99;
            assert_eq!(
                nux_player_step_result_rule_report_list_item(result, 0, index, &mut identity),
                NuxStatus::NotFound
            );
            assert_eq!(identity, 0);
        }
        nux_player_step_result_free(result);
    }
}
