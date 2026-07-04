use crate::data_bind_graph::{
    runtime_data_bind_graph_converter,
    runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source,
    runtime_data_bind_graph_converter_starts_with_to_string,
    runtime_data_bind_graph_group_formula_operation_accepts_non_number_source,
    runtime_data_bind_graph_group_operation_formula_accepts_non_number_source,
};
use crate::{
    RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue, RuntimeViewModelPointer,
    property_key_for_name,
};
use rive_binary::{RuntimeFile, RuntimeObject};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableNumber {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableNumberDefaultViewModelSource>,
    pub(crate) value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableNumberDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: RuntimeDataBindGraphValue,
    pub(crate) view_model_instance_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableInteger {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableIntegerDefaultViewModelSource>,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableIntegerDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableColor {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableColorDefaultViewModelSource>,
    pub(crate) value: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableColorDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableString {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableStringDefaultViewModelSource>,
    pub(crate) value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableStringDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableEnum {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableEnumDefaultViewModelSource>,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableEnumDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableAsset {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableAssetDefaultViewModelSource>,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableAssetDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableArtboard {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableArtboardDefaultViewModelSource>,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableArtboardDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableList {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableListDefaultViewModelSource>,
    pub(crate) value: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableListDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableTrigger {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
    pub(crate) source: RuntimeBindableTriggerSource,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableTriggerDefaultViewModelSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeBindableTriggerSource {
    None,
    DefaultViewModelTrigger { trigger_global_id: u32 },
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableTriggerDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableViewModel {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) source: RuntimeBindableViewModelSource,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableViewModelDefaultViewModelSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeBindableViewModelSource {
    Null,
    RootDataContext,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableViewModelDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: RuntimeViewModelPointer,
    pub(crate) view_model_instance_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableBoolean {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) default_view_model_sources: Vec<RuntimeBindableBooleanDefaultViewModelSource>,
    pub(crate) value: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableBooleanDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) value: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeViewModelTrigger {
    pub(crate) global_id: u32,
    pub(crate) view_model_property_id: u32,
    pub(crate) value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableNumberInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: f32,
}

impl StateMachineBindableNumberInstance {
    pub(crate) fn new(bindable_number: &RuntimeBindableNumber) -> Self {
        Self {
            global_id: bindable_number.global_id,
            data_bind_indices: bindable_number.data_bind_indices.clone(),
            value: bindable_number.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_number_value(
    bindable_numbers: &[StateMachineBindableNumberInstance],
    global_id: u32,
) -> Option<f32> {
    bindable_numbers
        .iter()
        .find(|bindable_number| bindable_number.global_id == global_id)
        .map(|bindable_number| bindable_number.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableIntegerInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
}

impl StateMachineBindableIntegerInstance {
    pub(crate) fn new(bindable_integer: &RuntimeBindableInteger) -> Self {
        Self {
            global_id: bindable_integer.global_id,
            data_bind_indices: bindable_integer.data_bind_indices.clone(),
            value: bindable_integer.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_integer_value(
    bindable_integers: &[StateMachineBindableIntegerInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_integers
        .iter()
        .find(|bindable_integer| bindable_integer.global_id == global_id)
        .map(|bindable_integer| bindable_integer.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableColorInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u32,
}

impl StateMachineBindableColorInstance {
    pub(crate) fn new(bindable_color: &RuntimeBindableColor) -> Self {
        Self {
            global_id: bindable_color.global_id,
            data_bind_indices: bindable_color.data_bind_indices.clone(),
            value: bindable_color.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_color_value(
    bindable_colors: &[StateMachineBindableColorInstance],
    global_id: u32,
) -> Option<u32> {
    bindable_colors
        .iter()
        .find(|bindable_color| bindable_color.global_id == global_id)
        .map(|bindable_color| bindable_color.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableStringInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: Vec<u8>,
}

impl StateMachineBindableStringInstance {
    pub(crate) fn new(bindable_string: &RuntimeBindableString) -> Self {
        Self {
            global_id: bindable_string.global_id,
            data_bind_indices: bindable_string.data_bind_indices.clone(),
            value: bindable_string.value.clone(),
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: &[u8]) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value.to_vec();
        true
    }
}

pub(crate) fn bindable_string_value(
    bindable_strings: &[StateMachineBindableStringInstance],
    global_id: u32,
) -> Option<&[u8]> {
    bindable_strings
        .iter()
        .find(|bindable_string| bindable_string.global_id == global_id)
        .map(|bindable_string| bindable_string.value.as_slice())
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableEnumInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
}

impl StateMachineBindableEnumInstance {
    pub(crate) fn new(bindable_enum: &RuntimeBindableEnum) -> Self {
        Self {
            global_id: bindable_enum.global_id,
            data_bind_indices: bindable_enum.data_bind_indices.clone(),
            value: bindable_enum.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_enum_value(
    bindable_enums: &[StateMachineBindableEnumInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_enums
        .iter()
        .find(|bindable_enum| bindable_enum.global_id == global_id)
        .map(|bindable_enum| bindable_enum.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableAssetInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
}

impl StateMachineBindableAssetInstance {
    pub(crate) fn new(bindable_asset: &RuntimeBindableAsset) -> Self {
        Self {
            global_id: bindable_asset.global_id,
            data_bind_indices: bindable_asset.data_bind_indices.clone(),
            value: bindable_asset.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_asset_value(
    bindable_assets: &[StateMachineBindableAssetInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_assets
        .iter()
        .find(|bindable_asset| bindable_asset.global_id == global_id)
        .map(|bindable_asset| bindable_asset.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableArtboardInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
}

impl StateMachineBindableArtboardInstance {
    pub(crate) fn new(bindable_artboard: &RuntimeBindableArtboard) -> Self {
        Self {
            global_id: bindable_artboard.global_id,
            data_bind_indices: bindable_artboard.data_bind_indices.clone(),
            value: bindable_artboard.value,
        }
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }
}

pub(crate) fn bindable_artboard_value(
    bindable_artboards: &[StateMachineBindableArtboardInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_artboards
        .iter()
        .find(|bindable_artboard| bindable_artboard.global_id == global_id)
        .map(|bindable_artboard| bindable_artboard.value)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableListInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) property_value: usize,
}

impl StateMachineBindableListInstance {
    pub(crate) fn new(bindable_list: &RuntimeBindableList) -> Self {
        Self {
            global_id: bindable_list.global_id,
            data_bind_indices: bindable_list.data_bind_indices.clone(),
            property_value: bindable_list.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: usize) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableTriggerInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: u64,
    pub(crate) source: RuntimeBindableTriggerSource,
}

impl StateMachineBindableTriggerInstance {
    pub(crate) fn new(bindable_trigger: &RuntimeBindableTrigger) -> Self {
        Self {
            global_id: bindable_trigger.global_id,
            data_bind_indices: bindable_trigger.data_bind_indices.clone(),
            value: bindable_trigger.value,
            source: bindable_trigger.source,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_trigger_value(
    bindable_triggers: &[StateMachineBindableTriggerInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_triggers
        .iter()
        .find(|bindable_trigger| bindable_trigger.global_id == global_id)
        .map(|bindable_trigger| bindable_trigger.value)
}

pub(crate) fn bindable_trigger_source_global_id(
    bindable_triggers: &[StateMachineBindableTriggerInstance],
    global_id: u32,
) -> Option<u32> {
    bindable_triggers
        .iter()
        .find(|bindable_trigger| bindable_trigger.global_id == global_id)
        .and_then(|bindable_trigger| match bindable_trigger.source {
            RuntimeBindableTriggerSource::DefaultViewModelTrigger { trigger_global_id } => {
                Some(trigger_global_id)
            }
            RuntimeBindableTriggerSource::None => None,
        })
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableViewModelInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    source: RuntimeBindableViewModelSource,
    pub(crate) value: RuntimeViewModelPointer,
}

impl StateMachineBindableViewModelInstance {
    pub(crate) fn new(bindable_view_model: &RuntimeBindableViewModel) -> Self {
        Self {
            global_id: bindable_view_model.global_id,
            data_bind_indices: bindable_view_model.data_bind_indices.clone(),
            source: bindable_view_model.source,
            value: RuntimeViewModelPointer::Null,
        }
    }

    fn pointer(&self, data_context_present: bool) -> RuntimeViewModelPointer {
        match self.source {
            RuntimeBindableViewModelSource::RootDataContext if data_context_present => {
                RuntimeViewModelPointer::DataContextRoot
            }
            RuntimeBindableViewModelSource::RootDataContext => RuntimeViewModelPointer::Null,
            RuntimeBindableViewModelSource::Null => self.value,
        }
    }

    pub(crate) fn set_value(&mut self, value: RuntimeViewModelPointer) {
        self.value = value;
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_imported_value(&mut self, value: RuntimeViewModelPointer) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_view_model_value(
    bindable_view_models: &[StateMachineBindableViewModelInstance],
    global_id: u32,
    data_context_present: bool,
) -> RuntimeViewModelPointer {
    bindable_view_models
        .iter()
        .find(|bindable_view_model| bindable_view_model.global_id == global_id)
        .map(|bindable_view_model| bindable_view_model.pointer(data_context_present))
        .unwrap_or(RuntimeViewModelPointer::Null)
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineBindableBooleanInstance {
    pub(crate) global_id: u32,
    pub(crate) data_bind_indices: Vec<usize>,
    pub(crate) value: bool,
}

impl StateMachineBindableBooleanInstance {
    pub(crate) fn new(bindable_boolean: &RuntimeBindableBoolean) -> Self {
        Self {
            global_id: bindable_boolean.global_id,
            data_bind_indices: bindable_boolean.data_bind_indices.clone(),
            value: bindable_boolean.value,
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: bool) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
}

pub(crate) fn bindable_boolean_value(
    bindable_booleans: &[StateMachineBindableBooleanInstance],
    global_id: u32,
) -> Option<bool> {
    bindable_booleans
        .iter()
        .find(|bindable_boolean| bindable_boolean.global_id == global_id)
        .map(|bindable_boolean| bindable_boolean.value)
}

pub(crate) fn runtime_bindable_numbers(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableNumber> {
    let mut values = BTreeMap::<u32, RuntimeBindableNumber>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyNumber" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_number| bindable_number.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableNumber {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target.double_property("propertyValue").unwrap_or(0.0),
            });
        if let Some(source) =
            runtime_bindable_number_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_number| {
                bindable_number.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_number_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableNumberDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyNumber", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let converter = runtime_data_bind_graph_converter(file, data_bind);
    let value = match converter.as_ref() {
        Some(RuntimeDataBindGraphConverter::ToNumber) => {
            if let Some(value) = file.view_model_instance_number_value_for_object(source) {
                RuntimeDataBindGraphValue::Number(value)
            } else if let Some(value) = file.view_model_instance_boolean_value_for_object(source) {
                RuntimeDataBindGraphValue::Boolean(value)
            } else if source.type_name == "ViewModelInstanceEnum" {
                RuntimeDataBindGraphValue::Enum(source.uint_property("propertyValue")?)
            } else if let Some(value) = file.view_model_instance_color_value_for_object(source) {
                RuntimeDataBindGraphValue::Color(value)
            } else if let Some(value) =
                file.view_model_instance_string_value_bytes_for_object(source)
            {
                RuntimeDataBindGraphValue::String(value.to_vec())
            } else if let Some(value) =
                file.view_model_instance_symbol_list_index_value_for_object(source)
            {
                RuntimeDataBindGraphValue::SymbolListIndex(value)
            } else {
                return None;
            }
        }
        Some(RuntimeDataBindGraphConverter::OperationValue { .. }) => {
            if let Some(value) = file.view_model_instance_number_value_for_object(source) {
                RuntimeDataBindGraphValue::Number(value)
            } else if let Some(value) =
                file.view_model_instance_symbol_list_index_value_for_object(source)
            {
                RuntimeDataBindGraphValue::SymbolListIndex(value)
            } else {
                return None;
            }
        }
        Some(RuntimeDataBindGraphConverter::Formula { .. }) => {
            if let Some(value) = file.view_model_instance_number_value_for_object(source) {
                RuntimeDataBindGraphValue::Number(value)
            } else if let Some(value) =
                file.view_model_instance_symbol_list_index_value_for_object(source)
            {
                RuntimeDataBindGraphValue::SymbolListIndex(value)
            } else if let Some(value) = file.view_model_instance_boolean_value_for_object(source) {
                RuntimeDataBindGraphValue::Boolean(value)
            } else if source.type_name == "ViewModelInstanceEnum" {
                RuntimeDataBindGraphValue::Enum(source.uint_property("propertyValue")?)
            } else if let Some(value) = file.view_model_instance_color_value_for_object(source) {
                RuntimeDataBindGraphValue::Color(value)
            } else if let Some(value) =
                file.view_model_instance_string_value_bytes_for_object(source)
            {
                RuntimeDataBindGraphValue::String(value.to_vec())
            } else if let Some(value) = file.view_model_instance_trigger_count_for_object(source) {
                RuntimeDataBindGraphValue::Trigger(value)
            } else if let Some(item_count) = file.view_model_instance_list_size_for_object(source) {
                RuntimeDataBindGraphValue::List { item_count }
            } else if source.type_name == "ViewModelInstanceAssetImage" {
                RuntimeDataBindGraphValue::Asset(source.uint_property("propertyValue")?)
            } else if source.type_name == "ViewModelInstanceArtboard" {
                RuntimeDataBindGraphValue::Artboard(source.uint_property("propertyValue")?)
            } else if let Some(reference) =
                file.data_context_view_model_instance_for_instance(default_instance.object, &path)
            {
                RuntimeDataBindGraphValue::ViewModel(RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                })
            } else {
                return None;
            }
        }
        Some(RuntimeDataBindGraphConverter::Group(converters))
            if converters.first().is_some_and(
                runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source,
            ) =>
        {
            if let Some(value) = file.view_model_instance_number_value_for_object(source) {
                RuntimeDataBindGraphValue::Number(value)
            } else if let Some(value) =
                file.view_model_instance_symbol_list_index_value_for_object(source)
            {
                RuntimeDataBindGraphValue::SymbolListIndex(value)
            } else if runtime_data_bind_graph_group_operation_formula_accepts_non_number_source(
                converters,
            ) || runtime_data_bind_graph_group_formula_operation_accepts_non_number_source(
                converters,
            ) {
                if let Some(value) = file.view_model_instance_boolean_value_for_object(source) {
                    RuntimeDataBindGraphValue::Boolean(value)
                } else if source.type_name == "ViewModelInstanceEnum" {
                    RuntimeDataBindGraphValue::Enum(source.uint_property("propertyValue")?)
                } else if let Some(value) = file.view_model_instance_color_value_for_object(source)
                {
                    RuntimeDataBindGraphValue::Color(value)
                } else if let Some(value) =
                    file.view_model_instance_string_value_bytes_for_object(source)
                {
                    RuntimeDataBindGraphValue::String(value.to_vec())
                } else if let Some(value) =
                    file.view_model_instance_trigger_count_for_object(source)
                {
                    RuntimeDataBindGraphValue::Trigger(value)
                } else if source.type_name == "ViewModelInstanceAssetImage" {
                    RuntimeDataBindGraphValue::Asset(source.uint_property("propertyValue")?)
                } else if source.type_name == "ViewModelInstanceArtboard" {
                    RuntimeDataBindGraphValue::Artboard(source.uint_property("propertyValue")?)
                } else if let Some(reference) = file
                    .data_context_view_model_instance_for_instance(default_instance.object, &path)
                {
                    RuntimeDataBindGraphValue::ViewModel(RuntimeViewModelPointer::Imported {
                        object_id: reference.object.id,
                    })
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(RuntimeDataBindGraphConverter::ListToLength) => RuntimeDataBindGraphValue::ListLength(
            file.view_model_instance_list_size_for_object(source)?,
        ),
        _ => RuntimeDataBindGraphValue::Number(
            file.view_model_instance_number_value_for_object(source)?,
        ),
    };
    let view_model_instance_ids = if matches!(&value, RuntimeDataBindGraphValue::ViewModel(_)) {
        let reference =
            file.data_context_view_model_instance_for_instance(default_instance.object, &path)?;
        file.view_model(reference.view_model_index)?
            .instances
            .into_iter()
            .map(|instance| instance.object.id)
            .collect()
    } else {
        Vec::new()
    };
    Some(RuntimeBindableNumberDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter,
        value,
        view_model_instance_ids,
    })
}

pub(crate) fn runtime_bindable_integers(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableInteger> {
    let mut values = BTreeMap::<u32, RuntimeBindableInteger>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyInteger" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_integer| bindable_integer.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableInteger {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target.uint_property("propertyValue").unwrap_or(0),
            });
        if let Some(source) =
            runtime_bindable_integer_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_integer| {
                bindable_integer.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_integer_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableIntegerDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyInteger", "propertyValue") != Some(property_key) {
        return None;
    }
    if runtime_data_bind_graph_converter(file, data_bind).is_some() {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let value = file.view_model_instance_symbol_list_index_value_for_object(source)?;
    Some(RuntimeBindableIntegerDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_colors(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableColor> {
    let mut values = BTreeMap::<u32, RuntimeBindableColor>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyColor" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_color| bindable_color.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableColor {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target.color_property("propertyValue").unwrap_or(0),
            });
        if let Some(source) =
            runtime_bindable_color_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_color| {
                bindable_color.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_color_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableColorDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyColor", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let value = file.view_model_instance_color_value_for_object(source)?;
    Some(RuntimeBindableColorDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_strings(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableString> {
    let mut values = BTreeMap::<u32, RuntimeBindableString>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyString" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_string| bindable_string.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableString {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target
                    .string_property_bytes("propertyValue")
                    .unwrap_or_default()
                    .to_vec(),
            });
        if let Some(source) =
            runtime_bindable_string_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_string| {
                bindable_string.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_string_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableStringDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyString", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let converter = runtime_data_bind_graph_converter(file, data_bind);
    let value = if runtime_data_bind_graph_converter_starts_with_to_string(converter.as_ref()) {
        if let Some(value) = file.view_model_instance_number_value_for_object(source) {
            RuntimeDataBindGraphValue::Number(value)
        } else if let Some(value) = file.view_model_instance_boolean_value_for_object(source) {
            RuntimeDataBindGraphValue::Boolean(value)
        } else if let Some(value) = file.view_model_instance_string_value_bytes_for_object(source) {
            RuntimeDataBindGraphValue::String(value.to_vec())
        } else if let Some(value) = file.view_model_instance_trigger_count_for_object(source) {
            RuntimeDataBindGraphValue::Trigger(value)
        } else if let Some(value) =
            file.view_model_instance_symbol_list_index_value_for_object(source)
        {
            RuntimeDataBindGraphValue::SymbolListIndex(value)
        } else if let Some(value) = file.view_model_instance_color_value_for_object(source) {
            RuntimeDataBindGraphValue::Color(value)
        } else if source.type_name == "ViewModelInstanceEnum" {
            RuntimeDataBindGraphValue::Enum(source.uint_property("propertyValue")?)
        } else {
            return None;
        }
    } else {
        RuntimeDataBindGraphValue::String(
            file.view_model_instance_string_value_bytes_for_object(source)?
                .to_vec(),
        )
    };
    Some(RuntimeBindableStringDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter,
        value,
    })
}

pub(crate) fn runtime_bindable_enums(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableEnum> {
    let mut values = BTreeMap::<u32, RuntimeBindableEnum>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyEnum" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_enum| bindable_enum.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableEnum {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target
                    .uint_property("propertyValue")
                    .unwrap_or(u64::from(u32::MAX)),
            });
        if let Some(source) =
            runtime_bindable_enum_default_view_model_source(file, data_bind_index, data_bind)
        {
            values
                .entry(target.id)
                .and_modify(|bindable_enum| bindable_enum.default_view_model_sources.push(source));
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_enum_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableEnumDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyEnum", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    if source.type_name != "ViewModelInstanceEnum" {
        return None;
    }
    let value = source.uint_property("propertyValue")?;
    Some(RuntimeBindableEnumDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_assets(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableAsset> {
    let mut values = BTreeMap::<u32, RuntimeBindableAsset>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyAsset" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_asset| bindable_asset.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableAsset {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target
                    .uint_property("propertyValue")
                    .unwrap_or(u64::from(u32::MAX)),
            });
        if let Some(source) =
            runtime_bindable_asset_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_asset| {
                bindable_asset.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_asset_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableAssetDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyAsset", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    if source.type_name != "ViewModelInstanceAssetImage" {
        return None;
    }
    let value = source.uint_property("propertyValue")?;
    Some(RuntimeBindableAssetDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_artboards(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableArtboard> {
    let mut values = BTreeMap::<u32, RuntimeBindableArtboard>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyArtboard" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_artboard| {
                bindable_artboard.data_bind_indices.push(data_bind_index)
            })
            .or_insert_with(|| RuntimeBindableArtboard {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target
                    .uint_property("propertyValue")
                    .unwrap_or(u64::from(u32::MAX)),
            });
        if let Some(source) =
            runtime_bindable_artboard_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_artboard| {
                bindable_artboard.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_artboard_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableArtboardDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyArtboard", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    if source.type_name != "ViewModelInstanceArtboard" {
        return None;
    }
    let value = source.uint_property("propertyValue")?;
    Some(RuntimeBindableArtboardDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_lists(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableList> {
    let mut values = BTreeMap::<u32, RuntimeBindableList>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyList" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_list| bindable_list.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableList {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target
                    .uint_property("propertyValue")
                    .and_then(|value| usize::try_from(value).ok())
                    .unwrap_or(usize::try_from(u64::from(u32::MAX)).unwrap_or(usize::MAX)),
            });
        if let Some(source) =
            runtime_bindable_list_default_view_model_source(file, data_bind_index, data_bind)
        {
            values
                .entry(target.id)
                .and_modify(|bindable_list| bindable_list.default_view_model_sources.push(source));
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_list_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableListDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyList", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source = file.data_context_view_model_property_for_instance(default_instance.object, &path);
    let converter = runtime_data_bind_graph_converter(file, data_bind);
    let value = match converter.as_ref() {
        Some(RuntimeDataBindGraphConverter::NumberToList { .. }) => {
            RuntimeDataBindGraphValue::Number(
                file.view_model_instance_number_value_for_object(source?)?,
            )
        }
        Some(RuntimeDataBindGraphConverter::Formula { .. }) => RuntimeDataBindGraphValue::List {
            item_count: file.view_model_instance_list_size_for_object(source?)?,
        },
        None => RuntimeDataBindGraphValue::List {
            item_count: match source {
                Some(source) => file.view_model_instance_list_size_for_object(source)?,
                None => {
                    runtime_view_model_property_at_path(file, &path)
                        .filter(|property| property.type_name == "ViewModelPropertyList")?;
                    0
                }
            },
        },
        _ => return None,
    };
    Some(RuntimeBindableListDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter,
        value,
    })
}

fn runtime_view_model_property_at_path<'a>(
    file: &'a RuntimeFile,
    path: &[u32],
) -> Option<&'a RuntimeObject> {
    let mut view_model = file.view_model(usize::try_from(*path.first()?).ok()?)?;
    let mut properties = &path[1..];
    while let Some((&property_id, rest)) = properties.split_first() {
        let property = view_model
            .properties
            .get(usize::try_from(property_id).ok()?)
            .copied()?;
        if rest.is_empty() {
            return Some(property);
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        view_model = file
            .view_model(usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?)?;
        properties = rest;
    }
    None
}

pub(crate) fn runtime_bindable_triggers(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableTrigger> {
    let mut values = BTreeMap::<u32, RuntimeBindableTrigger>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyTrigger" {
            continue;
        }
        let source = runtime_bindable_trigger_source(file, data_bind);
        let value = target.uint_property("propertyValue").unwrap_or(0);
        values
            .entry(target.id)
            .and_modify(|bindable_trigger| {
                bindable_trigger.data_bind_indices.push(data_bind_index);
                bindable_trigger.value = value;
                bindable_trigger.source = source;
            })
            .or_insert_with(|| RuntimeBindableTrigger {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                value,
                source,
                default_view_model_sources: Vec::new(),
            });
        if let Some(default_source) =
            runtime_bindable_trigger_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_trigger| {
                bindable_trigger
                    .default_view_model_sources
                    .push(default_source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_trigger_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableTriggerDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyTrigger", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let value = file.view_model_instance_trigger_count_for_object(source)?;
    Some(RuntimeBindableTriggerDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter: runtime_data_bind_graph_converter(file, data_bind),
        value,
    })
}

fn runtime_bindable_trigger_source(
    file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> RuntimeBindableTriggerSource {
    let Some(path) = file.data_bind_context_source_path_ids_for_object(data_bind) else {
        return RuntimeBindableTriggerSource::None;
    };
    let Some(default_instance) = file.view_model_default_instance(0) else {
        return RuntimeBindableTriggerSource::None;
    };
    let Some(target) =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)
    else {
        return RuntimeBindableTriggerSource::None;
    };
    if file
        .view_model_instance_trigger_count_for_object(target)
        .is_none()
    {
        return RuntimeBindableTriggerSource::None;
    }

    RuntimeBindableTriggerSource::DefaultViewModelTrigger {
        trigger_global_id: target.id,
    }
}

pub(crate) fn runtime_bindable_view_models(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableViewModel> {
    let mut values = BTreeMap::<u32, RuntimeBindableViewModel>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyViewModel" {
            continue;
        }
        let source = runtime_bindable_view_model_source(file, data_bind);
        values
            .entry(target.id)
            .and_modify(|bindable_view_model| {
                bindable_view_model.source = source;
                bindable_view_model.data_bind_indices.push(data_bind_index);
            })
            .or_insert_with(|| RuntimeBindableViewModel {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                source,
                default_view_model_sources: Vec::new(),
            });
        if let Some(source) =
            runtime_bindable_view_model_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_view_model| {
                bindable_view_model.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_view_model_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableViewModelDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyViewModel", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let Some(reference) =
        file.data_context_view_model_instance_for_instance(default_instance.object, &path)
    else {
        return None;
    };
    let view_model_instance_ids = file
        .view_model(reference.view_model_index)?
        .instances
        .into_iter()
        .map(|instance| instance.object.id)
        .collect();
    Some(RuntimeBindableViewModelDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter: runtime_data_bind_graph_converter(file, data_bind),
        value: RuntimeViewModelPointer::Imported {
            object_id: reference.object.id,
        },
        view_model_instance_ids,
    })
}

fn runtime_bindable_view_model_source(
    file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> RuntimeBindableViewModelSource {
    if data_bind.type_name == "DataBindContext"
        && file
            .data_bind_context_source_path_ids_for_object(data_bind)
            .is_some_and(|source_path_ids| source_path_ids.len() == 1)
    {
        RuntimeBindableViewModelSource::RootDataContext
    } else {
        RuntimeBindableViewModelSource::Null
    }
}

pub(crate) fn runtime_bindable_booleans(
    file: &RuntimeFile,
    state_machine: &rive_binary::RuntimeStateMachine<'_>,
) -> Vec<RuntimeBindableBoolean> {
    let mut values = BTreeMap::<u32, RuntimeBindableBoolean>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyBoolean" {
            continue;
        }
        values
            .entry(target.id)
            .and_modify(|bindable_boolean| bindable_boolean.data_bind_indices.push(data_bind_index))
            .or_insert_with(|| RuntimeBindableBoolean {
                global_id: target.id,
                data_bind_indices: vec![data_bind_index],
                default_view_model_sources: Vec::new(),
                value: target.bool_property("propertyValue").unwrap_or(false),
            });
        if let Some(source) =
            runtime_bindable_boolean_default_view_model_source(file, data_bind_index, data_bind)
        {
            values.entry(target.id).and_modify(|bindable_boolean| {
                bindable_boolean.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_boolean_default_view_model_source(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
) -> Option<RuntimeBindableBooleanDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyBoolean", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let default_instance = file.view_model_default_instance(0)?;
    let source =
        file.data_context_view_model_property_for_instance(default_instance.object, &path)?;
    let value = file.view_model_instance_boolean_value_for_object(source)?;
    Some(RuntimeBindableBooleanDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter: runtime_data_bind_graph_converter(file, data_bind),
        value,
    })
}

pub(crate) fn runtime_default_view_model_triggers(
    file: &RuntimeFile,
) -> Vec<RuntimeViewModelTrigger> {
    let Some(view_model) = file.view_model(0) else {
        return Vec::new();
    };
    let Some(instance) = view_model.instances.into_iter().next() else {
        return Vec::new();
    };

    instance
        .values
        .into_iter()
        .filter_map(|value| {
            let value_count = file.view_model_instance_trigger_count_for_object(value.object)?;
            let view_model_property_id = value
                .object
                .uint_property("viewModelPropertyId")
                .and_then(|id| u32::try_from(id).ok())?;
            Some(RuntimeViewModelTrigger {
                global_id: value.object.id,
                view_model_property_id,
                value: value_count,
            })
        })
        .collect()
}
