//! Virtual converter unbind, released receiver, and ownership contracts at pinned160085.
use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::{
        converters::{
            data_converter_group::DataConverterGroup,
            data_converter_group_item::DataConverterGroupItem,
            data_converter_rounder::DataConverterRounder,
        },
        data_values::{data_type::DataType, data_value::DataValue},
    },
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject, DataConverterCapability},
        data_bind::converters::data_converter_base::DataConverterBase,
    },
};
use std::{any::Any, cell::RefCell, rc::Rc};

struct OutputProbe {
    base: DataConverterRounder,
    label: &'static str,
    hook: Option<Rc<dyn Fn()>>,
    log: Rc<RefCell<Vec<&'static str>>>,
}
impl OutputProbe {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataConverterBase::is_type_of(key)
    }
}
impl CoreCapabilities for OutputProbe {
    fn as_data_converter(
        &self,
    ) -> Option<&nuxie_runtime::source::data_bind::converters::data_converter::DataConverter> {
        Some(&self.base.base.base)
    }
    fn as_data_converter_mut(
        &mut self,
    ) -> Option<&mut nuxie_runtime::source::data_bind::converters::data_converter::DataConverter>
    {
        Some(&mut self.base.base.base)
    }
    fn as_data_converter_capability(&self) -> Option<&dyn DataConverterCapability> {
        Some(self)
    }
    fn as_data_converter_capability_mut(&mut self) -> Option<&mut dyn DataConverterCapability> {
        Some(self)
    }
}
impl CoreObject for OutputProbe {
    fn core(&self) -> &Core {
        self.base.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.base.core_mut()
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
        self.base.deserialize(key, reader)
    }
}
impl CoreRegistryObject for OutputProbe {
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
        self.base.set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: CoreField,
        value: bool,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base.set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: CoreField,
        value: CallbackData<'_>,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.base.set_int_with_completion(field, value, completion);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.base.get_uint(field)
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.base.get_string(field)
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.base.get_color(field)
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.base.get_bool(field)
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.base.get_double(field)
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.base.get_int(field)
    }
}
impl DataConverterCapability for OutputProbe {
    fn convert(
        &mut self,
        value: &dyn DataValue,
        _: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(value);
    }
    fn reverse_convert(
        &mut self,
        value: &dyn DataValue,
        _: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(value);
    }
    fn output_type(&self) -> DataType {
        DataType::Number
    }
    fn bind_context_handler(
        &self,
    ) -> nuxie_runtime::source::data_bind::converters::data_converter::ConverterBindContextHandler
    {
        nuxie_runtime::source::data_bind::converters::data_converter::DataConverter::bind_from_context_handle
    }
    fn unbind(&mut self) {
        self.log.borrow_mut().push(self.label);
        if let Some(hook) = &self.hook {
            hook();
        }
        self.base.base.base.unbind();
    }
    fn update(&mut self) {
        self.base.base.base.update();
    }
    fn reset(&mut self) {
        self.base.base.base.reset();
    }
    fn advance(&mut self, elapsed: f32) -> bool {
        self.base.base.base.advance(elapsed)
    }
}
fn probe(
    arena: &CoreArena,
    label: &'static str,
    log: &Rc<RefCell<Vec<&'static str>>>,
    hook: Option<Rc<dyn Fn()>>,
) -> CoreHandle {
    arena.insert(OutputProbe {
        base: DataConverterRounder::default(),
        label,
        log: log.clone(),
        hook,
    })
}
fn item(arena: &CoreArena, converter: CoreHandle) -> CoreHandle {
    let mut item = DataConverterGroupItem::default();
    item.set_converter(Some(converter));
    arena.insert(item)
}
fn group(arena: &CoreArena, items: &[CoreHandle]) -> CoreHandle {
    let mut group = DataConverterGroup::default();
    for item in items {
        group.add_item(item.clone());
    }
    arena.insert(group)
}

