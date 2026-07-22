use crate::data_bind_graph::{
    RuntimeDataBindGraphConverterBuildCache,
    runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source,
    runtime_data_bind_graph_converter_starts_with_to_string,
    runtime_data_bind_graph_converter_with_cache,
    runtime_data_bind_graph_group_formula_operation_accepts_non_number_source,
    runtime_data_bind_graph_group_operation_formula_accepts_non_number_source,
};
use crate::properties::property_key_for_name;
use crate::view_model::RuntimeFontAssetValue;
use crate::{RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue, RuntimeViewModelPointer};
use nuxie_binary::{RuntimeFile, RuntimeObject};
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
    pub(crate) value: RuntimeBindableAssetValue,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableAssetDefaultViewModelSource {
    pub(crate) data_bind_index: usize,
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) value: RuntimeBindableAssetValue,
}

/// The value retained by C++ `BindablePropertyAsset`.
///
/// `asset_index` is the generated `propertyValue`. `font_value` models the
/// separate private `FontAsset`: `Some` identifies a font binding and retains
/// its live Font payload even when `propertyValue` is unchanged.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindableAssetValue {
    asset_index: u64,
    font_value: Option<RuntimeFontAssetValue>,
}

impl RuntimeBindableAssetValue {
    pub(crate) fn from_asset_index(asset_index: u64) -> Self {
        Self {
            asset_index,
            font_value: None,
        }
    }

    pub(crate) fn from_font_value(font_value: RuntimeFontAssetValue) -> Self {
        Self {
            asset_index: font_value.file_asset_index(),
            font_value: Some(font_value),
        }
    }

    pub(crate) fn asset_index(&self) -> u64 {
        self.asset_index
    }

    pub(crate) fn font_value(&self) -> Option<&RuntimeFontAssetValue> {
        self.font_value.as_ref()
    }

    pub(crate) fn data_bind_asset_index(&self) -> u64 {
        self.font_value
            .as_ref()
            .filter(|font_value| font_value.live_font_bytes_arc().is_some())
            .map(RuntimeFontAssetValue::file_asset_index)
            .unwrap_or(self.asset_index)
    }

    pub(crate) fn font_data_bind_value(&self) -> Option<RuntimeFontAssetValue> {
        let font_value = self.font_value.as_ref()?;
        if font_value.live_font_bytes_arc().is_some() {
            Some(font_value.clone())
        } else {
            Some(RuntimeFontAssetValue::from_file_asset_index(
                self.asset_index,
            ))
        }
    }

    pub(crate) fn mark_as_font(&mut self) {
        if self.font_value.is_none() {
            self.font_value = Some(RuntimeFontAssetValue::from_file_asset_index(
                self.asset_index,
            ));
        }
    }

    pub(crate) fn set_asset_index(&mut self, asset_index: u64) -> bool {
        let changed = self.asset_index != asset_index;
        self.asset_index = asset_index;
        changed
    }

    pub(crate) fn apply_font_value(&mut self, font_value: &RuntimeFontAssetValue) -> bool {
        let mut changed = self.asset_index != font_value.file_asset_index();
        self.asset_index = font_value.file_asset_index();
        match self.font_value.as_mut() {
            Some(current) => changed |= current.apply_data_bind_value(font_value),
            None => {
                self.font_value = Some(font_value.clone());
                changed = true;
            }
        }
        changed
    }
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
    pub(crate) default_view_model_sources: Vec<RuntimeBindableAssetDefaultViewModelSource>,
    pub(crate) value: RuntimeBindableAssetValue,
}

impl StateMachineBindableAssetInstance {
    pub(crate) fn new(bindable_asset: &RuntimeBindableAsset) -> Self {
        Self {
            global_id: bindable_asset.global_id,
            data_bind_indices: bindable_asset.data_bind_indices.clone(),
            default_view_model_sources: bindable_asset.default_view_model_sources.clone(),
            value: bindable_asset.value.clone(),
        }
    }

    pub(crate) fn has_data_bind_index(&self, data_bind_index: usize) -> bool {
        self.data_bind_indices.contains(&data_bind_index)
    }

    pub(crate) fn set_value(&mut self, value: u64) -> bool {
        self.value.set_asset_index(value)
    }

