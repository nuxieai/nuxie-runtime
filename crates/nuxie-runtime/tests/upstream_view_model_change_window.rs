//! Literal cases from runtime/view_model_change_window_test.cpp through c4d2c6cb.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        animation_state::AnimationState, linear_animation::LinearAnimation,
        nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard_component_list::ArtboardComponentList,
    custom_property_trigger::CustomPropertyTrigger,
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    event::Event,
    generated::{core_registry::CoreRegistry, nested_artboard_base::NestedArtboardBase},
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn read_file(asset: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}
fn artboard(file: &RuntimeFileHandle, name: &str) -> RuntimeArtboardInstanceHandle {
    file.with_file(|file| file.artboard_named(name)).unwrap()
}
fn default_artboard(file: &RuntimeFileHandle) -> RuntimeArtboardInstanceHandle {
    file.with_file(|file| file.artboard_default()).unwrap()
}
fn default_vm(file: &RuntimeFileHandle, artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    })
    .unwrap()
}
fn first_vm(file: &RuntimeFileHandle, artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    let id = artboard.with_artboard(|a| a.view_model_id());
    file.with_file(|file| file.create_view_model_instance_at(id as usize, 0))
        .unwrap()
}
fn bind(machine: &RuntimeStateMachineInstanceHandle, vm: &CoreHandle) {
    machine.with_instance_mut(|m| m.bind_view_model_instance(vm.clone()));
}
fn property(vm: &CoreHandle, name: &str) -> CoreHandle {
    vm.with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named(name))
        .flatten()
        .unwrap()
}
fn trigger(vm: &CoreHandle, name: &str) -> CoreHandle {
    let value = property(vm, name);
    assert!(
        value
            .with_downcast::<ViewModelInstanceTrigger, _>(|_| ())
            .is_some()
    );
    value
}
fn fire(vm: &CoreHandle, name: &str) {
    assert!(ViewModelInstanceTrigger::trigger_handle(&trigger(vm, name)));
}
fn state(machine: &RuntimeStateMachineInstanceHandle, layer: usize) -> String {
    machine
        .with_instance_mut(|m| m.layer_state(layer))
        .and_then(|s| {
            s.with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten()
        })
        .and_then(|a| a.with_downcast::<LinearAnimation, _>(|a| a.name().to_owned()))
        .unwrap_or_default()
}
fn string(vm: &CoreHandle, name: &str) -> String {
    property(vm, name)
        .with_downcast::<ViewModelInstanceString, _>(|v| v.value().to_owned())
        .unwrap()
}
fn nested_vm(vm: &CoreHandle, name: &str) -> CoreHandle {
    property(vm, name)
        .with_downcast::<ViewModelInstanceViewModel, _>(|v| v.reference_view_model_instance())
        .flatten()
        .unwrap()
}
fn nested_machine(nested: &CoreHandle) -> RuntimeStateMachineInstanceHandle {
    nested
        .with(|n| n.as_nested_artboard().unwrap().nested_animations().to_vec())
        .unwrap()
        .iter()
        .find_map(|a| {
            a.with_downcast::<NestedStateMachine, _>(|n| n.state_machine_instance())
                .flatten()
        })
        .unwrap()
}
fn named_nested_machine(
    a: &RuntimeArtboardInstanceHandle,
    name: &str,
) -> RuntimeStateMachineInstanceHandle {
    let nested = a
        .with_artboard(|a| a.nested_artboards())
        .into_iter()
        .find(|n| {
            n.with(|n| n.as_nested_artboard().unwrap().artboard_instance_handle(0))
                .flatten()
                .is_some_and(|a| a.with_artboard(|a| a.name() == name))
        })
        .unwrap();
    nested_machine(&nested)
}