use nuxie_runtime::source::{
    data_bind::{
        converters::{
            data_converter::{DataConverter, bind_converter_context},
            data_converter_formula::DataConverterFormula,
        },
        data_bind::{BINDINGS_TARGET, DataBind},
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    data_bind_flags::DataBindFlags,
    generated::{
        core_registry::CoreRegistry,
        data_bind::converters::data_converter_formula_base::DataConverterFormulaBase,
        scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    },
    scripted::scripted_data_converter::ScriptedDataConverter,
    viewmodel::{
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_value::ValueDependentHandle,
    },
};

fn observing_child(
    arena: &CoreArena,
    owner: &CoreHandle,
    key: u16,
    converter: Option<CoreHandle>,
) -> CoreHandle {
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
        let child = child.as_data_bind_mut().unwrap();
        child.set_target(Some(owner.clone()));
        child.set_converter(converter);
        assert!(child.target_supports_push());
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
fn group_unbind_dispatches_custom_child_virtual_override() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let converter = probe(&arena, "custom-unbind", &log, None);
    let group = group(&arena, &[item(&arena, converter)]);
    group
        .with_downcast_mut::<DataConverterGroup, _>(DataConverterGroup::unbind)
        .unwrap();
    assert_eq!(*log.borrow(), ["custom-unbind"]);
}

#[test]
fn direct_unbind_dispatches_custom_override_and_ignores_retired_generation() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let converter = probe(&arena, "custom-unbind", &log, None);
    DataConverter::unbind_handle(&converter);
    assert_eq!(*log.borrow(), ["custom-unbind"]);
    assert!(converter.remove_occurrence());
    let replacement = probe(&arena, "replacement", &log, None);
    DataConverter::unbind_handle(&converter);
    assert_eq!(*log.borrow(), ["custom-unbind"]);
    assert!(replacement.is_alive());
}

#[test]
fn group_unbind_reads_next_converter_after_callback_before_base_unbind() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let old = probe(&arena, "old", &log, None);
    let next = item(&arena, old);
    let replacement = probe(&arena, "replacement", &log, None);
    let group = group(&arena, &[]);
    let group_for_hook = group.clone();
    let next_for_hook = next.clone();
    let first = probe(
        &arena,
        "first",
        &log,
        Some(Rc::new(move || {
            assert!(
                group_for_hook
                    .data_bind_container()
                    .unwrap()
                    .data_bind_context()
                    .is_some()
            );
            next_for_hook
                .with_downcast_mut::<DataConverterGroupItem, _>(|item| {
                    item.set_converter(Some(replacement.clone()))
                })
                .unwrap();
        })),
    );
    let first_item = item(&arena, first);
    group
        .with_downcast_mut::<DataConverterGroup, _>(|group| {
            group.add_item(first_item);
            group.add_item(next);
        })
        .unwrap();
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    group
        .data_bind_container()
        .unwrap()
        .set_data_bind_context(Some(context.clone()));
    DataConverter::unbind_handle(&group);
    assert_eq!(*log.borrow(), ["first", "replacement"]);
    assert!(
        group
            .data_bind_container()
            .unwrap()
            .data_bind_context()
            .is_none()
    );
    assert_eq!(context.debugging_refcnt(), 1);
}

#[test]
fn formula_detaches_source_and_observer_before_custom_child_then_clears_context() {
    let arena = CoreArena::default();
    let formula = arena.insert(DataConverterFormula::default());
    let source = arena.insert(ViewModelInstanceNumber::default());
    let parent = arena.insert(DataBind::default());
    parent.with_mut(|parent| {
        parent
            .as_data_bind_mut()
            .unwrap()
            .set_source(source.clone())
    });
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    bind_converter_context(&formula, context.clone(), Some(parent));
    let log = Rc::new(RefCell::new(Vec::new()));
    let formula_for_hook = formula.clone();
    let source_for_hook = source.clone();
    let converter = probe(
        &arena,
        "formula-child",
        &log,
        Some(Rc::new(move || {
            // DataBind destruction also unbinds its converter after the parent
            // occurrence retires; the assertions below concern the live call.
            if !formula_for_hook.is_alive() {
                return;
            }
            assert!(
                formula_for_hook
                    .data_bind_container()
                    .unwrap()
                    .data_bind_context()
                    .is_some()
            );
            let attached = source_for_hook.with(|source| source.as_view_model_instance_value().unwrap().dependents().iter().any(|dep| matches!(dep, ValueDependentHandle::Core(owner) if owner == &formula_for_hook))).unwrap();
            assert!(
                !attached,
                "formula source is detached before base bindings unbind"
            );
            assert!(CoreRegistry::set_uint_handle(
                &formula_for_hook,
                i32::from(DataConverterFormulaBase::RANDOM_MODE_VALUE_PROPERTY_KEY),
                2
            ));
        })),
    );
    let key = DataConverterFormulaBase::RANDOM_MODE_VALUE_PROPERTY_KEY;
    let child = observing_child(&arena, &formula, key, Some(converter));
    assert_observing(&formula, &child, key, true);
    DataConverter::unbind_handle(&formula);
    assert_eq!(*log.borrow(), ["formula-child"]);
    assert_observing(&formula, &child, key, false);
    assert_eq!(context.debugging_refcnt(), 1);
}