    pub(crate) fn apply_font_value(&mut self, value: &RuntimeFontAssetValue) -> bool {
        self.value.apply_font_value(value)
    }
}

pub(crate) fn bindable_asset_value(
    bindable_assets: &[StateMachineBindableAssetInstance],
    global_id: u32,
) -> Option<u64> {
    bindable_assets
        .iter()
        .find(|bindable_asset| bindable_asset.global_id == global_id)
        .map(|bindable_asset| bindable_asset.value.asset_index())
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

pub(crate) fn runtime_bindable_numbers<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_number_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        )
        .or_else(|| {
            // C++ `DataBindContext::bindFromContext` unbinds a name-based
            // bind whose relative lookup fails: the source stays absent
            // instead of holding the serialized target value.
            if file
                .data_bind_is_name_based_for_object(data_bind)
                .unwrap_or(false)
            {
                return None;
            }
            runtime_bindable_number_unresolved_view_model_source(
                file,
                data_bind_index,
                data_bind,
                target.double_property("propertyValue").unwrap_or(0.0),
                converter_cache,
            )
        }) {
            values.entry(target.id).and_modify(|bindable_number| {
                bindable_number.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_number_unresolved_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    target_value: f32,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableNumberDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyNumber", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let converter = runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache);
    let unresolved = runtime_unresolved_view_model_value_at_path(file, &path);
    let value = match (converter.as_ref(), unresolved) {
        (Some(RuntimeDataBindGraphConverter::ListToLength), Some(unresolved)) => {
            if !matches!(unresolved, RuntimeDataBindGraphValue::List { .. }) {
                return None;
            }
            // The source side of ListToLength is still a list even when its
            // parent-relative context is unavailable while the child artboard
            // is constructed.
            RuntimeDataBindGraphValue::ListLength(0)
        }
        (Some(RuntimeDataBindGraphConverter::ListToLength), None) => {
            // Parent-relative child binds do not necessarily have a locally
            // resolvable ViewModel schema while the child is constructed.
            // Preserve the source kind until its parent context is attached.
            RuntimeDataBindGraphValue::ListLength(0)
        }
        (None, Some(unresolved)) => {
            if !matches!(unresolved, RuntimeDataBindGraphValue::Number(_)) {
                return None;
            }
            RuntimeDataBindGraphValue::Number(target_value)
        }
        (None, None) => RuntimeDataBindGraphValue::Number(target_value),
        (Some(_), Some(unresolved)) => match unresolved {
            RuntimeDataBindGraphValue::Number(_) => RuntimeDataBindGraphValue::Number(target_value),
            value => value,
        },
        (Some(_), None) => RuntimeDataBindGraphValue::Number(target_value),
    };
    Some(RuntimeBindableNumberDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter,
        value,
        view_model_instance_ids: Vec::new(),
    })
}

fn runtime_bindable_number_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableNumberDefaultViewModelSource> {
    runtime_number_default_view_model_source_for_instance_with_cache(
        file,
        data_bind_index,
        data_bind,
        "BindablePropertyNumber",
        "propertyValue",
        default_instance?,
        converter_cache,
    )
}

pub(crate) fn runtime_number_default_view_model_source_for_instance(
    file: &RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    target_type_name: &str,
    target_property_name: &str,
    default_instance: &RuntimeObject,
) -> Option<RuntimeBindableNumberDefaultViewModelSource> {
    let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
    runtime_number_default_view_model_source_for_instance_with_cache(
        file,
        data_bind_index,
        data_bind,
        target_type_name,
        target_property_name,
        default_instance,
        &mut converter_cache,
    )
}

fn runtime_number_default_view_model_source_for_instance_with_cache<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    target_type_name: &str,
    target_property_name: &str,
    default_instance: &RuntimeObject,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableNumberDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name(target_type_name, target_property_name) != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let source = file.data_context_view_model_property_for_instance(default_instance, &path)?;
    let converter = runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache);
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
                file.data_context_view_model_instance_for_instance(default_instance, &path)
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
                } else if let Some(reference) =
                    file.data_context_view_model_instance_for_instance(default_instance, &path)
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
            file.data_context_view_model_instance_for_instance(default_instance, &path)?;
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

