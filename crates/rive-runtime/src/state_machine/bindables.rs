use crate::{
    RuntimeBindableArtboard, RuntimeBindableAsset, RuntimeBindableBoolean, RuntimeBindableColor,
    RuntimeBindableEnum, RuntimeBindableInteger, RuntimeBindableList, RuntimeBindableNumber,
    RuntimeBindableString, RuntimeBindableTrigger, RuntimeBindableTriggerSource,
    RuntimeBindableViewModel, RuntimeBindableViewModelSource, RuntimeViewModelPointer,
};

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
