//! Supplemental tools callback/lifetime observations for 75a22f94's index walks.
#![cfg(feature = "tools")]
use nuxie_runtime::source::{
    core::{CoreArena, CoreHandle},
    generated::{
        core_registry::CoreRegistry,
        viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBase,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};
use std::cell::RefCell;
thread_local! { static REMOVE_FROM_LIST: RefCell<Option<CoreHandle>> = const { RefCell::new(None) }; }
thread_local! { static REMOVE_ARTBOARD_PROPERTY: RefCell<Option<(CoreHandle, u32)>> = const { RefCell::new(None) }; }
fn remove_current_value(trigger: &CoreHandle, value: u32) {
    assert_eq!(value, 0);
    let (parent, id) = trigger
        .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| {
            (
                trigger.base.view_model_instance().unwrap(),
                trigger.base.view_model_property_id(),
            )
        })
        .unwrap();
    assert!(
        parent
            .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.remove_value(id))
            .unwrap()
    );
}
fn remove_current_item(_: &CoreHandle, value: u32) {
    assert_eq!(value, 0);
    let list = REMOVE_FROM_LIST.with(|list| list.borrow().clone().unwrap());
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.remove_item_at(0))
        .unwrap();
}
fn remove_enclosing_artboard_property(_: &CoreHandle, value: u32) {
    assert_eq!(value, 0);
    let (owner, id) = REMOVE_ARTBOARD_PROPERTY.with(|slot| slot.borrow().clone().unwrap());
    assert!(
        owner
            .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.remove_value(id))
            .unwrap()
    );
}
fn add_trigger(arena: &CoreArena, owner: &CoreHandle, id: u32) -> CoreHandle {
    let trigger = arena.insert(ViewModelInstanceTrigger::default());
    assert!(CoreRegistry::set_uint_handle(
        &trigger,
        ViewModelInstanceValueBase::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY.into(),
        id
    ));
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(trigger.clone()))
        .unwrap();
    ViewModelInstanceTrigger::trigger_handle(&trigger);
    trigger
}
fn count(trigger: &CoreHandle) -> u32 {
    trigger
        .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| trigger.base.property_value())
        .unwrap()
}
#[test]
fn removing_current_value_during_advance_leaves_shifted_successor_for_next_frame() {
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let first = add_trigger(&arena, &owner, 1);
    let second = add_trigger(&arena, &owner, 2);
    first
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
            trigger.on_changed(Some(remove_current_value))
        })
        .unwrap();
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&first), 0);
    assert_eq!(count(&second), 1);
    assert_eq!(
        owner
            .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())
            .unwrap(),
        vec![second.clone()]
    );
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&second), 0);
}

#[test]
fn current_value_removal_unbinds_its_source_and_target_observer_before_advance_finishes() {
    use nuxie_runtime::source::{
        data_bind::data_bind::{DataBind, TO_SOURCE},
        generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase,
    };
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let trigger = add_trigger(&arena, &owner, 1);
    let successor = add_trigger(&arena, &owner, 2);
    let bind = arena.insert(DataBind::new(
        TO_SOURCE,
        ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        0,
    ));
    bind.with_mut(|object| {
        let binding = object.as_data_bind_mut().unwrap();
        binding.set_target(Some(trigger.clone()));
        binding.set_source(trigger.clone());
        assert!(binding.target_supports_push());
    })
    .unwrap();
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value_data_bind(bind.clone()))
        .unwrap();
    trigger
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
            trigger.on_changed(Some(remove_current_value))
        })
        .unwrap();
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&trigger), 0);
    assert_eq!(count(&successor), 1);
    assert!(
        owner
            .with_downcast::<ViewModelInstance, _>(|owner| owner.value_data_binds().is_empty())
            .unwrap()
    );
    assert!(bind.with(|_| ()).is_none(), "removed binding occurrence");
    trigger
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
            trigger.on_changed(None);
        })
        .unwrap();
    ViewModelInstanceTrigger::trigger_handle(&trigger);
    assert_eq!(count(&trigger), 1);
}
#[test]
fn removing_current_list_item_during_advance_leaves_shifted_successor_for_next_frame() {
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let list = arena.insert(ViewModelInstanceList::default());
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(list.clone()))
        .unwrap();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
        list.set_parent_view_model_instance(Some(owner.clone()))
    })
    .unwrap();
    let mut triggers = Vec::new();
    for id in 1..=2 {
        let instance = arena.insert(ViewModelInstance::default());
        triggers.push(add_trigger(&arena, &instance, id));
        let item = arena.insert(ViewModelInstanceListItem::default());
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
            item.set_view_model_instance(Some(instance))
        })
        .unwrap();
        list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
            .unwrap();
    }
    REMOVE_FROM_LIST.with(|slot| *slot.borrow_mut() = Some(list.clone()));
    triggers[0]
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
            trigger.on_changed(Some(remove_current_item))
        })
        .unwrap();
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&triggers[0]), 0);
    assert_eq!(count(&triggers[1]), 1);
    assert_eq!(
        list.with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            .unwrap(),
        1
    );
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&triggers[1]), 0);
    REMOVE_FROM_LIST.with(|slot| *slot.borrow_mut() = None);
}

#[test]
fn nested_artboard_trigger_can_remove_its_enclosing_property_during_advance() {
    use nuxie_runtime::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard;
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let bound = arena.insert(ViewModelInstance::default());
    let property = arena.insert(ViewModelInstanceArtboard::default());
    assert!(CoreRegistry::set_uint_handle(
        &property,
        ViewModelInstanceValueBase::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY.into(),
        42
    ));
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
            property.set_bound_view_model_instance(Some(bound.clone()));
            property.base.on_value_changed();
        })
        .unwrap();
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(property.clone()))
        .unwrap();
    let next = add_trigger(&arena, &owner, 43);
    let nested = add_trigger(&arena, &bound, 1);
    nested
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| {
            trigger.on_changed(Some(remove_enclosing_artboard_property))
        })
        .unwrap();
    REMOVE_ARTBOARD_PROPERTY.with(|slot| *slot.borrow_mut() = Some((owner.clone(), 42)));
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&nested), 0);
    assert_eq!(count(&next), 1);
    assert!(owner.with_downcast::<ViewModelInstance, _>(|owner| owner.property_values() == [next.clone()]).unwrap());
    assert!(
        !property
            .with_downcast::<ViewModelInstanceArtboard, _>(|property| property.base.has_changed())
            .unwrap()
    );
    ViewModelInstance::advanced_handle(&owner);
    assert_eq!(count(&next), 0);
    REMOVE_ARTBOARD_PROPERTY.with(|slot| *slot.borrow_mut() = None);
}
