fn wave_b4_context_indices(context: &RuntimeOwnedViewModelContext) -> Vec<usize> {
    context
        .handles()
        .map(|handle| handle.borrow().view_model_index())
        .collect()
}

fn wave_b4_machine_context_indices(machine: &StateMachineInstance) -> Vec<usize> {
    machine
        .data_context()
        .expect("retained DataContext")
        .snapshot()
        .handles()
        .map(|handle| handle.borrow().view_model_index())
        .collect()
}

fn wave_b4_view_model_name(file: &RuntimeFile, index: usize) -> String {
    file.view_model(index)
        .and_then(|view_model| view_model.object.string_property("name"))
        .unwrap_or("")
        .to_owned()
}

fn wave_b4_context_names(file: &RuntimeFile, indices: &[usize]) -> Vec<String> {
    indices
        .iter()
        .map(|index| wave_b4_view_model_name(file, *index))
        .collect()
}

// Narrow test-only representation of the missing Artboard owner. Keeping the
// seam on ArtboardInstance makes expected-red cases 4 and 5 fail at the exact
// `Artboard::setViewModelInstance` action instead of misusing the unrelated
// global-slot setter as a proxy.
impl ArtboardInstance {
    fn set_view_model_instance(
        &mut self,
        _view_model_instance: RuntimeOwnedViewModelHandle,
    ) -> Result<bool, &'static str> {
        Err("Artboard::setViewModelInstance owner is not implemented")
    }
}

#[test]
fn wave_b4_global_binding_case_001_file_names_are_globals_in_file_order() {
    let (file, _, _) = upstream_global_binding_fixture();
    let names = crate::runtime_global_view_model_names(&file);
    assert!(!names.is_empty());
    assert_eq!(
        names,
        crate::runtime_global_view_model_indices(&file)
            .into_iter()
            .map(|index| {
                let view_model = file.view_model(index).expect("listed ViewModel");
                assert_eq!(view_model.object.uint_property("viewModelType"), Some(2));
                view_model
                    .object
                    .string_property("name")
                    .expect("global ViewModel name")
                    .to_owned()
            })
            .collect::<Vec<_>>(),
    );
}

#[test]
fn wave_b4_global_binding_case_002_artboard_does_not_auto_create_globals() {
    let (_, artboard, _) = upstream_global_binding_fixture();
    assert!(artboard.owned_view_model_context().is_none());
}

#[test]
fn wave_b4_global_binding_case_003_getter_is_null_until_set() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let names = crate::runtime_global_view_model_names(&file);
    assert!(!names.is_empty());
    let target = &names[0];
    assert!(artboard.global_view_model_instance(&file, target).is_none());
    let instance = upstream_global_binding_handle(&file, target);
    assert!(artboard.set_global_view_model_instance(&file, target, Some(instance.clone())));
    assert!(
        artboard
            .global_view_model_instance(&file, target)
            .is_some_and(|actual| actual.ptr_eq(&instance))
    );
    assert!(!artboard.set_global_view_model_instance(&file, "not-a-global", Some(instance),));
    assert!(
        artboard
            .global_view_model_instance(&file, "not-a-global")
            .is_none()
    );
}

#[test]
#[ignore = "expected-red: exact Artboard::setViewModelInstance owner is absent"]
fn wave_b4_global_binding_case_004_set_without_bind_mutates_order() {
    let (file, mut artboard, artboard_index) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let main = upstream_global_binding_main_handle(&file, artboard_index);
    let main_index = main.borrow().view_model_index();
    assert_ne!(
        file.view_model(main_index)
            .and_then(|view_model| { view_model.object.uint_property("viewModelType") }),
        Some(2)
    );
    artboard
        .set_view_model_instance(main)
        .expect("exact Artboard::setViewModelInstance owner");
    for name in &global_names {
        assert!(artboard.set_global_view_model_instance(
            &file,
            name,
            Some(upstream_global_binding_handle(&file, name)),
        ));
    }
    let mut expected = vec![main_index];
    expected.extend(crate::runtime_global_view_model_indices(&file));
    assert_eq!(
        wave_b4_context_indices(
            artboard
                .owned_view_model_context()
                .expect("pre-bind context")
        ),
        expected,
    );
}

