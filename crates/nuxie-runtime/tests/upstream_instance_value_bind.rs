//! All six tests/unit_tests/runtime/instance_value_bind_test.cpp cases at 85d7f952.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_number::ViewModelInstanceNumber,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn file() -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(
        include_bytes!("../../../fixtures/sync/instance_value_binds.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("instance value binds fixture")
}
fn property(instance: &CoreHandle, name: &str) -> CoreHandle {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()
        .expect("named value")
}
fn boolean(instance: &CoreHandle, name: &str) -> CoreHandle {
    let value = property(instance, name);
    assert!(
        value
            .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())
            .is_some()
    );
    value
}
fn number(instance: &CoreHandle, name: &str) -> CoreHandle {
    let value = property(instance, name);
    assert!(
        value
            .with_downcast::<ViewModelInstanceNumber, _>(|_| ())
            .is_some()
    );
    value
}
fn bool_value(value: &CoreHandle) -> bool {
    value
        .with_downcast::<ViewModelInstanceBoolean, _>(|value| value.value())
        .unwrap()
}
fn set_bool(value: &CoreHandle, next: bool) {
    value
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|value| value.set_value(next))
        .unwrap();
}
fn number_value(value: &CoreHandle) -> f32 {
    value
        .with_downcast::<ViewModelInstanceNumber, _>(|value| value.value())
        .unwrap()
}
fn value_binds(instance: &CoreHandle) -> Vec<CoreHandle> {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.value_data_binds().to_vec())
        .unwrap()
}
fn target(bind: &CoreHandle) -> CoreHandle {
    bind.with(|bind| bind.as_data_bind().unwrap().target())
        .flatten()
        .unwrap()
}
fn copy(file: &RuntimeFileHandle) -> CoreHandle {
    file.with_file(|file| file.create_view_model_instance_named("Row", "Flag Row"))
        .unwrap()
}
fn row(
    file: &RuntimeFileHandle,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let artboard = file.with_file(|file| file.artboard_named("Row")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    (artboard, machine)
}
fn instance_binds(artboard: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    artboard
        .core_handle()
        .data_bind_container()
        .unwrap()
        .data_binds()
        .iter()
        .filter(|bind| {
            bind.with(|bind| bind.as_data_bind().unwrap().is_instance_value_bind())
                .unwrap()
        })
        .cloned()
        .collect()
}

#[test]
fn instance_value_binds_import_onto_the_instance_and_clone_with_it() {
    let file = file();
    let model = file.with_file(|file| file.view_model_named("Row")).unwrap();
    // File-level values have no property names yet; match their owner.
    let source = model
        .with_downcast::<ViewModel, _>(|model| model.instance_named("Flag Row"))
        .flatten()
        .unwrap();
    let source_binds = value_binds(&source);
    assert_eq!(source_binds.len(), 1);
    let source_bind = &source_binds[0];
    let source_target = target(source_bind);
    assert!(
        source_target
            .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())
            .is_some()
    );
    assert!(
        source
            .with_downcast::<ViewModelInstance, _>(|instance| instance
                .property_values()
                .contains(&source_target))
            .unwrap()
    );
    assert!(
        source_bind
            .with(|bind| bind.as_data_bind().unwrap().to_source())
            .unwrap()
    );
    assert!(
        source_bind
            .with(|bind| bind.as_data_bind().unwrap().to_target())
            .unwrap()
    );

    let copy = copy(&file);
    let copy_binds = value_binds(&copy);
    assert_eq!(copy_binds.len(), 1);
    assert_ne!(&copy_binds[0], source_bind);
    assert_eq!(target(&copy_binds[0]), boolean(&copy, "on"));

    let converted = model
        .with_downcast::<ViewModel, _>(|model| model.instance_named("Level Row"))
        .flatten()
        .unwrap();
    let converted_binds = value_binds(&converted);
    assert_eq!(converted_binds.len(), 1);
    assert!(
        converted_binds[0]
            .with(|bind| bind.as_data_bind().unwrap().converter().is_some())
            .unwrap()
    );
}

