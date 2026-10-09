use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    },
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_value::ViewModelInstanceValue,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use std::{any::Any, cell::RefCell, rc::Rc};
struct NumberCloneProbe {
    inner: ViewModelInstanceNumber,
    callback: Rc<dyn Fn(&CoreHandle)>,
}
impl NumberCloneProbe {
    fn subtype(key: u16) -> bool {
        key == 65533 || ViewModelInstanceNumberBase::is_type_of(key)
    }
}
impl CoreCapabilities for NumberCloneProbe {
    fn as_view_model_instance_value(&self) -> Option<&ViewModelInstanceValue> {
        Some(&self.inner.base.base)
    }
    fn as_view_model_instance_value_mut(&mut self) -> Option<&mut ViewModelInstanceValue> {
        Some(&mut self.inner.base.base)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, cloned| {
            let callback = source
                .with_downcast::<NumberCloneProbe, _>(|source| source.callback.clone())
                .unwrap();
            callback(cloned);
            true
        })
    }
}
impl CoreObject for NumberCloneProbe {
    fn core(&self) -> &Core {
        self.inner.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.inner.core_mut()
    }
    fn core_type(&self) -> u16 {
        65533
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(Box::new(ViewModelInstanceNumber::default()))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for NumberCloneProbe {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: CoreField,
        value: bool,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: CoreField,
        value: CallbackData<'_>,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner.set_int_with_completion(field, value, completion);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.inner.get_uint(field)
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.inner.get_string(field)
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.inner.get_color(field)
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.inner.get_bool(field)
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.inner.get_double(field)
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.inner.get_int(field)
    }
}

fn property(arena: &CoreArena, owner: &CoreHandle, callback: Rc<dyn Fn()>) -> CoreHandle {
    let value = arena.insert(NumberCloneProbe {
        inner: ViewModelInstanceNumber::default(),
        callback: Rc::new(move |_| callback()),
    });
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(value.clone()))
        .unwrap();
    value
}
fn instance(arena: &CoreArena, model: &CoreHandle) -> CoreHandle {
    let owner = arena.insert(ViewModelInstance::default());
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.view_model(model.clone()))
        .unwrap();
    owner
}
#[test]
fn clone_reads_view_model_after_open_child_clone() {
    let arena = CoreArena::default();
    let before = arena.insert(ViewModel::default());
    let after = arena.insert(ViewModel::default());
    let owner = instance(&arena, &before);
    let source = owner.clone();
    let changed = after.clone();
    property(
        &arena,
        &owner,
        Rc::new(move || {
            source
                .with_downcast_mut::<ViewModelInstance, _>(|source| {
                    source.view_model(changed.clone())
                })
                .unwrap();
        }),
    );
    let cloned = owner.clone_occurrence().unwrap();
    assert_eq!(
        cloned.with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model),
        Some(Some(after))
    );
}
#[test]
fn nested_clone_reads_parent_owner_after_recursive_instance_clone() {
    let arena = CoreArena::default();
    let model = arena.insert(ViewModel::default());
    let before = instance(&arena, &model);
    let after = instance(&arena, &model);
    let nested = instance(&arena, &model);
    let owner = arena.insert(ViewModelInstanceViewModel::default());
    owner
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|owner| {
            owner.base.set_view_model_instance(before.clone());
            owner.set_reference_view_model_instance(Some(nested.clone()));
        })
        .unwrap();
    let source = owner.clone();
    let changed = after.clone();
    property(
        &arena,
        &nested,
        Rc::new(move || {
            source
                .with_downcast_mut::<ViewModelInstanceViewModel, _>(|source| {
                    source.base.set_view_model_instance(changed.clone())
                })
                .unwrap();
        }),
    );
    let cloned = owner.clone_occurrence().unwrap();
    assert_eq!(
        cloned.with_downcast::<ViewModelInstanceViewModel, _>(|cloned| cloned
            .base
            .view_model_instance()),
        Some(Some(after))
    );
}
#[test]
fn clone_reads_binding_membership_after_property_clones() {
    let arena = CoreArena::default();
    let model = arena.insert(ViewModel::default());
    let owner = instance(&arena, &model);
    let bind = arena.insert(DataBind::default());
    let source = owner.clone();
    let added = bind.clone();
    let value = property(
        &arena,
        &owner,
        Rc::new(move || {
            source
                .with_downcast_mut::<ViewModelInstance, _>(|source| {
                    source.add_value_data_bind(added.clone())
                })
                .unwrap();
        }),
    );
    bind.with_downcast_mut::<DataBind, _>(|bind| bind.set_target(Some(value)))
        .unwrap();
    let cloned = owner.clone_occurrence().unwrap();
    let binds = cloned
        .with_downcast::<ViewModelInstance, _>(|cloned| cloned.value_data_binds().to_vec())
        .unwrap();
    assert_eq!(
        binds.len(),
        1,
        "source starts the binding loop after all property clones"
    );
    assert_ne!(binds[0], bind);
}