#[test]
fn earlier_fire_reaches_state_entered_later_in_frame() {
    let f = read_file("data_binding_test.riv");
    let a = artboard(&f, "artboard-2");
    let vm = default_vm(&f, &a);
    let m = a.default_state_machine_handle().unwrap();
    bind(&m, &vm);
    fire(&vm, "trigger-prop");
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 1), "right");
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 1), "right");
    fire(&vm, "trigger-prop");
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 1), "bottom");
}
#[test]
fn fires_before_machine_creation_are_ignored() {
    let f = read_file("state_action_vm_sync.riv");
    let a = artboard(&f, "StateEnter");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    fire(&vm, "go");
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 0), "done");
    property(&vm, "ready")
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|v| v.set_value(false))
        .unwrap();
    let a2 = artboard(&f, "StateEnter");
    let m2 = a2.state_machine_at(0).unwrap();
    bind(&m2, &vm);
    for _ in 0..3 {
        m2.advance_and_apply(0.016);
    }
    assert_eq!(state(&m2, 0), "idle");
    fire(&vm, "go");
    m2.advance_and_apply(0.016);
    assert_eq!(state(&m2, 0), "done");
}
#[test]
fn reset_state_drops_previous_changes() {
    let f = read_file("state_action_vm_sync.riv");
    let a = artboard(&f, "StateEnter");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 0), "idle");
    fire(&vm, "go");
    m.with_instance_mut(|m| m.reset_state());
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 0), "idle");
    fire(&vm, "go");
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 0), "done");
}
#[test]
fn settled_machine_rebind_ignores_past_fires() {
    let f = read_file("state_action_vm_sync.riv");
    let a = artboard(&f, "StateEnter");
    let m = a.state_machine_at(0).unwrap();
    let first = first_vm(&f, &a);
    bind(&m, &first);
    for _ in 0..3 {
        m.advance_and_apply(0.016);
    }
    assert_eq!(state(&m, 0), "idle");
    let a2 = artboard(&f, "StateEnter");
    let other = a2.state_machine_at(0).unwrap();
    let second = first_vm(&f, &a);
    bind(&other, &second);
    other.advance_and_apply(0.0);
    fire(&second, "go");
    for _ in 0..3 {
        other.advance_and_apply(0.016);
        m.advance_and_apply(0.016);
    }
    assert_eq!(state(&other, 0), "done");
    bind(&m, &second);
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 0), "idle");
    fire(&second, "go");
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 0), "done");
}
#[test]
fn untaken_fire_expires_with_frame() {
    let f = read_file("transition_self_comparator_test.riv");
    let a = artboard(&f, "main");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 1), "Timeline 1");
    let n = property(&vm, "num");
    fire(&vm, "tri");
    m.advance_and_apply(0.016);
    m.advance_and_apply(0.016);
    n.with_downcast_mut::<ViewModelInstanceNumber, _>(|n| n.set_value(n.value() + 1.0))
        .unwrap();
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 1), "Timeline 1");
    fire(&vm, "tri");
    n.with_downcast_mut::<ViewModelInstanceNumber, _>(|n| n.set_value(n.value() + 1.0))
        .unwrap();
    m.advance_and_apply(0.016);
    assert_eq!(state(&m, 1), "small-green");
}
#[test]
fn fire_during_uninterruptible_mix_is_not_retained() {
    let f = read_file("data_bind_test_cmdq.riv");
    let a = artboard(&f, "Test Artboard");
    let m = a.state_machine_at(0).unwrap();
    let vm = default_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 0), "TriggerHide");
    fire(&vm, "Test Trigger");
    m.advance_and_apply(0.016);
    assert_eq!(m.with_instance(|m| m.state_changed_count()), 1);
    assert_eq!(state(&m, 0), "TriggerHide");
    m.advance_and_apply(0.1);
    fire(&vm, "Test Trigger");
    let mut changes = 0;
    for _ in 0..60 {
        m.advance_and_apply(0.016);
        changes += m.with_instance(|m| m.state_changed_count());
    }
    assert_eq!(changes, 0);
}
fn child_state_after_fire(paused: bool) -> String {
    let f = read_file("collapsed_databinds_test.riv");
    let a = default_artboard(&f);
    let m = a.state_machine_at(0).unwrap();
    let vm = default_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    for _ in 0..9 {
        m.advance_and_apply(0.25);
    }
    let nested = a.with_artboard(|a| a.nested_artboards());
    assert_eq!(nested.len(), 1);
    assert!(
        !nested[0]
            .with(|n| n.as_nested_artboard().unwrap().is_collapsed())
            .unwrap()
    );
    assert_eq!(state(&nested_machine(&nested[0]), 0), "Timeline 1");
    assert!(CoreRegistry::set_bool_handle(
        &nested[0],
        NestedArtboardBase::IS_PAUSED_PROPERTY_KEY as i32,
        paused
    ));
    fire(&vm, "tri");
    for _ in 0..3 {
        m.advance_and_apply(0.016);
    }
    assert!(CoreRegistry::set_bool_handle(
        &nested[0],
        NestedArtboardBase::IS_PAUSED_PROPERTY_KEY as i32,
        false
    ));
    for _ in 0..3 {
        m.advance_and_apply(0.016);
    }
    state(&nested_machine(&nested[0]), 0)
}
#[test]
fn paused_nested_machine_ignores_fires() {
    assert_eq!(child_state_after_fire(false), "Timeline 2");
    assert_eq!(child_state_after_fire(true), "Timeline 1");
}
#[test]
fn late_nested_relay_reaches_sibling() {
    let f = read_file("nested_trigger_relay.riv");
    let a = artboard(&f, "Main");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    let consumer = named_nested_machine(&a, "Consumer");
    assert_eq!(state(&consumer, 0), "waiting");
    fire(&vm, "go");
    m.advance_and_apply(0.016);
    assert_eq!(state(&named_nested_machine(&a, "Producer1"), 0), "fired");
    assert_eq!(state(&named_nested_machine(&a, "Producer2"), 0), "fired");
    m.advance_and_apply(0.016);
    assert_eq!(state(&consumer, 0), "received");
}
#[test]
fn fire_before_listener_bind_reaches_it() {
    let f = read_file("vm_listener_fire_event.riv");
    let a = default_artboard(&f);
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    fire(&vm, "go");
    bind(&m, &vm);
    m.advance_and_apply(0.016);
    assert_eq!(m.with_instance(|m| m.reported_event_count()), 1);
    let event = m.with_instance(|m| m.reported_event_at(0).event).unwrap();
    assert_eq!(
        event
            .with_downcast::<Event, _>(|e| e.name().to_owned())
            .unwrap(),
        "ding"
    );
    m.advance_and_apply(0.016);
    assert_eq!(m.with_instance(|m| m.reported_event_count()), 0);
}
#[test]
fn listener_does_not_replay_expired_fire() {
    let f = read_file("vm_listener_fire_event.riv");
    let a = default_artboard(&f);
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    fire(&vm, "go");
    m.advance_and_apply(0.016);
    assert_eq!(m.with_instance(|m| m.reported_event_count()), 1);
    m.advance_and_apply(0.016);
    bind(&m, &vm);
    m.advance_and_apply(0.016);
    assert_eq!(m.with_instance(|m| m.reported_event_count()), 0);
    let a2 = default_artboard(&f);
    let m2 = a2.state_machine_at(0).unwrap();
    bind(&m2, &vm);
    m2.advance_and_apply(0.016);
    assert_eq!(m2.with_instance(|m| m.reported_event_count()), 0);
    fire(&vm, "go");
    m2.advance_and_apply(0.016);
    assert_eq!(m2.with_instance(|m| m.reported_event_count()), 1);
}
#[test]
fn moved_listener_hears_new_instances_fires() {
    let f = read_file("trigger_based_listeners.riv");
    let a = artboard(&f, "main");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    let first = nested_vm(&vm, "vm1");
    fire(&first, "tri1");
    m.advance_and_apply(0.016);
    assert_eq!(string(&first, "str1"), "changed-text-1");
    let model = first
        .with_downcast::<ViewModelInstance, _>(|v| v.get_view_model())
        .flatten()
        .unwrap();
    let name = model
        .with_downcast::<ViewModel, _>(|v| v.base.name().to_owned())
        .unwrap();
    let second = f
        .with_file(|f| f.create_view_model_instance_for_name(&name))
        .unwrap();
    assert!(ViewModelInstance::replace_view_model_by_name(
        &vm,
        "vm1",
        second.clone()
    ));
    m.advance_and_apply(0.016);
    fire(&second, "tri1");
    m.advance_and_apply(0.016);
    assert_eq!(string(&second, "str1"), "changed-text-1");
    let third = f
        .with_file(|f| f.create_view_model_instance_for_name(&name))
        .unwrap();
    fire(&third, "tri1");
    assert!(ViewModelInstance::replace_view_model_by_name(
        &vm,
        "vm1",
        third.clone()
    ));
    m.advance_and_apply(0.016);
    assert_eq!(string(&third, "str1"), "changed-text-1");
}
#[test]
fn listener_action_fires_each_time() {
    let f = read_file("trigger_based_listeners.riv");
    let a = artboard(&f, "main");
    let m = a.state_machine_at(0).unwrap();
    let vm = first_vm(&f, &a);
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    let vm1 = nested_vm(&vm, "vm1");
    let t1 = trigger(&vm1, "tri1");
    let t2 = trigger(&vm1, "tri2");
    for press in 1..=3 {
        m.with_instance_mut(|m| m.pointer_down(Vec2D::new(25.0, 25.0), 0, PointerButton::Primary));
        m.advance_and_apply(0.016);
        m.with_instance_mut(|m| m.pointer_up(Vec2D::new(25.0, 25.0), 0, PointerButton::Primary));
        m.advance_and_apply(0.016);
        for t in [&t1, &t2] {
            assert_eq!(
                t.with_downcast::<ViewModelInstanceTrigger, _>(|t| t.base.property_value())
                    .unwrap(),
                press
            );
        }
    }
}
#[test]
fn binding_trigger_does_not_fire_target() {
    let f = read_file("custom_property_trigger.riv");
    let first = artboard(&f, "Main");
    let vm = default_vm(&f, &first);
    let m = first.default_state_machine_handle().unwrap();
    bind(&m, &vm);
    m.advance_and_apply(0.0);
    let item = nested_vm(&vm, "property of ItemVM");
    fire(&item, "ItemTrigger");
    m.advance_and_apply(0.016);
    assert_ne!(
        trigger(&item, "ItemTrigger")
            .with_downcast::<ViewModelInstanceTrigger, _>(|t| t.base.property_value())
            .unwrap(),
        0
    );
    let second = artboard(&f, "Main");
    let t = second
        .with_artboard(|a| a.find_handle::<CustomPropertyTrigger>("Trig"))
        .unwrap();
    let m2 = second.default_state_machine_handle().unwrap();
    bind(&m2, &vm);
    m2.advance_and_apply(0.0);
    assert_eq!(
        t.with_downcast::<CustomPropertyTrigger, _>(|t| t.property_value())
            .unwrap(),
        0
    );
    fire(&item, "ItemTrigger");
    m2.advance_and_apply(0.016);
    assert_ne!(
        t.with_downcast::<CustomPropertyTrigger, _>(|t| t.property_value())
            .unwrap(),
        0
    );
}
fn component_fire(t: &CoreHandle) {
    use nuxie_runtime::source::core::field_types::core_callback_type::CallbackData;
    t.with_downcast_mut::<CustomPropertyTrigger, _>(|t| t.fire(&CallbackData::new(None, 0.0)))
        .unwrap();
}
#[test]
fn component_fire_is_pending_only_for_its_frame() {
    let f = read_file("component_trigger_condition.riv");
    let a = artboard(&f, "Main");
    let press = a
        .with_artboard(|a| a.find_handle::<CustomPropertyTrigger>("Press"))
        .unwrap();
    let release = a
        .with_artboard(|a| a.find_handle::<CustomPropertyTrigger>("Release"))
        .unwrap();
    let m = a.state_machine_at(0).unwrap();
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 0), "idle");
    for _ in 0..2 {
        component_fire(&press);
        m.advance_and_apply(0.016);
        assert_eq!(state(&m, 0), "pressed");
        m.advance_and_apply(0.016);
        assert_eq!(state(&m, 0), "pressed");
        component_fire(&release);
        m.advance_and_apply(0.016);
        assert_eq!(state(&m, 0), "idle");
        m.advance_and_apply(0.016);
        assert_eq!(state(&m, 0), "idle");
    }
}
#[test]
fn component_fire_before_first_advance_reaches_new_machine() {
    let f = read_file("component_trigger_condition.riv");
    let a = artboard(&f, "Main");
    let press = a
        .with_artboard(|a| a.find_handle::<CustomPropertyTrigger>("Press"))
        .unwrap();
    component_fire(&press);
    let m = a.state_machine_at(0).unwrap();
    m.advance_and_apply(0.0);
    assert_eq!(state(&m, 0), "idle");
    let fresh = a.state_machine_at(0).unwrap();
    component_fire(&press);
    fresh.advance_and_apply(0.0);
    assert_eq!(state(&fresh, 0), "pressed");
}

