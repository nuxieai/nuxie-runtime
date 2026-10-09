//! Embedded capabilities must keep released source operations without an exact outer type.
use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::{
        converters::{
            data_converter::{DataConverter, bind_converter_context},
            data_converter_formula::DataConverterFormula,
            data_converter_group::DataConverterGroup,
            data_converter_group_item::DataConverterGroupItem,
            data_converter_rounder::DataConverterRounder,
        },
        data_bind::{BINDINGS_TARGET, DataBind},
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    data_bind_flags::DataBindFlags,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject, DataConverterCapability},
        data_bind::converters::{
            data_converter_base::DataConverterBase,
            data_converter_formula_base::DataConverterFormulaBase,
        },
        scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    },
    scripted::scripted_data_converter::ScriptedDataConverter,
    viewmodel::{
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_value::ValueDependentHandle,
    },
};
use std::any::Any;

struct Projected<T> {
    inner: T,
}
impl<T> Projected<T> {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataConverterBase::is_type_of(key)
    }
}
impl<T: CoreObject + DataConverterCapability> CoreCapabilities for Projected<T> {
    fn as_data_converter(&self) -> Option<&DataConverter> {
        self.inner.as_data_converter()
    }
    fn as_data_converter_mut(&mut self) -> Option<&mut DataConverter> {
        self.inner.as_data_converter_mut()
    }
    fn as_data_converter_capability(&self) -> Option<&dyn DataConverterCapability> {
        Some(&self.inner)
    }
    fn as_data_converter_capability_mut(&mut self) -> Option<&mut dyn DataConverterCapability> {
        Some(&mut self.inner)
    }
}
impl<T: CoreObject + DataConverterCapability> CoreObject for Projected<T> {
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
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl<T: CoreObject + DataConverterCapability> CoreRegistryObject for Projected<T> {
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

fn observing_child(arena: &CoreArena, owner: &CoreHandle, key: u16) -> CoreHandle {
    let child = arena.insert(DataBind::new(
        u32::from(DataBindFlags::TO_SOURCE.0),
        u32::from(key),
        0,
    ));
    owner
        .data_bind_container()
        .unwrap()
        .add_data_bind(child.clone());
    child.with_mut(|child| {
        child
            .as_data_bind_mut()
            .unwrap()
            .set_target(Some(owner.clone()))
    });
    child
}
fn assert_observing(owner: &CoreHandle, child: &CoreHandle, key: u16, observing: bool) {
    child.with_mut(|child| child.as_data_bind_mut().unwrap().set_dirt(0));
    owner.with_mut(|owner| owner.core_mut().notify_property_changed(key));
    assert_eq!(
        child.with(|child| child.as_data_bind().unwrap().dirt() & BINDINGS_TARGET),
        Some(if observing { BINDINGS_TARGET } else { 0 })
    );
}
#[test]
fn projected_group_unbinds_children_and_base_without_holding_the_outer_receiver() {
    let arena = CoreArena::default();
    let converter = arena.insert(DataConverterRounder::default());
    let mut item = DataConverterGroupItem::default();
    item.set_converter(Some(converter.clone()));
    let mut inner = DataConverterGroup::default();
    inner.add_item(arena.insert(item));
    let owner = arena.insert(Projected { inner });
    assert!(
        owner
            .with_downcast::<DataConverterGroup, _>(|_| ())
            .is_none()
    );
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    DataConverter::bind_from_context_handle(&owner, context.clone(), None);
    bind_converter_context(&converter, context.clone(), None);
    assert_eq!(context.debugging_refcnt(), 3);
    let key = DataConverterBase::NAME_PROPERTY_KEY;
    let child = observing_child(&arena, &owner, key);
    assert_observing(&owner, &child, key, true);
    DataConverter::unbind_handle(&owner);
    assert_observing(&owner, &child, key, false);
    assert_eq!(context.debugging_refcnt(), 1);
}
#[test]
fn projected_formula_detaches_its_source_then_unbinds_self_observers() {
    let arena = CoreArena::default();
    let owner = arena.insert(Projected {
        inner: DataConverterFormula::default(),
    });
    let source = arena.insert(ViewModelInstanceNumber::default());
    let parent = arena.insert(DataBind::default());
    parent.with_mut(|parent| {
        parent
            .as_data_bind_mut()
            .unwrap()
            .set_source(source.clone())
    });
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    owner
        .with_downcast_mut::<Projected<DataConverterFormula>, _>(|owner| {
            owner.inner.bind_from_context(context.clone(), Some(parent))
        })
        .unwrap();
    let attached = || {
        source
            .with(|source| {
                source
                    .as_view_model_instance_value()
                    .unwrap()
                    .dependents()
                    .iter()
                    .any(|dep| matches!(dep,ValueDependentHandle::Core(handle) if handle==&owner))
            })
            .unwrap()
    };
    assert!(attached());
    let key = DataConverterFormulaBase::RANDOM_MODE_VALUE_PROPERTY_KEY;
    let child = observing_child(&arena, &owner, key);
    assert_observing(&owner, &child, key, true);
    DataConverter::unbind_handle(&owner);
    assert!(!attached());
    assert_observing(&owner, &child, key, false);
    assert_eq!(context.debugging_refcnt(), 1);
}
#[test]
fn projected_scripted_converter_clears_both_contexts_after_self_observers() {
    let arena = CoreArena::default();
    let owner = arena.insert(Projected {
        inner: ScriptedDataConverter::default(),
    });
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    DataConverter::bind_from_context_handle(&owner, context.clone(), None);
    owner
        .with_downcast_mut::<Projected<ScriptedDataConverter>, _>(|owner| {
            owner.inner.scripted.set_data_context(Some(context.clone()))
        })
        .unwrap();
    let key = ScriptedDataConverterBase::SCRIPT_ASSET_ID_PROPERTY_KEY;
    let child = observing_child(&arena, &owner, key);
    assert_observing(&owner, &child, key, true);
    assert_eq!(context.debugging_refcnt(), 3);
    DataConverter::unbind_handle(&owner);
    assert_observing(&owner, &child, key, false);
    owner
        .with_downcast::<Projected<ScriptedDataConverter>, _>(|owner| {
            assert!(owner.inner.scripted.data_context().is_none())
        })
        .unwrap();
    assert_eq!(context.debugging_refcnt(), 1);
}