pub(crate) fn runtime_bindable_integers<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_integer_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        ) {
            values.entry(target.id).and_modify(|bindable_integer| {
                bindable_integer.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_integer_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableIntegerDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyInteger", "propertyValue") != Some(property_key) {
        return None;
    }
    if runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache).is_some() {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        file.view_model_instance_symbol_list_index_value_for_object(source)?
    } else {
        if !matches!(
            runtime_unresolved_view_model_value_at_path(file, &path)?,
            RuntimeDataBindGraphValue::SymbolListIndex(_)
        ) {
            return None;
        }
        file.data_bind_target_for_object(data_bind)?
            .uint_property("propertyValue")
            .unwrap_or(0)
    };
    Some(RuntimeBindableIntegerDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_colors(
    file: &RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
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
        if let Some(source) = runtime_bindable_color_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
        ) {
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
    default_instance: Option<&RuntimeObject>,
) -> Option<RuntimeBindableColorDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyColor", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        file.view_model_instance_color_value_for_object(source)?
    } else {
        if !matches!(
            runtime_unresolved_view_model_value_at_path(file, &path)?,
            RuntimeDataBindGraphValue::Color(_)
        ) {
            return None;
        }
        file.data_bind_target_for_object(data_bind)?
            .color_property("propertyValue")
            .unwrap_or(0)
    };
    Some(RuntimeBindableColorDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_strings<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_string_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        ) {
            values.entry(target.id).and_modify(|bindable_string| {
                bindable_string.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_string_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableStringDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyString", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let converter = runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache);
    let source = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    });
    let value = if let Some(source) = source {
        if runtime_data_bind_graph_converter_starts_with_to_string(converter.as_ref()) {
            if let Some(value) = file.view_model_instance_number_value_for_object(source) {
                RuntimeDataBindGraphValue::Number(value)
            } else if let Some(value) = file.view_model_instance_boolean_value_for_object(source) {
                RuntimeDataBindGraphValue::Boolean(value)
            } else if let Some(value) =
                file.view_model_instance_string_value_bytes_for_object(source)
            {
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
        }
    } else {
        let unresolved = runtime_unresolved_view_model_value_at_path(file, &path)?;
        if runtime_data_bind_graph_converter_starts_with_to_string(converter.as_ref()) {
            match unresolved {
                RuntimeDataBindGraphValue::Number(_)
                | RuntimeDataBindGraphValue::Boolean(_)
                | RuntimeDataBindGraphValue::String(_)
                | RuntimeDataBindGraphValue::Trigger(_)
                | RuntimeDataBindGraphValue::SymbolListIndex(_)
                | RuntimeDataBindGraphValue::Color(_)
                | RuntimeDataBindGraphValue::Enum(_) => unresolved,
                _ => return None,
            }
        } else {
            if !matches!(unresolved, RuntimeDataBindGraphValue::String(_)) {
                return None;
            }
            RuntimeDataBindGraphValue::String(
                file.data_bind_target_for_object(data_bind)?
                    .string_property_bytes("propertyValue")
                    .unwrap_or_default()
                    .to_vec(),
            )
        }
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
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
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
        if let Some(source) = runtime_bindable_enum_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
        ) {
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
    default_instance: Option<&RuntimeObject>,
) -> Option<RuntimeBindableEnumDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyEnum", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        if source.type_name != "ViewModelInstanceEnum" {
            return None;
        }
        source.uint_property("propertyValue")?
    } else {
        let property = runtime_view_model_property_at_path(file, &path)?;
        if !matches!(
            property.type_name,
            "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom" | "ViewModelPropertyEnumSystem"
        ) {
            return None;
        }
        file.data_bind_target_for_object(data_bind)?
            .uint_property("propertyValue")
            .unwrap_or(u64::from(u32::MAX))
    };
    Some(RuntimeBindableEnumDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_assets(
    file: &RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
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
                    .map(RuntimeBindableAssetValue::from_asset_index)
                    .unwrap_or_else(|| {
                        RuntimeBindableAssetValue::from_asset_index(u64::from(u32::MAX))
                    }),
            });
        if let Some(source) = runtime_bindable_asset_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
        ) {
            values.entry(target.id).and_modify(|bindable_asset| {
                if source.value.font_value().is_some() {
                    bindable_asset.value.mark_as_font();
                }
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
    default_instance: Option<&RuntimeObject>,
) -> Option<RuntimeBindableAssetDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyAsset", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        let asset_index = source.uint_property("propertyValue")?;
        match source.type_name {
            "ViewModelInstanceAssetImage" => {
                RuntimeBindableAssetValue::from_asset_index(asset_index)
            }
            "ViewModelInstanceAssetFont" => RuntimeBindableAssetValue::from_font_value(
                RuntimeFontAssetValue::from_file_asset_index(asset_index),
            ),
            _ => return None,
        }
    } else {
        let property = runtime_view_model_property_at_path(file, &path)?;
        let asset_index = file
            .data_bind_target_for_object(data_bind)?
            .uint_property("propertyValue")
            .unwrap_or(u64::from(u32::MAX));
        match property.type_name {
            "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => {
                RuntimeBindableAssetValue::from_asset_index(asset_index)
            }
            "ViewModelPropertyAssetFont" => RuntimeBindableAssetValue::from_font_value(
                RuntimeFontAssetValue::from_file_asset_index(asset_index),
            ),
            _ => return None,
        }
    };
    Some(RuntimeBindableAssetDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_artboards(
    file: &RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
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
        if let Some(source) = runtime_bindable_artboard_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
        ) {
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
    default_instance: Option<&RuntimeObject>,
) -> Option<RuntimeBindableArtboardDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyArtboard", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        if source.type_name != "ViewModelInstanceArtboard" {
            return None;
        }
        source.uint_property("propertyValue")?
    } else {
        if runtime_view_model_property_at_path(file, &path)?.type_name
            != "ViewModelPropertyArtboard"
        {
            return None;
        }
        file.data_bind_target_for_object(data_bind)?
            .uint_property("propertyValue")
            .unwrap_or(u64::from(u32::MAX))
    };
    Some(RuntimeBindableArtboardDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        value,
    })
}