#[test]
#[ignore = "expected-red: exact Artboard::setViewModelInstance owner is absent"]
fn wave_b4_global_binding_case_005_globals_keep_file_order_when_main_is_set_later() {
    let (file, mut artboard, artboard_index) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(global_names.len() >= 2);
    for name in global_names.iter().rev() {
        assert!(artboard.set_global_view_model_instance(
            &file,
            name,
            Some(upstream_global_binding_handle(&file, name)),
        ));
    }
    assert_eq!(
        wave_b4_context_names(
            &file,
            &wave_b4_context_indices(
                artboard
                    .owned_view_model_context()
                    .expect("globals create context"),
            ),
        ),
        global_names,
    );
    let main = upstream_global_binding_main_handle(&file, artboard_index);
    let main_name = wave_b4_view_model_name(&file, main.borrow().view_model_index());
    artboard
        .set_view_model_instance(main)
        .expect("exact Artboard::setViewModelInstance owner");
    let mut expected = vec![main_name];
    expected.extend(global_names);
    assert_eq!(
        wave_b4_context_names(
            &file,
            &wave_b4_context_indices(
                artboard
                    .owned_view_model_context()
                    .expect("ordered context")
            ),
        ),
        expected,
    );
}

#[test]
fn wave_b4_global_binding_case_006_bind_completes_missing_global_slots() {
    let (file, mut artboard, artboard_index) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    let main = upstream_global_binding_main_handle(&file, artboard_index);
    machine
        .bind_view_model_instance(Some(&file), &mut artboard, Some(main))
        .expect("StateMachineInstance::bindViewModelInstance");
    for name in &global_names {
        assert!(
            machine
                .global_view_model_instance(Some(&file), name)
                .is_some()
        );
    }
    assert_eq!(
        wave_b4_machine_context_indices(&machine).len(),
        global_names.len() + 1
    );
}

#[test]
fn wave_b4_global_binding_case_007_slot_accepts_other_view_model_instance() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(global_names.len() >= 2);
    let slot_a = &global_names[0];
    let vm_b = &global_names[1];
    let override_instance = upstream_global_binding_handle(&file, vm_b);
    assert_eq!(
        wave_b4_view_model_name(&file, override_instance.borrow().view_model_index()),
        *vm_b,
    );
    assert!(artboard.set_global_view_model_instance(
        &file,
        slot_a,
        Some(override_instance.clone()),
    ));
    assert!(
        artboard
            .global_view_model_instance(&file, slot_a)
            .is_some_and(|actual| actual.ptr_eq(&override_instance))
    );
    assert!(artboard.global_view_model_instance(&file, vm_b).is_none());
}

#[test]
fn wave_b4_global_binding_case_008_state_machine_set_bind_get_and_replace() {
    let (file, mut artboard, artboard_index) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    assert!(
        machine
            .global_view_model_instance(Some(&file), &global_names[0])
            .is_none()
    );
    let main = upstream_global_binding_main_handle(&file, artboard_index);
    let main_name = wave_b4_view_model_name(&file, main.borrow().view_model_index());
    assert!(machine.set_view_model_instance(Some(main)));
    for name in &global_names {
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            name,
            Some(upstream_global_binding_handle(&file, name)),
        ));
    }
    machine.bind(Some(&file), &mut artboard).expect("bind");
    let names = wave_b4_context_names(&file, &wave_b4_machine_context_indices(&machine));
    assert_eq!(names.len(), global_names.len() + 1);
    assert_eq!(names[0], main_name);
    assert_eq!(
        wave_b4_view_model_name(
            &file,
            machine
                .global_view_model_instance(Some(&file), &global_names[0])
                .expect("first global")
                .borrow()
                .view_model_index(),
        ),
        global_names[0],
    );
    let custom = upstream_global_binding_handle(&file, &global_names[0]);
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        &global_names[0],
        Some(custom.clone()),
    ));
    assert!(
        machine
            .global_view_model_instance(Some(&file), &global_names[0])
            .is_some_and(|actual| actual.ptr_eq(&custom))
    );
    assert_eq!(
        wave_b4_context_names(&file, &wave_b4_machine_context_indices(&machine)),
        names,
    );
}

#[test]
fn wave_b4_global_binding_case_009_rejects_non_global_name() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    let non_global_index = file
        .view_models()
        .iter()
        .position(|view_model| view_model.object.uint_property("viewModelType") != Some(2))
        .expect("non-global ViewModel");
    let non_global = wave_b4_view_model_name(&file, non_global_index);
    assert!(!non_global.is_empty());
    let instance = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::new(&file, non_global_index).expect("non-global instance"),
    );
    assert!(!artboard.set_global_view_model_instance(&file, &non_global, Some(instance.clone()),));
    assert!(
        artboard
            .global_view_model_instance(&file, &non_global)
            .is_none()
    );
    assert!(!machine.set_global_view_model_instance(Some(&file), &non_global, Some(instance),));
    assert!(
        machine
            .global_view_model_instance(Some(&file), &non_global)
            .is_none()
    );
}

