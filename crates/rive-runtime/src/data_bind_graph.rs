use std::collections::BTreeMap;

use rive_binary::{RuntimeFile, RuntimeObject};

use crate::{
    RuntimeDataBindGraphFormulaState, RuntimeDataBindGraphInterpolatorState, RuntimeDataContext,
    RuntimeOwnedViewModelInstance, RuntimeTransitionInterpolator, RuntimeViewModelPointer,
};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataBindGraph {
    pub(crate) context_kind: RuntimeDataBindGraphContextKind,
    pub(crate) default_view_model_bindings_dirty: bool,
    pub(crate) formula_random_source: RuntimeDataBindGraphFormulaRandomSource,
    pub(crate) sources: Vec<RuntimeDataBindGraphSourceNode>,
    pub(crate) targets: Vec<RuntimeDataBindGraphTargetNode>,
    pub(crate) default_view_model_bindings: Vec<RuntimeDataBindGraphDefaultBinding>,
    pub(crate) imported_view_model_context: Option<RuntimeImportedViewModelContextKey>,
    pub(crate) imported_view_model_overrides:
        BTreeMap<RuntimeImportedViewModelOverrideKey, RuntimeViewModelPointer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDataBindGraphContextKind {
    None,
    Empty,
    DefaultViewModel,
    ImportedViewModel,
    OwnedViewModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeImportedViewModelContextKey {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RuntimeImportedViewModelOverrideKey {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataBindGraphFormulaRandomSource {
    values: Vec<f32>,
    next_index: usize,
    call_count: usize,
}

impl RuntimeDataBindGraphFormulaRandomSource {
    pub(crate) fn set_values(&mut self, values: &[f32]) {
        self.values.clear();
        self.values.extend_from_slice(values);
        self.next_index = 0;
        self.call_count = 0;
    }

    pub(crate) fn next_value(&mut self) -> f32 {
        self.call_count += 1;
        let value = self.values.get(self.next_index).copied().unwrap_or(0.0);
        if self.next_index < self.values.len() {
            self.next_index += 1;
        }
        value
    }

    pub(crate) fn call_count(&self) -> usize {
        self.call_count
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataBindGraphDefaultBinding {
    pub(crate) data_bind_index: usize,
    pub(crate) source: RuntimeDataBindGraphSourceHandle,
    pub(crate) target: RuntimeDataBindGraphTargetHandle,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeDataBindGraphSourceHandle(pub(crate) usize);

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeDataBindGraphTargetHandle(pub(crate) usize);

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataBindGraphSourceNode {
    pub(crate) path: Vec<u32>,
    pub(crate) flags: u64,
    pub(crate) bound: bool,
    pub(crate) target_to_source_dirty: bool,
    pub(crate) source_to_target_dirty_after_immediate: bool,
    pub(crate) source_to_target_dirty_after_target_to_source: bool,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) converter_state: RuntimeDataBindGraphConverterState,
    pub(crate) default_value: RuntimeDataBindGraphValue,
    pub(crate) value: RuntimeDataBindGraphValue,
    pub(crate) view_model_instance_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeDataBindGraphConverter {
    PassThrough,
    BooleanNegate,
    TriggerIncrement,
    ToNumber,
    ListToLength,
    NumberToList {
        has_view_model: bool,
    },
    ToString {
        flags: u64,
        decimals: u64,
        color_format: Vec<u8>,
    },
    OperationValue {
        operation_type: u64,
        operation_value: f32,
    },
    OperationViewModel {
        operation_type: u64,
        operation_value: f32,
        default_operation_value: f32,
        source_path: Option<Vec<u32>>,
    },
    SystemOperationValue {
        operation_type: u64,
        operation_value: f32,
        reverse: bool,
    },
    Rounder {
        decimals: u64,
    },
    RangeMapper {
        min_input: f32,
        max_input: f32,
        min_output: f32,
        max_output: f32,
        flags: u64,
        interpolation_type: u64,
        interpolator: Option<RuntimeTransitionInterpolator>,
    },
    StringTrim {
        trim_type: u64,
    },
    StringRemoveZeros,
    StringPad {
        length: u64,
        text: Vec<u8>,
        pad_type: u64,
    },
    Formula {
        tokens: Vec<RuntimeDataBindGraphFormulaToken>,
    },
    Interpolator {
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
    },
    Group(Vec<RuntimeDataBindGraphConverter>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeDataBindGraphFormulaToken {
    Input,
    Value(f32),
    Operation {
        operation_type: u64,
    },
    Function {
        function_type: u64,
        arguments_count: usize,
        random_mode: u64,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeDataBindGraphConverterState {
    None,
    Formula(RuntimeDataBindGraphFormulaState),
    Interpolator(RuntimeDataBindGraphInterpolatorState),
    Group(Vec<RuntimeDataBindGraphConverterState>),
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RuntimeDataBindGraphStatefulAdvance {
    pub(crate) changed: bool,
    pub(crate) keep_going: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeDataBindGraphApplyPhase {
    BeforeStatefulAdvance,
    AfterStatefulAdvance { elapsed_positive: bool },
    Immediate,
    PublicUpdate,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataBindGraphTargetNode {
    pub(crate) target: RuntimeDataBindGraphTarget,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeDataBindGraphTarget {
    Number { global_id: u32 },
    Integer { global_id: u32 },
    Boolean { global_id: u32 },
    String { global_id: u32 },
    Color { global_id: u32 },
    Enum { global_id: u32 },
    Asset { global_id: u32 },
    Artboard { global_id: u32 },
    List { global_id: u32 },
    Trigger { global_id: u32 },
    ViewModel { global_id: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeDataBindGraphValue {
    Number(f32),
    Boolean(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u64),
    SymbolListIndex(u64),
    List { item_count: usize },
    ListLength(usize),
    Asset(u64),
    Artboard(u64),
    Trigger(u64),
    ViewModel(RuntimeViewModelPointer),
}

impl RuntimeDataBindGraphValue {
    pub(crate) fn resolve_from_owned_view_model_instance(
        &self,
        context: &RuntimeOwnedViewModelInstance,
        path: &[u32],
    ) -> Option<Self> {
        if path.len() < 2 || usize::try_from(path[0]).ok()? != context.view_model_index {
            return None;
        }
        if path.len() != 2
            && !matches!(
                self,
                Self::Number(_)
                    | Self::Boolean(_)
                    | Self::String(_)
                    | Self::Color(_)
                    | Self::Enum(_)
                    | Self::SymbolListIndex(_)
                    | Self::List { .. }
                    | Self::Asset(_)
                    | Self::Artboard(_)
                    | Self::Trigger(_)
                    | Self::ViewModel(_)
            )
        {
            return None;
        }
        match self {
            Self::Number(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .number_value_by_property_path(&property_path)
                    .map(Self::Number)
            }
            Self::Boolean(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .boolean_value_by_property_path(&property_path)
                    .map(Self::Boolean)
            }
            Self::String(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .string_value_by_property_path(&property_path)
                    .map(|value| Self::String(value.to_vec()))
            }
            Self::Color(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .color_value_by_property_path(&property_path)
                    .map(Self::Color)
            }
            Self::Enum(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .enum_value_by_property_path(&property_path)
                    .map(Self::Enum)
            }
            Self::SymbolListIndex(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .symbol_list_index_value_by_property_path(&property_path)
                    .map(Self::SymbolListIndex)
            }
            Self::List { .. } => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .list_item_count_by_property_path(&property_path)
                    .map(|item_count| Self::List { item_count })
            }
            Self::ListLength(_) => None,
            Self::Asset(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .asset_value_by_property_path(&property_path)
                    .map(Self::Asset)
            }
            Self::Artboard(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .artboard_value_by_property_path(&property_path)
                    .map(Self::Artboard)
            }
            Self::Trigger(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .trigger_value_by_property_path(&property_path)
                    .map(Self::Trigger)
            }
            Self::ViewModel(_) => {
                let property_path = path[1..]
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()?;
                context
                    .view_model_value_by_property_path(&property_path)
                    .map(Self::ViewModel)
            }
        }
    }

    pub(crate) fn resolve_from_view_model_instance(
        &self,
        file: &RuntimeFile,
        view_model_instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<Self> {
        let context = RuntimeDataContext::from_instance_object(file, view_model_instance)?;
        self.resolve_from_data_context(file, &context, path)
    }

    pub(crate) fn resolve_from_data_context(
        &self,
        file: &RuntimeFile,
        context: &RuntimeDataContext<'_>,
        path: &[u32],
    ) -> Option<Self> {
        if matches!(self, Self::ViewModel(_)) {
            return context.absolute_instance(path).map(|reference| {
                Self::ViewModel(RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                })
            });
        }

        let source = context.absolute_property(path)?;
        match self {
            Self::Number(_) => file
                .view_model_instance_number_value_for_object(source)
                .map(Self::Number),
            Self::Boolean(_) => file
                .view_model_instance_boolean_value_for_object(source)
                .map(Self::Boolean),
            Self::String(_) => file
                .view_model_instance_string_value_bytes_for_object(source)
                .map(|value| Self::String(value.to_vec())),
            Self::Color(_) => file
                .view_model_instance_color_value_for_object(source)
                .map(Self::Color),
            Self::Enum(_) => (source.type_name == "ViewModelInstanceEnum")
                .then(|| source.uint_property("propertyValue"))
                .flatten()
                .map(Self::Enum),
            Self::SymbolListIndex(_) => file
                .view_model_instance_symbol_list_index_value_for_object(source)
                .map(Self::SymbolListIndex),
            Self::List { .. } => file
                .view_model_instance_list_size_for_object(source)
                .map(|item_count| Self::List { item_count }),
            Self::ListLength(_) => file
                .view_model_instance_list_size_for_object(source)
                .map(Self::ListLength),
            Self::Asset(_) => (source.type_name == "ViewModelInstanceAssetImage")
                .then(|| source.uint_property("propertyValue"))
                .flatten()
                .map(Self::Asset),
            Self::Artboard(_) => (source.type_name == "ViewModelInstanceArtboard")
                .then(|| source.uint_property("propertyValue"))
                .flatten()
                .map(Self::Artboard),
            Self::Trigger(_) => file
                .view_model_instance_trigger_count_for_object(source)
                .map(Self::Trigger),
            Self::ViewModel(_) => None,
        }
    }
}