pub(crate) fn runtime_bindable_lists<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_list_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        ) {
            values
                .entry(target.id)
                .and_modify(|bindable_list| bindable_list.default_view_model_sources.push(source));
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_list_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableListDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyList", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let source = file.data_context_view_model_property_for_instance(default_instance?, &path);
    let converter = runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache);
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

fn runtime_unresolved_view_model_value_at_path(
    file: &RuntimeFile,
    path: &[u32],
) -> Option<RuntimeDataBindGraphValue> {
    let property = runtime_view_model_property_at_path(file, path)?;
    match property.type_name {
        "ViewModelPropertyNumber" => Some(RuntimeDataBindGraphValue::Number(0.0)),
        "ViewModelPropertyBoolean" => Some(RuntimeDataBindGraphValue::Boolean(false)),
        "ViewModelPropertyString" => Some(RuntimeDataBindGraphValue::String(Vec::new())),
        "ViewModelPropertyColor" => Some(RuntimeDataBindGraphValue::Color(0)),
        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom" | "ViewModelPropertyEnumSystem" => {
            Some(RuntimeDataBindGraphValue::Enum(0))
        }
        "ViewModelPropertySymbolListIndex" => Some(RuntimeDataBindGraphValue::SymbolListIndex(0)),
        "ViewModelPropertyList" => Some(RuntimeDataBindGraphValue::List { item_count: 0 }),
        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" | "ViewModelPropertyAssetFont" => {
            Some(RuntimeDataBindGraphValue::Asset(u64::from(u32::MAX)))
        }
        "ViewModelPropertyArtboard" => {
            Some(RuntimeDataBindGraphValue::Artboard(u64::from(u32::MAX)))
        }
        "ViewModelPropertyTrigger" => Some(RuntimeDataBindGraphValue::Trigger(0)),
        "ViewModelPropertyViewModel" => Some(RuntimeDataBindGraphValue::ViewModel(
            RuntimeViewModelPointer::Null,
        )),
        _ => None,
    }
}