#[test]
fn scripted_unbind_releases_receiver_then_clears_both_context_owners() {
    let arena = CoreArena::default();
    let scripted = arena.insert(ScriptedDataConverter::default());
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    scripted
        .data_bind_container()
        .unwrap()
        .set_data_bind_context(Some(context.clone()));
    scripted
        .with_downcast_mut::<ScriptedDataConverter, _>(|scripted| {
            scripted.scripted.set_data_context(Some(context.clone()))
        })
        .unwrap();
    assert_eq!(context.debugging_refcnt(), 3);
    let log = Rc::new(RefCell::new(Vec::new()));
    let scripted_for_hook = scripted.clone();
    let converter = probe(
        &arena,
        "scripted-child",
        &log,
        Some(Rc::new(move || {
            if !scripted_for_hook.is_alive() {
                return;
            }
            scripted_for_hook
                .with_downcast_mut::<ScriptedDataConverter, _>(|scripted| {
                    assert!(scripted.data_context().is_some());
                    assert!(scripted.scripted.data_context().is_some());
                })
                .unwrap();
        })),
    );
    let key = ScriptedDataConverterBase::SCRIPT_ASSET_ID_PROPERTY_KEY;
    let child = observing_child(&arena, &scripted, key, Some(converter));
    assert_observing(&scripted, &child, key, true);
    DataConverter::unbind_handle(&scripted);
    assert_eq!(*log.borrow(), ["scripted-child"]);
    assert_observing(&scripted, &child, key, false);
    scripted
        .with_downcast::<ScriptedDataConverter, _>(|scripted| {
            assert!(scripted.data_context().is_none());
            assert!(scripted.scripted.data_context().is_none());
        })
        .unwrap();
    assert_eq!(context.debugging_refcnt(), 1);
}

#[test]
fn lifecycle_builtin_unbind_releases_receiver_for_self_observer() {
    use nuxie_runtime::source::data_bind::converters::{
        data_converter_interpolator::DataConverterInterpolator,
        data_converter_operation_viewmodel::DataConverterOperationViewModel,
    };
    let arena = CoreArena::default();
    for converter in [
        arena.insert(DataConverterRounder::default()),
        arena.insert(DataConverterInterpolator::default()),
        arena.insert(DataConverterOperationViewModel::default()),
    ] {
        let context = RuntimeDataContextHandle::new(DataContext::new(None));
        // Qualified base binding avoids unrelated OperationViewModel lookup.
        DataConverter::bind_from_context_handle(&converter, context.clone(), None);
        let key = DataConverterBase::NAME_PROPERTY_KEY;
        let child = observing_child(&arena, &converter, key, None);
        assert_observing(&converter, &child, key, true);
        DataConverter::unbind_handle(&converter);
        assert_observing(&converter, &child, key, false);
        assert_eq!(context.debugging_refcnt(), 1);
    }
}

#[test]
fn custom_unbind_panic_releases_receiver_and_preserves_pending_base_context() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let converter = probe(
        &arena,
        "panic-child",
        &log,
        Some(Rc::new(|| panic!("custom unbind"))),
    );
    let group = group(&arena, &[item(&arena, converter.clone())]);
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    group
        .data_bind_container()
        .unwrap()
        .set_data_bind_context(Some(context.clone()));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DataConverter::unbind_handle(&group)
    }));
    assert!(result.is_err());
    assert!(
        group
            .data_bind_container()
            .unwrap()
            .data_bind_context()
            .is_some()
    );
    converter
        .with_downcast_mut::<OutputProbe, _>(|probe| probe.hook = None)
        .unwrap();
    DataConverter::unbind_handle(&group);
    assert_eq!(*log.borrow(), ["panic-child", "panic-child"]);
    assert_eq!(context.debugging_refcnt(), 1);
}
