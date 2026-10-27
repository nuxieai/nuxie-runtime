//! Literal viewmodel_instance_parents_test.cpp through upstream a9239045.
use nuxie_runtime::source::{
    core::CoreArena,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};

#[test]
fn a_list_item_popped_off_loses_the_lists_view_model_as_its_parent() {
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let mut list = ViewModelInstanceList::default();
    list.set_parent_view_model_instance(Some(owner.clone()));
    let child = arena.insert(ViewModelInstance::default());
    let mut item = ViewModelInstanceListItem::default();
    item.set_view_model_instance(Some(child.clone()));
    let item = arena.insert(item);
    list.add_item(item.clone());
    assert_eq!(
        child
            .with_downcast::<ViewModelInstance, _>(|child| child.parents())
            .unwrap(),
        vec![owner]
    );
    assert_eq!(list.pop(), Some(item));
    assert!(
        !child
            .with_downcast::<ViewModelInstance, _>(|child| child.has_parents())
            .unwrap()
    );
}

#[test]
fn view_model_instance_keeps_its_other_parents_when_the_first_goes() {
    let arena = CoreArena::default();
    let mut child = ViewModelInstance::default();
    let a = arena.insert(ViewModelInstance::default());
    let b = arena.insert(ViewModelInstance::default());
    let c = arena.insert(ViewModelInstance::default());
    child.add_parent(a.clone());
    child.add_parent(b.clone());
    child.add_parent(c.clone());
    child.add_parent(b.clone());
    assert_eq!(child.parents(), vec![a.clone(), b.clone(), c.clone()]);
    child.remove_parent(&a);
    assert!(child.has_parents());
    let mut parents = child.parents();
    parents.sort_by_key(|p| p.identity_key());
    let mut expected = vec![b.clone(), c.clone()];
    expected.sort_by_key(|p| p.identity_key());
    assert_eq!(parents, expected);
    child.add_parent(a.clone());
    child.remove_parent(&c);
    child.remove_parent(&b);
    assert_eq!(child.parents(), vec![a.clone()]);
    child.remove_parent(&a);
    assert!(!child.has_parents());
}

#[test]
fn a_list_kept_past_its_view_model_stops_pointing_at_it() {
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
    let child = arena.insert(ViewModelInstance::default());
    let mut item = ViewModelInstanceListItem::default();
    item.set_view_model_instance(Some(child.clone()));
    let item = arena.insert(item);
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
        .unwrap();
    assert_eq!(
        child
            .with_downcast::<ViewModelInstance, _>(|child| child.parents())
            .unwrap(),
        vec![owner.clone()]
    );
    assert!(owner.remove_occurrence());
    list.with_downcast::<ViewModelInstanceList, _>(|list| {
        assert!(list.parent_view_model_instance().is_none());
        assert!(list.base.view_model_instance().is_none());
    })
    .unwrap();
    assert!(
        !child
            .with_downcast::<ViewModelInstance, _>(|child| child.has_parents())
            .unwrap()
    );
    let later = arena.insert(ViewModelInstance::default());
    let mut later_item = ViewModelInstanceListItem::default();
    later_item.set_view_model_instance(Some(later.clone()));
    let later_item = arena.insert(later_item);
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(later_item))
        .unwrap();
    assert!(
        !later
            .with_downcast::<ViewModelInstance, _>(|later| later.has_parents())
            .unwrap()
    );
}

#[test]
fn a_view_model_property_kept_past_its_view_model_stops_pointing_at_it() {
    use nuxie_runtime::source::viewmodel::viewmodel_instance_viewmodel::ViewModelInstanceViewModel;
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let property = arena.insert(ViewModelInstanceViewModel::default());
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(property.clone()))
        .unwrap();
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            property.set_parent_view_model_instance(Some(owner.clone()));
            property.set_reference_view_model_instance(Some(
                arena.insert(ViewModelInstance::default()),
            ));
        })
        .unwrap();
    assert!(owner.remove_occurrence());
    let later = arena.insert(ViewModelInstance::default());
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            assert!(property.parent_view_model_instance().is_none());
            assert!(property.base.view_model_instance().is_none());
            property.set_reference_view_model_instance(Some(later.clone()));
        })
        .unwrap();
    assert!(
        !later
            .with_downcast::<ViewModelInstance, _>(|later| later.has_parents())
            .unwrap()
    );
    let applied = arena.insert(ViewModelInstance::default());
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            property.update_view_model(applied);
            assert_eq!(property.reference_view_model_instance(), Some(later));
        })
        .unwrap();
}