pub(crate) fn runtime_bindable_triggers<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Vec<RuntimeBindableTrigger> {
    let mut values = BTreeMap::<u32, RuntimeBindableTrigger>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        if target.type_name != "BindablePropertyTrigger" {
            continue;
        }
        let source = runtime_bindable_trigger_source(file, data_bind, default_instance);
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
        if let Some(default_source) = runtime_bindable_trigger_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        )
        .or_else(|| {
            runtime_bindable_trigger_unresolved_view_model_source(
                file,
                data_bind_index,
                data_bind,
                value,
                converter_cache,
            )
        }) {
            values.entry(target.id).and_modify(|bindable_trigger| {
                bindable_trigger
                    .default_view_model_sources
                    .push(default_source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_trigger_unresolved_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    target_value: u64,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableTriggerDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyTrigger", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    (runtime_view_model_property_at_path(file, &path)?.type_name == "ViewModelPropertyTrigger")
        .then(|| RuntimeBindableTriggerDefaultViewModelSource {
            data_bind_index,
            path: path.to_vec(),
            flags: data_bind.uint_property("flags").unwrap_or(0),
            converter: runtime_data_bind_graph_converter_with_cache(
                file,
                data_bind,
                converter_cache,
            ),
            value: target_value,
        })
}

fn runtime_bindable_trigger_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableTriggerDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyTrigger", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let source = file.data_context_view_model_property_for_instance(default_instance?, &path)?;
    let value = file.view_model_instance_trigger_count_for_object(source)?;
    Some(RuntimeBindableTriggerDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter: runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache),
        value,
    })
}

fn runtime_bindable_trigger_source(
    file: &RuntimeFile,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
) -> RuntimeBindableTriggerSource {
    let Some(path) = file.data_bind_context_source_path_ids_for_object(data_bind) else {
        return RuntimeBindableTriggerSource::None;
    };
    let Some(default_instance) = default_instance else {
        return RuntimeBindableTriggerSource::None;
    };
    let Some(target) = file.data_context_view_model_property_for_instance(default_instance, &path)
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

pub(crate) fn runtime_bindable_view_models<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_view_model_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        ) {
            values.entry(target.id).and_modify(|bindable_view_model| {
                bindable_view_model.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_view_model_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableViewModelDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyViewModel", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let Some(reference) =
        file.data_context_view_model_instance_for_instance(default_instance?, &path)
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
        converter: runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache),
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

pub(crate) fn runtime_bindable_booleans<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
        if let Some(source) = runtime_bindable_boolean_default_view_model_source(
            file,
            data_bind_index,
            data_bind,
            default_instance,
            converter_cache,
        ) {
            values.entry(target.id).and_modify(|bindable_boolean| {
                bindable_boolean.default_view_model_sources.push(source)
            });
        }
    }

    values.into_values().collect()
}

fn runtime_bindable_boolean_default_view_model_source<'a>(
    file: &'a RuntimeFile,
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    default_instance: Option<&RuntimeObject>,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Option<RuntimeBindableBooleanDefaultViewModelSource> {
    let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
    if property_key_for_name("BindablePropertyBoolean", "propertyValue") != Some(property_key) {
        return None;
    }
    let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
    let value = if let Some(source) = default_instance.and_then(|default_instance| {
        file.data_context_view_model_property_for_instance(default_instance, &path)
    }) {
        file.view_model_instance_boolean_value_for_object(source)?
    } else {
        if !matches!(
            runtime_unresolved_view_model_value_at_path(file, &path)?,
            RuntimeDataBindGraphValue::Boolean(_)
        ) {
            return None;
        }
        file.data_bind_target_for_object(data_bind)?
            .bool_property("propertyValue")
            .unwrap_or(false)
    };
    Some(RuntimeBindableBooleanDefaultViewModelSource {
        data_bind_index,
        path: path.to_vec(),
        flags: data_bind.uint_property("flags").unwrap_or(0),
        converter: runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache),
        value,
    })
}

