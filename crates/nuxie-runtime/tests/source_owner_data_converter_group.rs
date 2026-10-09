//! Source-contract regressions for DataConverterGroup at pinned160085.
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
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

struct OutputProbe {
    base: DataConverterRounder,
    label: &'static str,
    outputs: Vec<DataType>,
    calls: Cell<usize>,
    log: Rc<RefCell<Vec<&'static str>>>,
    replacement: Option<(CoreHandle, CoreHandle)>,
}
impl OutputProbe {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataConverterBase::is_type_of(key)
    }
}
impl CoreCapabilities for OutputProbe {
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
        let call = self.calls.get();
        self.calls.set(call + 1);
        self.log.borrow_mut().push(self.label);
        if call == 0 {
            if let Some((item, next)) = &self.replacement {
                item.with_downcast_mut::<DataConverterGroupItem, _>(|item| {
                    item.set_converter(Some(next.clone()))
                })
                .unwrap();
            }
        }
        self.outputs[call.min(self.outputs.len() - 1)]
    }
    nuxie_runtime::data_converter_capability_lifecycle!(base.base.base);
}
fn probe(
    arena: &CoreArena,
    label: &'static str,
    outputs: Vec<DataType>,
    log: &Rc<RefCell<Vec<&'static str>>>,
    replacement: Option<(CoreHandle, CoreHandle)>,
) -> CoreHandle {
    arena.insert(OutputProbe {
        base: DataConverterRounder::default(),
        label,
        outputs,
        calls: Cell::new(0),
        log: log.clone(),
        replacement,
    })
}
fn item(arena: &CoreArena, converter: CoreHandle, owns: bool) -> CoreHandle {
    let mut item = DataConverterGroupItem::default();
    item.set_converter(Some(converter));
    item.set_owns_converter(owns);
    arena.insert(item)
}
fn group(arena: &CoreArena, items: &[CoreHandle]) -> CoreHandle {
    let mut group = DataConverterGroup::default();
    for item in items {
        group.add_item(item.clone());
    }
    arena.insert(group)
}
fn output(group: &CoreHandle) -> DataType {
    group
        .with(|object| object.as_data_converter_capability().unwrap().output_type())
        .unwrap()
}

#[test]
fn selected_output_type_is_queried_again_after_the_predicate() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let converter = probe(
        &arena,
        "selected",
        vec![DataType::Number, DataType::String],
        &log,
        None,
    );
    let item = item(&arena, converter, false);
    let group = group(&arena, &[item]);
    assert_eq!(output(&group), DataType::String);
    assert_eq!(*log.borrow(), ["selected", "selected"]);
}

#[test]
fn selected_converter_is_read_fresh_for_the_return_query() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let replacement = probe(&arena, "replacement", vec![DataType::Boolean], &log, None);
    let item = arena.insert(DataConverterGroupItem::default());
    let first = probe(
        &arena,
        "predicate",
        vec![DataType::Number],
        &log,
        Some((item.clone(), replacement)),
    );
    item.with_downcast_mut::<DataConverterGroupItem, _>(|item| item.set_converter(Some(first)))
        .unwrap();
    let group = group(&arena, &[item]);
    assert_eq!(output(&group), DataType::Boolean);
    assert_eq!(*log.borrow(), ["predicate", "replacement"]);
}

#[test]
fn output_search_is_reverse_order_and_does_not_requery_input_children() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let selected = probe(&arena, "selected", vec![DataType::Number], &log, None);
    let input = probe(&arena, "input", vec![DataType::Input], &log, None);
    let group = group(
        &arena,
        &[item(&arena, selected, false), item(&arena, input, false)],
    );
    assert_eq!(output(&group), DataType::Number);
    assert_eq!(*log.borrow(), ["input", "selected", "selected"]);
}

#[test]
fn deleting_group_retires_owned_items_and_only_owned_converters() {
    let arena = CoreArena::default();
    let borrowed_converter = arena.insert(DataConverterRounder::default());
    let owned_converter = arena.insert(DataConverterRounder::default());
    let borrowed_item = item(&arena, borrowed_converter.clone(), false);
    let owned_item = item(&arena, owned_converter.clone(), true);
    let group = group(&arena, &[borrowed_item.clone(), owned_item.clone()]);
    assert!(group.remove_occurrence());
    assert!(
        !borrowed_item.is_alive(),
        "Group destructor deletes each item"
    );
    assert!(!owned_item.is_alive(), "Group destructor deletes each item");
    assert!(
        borrowed_converter.is_alive(),
        "authored item only borrows converter"
    );
    assert!(
        !owned_converter.is_alive(),
        "owned item destructor deletes cloned converter"
    );
}

