//! The four regressions added to viewmodel_instance_replace_test.cpp by d732510c.
use nuxie_runtime::source::{
    component_dirt::ComponentDirt,
    core::{CoreArena, CoreHandle},
    dirtyable::Dirtyable,
    generated::{
        core_registry::CoreRegistry,
        viewmodel::{
            viewmodel_component_base::ViewModelComponentBase,
            viewmodel_instance_base::ViewModelInstanceBase,
            viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase,
        },
    },
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_value::ValueDependentHandle,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
        viewmodel_property_viewmodel::ViewModelPropertyViewModel,
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct RecordingDependent(Rc<Cell<usize>>);
impl Dirtyable for RecordingDependent {
    fn add_dirt(&mut self, _: ComponentDirt, _: bool) {}
}
impl ViewModelValueDependent for RecordingDependent {
    fn relink_data_bind(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn parent_with_property(arena: &CoreArena, initial: &CoreHandle) -> (CoreHandle, CoreHandle) {
    let parent = arena.insert(ViewModelInstance::default());
    let property = arena.insert(ViewModelInstanceViewModel::default());
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            property.set_parent_view_model_instance(Some(parent.clone()));
            property.set_reference_view_model_instance(Some(initial.clone()));
        })
        .unwrap();
    parent
        .with_downcast_mut::<ViewModelInstance, _>(|parent| parent.add_value(property.clone()))
        .unwrap();
    (parent, property)
}

fn dependent(property: &CoreHandle) -> (Rc<RefCell<dyn ViewModelValueDependent>>, Rc<Cell<usize>>) {
    let count = Rc::new(Cell::new(0));
    let dependent: Rc<RefCell<dyn ViewModelValueDependent>> =
        Rc::new(RefCell::new(RecordingDependent(count.clone())));
    property
        .with_mut(|property| {
            property
                .as_view_model_instance_value_mut()
                .unwrap()
                .add_dependent(ValueDependentHandle::runtime(&dependent))
        })
        .unwrap();
    (dependent, count)
}

fn referenced(property: &CoreHandle) -> Option<CoreHandle> {
    property
        .with_downcast::<ViewModelInstanceViewModel, _>(
            ViewModelInstanceViewModel::reference_view_model_instance,
        )
        .unwrap()
}

#[test]
fn replace_view_model_by_property_is_a_no_op_when_the_value_is_unchanged() {
    let arena = CoreArena::default();
    let a = arena.insert(ViewModelInstance::default());
    let (parent, property) = parent_with_property(&arena, &a);
    let (_dependent, count) = dependent(&property);
    assert!(ViewModelInstance::replace_view_model_property_occurrence(
        &parent,
        &property,
        Some(a.clone())
    ));
    assert_eq!(referenced(&property), Some(a));
    assert_eq!(count.get(), 0);
}

#[cfg(feature = "tools")]
thread_local! { static CHANGED_COUNT: Cell<usize> = const { Cell::new(0) }; }
#[cfg(feature = "tools")]
fn count_changed(_: &mut ViewModelInstanceViewModel) {
    CHANGED_COUNT.with(|count| count.set(count.get() + 1));
}
#[cfg(feature = "tools")]
#[test]
fn replace_view_model_by_property_only_notifies_tools_on_a_real_swap() {
    let arena = CoreArena::default();
    let a = arena.insert(ViewModelInstance::default());
    let b = arena.insert(ViewModelInstance::default());
    let (parent, property) = parent_with_property(&arena, &a);
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            property.on_changed(Some(count_changed))
        })
        .unwrap();
    CHANGED_COUNT.with(|count| count.set(0));
    ViewModelInstance::replace_view_model_property_occurrence(&parent, &property, Some(a));
    assert_eq!(CHANGED_COUNT.with(Cell::get), 0);
    ViewModelInstance::replace_view_model_property_occurrence(&parent, &property, Some(b));
    assert_eq!(CHANGED_COUNT.with(Cell::get), 1);
}

fn named_scene(arena: &CoreArena, initial: &CoreHandle) -> (CoreHandle, CoreHandle) {
    let _child_view_model = arena.insert(ViewModel::default());
    let parent_view_model = arena.insert(ViewModel::default());
    let child_property = arena.insert(ViewModelPropertyViewModel::default());
    CoreRegistry::set_string_handle(
        &child_property,
        ViewModelComponentBase::NAME_PROPERTY_KEY as i32,
        "child".to_owned(),
    );
    let id = initial
        .with_downcast::<ViewModelInstance, _>(|instance| instance.base.view_model_id())
        .unwrap();
    CoreRegistry::set_uint_handle(
        &child_property,
        ViewModelPropertyViewModelBase::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY as i32,
        id,
    );
    parent_view_model
        .with_downcast_mut::<ViewModel, _>(|model| model.add_property(child_property.clone()))
        .unwrap();
    let parent = arena.insert(ViewModelInstance::default());
    parent
        .with_downcast_mut::<ViewModelInstance, _>(|parent| parent.view_model(parent_view_model))
        .unwrap();
    let property = arena.insert(ViewModelInstanceViewModel::default());
    property
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|property| {
            property.base.set_view_model_property(child_property);
            property.set_parent_view_model_instance(Some(parent.clone()));
            property.set_reference_view_model_instance(Some(initial.clone()));
        })
        .unwrap();
    parent
        .with_downcast_mut::<ViewModelInstance, _>(|parent| parent.add_value(property.clone()))
        .unwrap();
    (parent, property)
}

#[test]
fn replace_view_model_by_name_is_a_no_op_when_the_value_is_unchanged() {
    let arena = CoreArena::default();
    let a = arena.insert(ViewModelInstance::default());
    CoreRegistry::set_uint_handle(
        &a,
        ViewModelInstanceBase::VIEW_MODEL_ID_PROPERTY_KEY as i32,
        7,
    );
    let (parent, property) = named_scene(&arena, &a);
    let (_dependent, count) = dependent(&property);
    assert!(ViewModelInstance::replace_view_model_by_name(
        &parent,
        "child",
        a.clone()
    ));
    assert_eq!(referenced(&property), Some(a));
    assert_eq!(count.get(), 0);
}

#[test]
fn replace_view_model_by_name_still_swaps_a_different_instance() {
    let arena = CoreArena::default();
    let a = arena.insert(ViewModelInstance::default());
    let b = arena.insert(ViewModelInstance::default());
    for instance in [&a, &b] {
        CoreRegistry::set_uint_handle(
            instance,
            ViewModelInstanceBase::VIEW_MODEL_ID_PROPERTY_KEY as i32,
            7,
        );
    }
    let (parent, property) = named_scene(&arena, &a);
    let (_dependent, count) = dependent(&property);
    assert!(ViewModelInstance::replace_view_model_by_name(
        &parent,
        "child",
        b.clone()
    ));
    assert_eq!(referenced(&property), Some(b));
    assert_eq!(count.get(), 1);
}