fn row_states(artboard: &RuntimeArtboardInstanceHandle) -> Vec<String> {
    let list = artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("Rows"))
        .expect("Rows list");
    let count = list
        .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::artboard_count)
        .unwrap();
    (0..count)
        .map(|index| {
            let machine = list
                .with_downcast::<ArtboardComponentList, _>(|list| {
                    list.state_machine_instance(index as i32)
                })
                .unwrap();
            machine.map_or_else(String::new, |machine| state(&machine, 0))
        })
        .collect()
}

#[test]
fn fire_before_lists_first_advance_reaches_rows_it_creates() {
    let file = read_file("list_item_parent_trigger.riv");
    let artboard = artboard(&file, "Main");
    let machine = artboard.state_machine_at(0).unwrap();
    let view_model = default_vm(&file, &artboard);
    bind(&machine, &view_model);
    fire(&view_model, "play");
    machine.advance_and_apply(0.0);
    assert_eq!(state(&machine, 0), "played");
    assert_eq!(row_states(&artboard), ["played", "played"]);
}

#[test]
fn host_without_value_conditions_sets_rows_frame_through_both_binding_paths() {
    let file = read_file("list_item_parent_trigger.riv");
    for to_data_context in [false, true] {
        let artboard = artboard(&file, "Quiet");
        let machine = artboard.state_machine_at(0).unwrap();
        let view_model = default_vm(&file, &artboard);
        if to_data_context {
            machine.with_instance_mut(|machine| {
                machine.bind_data_context(RuntimeDataContextHandle::new(DataContext::new(Some(
                    view_model.clone(),
                ))));
            });
        } else {
            bind(&machine, &view_model);
        }
        fire(&view_model, "play");
        machine.advance_and_apply(0.0);
        assert_eq!(
            row_states(&artboard),
            ["played", "played"],
            "data-context binding: {to_data_context}"
        );
    }
}