#[test]
fn wave_b4_global_binding_case_010_bind_creates_context_when_none_is_set() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    assert!(machine.data_context().is_none());
    machine.bind(Some(&file), &mut artboard).expect("bind");
    assert_eq!(
        wave_b4_machine_context_indices(&machine).len(),
        global_names.len() + 1
    );
    for name in &global_names {
        assert!(
            machine
                .global_view_model_instance(Some(&file), name)
                .is_some()
        );
    }
}

#[test]
fn wave_b4_global_binding_case_011_machine_null_empties_only_selected_slot() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(global_names.len() >= 2);
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    for name in &global_names {
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            name,
            Some(upstream_global_binding_handle(&file, name)),
        ));
    }
    assert!(
        machine
            .global_view_model_instance(Some(&file), &global_names[0])
            .is_some()
    );
    assert_eq!(
        wave_b4_machine_context_indices(&machine).len(),
        global_names.len()
    );
    assert!(machine.set_global_view_model_instance(Some(&file), &global_names[0], None));
    assert!(
        machine
            .global_view_model_instance(Some(&file), &global_names[0])
            .is_none()
    );
    for name in &global_names[1..] {
        assert!(
            machine
                .global_view_model_instance(Some(&file), name)
                .is_some()
        );
    }
    assert_eq!(
        wave_b4_machine_context_indices(&machine).len(),
        global_names.len() - 1
    );
}

#[test]
fn wave_b4_global_binding_case_012_bind_adds_main_when_only_global_is_set() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        &global_names[0],
        Some(upstream_global_binding_handle(&file, &global_names[0])),
    ));
    let retained_context = machine
        .data_context()
        .expect("global creates context")
        .clone();
    assert!(retained_context.main_handle().is_none());
    machine.bind(Some(&file), &mut artboard).expect("bind");
    let completed_context = machine.data_context().expect("completed context");
    assert!(
        retained_context.ptr_eq(completed_context),
        "bind mutates the retained DataContext instead of replacing it"
    );
    let snapshot = completed_context.snapshot();
    assert!(snapshot.main_handle().is_some());
    let names = wave_b4_context_names(&file, &wave_b4_context_indices(&snapshot));
    assert_eq!(names.len(), global_names.len() + 1);
    assert_eq!(
        names[0],
        wave_b4_view_model_name(
            &file,
            snapshot
                .main_handle()
                .expect("main")
                .borrow()
                .view_model_index(),
        ),
    );
}

#[test]
fn wave_b4_global_binding_case_013_machine_null_on_empty_context_is_noop() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    let mut machine = artboard.state_machine_instance(0).expect("state machine");
    assert!(machine.data_context().is_none());
    assert!(machine.set_global_view_model_instance(Some(&file), &global_names[0], None));
    assert!(machine.data_context().is_none());
    assert!(!machine.set_global_view_model_instance(Some(&file), "not-a-global", None));
}

#[test]
fn wave_b4_global_binding_case_014_artboard_null_empties_only_selected_slot() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(global_names.len() >= 2);
    for name in &global_names {
        assert!(artboard.set_global_view_model_instance(
            &file,
            name,
            Some(upstream_global_binding_handle(&file, name)),
        ));
    }
    assert!(
        artboard
            .global_view_model_instance(&file, &global_names[0])
            .is_some()
    );
    assert_eq!(
        wave_b4_context_indices(artboard.owned_view_model_context().expect("global context")).len(),
        global_names.len(),
    );
    assert!(artboard.set_global_view_model_instance(&file, &global_names[0], None));
    assert!(
        artboard
            .global_view_model_instance(&file, &global_names[0])
            .is_none()
    );
    for name in &global_names[1..] {
        assert!(artboard.global_view_model_instance(&file, name).is_some());
    }
    assert_eq!(
        wave_b4_context_indices(
            artboard
                .owned_view_model_context()
                .expect("remaining globals")
        )
        .len(),
        global_names.len() - 1,
    );
}

#[test]
fn wave_b4_global_binding_case_015_artboard_null_on_empty_context_is_noop() {
    let (file, mut artboard, _) = upstream_global_binding_fixture();
    let global_names = crate::runtime_global_view_model_names(&file);
    assert!(!global_names.is_empty());
    assert!(artboard.owned_view_model_context().is_none());
    assert!(artboard.set_global_view_model_instance(&file, &global_names[0], None));
    assert!(artboard.owned_view_model_context().is_none());
    assert!(!artboard.set_global_view_model_instance(&file, "not-a-global", None));
}
