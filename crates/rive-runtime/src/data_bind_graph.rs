use std::collections::BTreeMap;

use crate::{
    RuntimeDataBindGraphFormulaState, RuntimeDataBindGraphInterpolatorState,
    RuntimeTransitionInterpolator, RuntimeViewModelPointer,
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