#[test]
fn fire_reaches_row_added_in_its_frame_but_not_later_frame() {
    let file = read_file("list_item_parent_trigger.riv");
    let artboard = artboard(&file, "Main");
    let machine = artboard.state_machine_at(0).unwrap();
    let view_model = default_vm(&file, &artboard);
    bind(&machine, &view_model);
    machine.advance_and_apply(0.0);
    assert_eq!(row_states(&artboard), ["waiting", "waiting"]);

    let items = property(&view_model, "items");
    let add_row = || {
        let item = items
            .insert_sibling(ViewModelInstanceListItem::default())
            .unwrap();
        let instance = file
            .with_file(|file| {
                file.create_default_view_model_instance(file.view_model_named("Item").unwrap())
            })
            .unwrap();
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
            item.set_view_model_instance(Some(instance));
        })
        .unwrap();
        items
            .with_downcast_mut::<ViewModelInstanceList, _>(|items| items.add_item(item))
            .unwrap();
    };
    fire(&view_model, "play");
    add_row();
    machine.advance_and_apply(0.016);
    assert_eq!(row_states(&artboard), ["played", "played", "played"]);

    add_row();
    machine.advance_and_apply(0.016);
    assert_eq!(
        row_states(&artboard),
        ["played", "played", "played", "waiting"]
    );
}