#[test]
fn list_item_instance_value_binds_follow_the_host_both_ways() {
    let file = file();
    let artboard = file.with_file(|file| file.artboard_named("Host")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let host = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    let flag = boolean(&host, "flag");
    let level = number(&host, "level");
    set_bool(&flag, true);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(host));
    machine.advance_and_apply(0.0);
    let list = artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("Rows"))
        .unwrap();
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|list| list.artboard_count()),
        Some(2)
    );
    let item_instance = |index| {
        let item = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.list_item(index))
            .flatten()
            .unwrap();
        item.with(|item| {
            item.as_view_model_instance_list_item()
                .unwrap()
                .view_model_instance()
        })
        .flatten()
        .unwrap()
    };
    let on = boolean(&item_instance(0), "on");
    assert!(bool_value(&on));
    assert!(bool_value(&flag));
    set_bool(&flag, false);
    machine.advance_and_apply(0.016);
    assert!(!bool_value(&on));
    set_bool(&on, true);
    machine.advance_and_apply(0.016);
    assert!(bool_value(&flag));
    let amount = number(&item_instance(1), "amount");
    assert_eq!(number_value(&amount), 20.0);
    level
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|level| level.set_value(4.0))
        .unwrap();
    machine.advance_and_apply(0.016);
    assert_eq!(number_value(&amount), 8.0);
}

#[test]
fn rebinding_an_artboard_swaps_its_instance_value_binds() {
    let file = file();
    let (artboard, machine) = row(&file);
    let first = copy(&file);
    let second = copy(&file);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(first));
    machine.advance_and_apply(0.0);
    assert_eq!(instance_binds(&artboard).len(), 1);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(second.clone()));
    machine.advance_and_apply(0.016);
    let binds = instance_binds(&artboard);
    assert_eq!(binds.len(), 1);
    for bind in binds {
        assert_eq!(target(&bind), boolean(&second, "on"));
    }
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(None));
    assert!(instance_binds(&artboard).is_empty());
}

#[test]
fn removing_a_bound_value_drops_the_binds_aimed_at_it() {
    let file = file();
    let (artboard, machine) = row(&file);
    let instance = copy(&file);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    machine.advance_and_apply(0.0);
    assert_eq!(instance_binds(&artboard).len(), 1);
    let on = boolean(&instance, "on");
    let property_id = on
        .with(|on| {
            on.as_view_model_instance_value()
                .unwrap()
                .base
                .view_model_property_id()
        })
        .unwrap();
    assert!(instance.with_downcast_mut::<ViewModelInstance, _>(|instance| instance.remove_value(property_id)).unwrap());
    assert!(value_binds(&instance).is_empty());
    assert!(instance_binds(&artboard).is_empty());
    machine.advance_and_apply(0.016);
}

#[test]
fn swapping_the_contexts_main_instance_resyncs_instance_value_binds() {
    let file = file();
    let (artboard, machine) = row(&file);
    let first = copy(&file);
    let second = copy(&file);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(first.clone()));
    machine.advance_and_apply(0.0);
    assert_eq!(instance_binds(&artboard).len(), 1);
    let context = artboard
        .with_artboard(|artboard| artboard.data_context())
        .unwrap();
    context.set_main_view_model_instance(None);
    assert!(instance_binds(&artboard).is_empty());
    context.set_main_view_model_instance(Some(first));
    assert_eq!(instance_binds(&artboard).len(), 1);
    context.remove_main_view_model_instance();
    assert!(instance_binds(&artboard).is_empty());
    context.set_main_view_model_instance(Some(second.clone()));
    let binds = instance_binds(&artboard);
    assert_eq!(binds.len(), 1);
    assert_eq!(target(&binds[0]), boolean(&second, "on"));
}

#[test]
fn relinking_the_contexts_main_instance_resyncs_instance_value_binds() {
    let file = file();
    let (artboard, machine) = row(&file);
    let first = copy(&file);
    let second = copy(&file);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(first));
    machine.advance_and_apply(0.0);
    let context = artboard
        .with_artboard(|artboard| artboard.data_context())
        .unwrap();
    context.set_view_model_instance(Some(second.clone()));
    let binds = instance_binds(&artboard);
    for bind in &binds {
        assert_eq!(target(bind), boolean(&second, "on"));
    }
    assert_eq!(binds.len(), 1);
}