pub(crate) fn runtime_default_view_model_triggers(
    file: &RuntimeFile,
    view_model_index: Option<usize>,
) -> Vec<RuntimeViewModelTrigger> {
    let Some(view_model_index) = view_model_index else {
        return Vec::new();
    };
    let Some(view_model) = file.view_model(view_model_index) else {
        return Vec::new();
    };
    let Some(instance) = view_model.instances.into_iter().next() else {
        return Vec::new();
    };

    instance
        .values
        .into_iter()
        .filter_map(|value| {
            file.view_model_instance_trigger_count_for_object(value.object)?;
            let view_model_property_id = value
                .object
                .uint_property("viewModelPropertyId")
                .and_then(|id| u32::try_from(id).ok())?;
            Some(RuntimeViewModelTrigger {
                global_id: value.object.id,
                view_model_property_id,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use std::sync::Arc;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    #[test]
    fn unresolved_parent_number_bind_retains_its_dynamic_source() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(
                            property_key_for_name("BindablePropertyNumber", "propertyValue")
                                .expect("number property key"),
                        )),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![1, 0]),
                    ),
                    property("DataBindContext", "flags", AuthoringValue::Uint(4)),
                ],
            ),
        ])
        .expect("unresolved parent binding fixture imports");
        let data_bind = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "DataBindContext")
            .expect("fixture has a data bind");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();

        let source = runtime_bindable_number_unresolved_view_model_source(
            &file,
            3,
            data_bind,
            7.0,
            &mut converter_cache,
        )
        .expect("a parent-relative source is retained before a parent context is available");

        assert_eq!(source.data_bind_index, 3);
        assert_eq!(source.path, [1, 0]);
        assert_eq!(source.flags, 4);
        assert!(source.converter.is_none());
        assert!(matches!(
            source.value,
            RuntimeDataBindGraphValue::Number(7.0)
        ));
    }

    #[test]
    fn unresolved_parent_list_to_length_bind_retains_its_list_source_kind() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(
                            property_key_for_name("BindablePropertyNumber", "propertyValue")
                                .expect("number property key"),
                        )),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![1, 0]),
                    ),
                    property("DataBindContext", "flags", AuthoringValue::Uint(4)),
                    property("DataBindContext", "converterId", AuthoringValue::Uint(0)),
                ],
            ),
            record("DataConverterListToLength", Vec::new()),
        ])
        .expect("unresolved list-to-length fixture imports");
        let data_bind = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "DataBindContext")
            .expect("fixture has a data bind");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();

        let source = runtime_bindable_number_unresolved_view_model_source(
            &file,
            3,
            data_bind,
            7.0,
            &mut converter_cache,
        )
        .expect("a parent-relative list source is retained before binding its parent");

        assert!(matches!(
            source.converter,
            Some(RuntimeDataBindGraphConverter::ListToLength)
        ));
        assert!(matches!(
            source.value,
            RuntimeDataBindGraphValue::ListLength(0)
        ));
    }

    #[test]
    fn bindable_font_retains_live_payload_for_property_writes_and_replaces_it_for_data_binds() {
        let live: Arc<[u8]> = vec![2, 4, 6, 8].into();
        let mut font = RuntimeFontAssetValue::default();
        assert!(font.set_live_font_bytes(Some(Arc::clone(&live))));
        let mut bindable = RuntimeBindableAssetValue::from_font_value(font);

        assert!(bindable.set_asset_index(3));
        assert_eq!(bindable.asset_index(), 3);
        assert!(
            bindable
                .font_value()
                .and_then(RuntimeFontAssetValue::live_font_bytes_arc)
                .is_some_and(|value| Arc::ptr_eq(value, &live)),
            "a generated propertyValue write preserves BindablePropertyAsset::fontValue"
        );
        assert_eq!(
            bindable.data_bind_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX,
            "a live Font wins over the independent generated propertyValue when applied to a source"
        );

        let file_font = RuntimeFontAssetValue::from_file_asset_index(1);
        assert!(bindable.apply_font_value(&file_font));
        assert_eq!(bindable.asset_index(), 1);
        assert_eq!(
            bindable
                .font_value()
                .and_then(RuntimeFontAssetValue::live_font_bytes),
            None,
            "a source-to-target font bind replaces the retained Font payload"
        );
        assert!(bindable.set_asset_index(5));
        let copied_file = bindable
            .font_data_bind_value()
            .expect("font bindable has a font channel");
        assert_eq!(copied_file.file_asset_index(), 5);
        assert_eq!(copied_file.live_font_bytes(), None);
    }
}
