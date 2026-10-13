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