use nuxie_runtime::source::generated::data_bind::data_bind_base::DataBindBase;
struct BindingCloneProbe {
    inner: DataBind,
    callback: Rc<dyn Fn()>,
}
impl BindingCloneProbe {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataBindBase::is_type_of(key)
    }
}
impl CoreCapabilities for BindingCloneProbe {
    fn as_data_bind(&self) -> Option<&DataBind> {
        Some(&self.inner)
    }
    fn as_data_bind_mut(&mut self) -> Option<&mut DataBind> {
        Some(&mut self.inner)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, _cloned| {
            let callback = source
                .with_downcast::<BindingCloneProbe, _>(|source| source.callback.clone())
                .unwrap();
            callback();
            true
        })
    }
}
impl CoreObject for BindingCloneProbe {
    fn core(&self) -> &Core {
        self.inner.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.inner.core_mut()
    }
    fn core_type(&self) -> u16 {
        65533
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(Box::new(DataBind::default()))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for BindingCloneProbe {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: CoreField,
        value: bool,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: CoreField,
        value: CallbackData<'_>,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner.set_int_with_completion(field, value, completion);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.inner.get_uint(field)
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.inner.get_string(field)
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.inner.get_color(field)
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.inner.get_bool(field)
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.inner.get_double(field)
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.inner.get_int(field)
    }
}

#[test]
fn clone_remaps_each_binding_against_current_source_property_indices() {
    let arena = CoreArena::default();
    let model = arena.insert(ViewModel::default());
    let owner = instance(&arena, &model);
    let p0 = arena.insert(ViewModelInstanceNumber::default());
    let p1 = arena.insert(ViewModelInstanceNumber::default());
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| {
            owner.add_value(p0);
            owner.add_value(p1.clone());
        })
        .unwrap();
    let source = owner.clone();
    let first = arena.insert(BindingCloneProbe {
        inner: DataBind::default(),
        callback: Rc::new(move || {
            assert!(
                source
                    .with_downcast_mut::<ViewModelInstance, _>(|source| source.remove_value(0))
                    .unwrap()
            );
        }),
    });
    let second = arena.insert(DataBind::default());
    for bind in [&first, &second] {
        bind.with_mut(|bind| {
            bind.as_data_bind_mut()
                .unwrap()
                .set_target(Some(p1.clone()))
        })
        .unwrap();
        owner
            .with_downcast_mut::<ViewModelInstance, _>(|owner| {
                owner.add_value_data_bind(bind.clone())
            })
            .unwrap();
    }
    let cloned = owner.clone_occurrence().unwrap();
    let (properties, binds) = cloned
        .with_downcast::<ViewModelInstance, _>(|cloned| {
            (
                cloned.property_values().to_vec(),
                cloned.value_data_binds().to_vec(),
            )
        })
        .unwrap();
    assert_eq!(binds.len(), 2);
    let target = |bind: &CoreHandle| {
        bind.with(|bind| bind.as_data_bind().unwrap().target())
            .unwrap()
    };
    assert_eq!(target(&binds[0]), Some(properties[1].clone()));
    assert_eq!(
        target(&binds[1]),
        Some(properties[0].clone()),
        "source re-evaluates current property index for the second bind"
    );
}

#[test]
fn clone_preserves_existing_model_fallback_when_source_has_no_model() {
    let arena = CoreArena::default();
    let owner = arena.insert(ViewModelInstance::default());
    let model = arena.insert(ViewModel::default());
    let expected_model = model.clone();
    let captured = Rc::new(RefCell::new(None::<CoreHandle>));
    let capture = captured.clone();
    let value = arena.insert(NumberCloneProbe {
        inner: ViewModelInstanceNumber::default(),
        callback: Rc::new(move |cloned| *capture.borrow_mut() = Some(cloned.clone())),
    });
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value(value.clone()))
        .unwrap();
    let bind = arena.insert(BindingCloneProbe {
        inner: DataBind::default(),
        callback: Rc::new(move || {
            let child = captured.borrow().clone().unwrap();
            let cloned_owner = child
                .with(|child| {
                    child
                        .as_view_model_instance_value()
                        .unwrap()
                        .view_model_instance()
                })
                .flatten()
                .unwrap();
            cloned_owner
                .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.view_model(model.clone()))
                .unwrap();
        }),
    });
    bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_target(Some(value)))
        .unwrap();
    owner
        .with_downcast_mut::<ViewModelInstance, _>(|owner| owner.add_value_data_bind(bind))
        .unwrap();
    let cloned = owner.clone_occurrence().unwrap();
    assert_eq!(
        cloned.with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model),
        Some(Some(expected_model)),
        "preserve Rust fallback: C++ viewModel(nullptr) unconditionally dereferences value"
    );
}

#[test]
fn nested_clone_final_none_clears_owner_installed_during_reference_change() {
    fn install_owner(cloned: &mut ViewModelInstanceViewModel) {
        let reference = cloned.reference_view_model_instance().unwrap();
        cloned.base.set_view_model_instance(reference);
    }
    let arena = CoreArena::default();
    let nested = arena.insert(ViewModelInstance::default());
    let source = arena.insert(ViewModelInstanceViewModel::default());
    source
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|source| {
            source.set_reference_view_model_instance(Some(nested))
        })
        .unwrap();
    let cloned = arena.insert(ViewModelInstanceViewModel::default());
    cloned
        .with_downcast_mut::<ViewModelInstanceViewModel, _>(|cloned| {
            cloned.on_changed(Some(install_owner))
        })
        .unwrap();
    assert!(ViewModelInstanceViewModel::complete_clone(&source, &cloned));
    assert_eq!(
        cloned.with_downcast::<ViewModelInstanceViewModel, _>(|cloned| cloned
            .base
            .view_model_instance()),
        Some(None),
        "source's final nullable assignment clears the callback-installed owner"
    );
}
