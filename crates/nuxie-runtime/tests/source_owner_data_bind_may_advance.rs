//! Independent mayAdvance callback reproduction; no membership mutation.
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
    result: bool,
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
    fn may_advance(&self) -> bool {
        if let Some(hook) = &self.hook {
            hook();
        }
        self.result
    }
    fn advance(&mut self, elapsed: f32) -> bool {
        self.base.base.base.advance(elapsed)
    }
}

use nuxie_runtime::source::data_bind::{
    data_bind::DataBind, data_bind_container::DataBindContainer,
};
#[test]
fn may_advance_callback_can_dirty_current_bind_without_changing_membership() {
    let arena = CoreArena::default();
    let container = DataBindContainer::default();
    let bind = arena.insert(DataBind::default());
    let entered = Rc::new(std::cell::Cell::new(false));
    let hook_container = container.clone();
    let hook_bind = bind.clone();
    let hook_entered = entered.clone();
    let converter = arena.insert(OutputProbe {
        base: DataConverterRounder::default(),
        label: "probe",
        result: true,
        log: Rc::default(),
        hook: Some(Rc::new(move || {
            hook_entered.set(true);
            hook_container.add_dirty_data_bind(hook_bind.clone());
        })),
    });
    bind.with_mut(|bind| {
        bind.as_data_bind_mut()
            .unwrap()
            .set_converter(Some(converter))
    });
    container.add_data_bind(bind.clone());
    assert!(!container.has_data_bind_work());
    assert!(container.may_advance_data_binds());
    assert!(entered.get());
    assert_eq!(container.data_binds().as_ref(), &[bind]);
    assert!(container.has_data_bind_work());
}

fn query_probe(
    arena: &CoreArena,
    label: &'static str,
    result: bool,
    log: Rc<RefCell<Vec<&'static str>>>,
    hook: Option<Rc<dyn Fn()>>,
) -> CoreHandle {
    let query_log = log.clone();
    arena.insert(OutputProbe {
        base: DataConverterRounder::default(),
        label,
        result,
        log,
        hook: Some(Rc::new(move || {
            query_log.borrow_mut().push(label);
            if let Some(hook) = &hook {
                hook();
            }
        })),
    })
}

#[test]
fn earlier_query_changes_the_later_bind_converter_before_its_query() {
    let arena = CoreArena::default();
    let container = DataBindContainer::default();
    let first = arena.insert(DataBind::default());
    let second = arena.insert(DataBind::default());
    let log = Rc::new(RefCell::new(Vec::new()));
    let stale = query_probe(&arena, "stale", false, log.clone(), None);
    let fresh = query_probe(&arena, "fresh", true, log.clone(), None);
    second.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_converter(Some(stale)));
    let target = second.clone();
    let before = query_probe(
        &arena,
        "first",
        false,
        log.clone(),
        Some(Rc::new(move || {
            target.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .unwrap()
                    .set_converter(Some(fresh.clone()))
            });
        })),
    );
    first.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_converter(Some(before)));
    container.add_data_bind(first);
    container.add_data_bind(second);
    assert!(container.may_advance_data_binds());
    assert_eq!(&*log.borrow(), &["first", "fresh"]);
}

#[test]
fn first_true_query_short_circuits_later_open_queries() {
    let arena = CoreArena::default();
    let container = DataBindContainer::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    for (label, hook) in [
        ("first", None),
        (
            "unreachable",
            Some(Rc::new(|| -> () { panic!("query must short circuit") }) as Rc<dyn Fn()>),
        ),
    ] {
        let converter = query_probe(&arena, label, true, log.clone(), hook);
        let bind = arena.insert(DataBind::default());
        bind.with_mut(|bind| {
            bind.as_data_bind_mut()
                .unwrap()
                .set_converter(Some(converter))
        });
        container.add_data_bind(bind);
    }
    assert!(container.may_advance_data_binds());
    assert_eq!(&*log.borrow(), &["first"]);
}