impl Drop for OutputProbe {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.label);
    }
}

#[test]
fn empty_and_input_only_groups_keep_the_super_output() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let empty = group(&arena, &[]);
    assert_eq!(output(&empty), DataType::None);
    let input = probe(&arena, "input", vec![DataType::Input], &log, None);
    let only_input = group(&arena, &[item(&arena, input, false)]);
    assert_eq!(output(&only_input), DataType::None);
    assert_eq!(*log.borrow(), ["input"]);
}

#[test]
fn second_query_returning_input_does_not_resume_the_search() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let earlier = probe(&arena, "earlier", vec![DataType::String], &log, None);
    let selected = probe(
        &arena,
        "selected",
        vec![DataType::Number, DataType::Input],
        &log,
        None,
    );
    let group = group(
        &arena,
        &[item(&arena, earlier, false), item(&arena, selected, false)],
    );
    assert_eq!(output(&group), DataType::Input);
    assert_eq!(*log.borrow(), ["selected", "selected"]);
}

#[test]
fn explicit_group_removal_destroys_owned_converters_in_item_order() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = probe(&arena, "first", vec![DataType::Number], &log, None);
    let second = probe(&arena, "second", vec![DataType::Number], &log, None);
    let group = group(
        &arena,
        &[item(&arena, second, true), item(&arena, first, true)],
    );
    assert!(group.remove_occurrence());
    assert_eq!(*log.borrow(), ["second", "first"]);
}

#[test]
fn cloned_group_owns_independent_items_and_converters_and_skips_empty_items() {
    use nuxie_runtime::source::generated::core_registry::CoreRegistry;
    let arena = CoreArena::default();
    let original_converter = arena.insert(DataConverterRounder::default());
    let original_item = item(&arena, original_converter.clone(), false);
    CoreRegistry::set_uint_handle(&original_item, 679, 7);
    let empty_item = arena.insert(DataConverterGroupItem::default());
    let original = group(&arena, &[original_item.clone(), empty_item]);
    let cloned = original.clone_occurrence().unwrap();
    let cloned_items = cloned
        .with_downcast::<DataConverterGroup, _>(|group| group.items().to_vec())
        .unwrap();
    assert_eq!(cloned_items.len(), 1);
    let cloned_item = &cloned_items[0];
    assert_ne!(cloned_item.identity_key(), original_item.identity_key());
    assert_eq!(CoreRegistry::get_uint_handle(cloned_item, 679), Some(7));
    let cloned_converter = cloned_item
        .with_downcast::<DataConverterGroupItem, _>(DataConverterGroupItem::converter)
        .flatten()
        .unwrap();
    assert_ne!(
        cloned_converter.identity_key(),
        original_converter.identity_key()
    );
    assert!(cloned.remove_occurrence());
    assert!(!cloned_item.is_alive());
    assert!(!cloned_converter.is_alive());
    assert!(original_item.is_alive());
    assert!(original_converter.is_alive());
    assert!(original.remove_occurrence());
    assert!(!original_item.is_alive());
    assert!(original_converter.is_alive());
}

#[test]
fn whole_arena_teardown_retires_all_owned_converters() {
    let arena = CoreArena::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = probe(&arena, "first", vec![DataType::Number], &log, None);
    let second = probe(&arena, "second", vec![DataType::Number], &log, None);
    let group = group(
        &arena,
        &[
            item(&arena, second.clone(), true),
            item(&arena, first.clone(), true),
        ],
    );
    drop(arena);
    assert!(!group.is_alive());
    assert!(!first.is_alive());
    assert!(!second.is_alive());
    let observed = log.borrow().clone();
    println!(
        "whole-arena observed converter destruction order: {observed:?}; source group order: [second, first]"
    );
    let mut destroyed = observed;
    destroyed.sort_unstable();
    assert_eq!(destroyed, ["first", "second"]);
}
