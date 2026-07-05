use std::collections::{BTreeMap, BTreeSet};

use rive_binary::{RuntimeFile, RuntimeObject};

use crate::draw::color_lerp;
use crate::{
    RuntimeDataContext, RuntimeImportedViewModelInstanceContext, RuntimeOwnedViewModelInstance,
    RuntimeStateMachine, RuntimeTransitionInterpolator, RuntimeViewModelPointer,
    StateMachineBindableArtboardInstance, StateMachineBindableAssetInstance,
    StateMachineBindableBooleanInstance, StateMachineBindableColorInstance,
    StateMachineBindableEnumInstance, StateMachineBindableIntegerInstance,
    StateMachineBindableListInstance, StateMachineBindableNumberInstance,
    StateMachineBindableStringInstance, StateMachineBindableTriggerInstance,
    StateMachineBindableViewModelInstance,
    runtime_view_model_view_model_property_path_for_name_path,
};

pub(crate) const DATA_BIND_FLAG_DIRECTION_TO_SOURCE: u64 = 1 << 0;
pub(crate) const DATA_BIND_FLAG_TWO_WAY: u64 = 1 << 1;

pub(crate) fn data_bind_flags_apply_source_to_target(flags: u64) -> bool {
    flags & DATA_BIND_FLAG_TWO_WAY != 0 || flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE == 0
}

pub(crate) fn data_bind_flags_apply_target_to_source(flags: u64) -> bool {
    flags & DATA_BIND_FLAG_TWO_WAY != 0 || flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0
}

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

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataBindGraphFormulaRandomSource {
    values: Vec<f32>,
    next_index: usize,
    call_count: usize,
    seeded_values: bool,
    fallback_seed: u32,
}

impl Default for RuntimeDataBindGraphFormulaRandomSource {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            next_index: 0,
            call_count: 0,
            seeded_values: false,
            fallback_seed: 1,
        }
    }
}

impl RuntimeDataBindGraphFormulaRandomSource {
    pub(crate) fn set_values(&mut self, values: &[f32]) {
        self.values.clear();
        self.values.extend_from_slice(values);
        self.next_index = 0;
        self.call_count = 0;
        self.seeded_values = true;
    }

    pub(crate) fn next_value(&mut self) -> f32 {
        self.call_count += 1;
        if self.seeded_values {
            let value = self.values.get(self.next_index).copied().unwrap_or(0.0);
            if self.next_index < self.values.len() {
                self.next_index += 1;
            }
            return value;
        }

        // Mirrors C++ include/rive/math/random.hpp's non-TESTING rand()
        // sequence on macOS, where the default seed behaves like srand(1).
        self.fallback_seed = ((16807_u64 * u64::from(self.fallback_seed)) % 2147483647) as u32;
        self.fallback_seed as f32 / 2147483647_f32
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

pub(crate) struct RuntimeDataBindGraphTargetsMut<'a> {
    pub(crate) numbers: &'a mut [StateMachineBindableNumberInstance],
    pub(crate) integers: &'a mut [StateMachineBindableIntegerInstance],
    pub(crate) booleans: &'a mut [StateMachineBindableBooleanInstance],
    pub(crate) strings: &'a mut [StateMachineBindableStringInstance],
    pub(crate) colors: &'a mut [StateMachineBindableColorInstance],
    pub(crate) enums: &'a mut [StateMachineBindableEnumInstance],
    pub(crate) assets: &'a mut [StateMachineBindableAssetInstance],
    pub(crate) artboards: &'a mut [StateMachineBindableArtboardInstance],
    pub(crate) lists: &'a mut [StateMachineBindableListInstance],
    pub(crate) triggers: &'a mut [StateMachineBindableTriggerInstance],
    pub(crate) view_models: &'a mut [StateMachineBindableViewModelInstance],
    pub(crate) include_view_models: bool,
}

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

pub(crate) fn runtime_data_bind_graph_converter_contains_source_change_random(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::Formula { tokens } => {
            tokens.iter().any(|token| match token {
                RuntimeDataBindGraphFormulaToken::Function {
                    function_type,
                    random_mode,
                    ..
                } => *function_type == 16 && *random_mode == 2,
                _ => false,
            })
        }
        RuntimeDataBindGraphConverter::Group(converters) => converters
            .iter()
            .any(runtime_data_bind_graph_converter_contains_source_change_random),
        _ => false,
    }
}

pub(crate) fn runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::ToNumber
        | RuntimeDataBindGraphConverter::OperationValue { .. }
        | RuntimeDataBindGraphConverter::OperationViewModel { .. }
        | RuntimeDataBindGraphConverter::Formula { .. } => true,
        RuntimeDataBindGraphConverter::Group(converters) => converters
            .first()
            .is_some_and(runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source),
        _ => false,
    }
}

fn runtime_data_bind_graph_converter_preserves_symbol_list_index_source_on_number_target_apply(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    matches!(
        converter,
        RuntimeDataBindGraphConverter::ToNumber | RuntimeDataBindGraphConverter::Formula { .. }
    ) || (matches!(converter, RuntimeDataBindGraphConverter::Group(_))
        && runtime_data_bind_graph_converter_accepts_symbol_list_index_number_source(converter))
}

fn runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    matches!(
        converter,
        RuntimeDataBindGraphConverter::ToNumber | RuntimeDataBindGraphConverter::Formula { .. }
    ) || matches!(
        converter,
        RuntimeDataBindGraphConverter::Group(converters)
            if runtime_data_bind_graph_group_operation_formula_accepts_non_number_source(converters)
                || runtime_data_bind_graph_group_formula_operation_accepts_non_number_source(converters)
    )
}

fn runtime_data_bind_graph_converter_preserves_trigger_source_on_number_target_apply(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    matches!(converter, RuntimeDataBindGraphConverter::Formula { .. })
        || matches!(
            converter,
            RuntimeDataBindGraphConverter::Group(converters)
                if runtime_data_bind_graph_group_operation_formula_accepts_non_number_source(converters)
                    || runtime_data_bind_graph_group_formula_operation_accepts_non_number_source(converters)
        )
}

pub(crate) fn runtime_data_bind_graph_group_operation_formula_accepts_non_number_source(
    converters: &[RuntimeDataBindGraphConverter],
) -> bool {
    let Some(formula_index) = converters
        .iter()
        .position(|converter| matches!(converter, RuntimeDataBindGraphConverter::Formula { .. }))
    else {
        return false;
    };
    converters.len() >= 2
        && formula_index > 0
        && converters.iter().enumerate().all(|(index, converter)| {
            index == formula_index
                || matches!(
                    converter,
                    RuntimeDataBindGraphConverter::OperationValue { .. }
                )
        })
}

pub(crate) fn runtime_data_bind_graph_group_formula_operation_accepts_non_number_source(
    converters: &[RuntimeDataBindGraphConverter],
) -> bool {
    converters.len() >= 2
        && matches!(
            converters.first(),
            Some(RuntimeDataBindGraphConverter::Formula { .. })
        )
        && converters[1..].iter().all(|converter| {
            matches!(
                converter,
                RuntimeDataBindGraphConverter::OperationValue { .. }
            )
        })
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataBindGraphFormulaState {
    randoms: Vec<f32>,
}

impl RuntimeDataBindGraphFormulaState {
    fn random_value(
        &mut self,
        random_mode: u64,
        index: usize,
        source: &mut RuntimeDataBindGraphFormulaRandomSource,
    ) -> f32 {
        if random_mode == 1 {
            return source.next_value();
        }
        while self.randoms.len() <= index {
            self.randoms.push(source.next_value());
        }
        self.randoms[index]
    }

    fn clear(&mut self) {
        self.randoms.clear();
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataBindGraphInterpolatorState {
    advance_count: u8,
    advancer: Option<RuntimeDataBindGraphInterpolatorAdvancer>,
}

impl RuntimeDataBindGraphInterpolatorState {
    fn new() -> Self {
        Self::default()
    }

    fn is_initialized(&self) -> bool {
        self.advancer.is_some()
    }

    fn convert(
        &mut self,
        duration: f32,
        _interpolator: Option<RuntimeTransitionInterpolator>,
        input: &RuntimeDataBindGraphValue,
    ) -> Option<RuntimeDataBindGraphValue> {
        if duration == 0.0
            && let Some(advancer) = &mut self.advancer
        {
            if let Some(input_value) = RuntimeDataBindGraphInterpolatorValue::from_graph(input) {
                advancer.reset_to_start(&input_value);
            }
            return Some(input.clone());
        }

        if self.advancer.is_none() {
            let Some(input_value) = RuntimeDataBindGraphInterpolatorValue::from_graph(input) else {
                return Some(input.clone());
            };
            self.advancer = Some(RuntimeDataBindGraphInterpolatorAdvancer::new(&input_value));
        }

        let Some(input_value) = RuntimeDataBindGraphInterpolatorValue::from_graph(input) else {
            return Some(input.clone());
        };
        let advancer = self.advancer.as_mut().expect("advancer initialized");
        if self.advance_count < 2 {
            advancer.reset_values(&input_value);
        } else {
            advancer.update_values(&input_value);
        }
        Some(advancer.current_value().to_graph_value())
    }

    fn advance(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        if self.advance_count < 2 && elapsed_seconds > 0.0 {
            self.advance_count += 1;
        }
        let Some(advancer) = &mut self.advancer else {
            return RuntimeDataBindGraphStatefulAdvance {
                changed: true,
                keep_going: true,
            };
        };
        advancer.advance(duration, interpolator, elapsed_seconds)
    }
}

#[derive(Debug, Clone)]
struct RuntimeDataBindGraphInterpolatorAdvancer {
    animation_data_a: RuntimeDataBindGraphInterpolatorAnimationData,
    animation_data_b: RuntimeDataBindGraphInterpolatorAnimationData,
    current_value: RuntimeDataBindGraphInterpolatorValue,
    is_smoothing_animation: bool,
}

impl RuntimeDataBindGraphInterpolatorAdvancer {
    fn new(input: &RuntimeDataBindGraphInterpolatorValue) -> Self {
        let default_value = input.default_for_kind();
        Self {
            animation_data_a: RuntimeDataBindGraphInterpolatorAnimationData::new(
                default_value.clone(),
            ),
            animation_data_b: RuntimeDataBindGraphInterpolatorAnimationData::new(
                default_value.clone(),
            ),
            current_value: default_value,
            is_smoothing_animation: false,
        }
    }

    fn current_value(&self) -> &RuntimeDataBindGraphInterpolatorValue {
        &self.current_value
    }

    fn reset_values(&mut self, input: &RuntimeDataBindGraphInterpolatorValue) {
        if self.is_smoothing_animation {
            self.animation_data_b.reset_values(input);
        } else {
            self.animation_data_a.reset_values(input);
        }
        self.current_value.copy_from(input);
    }

    fn reset_to_start(&mut self, input: &RuntimeDataBindGraphInterpolatorValue) {
        self.reset_values(input);
        self.is_smoothing_animation = false;
        self.animation_data_a.elapsed_seconds = 0.0;
        self.animation_data_b.elapsed_seconds = 0.0;
    }

    fn update_values(&mut self, input: &RuntimeDataBindGraphInterpolatorValue) {
        if self.current_animation_data().to.compare(input) {
            return;
        }

        if self.current_animation_data().elapsed_seconds != 0.0 {
            if self.is_smoothing_animation {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
            }
            self.is_smoothing_animation = true;
        } else {
            self.is_smoothing_animation = false;
        }

        let current_value = self.current_value.clone();
        let animation_data = self.current_animation_data_mut();
        animation_data.from.copy_from(&current_value);
        animation_data.to.copy_from(input);
        animation_data.elapsed_seconds = 0.0;
    }

    fn advance(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        let animation_index = self.current_animation_index();
        if self.animation_data(animation_index).to == self.current_value || elapsed_seconds == 0.0 {
            return RuntimeDataBindGraphStatefulAdvance::default();
        }

        let previous_value = self.current_value.clone();
        self.advance_animation_data(duration, interpolator, elapsed_seconds, animation_index);
        RuntimeDataBindGraphStatefulAdvance {
            changed: self.current_value != previous_value,
            keep_going: self.animation_data(animation_index).elapsed_seconds < duration,
        }
    }

    fn advance_animation_data(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
        animation_index: usize,
    ) {
        if self.is_smoothing_animation {
            let factor = runtime_data_bind_graph_interpolator_factor(
                duration,
                interpolator,
                self.animation_data_a.elapsed_seconds,
            );
            let interpolated = self.animation_data_a.interpolate(factor);
            self.animation_data_b.from.copy_from(&interpolated);
            if factor == 1.0 {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.is_smoothing_animation = false;
            } else {
                self.animation_data_a.elapsed_seconds += elapsed_seconds;
            }
        }

        if self.animation_data(animation_index).elapsed_seconds >= duration {
            self.current_value
                .copy_from(&self.animation_data(animation_index).to.clone());
            if self.is_smoothing_animation {
                self.is_smoothing_animation = false;
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.animation_data_a.elapsed_seconds = 0.0;
                self.animation_data_b.elapsed_seconds = 0.0;
            } else {
                self.animation_data_a.elapsed_seconds = 0.0;
            }
            return;
        }

        self.animation_data_mut(animation_index).elapsed_seconds += elapsed_seconds;
        let factor = runtime_data_bind_graph_interpolator_factor(
            duration,
            interpolator,
            self.animation_data(animation_index).elapsed_seconds,
        );
        let interpolated = self.animation_data(animation_index).interpolate(factor);
        self.current_value.copy_from(&interpolated);
    }

    fn current_animation_data(&self) -> &RuntimeDataBindGraphInterpolatorAnimationData {
        self.animation_data(self.current_animation_index())
    }

    fn current_animation_data_mut(&mut self) -> &mut RuntimeDataBindGraphInterpolatorAnimationData {
        self.animation_data_mut(self.current_animation_index())
    }

    fn current_animation_index(&self) -> usize {
        usize::from(self.is_smoothing_animation)
    }

    fn animation_data(&self, index: usize) -> &RuntimeDataBindGraphInterpolatorAnimationData {
        if index == 0 {
            &self.animation_data_a
        } else {
            &self.animation_data_b
        }
    }

    fn animation_data_mut(
        &mut self,
        index: usize,
    ) -> &mut RuntimeDataBindGraphInterpolatorAnimationData {
        if index == 0 {
            &mut self.animation_data_a
        } else {
            &mut self.animation_data_b
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeDataBindGraphInterpolatorAnimationData {
    elapsed_seconds: f32,
    from: RuntimeDataBindGraphInterpolatorValue,
    to: RuntimeDataBindGraphInterpolatorValue,
}

impl RuntimeDataBindGraphInterpolatorAnimationData {
    fn new(value: RuntimeDataBindGraphInterpolatorValue) -> Self {
        Self {
            elapsed_seconds: 0.0,
            from: value.clone(),
            to: value,
        }
    }

    fn reset_values(&mut self, input: &RuntimeDataBindGraphInterpolatorValue) {
        self.from.copy_from(input);
        self.to.copy_from(input);
    }

    fn copy_from(&mut self, source: &Self) {
        self.from.copy_from(&source.from);
        self.to.copy_from(&source.to);
        self.elapsed_seconds = source.elapsed_seconds;
    }

    fn interpolate(&self, factor: f32) -> RuntimeDataBindGraphInterpolatorValue {
        self.from.interpolate(&self.to, factor)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeDataBindGraphInterpolatorValue {
    Number(f32),
    Color(u32),
}

impl RuntimeDataBindGraphInterpolatorValue {
    fn from_graph(value: &RuntimeDataBindGraphValue) -> Option<Self> {
        match value {
            RuntimeDataBindGraphValue::Number(value) => Some(Self::Number(*value)),
            RuntimeDataBindGraphValue::Color(value) => Some(Self::Color(*value)),
            _ => None,
        }
    }

    fn default_for_kind(&self) -> Self {
        match self {
            Self::Number(_) => Self::Number(0.0),
            Self::Color(_) => Self::Color(0),
        }
    }

    fn copy_from(&mut self, source: &Self) {
        if std::mem::discriminant(self) == std::mem::discriminant(source) {
            *self = source.clone();
        }
    }

    fn compare(&self, comparand: &Self) -> bool {
        self == comparand
    }

    fn interpolate(&self, to: &Self, factor: f32) -> Self {
        match (self, to) {
            (Self::Number(from), Self::Number(to)) => {
                Self::Number(*to * factor + *from * (1.0 - factor))
            }
            (Self::Color(from), Self::Color(to)) => Self::Color(color_lerp(*from, *to, factor)),
            _ => self.clone(),
        }
    }

    fn to_graph_value(&self) -> RuntimeDataBindGraphValue {
        match self {
            Self::Number(value) => RuntimeDataBindGraphValue::Number(*value),
            Self::Color(value) => RuntimeDataBindGraphValue::Color(*value),
        }
    }
}

fn runtime_data_bind_graph_interpolator_factor(
    duration: f32,
    interpolator: Option<RuntimeTransitionInterpolator>,
    elapsed_seconds: f32,
) -> f32 {
    let mut factor = if duration > 0.0 {
        f32::min(1.0, elapsed_seconds / duration)
    } else {
        1.0
    };
    if let Some(interpolator) = interpolator {
        factor = interpolator.transform(factor);
    }
    factor
}

fn runtime_data_bind_graph_converter_preserves_string_source_on_main_to_source_target_apply(
    converter: &RuntimeDataBindGraphConverter,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::StringTrim { .. }
        | RuntimeDataBindGraphConverter::StringRemoveZeros
        | RuntimeDataBindGraphConverter::StringPad { .. } => true,
        RuntimeDataBindGraphConverter::Group(converters) => {
            !converters.is_empty()
                && converters.iter().all(
                    runtime_data_bind_graph_converter_preserves_string_source_on_main_to_source_target_apply,
                )
        }
        _ => false,
    }
}

fn runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path(
    converter: &mut RuntimeDataBindGraphConverter,
    path: &[u32],
    value: f32,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            default_operation_value,
            source_path: Some(source_path),
            ..
        } if source_path.as_slice() == path
            && (*operation_value != value || *default_operation_value != value) =>
        {
            *operation_value = value;
            *default_operation_value = value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            let mut changed = false;
            for converter in converters {
                changed |=
                    runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path(
                        converter, path, value,
                    );
            }
            changed
        }
        _ => false,
    }
}

fn runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_imported_context_path(
    converter: &mut RuntimeDataBindGraphConverter,
    path: &[u32],
    value: f32,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            source_path: Some(source_path),
            ..
        } if source_path.as_slice() == path && *operation_value != value => {
            *operation_value = value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            let mut changed = false;
            for converter in converters {
                changed |= runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_imported_context_path(
                    converter, path, value,
                );
            }
            changed
        }
        _ => false,
    }
}

fn runtime_data_bind_graph_reset_operation_view_model_converter_to_default(
    converter: &mut RuntimeDataBindGraphConverter,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            default_operation_value,
            ..
        } if *operation_value != *default_operation_value => {
            *operation_value = *default_operation_value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            let mut changed = false;
            for converter in converters {
                changed |= runtime_data_bind_graph_reset_operation_view_model_converter_to_default(
                    converter,
                );
            }
            changed
        }
        _ => false,
    }
}

fn runtime_data_bind_graph_refresh_operation_view_model_converter_for_imported_context(
    file: &RuntimeFile,
    converter: &mut RuntimeDataBindGraphConverter,
    context: &RuntimeDataContext<'_>,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            source_path: Some(source_path),
            ..
        } => {
            let value = context
                .absolute_property(source_path)
                .and_then(|source| file.view_model_instance_number_value_for_object(source))
                .unwrap_or(0.0);
            if *operation_value == value {
                return false;
            }
            *operation_value = value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            let mut changed = false;
            for converter in converters {
                changed |=
                    runtime_data_bind_graph_refresh_operation_view_model_converter_for_imported_context(
                        file, converter, context,
                    );
            }
            changed
        }
        _ => false,
    }
}

fn runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
    converter: &mut RuntimeDataBindGraphConverter,
    context: &RuntimeOwnedViewModelInstance,
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            source_path: Some(source_path),
            ..
        } => {
            let value = runtime_owned_view_model_number_value_for_source_path(context, source_path)
                .unwrap_or(0.0);
            if *operation_value == value {
                return false;
            }
            *operation_value = value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            let mut changed = false;
            for converter in converters {
                changed |=
                    runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                        converter, context,
                    );
            }
            changed
        }
        _ => false,
    }
}

fn runtime_owned_view_model_number_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<f32> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.number_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_symbol_list_index_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u64> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.symbol_list_index_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_boolean_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<bool> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.boolean_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_enum_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u64> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.enum_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_color_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u32> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.color_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_string_value_for_source_path<'a>(
    context: &'a RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<&'a [u8]> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.string_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_trigger_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u64> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.trigger_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_list_item_count_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<usize> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.list_item_count_by_property_path(&property_path)
}

fn runtime_owned_view_model_asset_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u64> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.asset_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_artboard_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<u64> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.artboard_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_view_model_value_for_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<RuntimeViewModelPointer> {
    let property_path =
        runtime_owned_view_model_property_path_from_source_path(context, source_path)?;
    context.view_model_value_by_property_path(&property_path)
}

fn runtime_owned_view_model_property_path_from_source_path(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<Vec<usize>> {
    if source_path.len() < 2 || usize::try_from(source_path[0]).ok()? != context.view_model_index {
        return None;
    }
    source_path[1..]
        .iter()
        .map(|property_index| usize::try_from(*property_index).ok())
        .collect()
}

pub(crate) fn runtime_data_bind_graph_convert_value(
    converter: &RuntimeDataBindGraphConverter,
    value: &RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    match (converter, value) {
        (RuntimeDataBindGraphConverter::PassThrough, value) => Some(value.clone()),
        (
            RuntimeDataBindGraphConverter::BooleanNegate,
            RuntimeDataBindGraphValue::Boolean(value),
        ) => Some(RuntimeDataBindGraphValue::Boolean(!value)),
        (RuntimeDataBindGraphConverter::BooleanNegate, _) => None,
        (
            RuntimeDataBindGraphConverter::TriggerIncrement,
            RuntimeDataBindGraphValue::Trigger(value),
        ) => Some(RuntimeDataBindGraphValue::Trigger(u64::from(
            (*value as u32).wrapping_add(1),
        ))),
        (RuntimeDataBindGraphConverter::TriggerIncrement, _) => None,
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::Number(value)) => {
            Some(RuntimeDataBindGraphValue::Number(*value))
        }
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::Boolean(value)) => {
            Some(RuntimeDataBindGraphValue::Number(if *value {
                1.0
            } else {
                0.0
            }))
        }
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::Enum(value)) => {
            Some(RuntimeDataBindGraphValue::Number(*value as f32))
        }
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::Color(value)) => {
            Some(RuntimeDataBindGraphValue::Number((*value as i32) as f32))
        }
        (
            RuntimeDataBindGraphConverter::ToNumber,
            RuntimeDataBindGraphValue::SymbolListIndex(value),
        ) => Some(RuntimeDataBindGraphValue::Number(*value as f32)),
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::String(value)) => {
            Some(RuntimeDataBindGraphValue::Number(
                rive_binary::data_converter_to_number_string_value(value),
            ))
        }
        (RuntimeDataBindGraphConverter::ToNumber, _) => None,
        (
            RuntimeDataBindGraphConverter::ListToLength,
            RuntimeDataBindGraphValue::ListLength(value),
        ) => Some(RuntimeDataBindGraphValue::Number(*value as f32)),
        (
            RuntimeDataBindGraphConverter::ListToLength,
            RuntimeDataBindGraphValue::List { item_count },
        ) => Some(RuntimeDataBindGraphValue::Number(*item_count as f32)),
        (RuntimeDataBindGraphConverter::ListToLength, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::NumberToList { .. },
            RuntimeDataBindGraphValue::List { item_count },
        ) => Some(RuntimeDataBindGraphValue::List {
            item_count: *item_count,
        }),
        (
            RuntimeDataBindGraphConverter::NumberToList { has_view_model },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::List {
            item_count: if *has_view_model {
                value.floor().max(0.0) as usize
            } else {
                0
            },
        }),
        (RuntimeDataBindGraphConverter::NumberToList { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::ToString {
                flags, decimals, ..
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_number_value(*value, *flags, *decimals),
        )),
        (
            RuntimeDataBindGraphConverter::ToString { .. },
            RuntimeDataBindGraphValue::Boolean(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_boolean_value(*value),
        )),
        (
            RuntimeDataBindGraphConverter::ToString { .. },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_string_value(value),
        )),
        (
            RuntimeDataBindGraphConverter::ToString { .. },
            RuntimeDataBindGraphValue::Trigger(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_trigger_value(*value),
        )),
        (
            RuntimeDataBindGraphConverter::ToString { .. },
            RuntimeDataBindGraphValue::SymbolListIndex(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_symbol_list_index_value(*value),
        )),
        (
            RuntimeDataBindGraphConverter::ToString { color_format, .. },
            RuntimeDataBindGraphValue::Color(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_to_string_color_value(*value, color_format),
        )),
        (RuntimeDataBindGraphConverter::ToString { .. }, RuntimeDataBindGraphValue::Enum(_)) => {
            Some(RuntimeDataBindGraphValue::String(Vec::new()))
        }
        (RuntimeDataBindGraphConverter::ToString { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::OperationValue {
                operation_type,
                operation_value,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            ),
        )),
        (
            RuntimeDataBindGraphConverter::OperationValue {
                operation_type,
                operation_value,
            },
            RuntimeDataBindGraphValue::SymbolListIndex(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_operation_value(
                *value as f32,
                *operation_value,
                *operation_type,
            ),
        )),
        (RuntimeDataBindGraphConverter::OperationValue { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::OperationViewModel {
                operation_type,
                operation_value,
                ..
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            ),
        )),
        (
            RuntimeDataBindGraphConverter::OperationViewModel {
                operation_type,
                operation_value,
                ..
            },
            RuntimeDataBindGraphValue::SymbolListIndex(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_operation_value(
                *value as f32,
                *operation_value,
                *operation_type,
            ),
        )),
        (RuntimeDataBindGraphConverter::OperationViewModel { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::SystemOperationValue {
                operation_type,
                operation_value,
                reverse,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(if *reverse {
            runtime_data_bind_graph_reverse_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            )
        } else {
            runtime_data_bind_graph_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            )
        })),
        (RuntimeDataBindGraphConverter::SystemOperationValue { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::Rounder { decimals },
            RuntimeDataBindGraphValue::Number(value),
        ) => {
            let rounder = 10.0_f32.powf(*decimals as f32);
            Some(RuntimeDataBindGraphValue::Number(
                (*value * rounder).round() / rounder,
            ))
        }
        (RuntimeDataBindGraphConverter::Rounder { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::RangeMapper {
                min_input,
                max_input,
                min_output,
                max_output,
                flags,
                interpolation_type,
                interpolator,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_range_mapper(
                *value,
                *min_input,
                *max_input,
                *min_output,
                *max_output,
                *flags,
                *interpolation_type,
                *interpolator,
            ),
        )),
        (RuntimeDataBindGraphConverter::RangeMapper { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::StringTrim { trim_type },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_string_trim_value(value, *trim_type),
        )),
        (RuntimeDataBindGraphConverter::StringTrim { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::StringRemoveZeros,
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_string_remove_zeros_value(value),
        )),
        (RuntimeDataBindGraphConverter::StringRemoveZeros, _) => None,
        (
            RuntimeDataBindGraphConverter::StringPad {
                length,
                text,
                pad_type,
            },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(
            rive_binary::data_converter_string_pad_value(value, *length, text, *pad_type),
        )),
        (RuntimeDataBindGraphConverter::StringPad { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::Formula { tokens },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_formula(*value, tokens),
        )),
        (
            RuntimeDataBindGraphConverter::Formula { tokens },
            RuntimeDataBindGraphValue::SymbolListIndex(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_formula(*value as f32, tokens),
        )),
        (RuntimeDataBindGraphConverter::Formula { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (RuntimeDataBindGraphConverter::Group(converters), value) => {
            let mut value = value.clone();
            for converter in converters {
                value = runtime_data_bind_graph_convert_value(converter, &value)?;
            }
            Some(value)
        }
        (RuntimeDataBindGraphConverter::Interpolator { .. }, _) => None,
        (RuntimeDataBindGraphConverter::Unsupported, _) => None,
    }
}

pub(crate) fn runtime_data_bind_graph_reverse_convert_value(
    converter: &RuntimeDataBindGraphConverter,
    value: &RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    match (converter, value) {
        (RuntimeDataBindGraphConverter::PassThrough, value) => Some(value.clone()),
        (
            RuntimeDataBindGraphConverter::BooleanNegate,
            RuntimeDataBindGraphValue::Boolean(value),
        ) => Some(RuntimeDataBindGraphValue::Boolean(!value)),
        (RuntimeDataBindGraphConverter::BooleanNegate, _) => None,
        (
            RuntimeDataBindGraphConverter::TriggerIncrement,
            RuntimeDataBindGraphValue::Trigger(value),
        ) => Some(RuntimeDataBindGraphValue::Trigger(*value)),
        (RuntimeDataBindGraphConverter::TriggerIncrement, _) => None,
        (RuntimeDataBindGraphConverter::ToNumber, RuntimeDataBindGraphValue::Number(value)) => {
            Some(RuntimeDataBindGraphValue::Number(*value))
        }
        (RuntimeDataBindGraphConverter::ToNumber, _) => None,
        (
            RuntimeDataBindGraphConverter::ToString { .. },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(value.clone())),
        (RuntimeDataBindGraphConverter::ToString { .. }, _) => {
            Some(RuntimeDataBindGraphValue::String(Vec::new()))
        }
        (
            RuntimeDataBindGraphConverter::StringTrim { .. },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(value.clone())),
        (RuntimeDataBindGraphConverter::StringTrim { .. }, value) => Some(value.clone()),
        (
            RuntimeDataBindGraphConverter::StringRemoveZeros,
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(value.clone())),
        (RuntimeDataBindGraphConverter::StringRemoveZeros, value) => Some(value.clone()),
        (
            RuntimeDataBindGraphConverter::StringPad { .. },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(RuntimeDataBindGraphValue::String(value.clone())),
        (RuntimeDataBindGraphConverter::StringPad { .. }, value) => Some(value.clone()),
        (RuntimeDataBindGraphConverter::ListToLength, RuntimeDataBindGraphValue::Number(value)) => {
            Some(RuntimeDataBindGraphValue::Number(*value))
        }
        (RuntimeDataBindGraphConverter::ListToLength, RuntimeDataBindGraphValue::ListLength(_)) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (RuntimeDataBindGraphConverter::ListToLength, _) => None,
        (
            RuntimeDataBindGraphConverter::OperationValue {
                operation_type,
                operation_value,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_reverse_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            ),
        )),
        (RuntimeDataBindGraphConverter::OperationValue { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::OperationViewModel {
                operation_type,
                operation_value,
                ..
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_reverse_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            ),
        )),
        (RuntimeDataBindGraphConverter::OperationViewModel { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::RangeMapper {
                min_input,
                max_input,
                min_output,
                max_output,
                flags,
                interpolation_type,
                interpolator,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_range_mapper(
                *value,
                *min_output,
                *max_output,
                *min_input,
                *max_input,
                *flags,
                *interpolation_type,
                *interpolator,
            ),
        )),
        (RuntimeDataBindGraphConverter::RangeMapper { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::Rounder { .. },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(*value)),
        (RuntimeDataBindGraphConverter::Rounder { .. }, _) => None,
        (
            RuntimeDataBindGraphConverter::SystemOperationValue {
                operation_type,
                operation_value,
                reverse,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(if *reverse {
            runtime_data_bind_graph_reverse_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            )
        } else {
            runtime_data_bind_graph_convert_operation_value(
                *value,
                *operation_value,
                *operation_type,
            )
        })),
        (RuntimeDataBindGraphConverter::SystemOperationValue { .. }, _) => {
            Some(RuntimeDataBindGraphValue::Number(0.0))
        }
        (
            RuntimeDataBindGraphConverter::Formula { tokens },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_formula(*value, tokens),
        )),
        (RuntimeDataBindGraphConverter::Formula { .. }, _) => None,
        (RuntimeDataBindGraphConverter::Group(converters), value) => {
            let mut value = value.clone();
            for converter in converters.iter().rev() {
                value = runtime_data_bind_graph_reverse_convert_value(converter, &value)?;
            }
            Some(value)
        }
        _ => None,
    }
}

fn runtime_data_bind_graph_convert_formula(
    input: f32,
    tokens: &[RuntimeDataBindGraphFormulaToken],
) -> f32 {
    let mut state = RuntimeDataBindGraphFormulaState::default();
    let mut random_source = RuntimeDataBindGraphFormulaRandomSource::default();
    runtime_data_bind_graph_convert_formula_with_state(
        input,
        tokens,
        &mut state,
        &mut random_source,
    )
}

fn runtime_data_bind_graph_convert_formula_value_with_state(
    value: &RuntimeDataBindGraphValue,
    tokens: &[RuntimeDataBindGraphFormulaToken],
    state: &mut RuntimeDataBindGraphFormulaState,
    random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
) -> Option<RuntimeDataBindGraphValue> {
    match value {
        RuntimeDataBindGraphValue::Number(value) => Some(RuntimeDataBindGraphValue::Number(
            runtime_data_bind_graph_convert_formula_with_state(
                *value,
                tokens,
                state,
                random_source,
            ),
        )),
        RuntimeDataBindGraphValue::SymbolListIndex(value) => Some(
            RuntimeDataBindGraphValue::Number(runtime_data_bind_graph_convert_formula_with_state(
                *value as f32,
                tokens,
                state,
                random_source,
            )),
        ),
        _ => Some(RuntimeDataBindGraphValue::Number(0.0)),
    }
}

fn runtime_data_bind_graph_convert_formula_with_state(
    input: f32,
    tokens: &[RuntimeDataBindGraphFormulaToken],
    state: &mut RuntimeDataBindGraphFormulaState,
    random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
) -> f32 {
    let mut result = input;
    let mut stack = Vec::new();
    let mut current_random = 0;
    for token in tokens {
        match token {
            RuntimeDataBindGraphFormulaToken::Input => stack.push(input),
            RuntimeDataBindGraphFormulaToken::Value(value) => stack.push(*value),
            RuntimeDataBindGraphFormulaToken::Operation { operation_type } => {
                if stack.len() > 1 {
                    let right = stack.pop().expect("stack length checked");
                    let left = stack.pop().expect("stack length checked");
                    stack.push(runtime_data_bind_graph_apply_formula_operation(
                        left,
                        right,
                        *operation_type,
                    ));
                }
            }
            RuntimeDataBindGraphFormulaToken::Function {
                function_type,
                arguments_count,
                random_mode,
            } => {
                let random_value = if *function_type == 16 {
                    let value = state.random_value(*random_mode, current_random, random_source);
                    current_random += 1;
                    Some(value)
                } else {
                    None
                };
                let value = runtime_data_bind_graph_apply_formula_function(
                    &mut stack,
                    *function_type,
                    *arguments_count,
                    random_value,
                );
                stack.push(value);
            }
        }
    }
    if stack.len() == 1 {
        result = stack.pop().expect("stack length checked");
    }
    result
}

fn runtime_data_bind_graph_apply_formula_operation(
    left: f32,
    right: f32,
    operation_type: u64,
) -> f32 {
    match operation_type {
        0 => left + right,
        1 => left - right,
        2 => left * right,
        3 => left / right,
        4 => runtime_data_bind_graph_positive_mod(left, right),
        _ => 0.0,
    }
}

fn runtime_data_bind_graph_apply_formula_function(
    stack: &mut Vec<f32>,
    function_type: u64,
    total_arguments: usize,
    random_value: Option<f32>,
) -> f32 {
    let mut function_arguments = Vec::new();
    for _ in 0..total_arguments {
        if let Some(function_argument) = stack.pop() {
            function_arguments.push(function_argument);
        }
    }

    match function_type {
        0 => {
            if function_arguments.is_empty() {
                0.0
            } else {
                let mut min_value = function_arguments[0];
                for value in function_arguments.iter().skip(1) {
                    if *value < min_value {
                        min_value = *value;
                    }
                }
                min_value
            }
        }
        1 => {
            if function_arguments.is_empty() {
                0.0
            } else {
                let mut max_value = function_arguments[0];
                for value in function_arguments.iter().skip(1) {
                    if *value > max_value {
                        max_value = *value;
                    }
                }
                max_value
            }
        }
        2 => function_arguments
            .last()
            .copied()
            .map(f32::round)
            .unwrap_or(0.0),
        3 => function_arguments
            .last()
            .copied()
            .map(f32::ceil)
            .unwrap_or(0.0),
        4 => function_arguments
            .last()
            .copied()
            .map(f32::floor)
            .unwrap_or(0.0),
        5 => function_arguments
            .last()
            .copied()
            .map(f32::sqrt)
            .unwrap_or(0.0),
        6 => {
            if function_arguments.len() > 1 {
                let exponent = function_arguments[function_arguments.len() - 2];
                let x = function_arguments[function_arguments.len() - 1];
                x.powf(exponent)
            } else {
                0.0
            }
        }
        7 => function_arguments
            .last()
            .copied()
            .map(f32::exp)
            .unwrap_or(0.0),
        8 => function_arguments
            .last()
            .copied()
            .map(f32::ln)
            .unwrap_or(0.0),
        9 => function_arguments
            .last()
            .copied()
            .map(f32::cos)
            .unwrap_or(0.0),
        10 => function_arguments
            .last()
            .copied()
            .map(f32::sin)
            .unwrap_or(0.0),
        11 => function_arguments
            .last()
            .copied()
            .map(f32::tan)
            .unwrap_or(0.0),
        12 => function_arguments
            .last()
            .copied()
            .map(f32::acos)
            .unwrap_or(0.0),
        13 => function_arguments
            .last()
            .copied()
            .map(f32::asin)
            .unwrap_or(0.0),
        14 => function_arguments
            .last()
            .copied()
            .map(f32::atan)
            .unwrap_or(0.0),
        15 => {
            if function_arguments.len() > 1 {
                let argument1 = function_arguments[function_arguments.len() - 1];
                let argument2 = function_arguments[function_arguments.len() - 2];
                argument1.atan2(argument2)
            } else {
                0.0
            }
        }
        16 => {
            let random_value = random_value.unwrap_or(0.0);
            let mut lower_bound = 0.0;
            let mut upper_bound = 1.0;
            if function_arguments.len() == 1 {
                upper_bound = function_arguments[function_arguments.len() - 1];
            } else if function_arguments.len() > 1 {
                lower_bound = function_arguments[function_arguments.len() - 1];
                upper_bound = function_arguments[function_arguments.len() - 2];
            }
            lower_bound + (upper_bound - lower_bound) * random_value
        }
        _ => 0.0,
    }
}

fn runtime_data_bind_graph_reverse_convert_operation_value(
    input: f32,
    operation_value: f32,
    operation_type: u64,
) -> f32 {
    match operation_type {
        0 => input - operation_value,
        1 => input + operation_value,
        2 => input / operation_value,
        3 => input * operation_value,
        4 => input,
        5 => input.powf(2.0),
        6 => input.powf(1.0 / operation_value),
        7 => input.ln(),
        8 => input.exp(),
        9 => input.acos(),
        10 => input.asin(),
        11 => input.atan(),
        12 => input.cos(),
        13 => input.sin(),
        14 => input.tan(),
        15..=18 => input,
        _ => operation_value,
    }
}

fn runtime_data_bind_graph_convert_operation_value(
    input: f32,
    operation_value: f32,
    operation_type: u64,
) -> f32 {
    match operation_type {
        0 => input + operation_value,
        1 => input - operation_value,
        2 => input * operation_value,
        3 => input / operation_value,
        4 => runtime_data_bind_graph_positive_mod(input, operation_value),
        5 => input.sqrt(),
        6 => input.powf(operation_value),
        7 => input.exp(),
        8 => input.ln(),
        9 => input.cos(),
        10 => input.sin(),
        11 => input.tan(),
        12 => input.acos(),
        13 => input.asin(),
        14 => input.atan(),
        15 => input.atan2(operation_value),
        16 => input.round(),
        17 => input.floor(),
        18 => input.ceil(),
        _ => operation_value,
    }
}

fn runtime_data_bind_graph_convert_range_mapper(
    input: f32,
    min_input: f32,
    max_input: f32,
    min_output: f32,
    max_output: f32,
    flags: u64,
    interpolation_type: u64,
    interpolator: Option<RuntimeTransitionInterpolator>,
) -> f32 {
    if min_output == max_output {
        return min_output;
    }

    const CLAMP_LOWER: u64 = 1 << 0;
    const CLAMP_UPPER: u64 = 1 << 1;
    const MODULO: u64 = 1 << 2;
    const REVERSE: u64 = 1 << 3;

    let mut value = input;
    if value < min_input && flags & CLAMP_LOWER != 0 {
        value = min_input;
    } else if value > max_input && flags & CLAMP_UPPER != 0 {
        value = max_input;
    }
    if (value < min_input || value > max_input) && flags & MODULO != 0 {
        value =
            (runtime_data_bind_graph_positive_mod(value, max_input - min_input) + min_input).abs();
    }

    let mut percent = (value - min_input) / (max_input - min_input);
    if flags & REVERSE != 0 {
        percent = 1.0 - percent;
    }
    if let Some(interpolator) = interpolator {
        if percent > 0.0 && percent < 1.0 {
            percent = interpolator.transform(percent);
        } else if interpolation_type == 0 {
            percent = if percent <= 0.0 { 0.0 } else { 1.0 };
        }
    } else if interpolation_type == 0 {
        percent = if percent <= 0.0 { 0.0 } else { 1.0 };
    }

    percent * max_output + (1.0 - percent) * min_output
}

fn runtime_data_bind_graph_positive_mod(value: f32, mut range: f32) -> f32 {
    if range < 0.0 {
        range = -range;
    }
    let mut value = value % range;
    if value < 0.0 {
        value += range;
    }
    value
}

pub(crate) fn runtime_data_bind_graph_converter_starts_with_to_string(
    converter: Option<&RuntimeDataBindGraphConverter>,
) -> bool {
    match converter {
        Some(RuntimeDataBindGraphConverter::ToString { .. }) => true,
        Some(RuntimeDataBindGraphConverter::Group(converters)) => {
            runtime_data_bind_graph_converter_starts_with_to_string(converters.first())
        }
        _ => false,
    }
}

pub(crate) fn runtime_data_bind_graph_converter(
    file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> Option<RuntimeDataBindGraphConverter> {
    let converter = file.resolved_data_converter_for_data_bind_object(data_bind)?;
    let flags = data_bind.uint_property("flags").unwrap_or(0);
    runtime_data_bind_graph_converter_for_object(file, converter, &mut BTreeSet::new(), true, flags)
}

fn runtime_data_bind_graph_system_operation_value_converter(
    converter: &RuntimeObject,
    flags: u64,
) -> RuntimeDataBindGraphConverter {
    if matches!(
        converter.type_name,
        "DataConverterSystemDegsToRads" | "DataConverterSystemNormalizer"
    ) {
        let to_target = flags & 0b10 != 0 || flags & 0b1 == 0;
        if to_target {
            RuntimeDataBindGraphConverter::SystemOperationValue {
                operation_type: converter.uint_property("operationType").unwrap_or(0),
                operation_value: converter.double_property("operationValue").unwrap_or(1.0),
                reverse: flags & 0b1 != 0,
            }
        } else {
            RuntimeDataBindGraphConverter::Unsupported
        }
    } else {
        RuntimeDataBindGraphConverter::Unsupported
    }
}

fn runtime_data_bind_graph_converter_for_object(
    file: &RuntimeFile,
    converter: &RuntimeObject,
    visiting: &mut BTreeSet<u32>,
    allow_stateful_interpolator: bool,
    data_bind_flags: u64,
) -> Option<RuntimeDataBindGraphConverter> {
    if !visiting.insert(converter.id) {
        return Some(RuntimeDataBindGraphConverter::Unsupported);
    }

    let graph_converter = match converter.type_name {
        "DataConverterGroup" => RuntimeDataBindGraphConverter::Group(
            file.data_converter_group_items_for_object(converter)
                .into_iter()
                .map(|item| {
                    item.converter
                        .and_then(|converter| {
                            runtime_data_bind_graph_converter_for_object(
                                file,
                                converter,
                                visiting,
                                true,
                                data_bind_flags,
                            )
                        })
                        .unwrap_or(RuntimeDataBindGraphConverter::Unsupported)
                })
                .collect(),
        ),
        "DataConverterOperation" | "ScriptedDataConverter" => {
            RuntimeDataBindGraphConverter::PassThrough
        }
        "DataConverterBooleanNegate" => RuntimeDataBindGraphConverter::BooleanNegate,
        "DataConverterTrigger" => RuntimeDataBindGraphConverter::TriggerIncrement,
        "DataConverterToNumber" => RuntimeDataBindGraphConverter::ToNumber,
        "DataConverterListToLength" => RuntimeDataBindGraphConverter::ListToLength,
        "DataConverterNumberToList" => RuntimeDataBindGraphConverter::NumberToList {
            has_view_model: file
                .resolved_view_model_for_number_to_list_converter_object(converter)
                .is_some(),
        },
        "DataConverterToString" => RuntimeDataBindGraphConverter::ToString {
            flags: converter.uint_property("flags").unwrap_or(0),
            decimals: converter.uint_property("decimals").unwrap_or(0),
            color_format: converter
                .string_property_bytes("colorFormat")
                .unwrap_or_default()
                .to_vec(),
        },
        "DataConverterOperationValue" => RuntimeDataBindGraphConverter::OperationValue {
            operation_type: converter.uint_property("operationType").unwrap_or(0),
            operation_value: converter.double_property("operationValue").unwrap_or(1.0),
        },
        "DataConverterOperationViewModel" => {
            let operand =
                runtime_data_bind_graph_default_operation_view_model_operand(file, converter);
            RuntimeDataBindGraphConverter::OperationViewModel {
                operation_type: converter.uint_property("operationType").unwrap_or(0),
                operation_value: operand.as_ref().map(|operand| operand.value).unwrap_or(0.0),
                default_operation_value: operand
                    .as_ref()
                    .map(|operand| operand.value)
                    .unwrap_or(0.0),
                source_path: operand.map(|operand| operand.path),
            }
        }
        "DataConverterSystemDegsToRads" | "DataConverterSystemNormalizer" => {
            runtime_data_bind_graph_system_operation_value_converter(converter, data_bind_flags)
        }
        "DataConverterRounder" => RuntimeDataBindGraphConverter::Rounder {
            decimals: converter.uint_property("decimals").unwrap_or(0),
        },
        "DataConverterRangeMapper" => {
            runtime_data_bind_graph_range_mapper_converter(file, converter)
        }
        "DataConverterStringTrim" => RuntimeDataBindGraphConverter::StringTrim {
            trim_type: converter.uint_property("trimType").unwrap_or(1),
        },
        "DataConverterStringRemoveZeros" => RuntimeDataBindGraphConverter::StringRemoveZeros,
        "DataConverterStringPad" => RuntimeDataBindGraphConverter::StringPad {
            length: converter.uint_property("length").unwrap_or(1),
            text: converter
                .string_property_bytes("text")
                .unwrap_or_default()
                .to_vec(),
            pad_type: converter.uint_property("padType").unwrap_or(0),
        },
        "DataConverterFormula" => runtime_data_bind_graph_formula_converter(file, converter),
        "DataConverterInterpolator" if allow_stateful_interpolator => {
            runtime_data_bind_graph_interpolator_converter(file, converter)
        }
        "DataConverterInterpolator" => RuntimeDataBindGraphConverter::Unsupported,
        _ => RuntimeDataBindGraphConverter::Unsupported,
    };

    visiting.remove(&converter.id);
    Some(graph_converter)
}

fn runtime_data_bind_graph_formula_converter(
    file: &RuntimeFile,
    converter: &RuntimeObject,
) -> RuntimeDataBindGraphConverter {
    let mut tokens = Vec::new();
    for token in file.data_converter_formula_output_tokens_for_object(converter) {
        match token.object.type_name {
            "FormulaTokenInput" => tokens.push(RuntimeDataBindGraphFormulaToken::Input),
            "FormulaTokenValue" => tokens.push(RuntimeDataBindGraphFormulaToken::Value(
                token
                    .object
                    .double_property("operationValue")
                    .unwrap_or(1.0),
            )),
            "FormulaTokenOperation" => {
                tokens.push(RuntimeDataBindGraphFormulaToken::Operation {
                    operation_type: token.object.uint_property("operationType").unwrap_or(0),
                });
            }
            "FormulaTokenFunction" => {
                let function_type = token.object.uint_property("functionType").unwrap_or(0);
                let random_mode = converter.uint_property("randomModeValue").unwrap_or(0);
                if function_type == 16 && random_mode > 2 {
                    return RuntimeDataBindGraphConverter::Unsupported;
                }
                tokens.push(RuntimeDataBindGraphFormulaToken::Function {
                    function_type,
                    arguments_count: token.arguments_count,
                    random_mode,
                });
            }
            _ => return RuntimeDataBindGraphConverter::Unsupported,
        }
    }

    RuntimeDataBindGraphConverter::Formula { tokens }
}

fn runtime_data_bind_graph_interpolator_converter(
    file: &RuntimeFile,
    converter: &RuntimeObject,
) -> RuntimeDataBindGraphConverter {
    let interpolator = match file.resolved_interpolator_for_data_converter_object(converter) {
        Some(interpolator) => match RuntimeTransitionInterpolator::from_object(interpolator) {
            Some(interpolator) => Some(interpolator),
            None => return RuntimeDataBindGraphConverter::Unsupported,
        },
        None => None,
    };

    RuntimeDataBindGraphConverter::Interpolator {
        duration: converter.double_property("duration").unwrap_or(1.0),
        interpolator,
    }
}

fn runtime_data_bind_graph_range_mapper_converter(
    file: &RuntimeFile,
    converter: &RuntimeObject,
) -> RuntimeDataBindGraphConverter {
    let interpolator = match file.resolved_interpolator_for_data_converter_object(converter) {
        Some(interpolator) => match RuntimeTransitionInterpolator::from_object(interpolator) {
            Some(interpolator) => Some(interpolator),
            None => return RuntimeDataBindGraphConverter::Unsupported,
        },
        None => None,
    };

    RuntimeDataBindGraphConverter::RangeMapper {
        min_input: converter.double_property("minInput").unwrap_or(1.0),
        max_input: converter.double_property("maxInput").unwrap_or(1.0),
        min_output: converter.double_property("minOutput").unwrap_or(1.0),
        max_output: converter.double_property("maxOutput").unwrap_or(1.0),
        flags: converter.uint_property("flags").unwrap_or(0),
        interpolation_type: converter.uint_property("interpolationType").unwrap_or(1),
        interpolator,
    }
}

struct RuntimeDataBindGraphOperationViewModelOperand {
    path: Vec<u32>,
    value: f32,
}

fn runtime_data_bind_graph_default_operation_view_model_operand(
    file: &RuntimeFile,
    converter: &RuntimeObject,
) -> Option<RuntimeDataBindGraphOperationViewModelOperand> {
    let Some(path) = converter.id_list_property("sourcePathIds") else {
        return None;
    };
    let Some(default_instance) = file.view_model_default_instance(0) else {
        return Some(RuntimeDataBindGraphOperationViewModelOperand { path, value: 0.0 });
    };
    let Some(context) = RuntimeDataContext::from_instance_reference(file, default_instance) else {
        return Some(RuntimeDataBindGraphOperationViewModelOperand { path, value: 0.0 });
    };
    let Some(value) = context.absolute_property(&path) else {
        return Some(RuntimeDataBindGraphOperationViewModelOperand { path, value: 0.0 });
    };
    if file.view_model_instance_value_data_type_for_object(value)
        != Some(rive_binary::RuntimeDataType::Number)
    {
        return Some(RuntimeDataBindGraphOperationViewModelOperand { path, value: 0.0 });
    }
    Some(RuntimeDataBindGraphOperationViewModelOperand {
        path,
        value: value.double_property("propertyValue").unwrap_or(0.0),
    })
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeDataBindGraphConverterState {
    None,
    Formula(RuntimeDataBindGraphFormulaState),
    Interpolator(RuntimeDataBindGraphInterpolatorState),
    Group(Vec<RuntimeDataBindGraphConverterState>),
}

impl RuntimeDataBindGraphConverterState {
    pub(crate) fn for_converter(converter: Option<&RuntimeDataBindGraphConverter>) -> Self {
        match converter {
            Some(RuntimeDataBindGraphConverter::Formula { .. }) => {
                Self::Formula(RuntimeDataBindGraphFormulaState::default())
            }
            Some(RuntimeDataBindGraphConverter::Interpolator { .. }) => {
                Self::Interpolator(RuntimeDataBindGraphInterpolatorState::new())
            }
            Some(RuntimeDataBindGraphConverter::Group(converters)) => Self::Group(
                converters
                    .iter()
                    .map(|converter| Self::for_converter(Some(converter)))
                    .collect(),
            ),
            _ => Self::None,
        }
    }

    pub(crate) fn convert_value(
        &mut self,
        converter: &RuntimeDataBindGraphConverter,
        value: &RuntimeDataBindGraphValue,
    ) -> Option<RuntimeDataBindGraphValue> {
        let mut formula_random_source = RuntimeDataBindGraphFormulaRandomSource::default();
        self.convert_value_with_formula_randoms(converter, value, &mut formula_random_source)
    }

    pub(crate) fn convert_value_with_formula_randoms(
        &mut self,
        converter: &RuntimeDataBindGraphConverter,
        value: &RuntimeDataBindGraphValue,
        formula_random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
    ) -> Option<RuntimeDataBindGraphValue> {
        match (converter, self) {
            (RuntimeDataBindGraphConverter::Formula { tokens }, Self::Formula(state)) => {
                runtime_data_bind_graph_convert_formula_value_with_state(
                    value,
                    tokens,
                    state,
                    formula_random_source,
                )
            }
            (
                RuntimeDataBindGraphConverter::Interpolator {
                    duration,
                    interpolator,
                },
                Self::Interpolator(state),
            ) => state.convert(*duration, *interpolator, value),
            (RuntimeDataBindGraphConverter::Group(converters), Self::Group(states))
                if converters.len() == states.len() =>
            {
                let mut value = value.clone();
                for (converter, state) in converters.iter().zip(states) {
                    value = state.convert_value_with_formula_randoms(
                        converter,
                        &value,
                        formula_random_source,
                    )?;
                }
                Some(value)
            }
            _ => runtime_data_bind_graph_convert_value(converter, value),
        }
    }

    pub(crate) fn reverse_convert_value(
        &mut self,
        converter: &RuntimeDataBindGraphConverter,
        value: &RuntimeDataBindGraphValue,
    ) -> Option<RuntimeDataBindGraphValue> {
        let mut formula_random_source = RuntimeDataBindGraphFormulaRandomSource::default();
        self.reverse_convert_value_with_formula_randoms(
            converter,
            value,
            &mut formula_random_source,
        )
    }

    pub(crate) fn reverse_convert_value_with_formula_randoms(
        &mut self,
        converter: &RuntimeDataBindGraphConverter,
        value: &RuntimeDataBindGraphValue,
        formula_random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
    ) -> Option<RuntimeDataBindGraphValue> {
        match (converter, self) {
            (RuntimeDataBindGraphConverter::Formula { tokens }, Self::Formula(state)) => {
                runtime_data_bind_graph_convert_formula_value_with_state(
                    value,
                    tokens,
                    state,
                    formula_random_source,
                )
            }
            (
                RuntimeDataBindGraphConverter::Interpolator {
                    duration,
                    interpolator,
                },
                Self::Interpolator(state),
            ) => state.convert(*duration, *interpolator, value),
            (RuntimeDataBindGraphConverter::Group(converters), Self::Group(states))
                if converters.len() == states.len() =>
            {
                let mut value = value.clone();
                for (converter, state) in converters.iter().rev().zip(states.iter_mut().rev()) {
                    value = state.reverse_convert_value_with_formula_randoms(
                        converter,
                        &value,
                        formula_random_source,
                    )?;
                }
                Some(value)
            }
            _ => runtime_data_bind_graph_reverse_convert_value(converter, value),
        }
    }

    pub(crate) fn advance_converter(
        &mut self,
        converter: Option<&RuntimeDataBindGraphConverter>,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        match (converter, self) {
            (
                Some(RuntimeDataBindGraphConverter::Interpolator {
                    duration,
                    interpolator,
                }),
                Self::Interpolator(state),
            ) => state.advance(*duration, *interpolator, elapsed_seconds),
            (Some(RuntimeDataBindGraphConverter::Group(converters)), Self::Group(states))
                if converters.len() == states.len() =>
            {
                let mut aggregate = RuntimeDataBindGraphStatefulAdvance::default();
                for (converter, state) in converters.iter().zip(states) {
                    let advance = state.advance_converter(Some(converter), elapsed_seconds);
                    aggregate.changed |= advance.changed;
                    aggregate.keep_going |= advance.keep_going;
                }
                aggregate
            }
            _ => RuntimeDataBindGraphStatefulAdvance::default(),
        }
    }

    pub(crate) fn is_initialized_stateful(&self) -> bool {
        match self {
            Self::Interpolator(state) => state.is_initialized(),
            Self::Group(states) => states.iter().any(Self::is_initialized_stateful),
            Self::Formula(_) | Self::None => false,
        }
    }

    pub(crate) fn reset_formula_randoms(&mut self) {
        match self {
            Self::Formula(state) => state.clear(),
            Self::Group(states) => {
                for state in states {
                    state.reset_formula_randoms();
                }
            }
            Self::Interpolator(_) | Self::None => {}
        }
    }
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

impl RuntimeDataBindGraph {
    pub(crate) fn new(state_machine: &RuntimeStateMachine) -> Self {
        let mut sources = Vec::new();
        let mut targets = Vec::new();
        let mut default_view_model_bindings = Vec::new();

        for bindable in &state_machine.bindable_numbers {
            for source in &bindable.default_view_model_sources {
                let source_handle = Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::Number {
                        global_id: bindable.global_id,
                    },
                    source.value.clone(),
                );
                if let Some(node) = sources.get_mut(source_handle.0) {
                    node.view_model_instance_ids = source.view_model_instance_ids.clone();
                }
            }
        }
        for bindable in &state_machine.bindable_integers {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    None,
                    RuntimeDataBindGraphTarget::Integer {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::SymbolListIndex(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_booleans {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::Boolean {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Boolean(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_strings {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::String {
                        global_id: bindable.global_id,
                    },
                    source.value.clone(),
                );
            }
        }
        for bindable in &state_machine.bindable_colors {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    None,
                    RuntimeDataBindGraphTarget::Color {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Color(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_enums {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    None,
                    RuntimeDataBindGraphTarget::Enum {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Enum(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_assets {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    None,
                    RuntimeDataBindGraphTarget::Asset {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Asset(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_artboards {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    None,
                    RuntimeDataBindGraphTarget::Artboard {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Artboard(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_lists {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::List {
                        global_id: bindable.global_id,
                    },
                    source.value.clone(),
                );
            }
        }
        for bindable in &state_machine.bindable_triggers {
            for source in &bindable.default_view_model_sources {
                Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::Trigger {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::Trigger(source.value),
                );
            }
        }
        for bindable in &state_machine.bindable_view_models {
            for source in &bindable.default_view_model_sources {
                let source_handle = Self::push_default_view_model_binding(
                    &mut sources,
                    &mut targets,
                    &mut default_view_model_bindings,
                    source.data_bind_index,
                    &source.path,
                    source.flags,
                    source.converter.clone(),
                    RuntimeDataBindGraphTarget::ViewModel {
                        global_id: bindable.global_id,
                    },
                    RuntimeDataBindGraphValue::ViewModel(source.value),
                );
                if let Some(node) = sources.get_mut(source_handle.0) {
                    node.view_model_instance_ids = source.view_model_instance_ids.clone();
                }
            }
        }

        default_view_model_bindings.sort_by_key(|binding| binding.data_bind_index);

        Self {
            context_kind: RuntimeDataBindGraphContextKind::None,
            default_view_model_bindings_dirty: false,
            formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            sources,
            targets,
            default_view_model_bindings,
            imported_view_model_context: None,
            imported_view_model_overrides: BTreeMap::new(),
        }
    }

    pub(crate) fn push_default_view_model_binding(
        sources: &mut Vec<RuntimeDataBindGraphSourceNode>,
        targets: &mut Vec<RuntimeDataBindGraphTargetNode>,
        bindings: &mut Vec<RuntimeDataBindGraphDefaultBinding>,
        data_bind_index: usize,
        path: &[u32],
        flags: u64,
        converter: Option<RuntimeDataBindGraphConverter>,
        target: RuntimeDataBindGraphTarget,
        value: RuntimeDataBindGraphValue,
    ) -> RuntimeDataBindGraphSourceHandle {
        let source = RuntimeDataBindGraphSourceHandle(sources.len());
        let converter_state = RuntimeDataBindGraphConverterState::for_converter(converter.as_ref());
        sources.push(RuntimeDataBindGraphSourceNode {
            path: path.to_vec(),
            flags,
            bound: true,
            target_to_source_dirty: false,
            source_to_target_dirty_after_immediate: false,
            source_to_target_dirty_after_target_to_source: false,
            converter,
            converter_state,
            default_value: value.clone(),
            value,
            view_model_instance_ids: Vec::new(),
        });
        let target_handle = RuntimeDataBindGraphTargetHandle(targets.len());
        targets.push(RuntimeDataBindGraphTargetNode { target });
        bindings.push(RuntimeDataBindGraphDefaultBinding {
            data_bind_index,
            source,
            target: target_handle,
        });
        source
    }

    pub(crate) fn set_formula_random_values(&mut self, values: &[f32]) {
        self.formula_random_source.set_values(values);
        for source in &mut self.sources {
            source.reset_formula_random_state();
        }
    }

    pub(crate) fn formula_random_call_count(&self) -> usize {
        self.formula_random_source.call_count()
    }

    pub(crate) fn data_context_present(&self) -> bool {
        self.context_kind != RuntimeDataBindGraphContextKind::None
    }

    pub(crate) fn default_view_model_context_bound(&self) -> bool {
        matches!(
            self.context_kind,
            RuntimeDataBindGraphContextKind::DefaultViewModel
                | RuntimeDataBindGraphContextKind::ImportedViewModel
                | RuntimeDataBindGraphContextKind::OwnedViewModel
        )
    }

    pub(crate) fn default_view_model_source_context_bound(&self) -> bool {
        self.context_kind == RuntimeDataBindGraphContextKind::DefaultViewModel
    }

    pub(crate) fn mark_default_view_model_bindings_dirty(&mut self) {
        self.default_view_model_bindings_dirty = true;
    }

    pub(crate) fn bind_empty_data_context(&mut self) -> bool {
        if self.data_context_present() {
            return false;
        }
        self.reset_converter_states();
        self.context_kind = RuntimeDataBindGraphContextKind::Empty;
        self.imported_view_model_context = None;
        self.default_view_model_bindings_dirty = false;
        true
    }

    pub(crate) fn bind_default_view_model_context(&mut self) -> bool {
        if self.context_kind == RuntimeDataBindGraphContextKind::DefaultViewModel {
            return false;
        }
        for source in &mut self.sources {
            source.value = source.default_value.clone();
            source.bound = true;
            if let Some(converter) = source.converter.as_mut() {
                runtime_data_bind_graph_reset_operation_view_model_converter_to_default(converter);
            }
            source.reset_converter_state();
        }
        self.context_kind = RuntimeDataBindGraphContextKind::DefaultViewModel;
        self.imported_view_model_context = None;
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn bind_view_model_instance_context(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> bool {
        let context = RuntimeImportedViewModelInstanceContext {
            view_model_index,
            instance_index,
            number_overrides: BTreeMap::new(),
            boolean_overrides: BTreeMap::new(),
            string_overrides: BTreeMap::new(),
            color_overrides: BTreeMap::new(),
            enum_overrides: BTreeMap::new(),
            symbol_list_index_overrides: BTreeMap::new(),
            asset_overrides: BTreeMap::new(),
            artboard_overrides: BTreeMap::new(),
            trigger_overrides: BTreeMap::new(),
            list_overrides: BTreeMap::new(),
            view_model_overrides: BTreeMap::new(),
        };
        self.bind_imported_view_model_context(file, &context)
    }

    pub(crate) fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        let view_model_index = context.view_model_index;
        let instance_index = context.instance_index;
        let Some(view_model) = file.view_model(view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(instance_index) else {
            return false;
        };
        let Some(runtime_context) = RuntimeDataContext::from_instance_object(file, instance.object)
        else {
            return false;
        };

        for source in &mut self.sources {
            if let Some(value) =
                source
                    .value
                    .resolve_from_data_context(file, &runtime_context, &source.path)
            {
                let value = match value {
                    RuntimeDataBindGraphValue::Number(_) => context
                        .number_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Number)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Boolean(_) => context
                        .boolean_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Boolean)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::String(_) => context
                        .string_overrides
                        .get(&source.path)
                        .cloned()
                        .map(RuntimeDataBindGraphValue::String)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Color(_) => context
                        .color_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Color)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Enum(_) => context
                        .enum_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Enum)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::SymbolListIndex(_) => context
                        .symbol_list_index_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::SymbolListIndex)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Asset(_) => context
                        .asset_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Asset)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Artboard(_) => context
                        .artboard_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Artboard)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::Trigger(_) => context
                        .trigger_overrides
                        .get(&source.path)
                        .copied()
                        .map(RuntimeDataBindGraphValue::Trigger)
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::List { .. } => context
                        .list_overrides
                        .get(&source.path)
                        .copied()
                        .map(|item_count| RuntimeDataBindGraphValue::List { item_count })
                        .unwrap_or(value),
                    RuntimeDataBindGraphValue::ViewModel(_) => context
                        .view_model_overrides
                        .get(&source.path)
                        .copied()
                        .or_else(|| {
                            self.imported_view_model_overrides
                                .get(&RuntimeImportedViewModelOverrideKey {
                                    view_model_index,
                                    instance_index,
                                    path: source.path.clone(),
                                })
                                .copied()
                        })
                        .map(RuntimeDataBindGraphValue::ViewModel)
                        .unwrap_or(value),
                    _ => value,
                };
                source.value = value;
                source.bound = true;
            } else {
                source.bound = false;
            }
            if let Some(converter) = source.converter.as_mut() {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_imported_context(
                    file,
                    converter,
                    &runtime_context,
                );
            }
            source.reset_converter_state();
        }
        self.context_kind = RuntimeDataBindGraphContextKind::ImportedViewModel;
        self.imported_view_model_context = Some(RuntimeImportedViewModelContextKey {
            view_model_index,
            instance_index,
        });
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn bind_owned_view_model_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        for source in &mut self.sources {
            if let Some(value) = source
                .value
                .resolve_from_owned_view_model_instance(context, &source.path)
            {
                source.value = value;
                source.bound = true;
            } else {
                source.bound = false;
            }
            if let Some(converter) = source.converter.as_mut() {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                    converter, context,
                );
            }
            source.reset_converter_state();
        }
        self.context_kind = RuntimeDataBindGraphContextKind::OwnedViewModel;
        self.imported_view_model_context = None;
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn reset_converter_states(&mut self) {
        for source in &mut self.sources {
            source.reset_converter_state();
        }
    }

    pub(crate) fn set_default_view_model_number_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_number_source_for_path(&path, value)
    }

    pub(crate) fn set_active_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        if !self.default_view_model_context_bound() {
            return false;
        }
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        let update_default = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && std::mem::discriminant(&source.default_value) == std::mem::discriminant(&value)
        }) {
            if update_default && source.default_value != value {
                source.default_value = value.clone();
                changed = true;
            }
            if !source.bound || source.value != value {
                source.value = value.clone();
                source.bound = true;
                source.reset_formula_random_state_for_source_change();
                changed = true;
            }
        }
        if !changed {
            return false;
        }
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn source_path_for_data_bind(&self, data_bind_index: usize) -> Option<Vec<u32>> {
        self.default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
    }

    pub(crate) fn set_default_view_model_number_source_for_path(
        &mut self,
        path: &[u32],
        value: f32,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Number(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Number(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        let refreshed_dependents =
            self.refresh_operation_view_model_number_dependents_for_path(path, value);
        if default_context_bound || refreshed_dependents {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn refresh_operation_view_model_number_dependents_for_path(
        &mut self,
        path: &[u32],
        value: f32,
    ) -> bool {
        let mut changed = false;
        for source in &mut self.sources {
            let Some(converter) = source.converter.as_mut() else {
                continue;
            };
            if !runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path(
                converter, path, value,
            ) {
                continue;
            }
            source.source_to_target_dirty_after_target_to_source = true;
            changed = true;
        }
        changed
    }

    pub(crate) fn refresh_operation_view_model_number_dependents_for_imported_context_path(
        &mut self,
        path: &[u32],
        value: f32,
    ) -> bool {
        let mut changed = false;
        for source in &mut self.sources {
            let Some(converter) = source.converter.as_mut() else {
                continue;
            };
            if !runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_imported_context_path(
                converter, path, value,
            ) {
                continue;
            }
            source.source_to_target_dirty_after_target_to_source = true;
            changed = true;
        }
        changed
    }

    pub(crate) fn refresh_operation_view_model_number_dependents_for_owned_context_path(
        &mut self,
        path: &[u32],
        value: f32,
    ) -> bool {
        let mut changed = false;
        for source in &mut self.sources {
            let Some(converter) = source.converter.as_mut() else {
                continue;
            };
            if !runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_imported_context_path(
                converter, path, value,
            ) {
                continue;
            }
            source.source_to_target_dirty_after_target_to_source = true;
            changed = true;
        }
        changed
    }

    pub(crate) fn set_owned_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Number(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_number_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Number(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Number(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_number_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Number(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Number(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Number(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.refresh_operation_view_model_number_dependents_for_owned_context_path(&path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::SymbolListIndex(_)
        ) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_symbol_list_index_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::SymbolListIndex(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::SymbolListIndex(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_symbol_list_index_by_property_path(&property_path, value)
        {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::SymbolListIndex(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::SymbolListIndex(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::SymbolListIndex(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Boolean(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_boolean_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Boolean(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Boolean(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_boolean_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Boolean(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Boolean(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Boolean(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Enum(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_enum_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Enum(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Enum(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_enum_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Enum(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Enum(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Enum(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Color(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_color_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Color(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Color(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_color_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Color(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Color(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Color(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::String(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_string_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::String(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::String(current) if current.as_slice() == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_string_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::String(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::String(current) if current.as_slice() == value);
            source.value = RuntimeDataBindGraphValue::String(value.to_vec());
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Trigger(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_trigger_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Trigger(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_trigger_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Trigger(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Trigger(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn fire_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(binding) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
        else {
            return false;
        };
        let Some(target) = self.targets.get(binding.target.0) else {
            return false;
        };
        if !matches!(target.target, RuntimeDataBindGraphTarget::Trigger { .. }) {
            return false;
        }
        let Some(source) = self.sources.get_mut(binding.source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Trigger(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(value) = source.trigger_target_to_source_value(value) else {
            return false;
        };
        let RuntimeDataBindGraphValue::Trigger(value) = value else {
            return false;
        };
        source.target_to_source_dirty = false;
        source.source_to_target_dirty_after_immediate = false;

        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_trigger_value_for_source_path(context, &path)
        else {
            return false;
        };

        if current_context_value != value
            && !context.set_trigger_by_property_path(&property_path, value)
        {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Trigger(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Trigger(value);
            source.bound = true;
            source.source_to_target_dirty_after_target_to_source = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::List { .. }
        ) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_item_count) =
            runtime_owned_view_model_list_item_count_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_item_count != item_count;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::List { .. }
                )
                && (!source.bound
                    || !matches!(
                        &source.value,
                        RuntimeDataBindGraphValue::List { item_count: current } if *current == item_count
                    ))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed
            && !context.set_list_item_count_by_property_path(&property_path, item_count)
        {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::List { .. })
        }) {
            let changed = !source.bound
                || !matches!(
                    &source.value,
                    RuntimeDataBindGraphValue::List { item_count: current } if *current == item_count
                );
            source.value = RuntimeDataBindGraphValue::List { item_count };
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_default_view_model_boolean_source_for_path(
        &mut self,
        path: &[u32],
        value: bool,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Boolean(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Boolean(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_boolean_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_boolean_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_string_source_for_path(
        &mut self,
        path: &[u32],
        value: &[u8],
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::String(current) = &mut source.default_value else {
                continue;
            };
            if current == value {
                continue;
            }
            *current = value.to_vec();
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::String(value.to_vec());
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_string_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_string_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_color_source_for_path(
        &mut self,
        path: &[u32],
        value: u32,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Color(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Color(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_color_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_color_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_enum_source_for_path(
        &mut self,
        path: &[u32],
        value: u64,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Enum(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Enum(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_enum_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_enum_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_symbol_list_index_source_for_path(
        &mut self,
        path: &[u32],
        value: u64,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::SymbolListIndex(current) = &mut source.default_value
            else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::SymbolListIndex(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_symbol_list_index_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_symbol_list_index_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_asset_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_asset_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_asset_source_for_path(
        &mut self,
        path: &[u32],
        value: u64,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Asset(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Asset(value);
                source.bound = true;
            }
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_artboard_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_artboard_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_artboard_source_for_path(
        &mut self,
        path: &[u32],
        value: u64,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Artboard(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Artboard(value);
                source.bound = true;
            }
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_trigger_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_trigger_source_for_path(&path, value)
    }

    pub(crate) fn set_default_view_model_trigger_source_for_path(
        &mut self,
        path: &[u32],
        value: u64,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::Trigger(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::Trigger(value);
                source.bound = true;
            }
            source.reset_formula_random_state_for_source_change();
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_list_source_item_count_for_data_bind(
        &mut self,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.set_default_view_model_list_source_item_count_for_path(&path, item_count)
    }

    pub(crate) fn set_default_view_model_list_source_item_count_for_path(
        &mut self,
        path: &[u32],
        item_count: usize,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::List {
                item_count: current,
            } = &mut source.default_value
            else {
                continue;
            };
            if *current == item_count {
                continue;
            }
            *current = item_count;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::List { item_count };
                source.bound = true;
            }
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn set_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get_mut(source.0) else {
            return false;
        };
        let Some(_) = source.view_model_instance_ids.get(instance_index).copied() else {
            return false;
        };
        matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::ViewModel(RuntimeViewModelPointer::Imported { .. })
        )
    }

    pub(crate) fn relink_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        let Some(path) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
            .and_then(|source| self.sources.get(source.0))
            .map(|source| source.path.clone())
        else {
            return false;
        };

        self.relink_default_view_model_view_model_source_for_path(&path, instance_index)
    }

    pub(crate) fn relink_default_view_model_view_model_source_for_path(
        &mut self,
        path: &[u32],
        instance_index: usize,
    ) -> bool {
        let default_context_bound = self.default_view_model_source_context_bound();
        let Some(object_id) = self
            .sources
            .iter()
            .find(|source| {
                source.path == path
                    && matches!(
                        &source.default_value,
                        RuntimeDataBindGraphValue::ViewModel(_)
                    )
            })
            .and_then(|source| source.view_model_instance_ids.get(instance_index).copied())
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        let mut changed = false;
        for source in self.sources.iter_mut().filter(|source| source.path == path) {
            let RuntimeDataBindGraphValue::ViewModel(current) = &mut source.default_value else {
                continue;
            };
            if *current == value {
                continue;
            }
            *current = value;
            if default_context_bound {
                source.value = RuntimeDataBindGraphValue::ViewModel(value);
                source.bound = true;
            }
            changed = true;
        }
        if !changed {
            return false;
        }
        if default_context_bound {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn relink_view_model_instance_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        referenced_instance_index: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        let Some(context) = self.imported_view_model_context else {
            return false;
        };
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        let Some(object_id) = source
            .view_model_instance_ids
            .get(referenced_instance_index)
            .copied()
        else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::ViewModel(_)
        ) {
            return false;
        }
        let value = RuntimeViewModelPointer::Imported { object_id };
        let path = source.path.clone();
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value))
        });
        if !source_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::ViewModel(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }
        self.imported_view_model_overrides.insert(
            RuntimeImportedViewModelOverrideKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
                path,
            },
            value,
        );
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn relink_imported_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        referenced_instance_index: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get_mut(source.0) else {
            return false;
        };
        let Some(object_id) = source
            .view_model_instance_ids
            .get(referenced_instance_index)
            .copied()
        else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::ViewModel(_)
        ) {
            return false;
        }
        let value = RuntimeViewModelPointer::Imported { object_id };
        let path = source.path.clone();
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value))
        });
        let context_changed = context.view_model_overrides.get(&path) != Some(&value);
        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::ViewModel(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }
        context.view_model_overrides.insert(path.clone(), value);
        self.imported_view_model_overrides.insert(
            RuntimeImportedViewModelOverrideKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
                path,
            },
            value,
        );
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Number(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.number_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Number(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Number(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Number(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Number(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Number(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.refresh_operation_view_model_number_dependents_for_imported_context_path(&path, value);
        context.number_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Boolean(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.boolean_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Boolean(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Boolean(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Boolean(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Boolean(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Boolean(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.boolean_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::String(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context
            .string_overrides
            .get(&path)
            .map(|current| current.as_slice())
            != Some(value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::String(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::String(current) if current.as_slice() == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::String(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::String(current) if current.as_slice() == value);
            source.value = RuntimeDataBindGraphValue::String(value.to_vec());
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.string_overrides.insert(path, value.to_vec());
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Color(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.color_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Color(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Color(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Color(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Color(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Color(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.color_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Enum(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.enum_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Enum(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Enum(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Enum(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Enum(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Enum(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.enum_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::SymbolListIndex(_)
        ) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.symbol_list_index_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::SymbolListIndex(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::SymbolListIndex(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::SymbolListIndex(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::SymbolListIndex(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::SymbolListIndex(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.symbol_list_index_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Asset(_)) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_asset_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Asset(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Asset(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_asset_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Asset(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Asset(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Asset(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::Artboard(_)
        ) {
            return false;
        }
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_artboard_value_for_source_path(context, &path)
        else {
            return false;
        };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::Artboard(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Artboard(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed && !context.set_artboard_by_property_path(&property_path, value) {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Artboard(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Artboard(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Artboard(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_owned_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::OwnedViewModel {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::ViewModel(_)
        ) {
            return false;
        }
        let Some(object_id) = source.view_model_instance_ids.get(instance_index).copied() else {
            return false;
        };
        let path = source.path.clone();
        let Some(property_path) =
            runtime_owned_view_model_property_path_from_source_path(context, &path)
        else {
            return false;
        };
        let Some(current_context_value) =
            runtime_owned_view_model_view_model_value_for_source_path(context, &path)
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        let context_changed = current_context_value != value;
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        if context_changed
            && !context.set_view_model_by_property_path(&property_path, instance_index)
        {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::ViewModel(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Asset(_)) {
            return false;
        }
        let path = source.path.clone();
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(&source.default_value, RuntimeDataBindGraphValue::Asset(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Asset(current) if *current == value))
        });
        let context_changed = context.asset_overrides.get(&path) != Some(&value);
        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(&source.default_value, RuntimeDataBindGraphValue::Asset(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Asset(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Asset(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }
        context.asset_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::Artboard(_)
        ) {
            return false;
        }
        let path = source.path.clone();
        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::Artboard(_)
                )
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Artboard(current) if *current == value))
        });
        let context_changed = context.artboard_overrides.get(&path) != Some(&value);
        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::Artboard(_)
                )
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Artboard(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Artboard(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }
        context.artboard_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(&source.default_value, RuntimeDataBindGraphValue::Trigger(_)) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.trigger_overrides.get(&path) != Some(&value);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
                && (!source.bound
                    || !matches!(&source.value, RuntimeDataBindGraphValue::Trigger(current) if *current == value))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
        }) {
            let changed = !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::Trigger(current) if *current == value);
            source.value = RuntimeDataBindGraphValue::Trigger(value);
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.trigger_overrides.insert(path, value);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn set_imported_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        if self.imported_view_model_context
            != Some(RuntimeImportedViewModelContextKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
            })
        {
            return false;
        }
        let Some(source) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source)
        else {
            return false;
        };
        let Some(source) = self.sources.get(source.0) else {
            return false;
        };
        if !matches!(
            &source.default_value,
            RuntimeDataBindGraphValue::List { .. }
        ) {
            return false;
        }
        let path = source.path.clone();
        let context_changed = context.list_overrides.get(&path) != Some(&item_count);

        let source_changed = self.sources.iter().any(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::List { .. }
                )
                && (!source.bound
                    || !matches!(
                        &source.value,
                        RuntimeDataBindGraphValue::List { item_count: current } if *current == item_count
                    ))
        });

        if !source_changed && !context_changed {
            return false;
        }

        for source in self.sources.iter_mut().filter(|source| {
            source.path == path
                && matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::List { .. }
                )
        }) {
            let changed = !source.bound
                || !matches!(
                    &source.value,
                    RuntimeDataBindGraphValue::List { item_count: current } if *current == item_count
                );
            source.value = RuntimeDataBindGraphValue::List { item_count };
            source.bound = true;
            if changed {
                source.reset_formula_random_state_for_source_change();
            }
        }

        context.list_overrides.insert(path, item_count);
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn relink_view_model_instance_view_model_source_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        referenced_instance_index: usize,
    ) -> bool {
        if self.context_kind != RuntimeDataBindGraphContextKind::ImportedViewModel {
            return false;
        }
        let Some(context) = self.imported_view_model_context else {
            return false;
        };
        let Some(path) = runtime_view_model_view_model_property_path_for_name_path(
            file,
            context.view_model_index,
            property_path,
        ) else {
            return false;
        };
        let Some(object_id) = self
            .sources
            .iter()
            .find(|source| {
                source.path == path
                    && matches!(
                        &source.default_value,
                        RuntimeDataBindGraphValue::ViewModel(_)
                    )
            })
            .and_then(|source| {
                source
                    .view_model_instance_ids
                    .get(referenced_instance_index)
                    .copied()
            })
        else {
            return false;
        };

        let value = RuntimeViewModelPointer::Imported { object_id };
        let mut changed = false;
        for source in &mut self.sources {
            if source.path != path
                || !matches!(
                    &source.default_value,
                    RuntimeDataBindGraphValue::ViewModel(_)
                )
            {
                continue;
            }
            if !source.bound
                || !matches!(&source.value, RuntimeDataBindGraphValue::ViewModel(current) if *current == value)
            {
                changed = true;
            }
            source.value = RuntimeDataBindGraphValue::ViewModel(value);
            source.bound = true;
        }
        if !changed {
            return false;
        }
        self.imported_view_model_overrides.insert(
            RuntimeImportedViewModelOverrideKey {
                view_model_index: context.view_model_index,
                instance_index: context.instance_index,
                path,
            },
            value,
        );
        self.mark_default_view_model_bindings_dirty();
        true
    }

    pub(crate) fn mark_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
        target_matches: impl FnOnce(RuntimeDataBindGraphTarget) -> bool,
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let Some(binding) = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
        else {
            return false;
        };
        let Some(target) = self.targets.get(binding.target.0) else {
            return false;
        };
        if !target_matches(target.target) {
            return false;
        }

        let mut defer_source_to_target = false;
        let Some(source) = self.sources.get_mut(binding.source.0) else {
            return false;
        };
        if !source.applies_target_to_source() {
            return false;
        }
        if source.is_main_to_source() {
            source.target_to_source_dirty = true;
        } else if source.applies_source_to_target() {
            source.source_to_target_dirty_after_immediate = true;
            defer_source_to_target = true;
        } else {
            return false;
        }
        if defer_source_to_target {
            self.mark_default_view_model_bindings_dirty();
        }
        true
    }

    pub(crate) fn mark_number_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Number { .. })
        })
    }

    pub(crate) fn mark_integer_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Integer { .. })
        })
    }

    pub(crate) fn mark_boolean_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Boolean { .. })
        })
    }

    pub(crate) fn mark_string_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::String { .. })
        })
    }

    pub(crate) fn mark_color_target_dirty_for_data_bind(&mut self, data_bind_index: usize) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Color { .. })
        })
    }

    pub(crate) fn mark_enum_target_dirty_for_data_bind(&mut self, data_bind_index: usize) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Enum { .. })
        })
    }

    pub(crate) fn mark_asset_target_dirty_for_data_bind(&mut self, data_bind_index: usize) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Asset { .. })
        })
    }

    pub(crate) fn mark_artboard_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Artboard { .. })
        })
    }

    pub(crate) fn mark_list_target_dirty_for_data_bind(&mut self, data_bind_index: usize) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::List { .. })
        })
    }

    pub(crate) fn mark_trigger_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::Trigger { .. })
        })
    }

    pub(crate) fn mark_view_model_target_dirty_for_data_bind(
        &mut self,
        data_bind_index: usize,
    ) -> bool {
        self.mark_target_dirty_for_data_bind(data_bind_index, |target| {
            matches!(target, RuntimeDataBindGraphTarget::ViewModel { .. })
        })
    }

    pub(crate) fn imported_view_model_target_value_for_data_bind(
        &self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> Option<RuntimeViewModelPointer> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let target = self.targets.get(binding.target.0)?;
        let RuntimeDataBindGraphTarget::ViewModel { .. } = target.target else {
            return None;
        };
        let source = self.sources.get(binding.source.0)?;
        let object_id = source
            .view_model_instance_ids
            .get(instance_index)
            .copied()?;
        Some(RuntimeViewModelPointer::Imported { object_id })
    }

    pub(crate) fn view_model_instance_index_for_data_bind_value(
        &self,
        data_bind_index: usize,
        value: RuntimeViewModelPointer,
    ) -> Option<usize> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeViewModelPointer::Imported { object_id } = value else {
            return None;
        };
        source
            .view_model_instance_ids
            .iter()
            .position(|candidate| *candidate == object_id)
    }

    pub(crate) fn default_view_model_view_model_source_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::ViewModel(value) = &source.value else {
            return None;
        };
        self.view_model_instance_index_for_data_bind_value(data_bind_index, *value)
    }

    pub(crate) fn default_view_model_number_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Number(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_boolean_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Boolean(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_list_source_item_count_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::List { item_count } = source.value else {
            return None;
        };
        Some(item_count)
    }

    pub(crate) fn default_view_model_string_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<&[u8]> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::String(value) = &source.value else {
            return None;
        };
        Some(value.as_slice())
    }

    pub(crate) fn default_view_model_color_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Color(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_enum_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Enum(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_symbol_list_index_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::SymbolListIndex(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_asset_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Asset(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_artboard_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Artboard(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_trigger_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        let RuntimeDataBindGraphValue::Trigger(value) = source.value else {
            return None;
        };
        Some(value)
    }

    pub(crate) fn default_view_model_trigger_source_property_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let binding = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)?;
        let source = self.sources.get(binding.source.0)?;
        if !matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_)) {
            return None;
        }
        source.path.last().copied()
    }

    pub(crate) fn number_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Number { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn integer_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Integer { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn boolean_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Boolean { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn asset_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Asset { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn artboard_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Artboard { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn string_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::String { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn color_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Color { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn enum_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Enum { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn view_model_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::ViewModel { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn list_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::List { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn default_view_model_trigger_target_global_id_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        let target = self
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| self.targets.get(binding.target.0))?;
        let RuntimeDataBindGraphTarget::Trigger { global_id } = target.target else {
            return None;
        };
        Some(global_id)
    }

    pub(crate) fn default_view_model_trigger_target_global_ids_for_source_path(
        &self,
        path: &[u32],
    ) -> Vec<u32> {
        self.default_view_model_bindings
            .iter()
            .filter_map(|binding| {
                let source = self.sources.get(binding.source.0)?;
                if source.path != path
                    || !matches!(source.default_value, RuntimeDataBindGraphValue::Trigger(_))
                {
                    return None;
                }
                let target = self.targets.get(binding.target.0)?;
                let RuntimeDataBindGraphTarget::Trigger { global_id } = target.target else {
                    return None;
                };
                Some(global_id)
            })
            .collect()
    }

    pub(crate) fn reset_bound_trigger_sources(&mut self) -> bool {
        if !self.default_view_model_context_bound() {
            return false;
        }
        let default_context_bound = self.default_view_model_source_context_bound();
        let mut changed = false;
        for source in &mut self.sources {
            if !source.bound {
                continue;
            }
            let RuntimeDataBindGraphValue::Trigger(value) = &mut source.value else {
                continue;
            };
            let mut source_reset_changed = false;
            if *value != 0 {
                changed = true;
                source_reset_changed = true;
            }
            *value = 0;
            if default_context_bound {
                let RuntimeDataBindGraphValue::Trigger(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != 0 {
                    changed = true;
                    source_reset_changed = true;
                }
                *default_value = 0;
            }
            if source_reset_changed {
                source.reset_formula_random_state_for_source_change();
                if source.applies_source_to_target() {
                    source.source_to_target_dirty_after_target_to_source = true;
                }
            }
        }
        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn advance_stateful_converters(
        &mut self,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        if !self.default_view_model_context_bound() {
            return RuntimeDataBindGraphStatefulAdvance::default();
        }
        let mut keep_going = false;
        let mut changed = false;
        for source in &mut self.sources {
            if !source.bound {
                continue;
            }
            let advance = source.advance_stateful_converter(elapsed_seconds);
            changed |= advance.changed;
            keep_going |= advance.keep_going;
        }
        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        RuntimeDataBindGraphStatefulAdvance {
            changed,
            keep_going,
        }
    }

    pub(crate) fn apply_default_view_model_number_targets_to_sources(
        &mut self,
        numbers: &[StateMachineBindableNumberInstance],
    ) -> bool {
        self.apply_default_view_model_number_targets_to_sources_with_options(numbers, false)
    }

    pub(crate) fn apply_default_view_model_number_public_update_targets_to_sources(
        &mut self,
        numbers: &[StateMachineBindableNumberInstance],
    ) -> bool {
        self.apply_default_view_model_number_targets_to_sources_with_options(numbers, true)
    }

    pub(crate) fn apply_default_view_model_number_targets_to_sources_with_options(
        &mut self,
        numbers: &[StateMachineBindableNumberInstance],
        include_deferred_main_to_target: bool,
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut updates = Vec::<(Vec<u32>, RuntimeDataBindGraphValue)>::new();
        let mut applied_target_to_source = false;
        let mut formula_random_source = std::mem::take(&mut self.formula_random_source);

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Number { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty = source.target_to_source_dirty
                || (include_deferred_main_to_target
                    && source.source_to_target_dirty_after_immediate);
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if include_deferred_main_to_target {
                source.source_to_target_dirty_after_immediate = false;
            }
            let Some(value) = numbers
                .iter()
                .find(|number| number.global_id == global_id)
                .map(|number| number.value)
            else {
                continue;
            };
            let Some(value) =
                source.number_target_to_source_value(value, &mut formula_random_source)
            else {
                continue;
            };
            if !include_deferred_main_to_target
                && source.is_main_to_source()
                && matches!(
                    source.converter.as_ref(),
                    Some(
                        RuntimeDataBindGraphConverter::Formula { .. }
                            | RuntimeDataBindGraphConverter::Group(_)
                            | RuntimeDataBindGraphConverter::Interpolator { .. }
                            | RuntimeDataBindGraphConverter::ListToLength
                    )
                )
            {
                applied_target_to_source = true;
                source.source_to_target_dirty_after_target_to_source = true;
            }
            if include_deferred_main_to_target {
                applied_target_to_source = true;
                source.source_to_target_dirty_after_target_to_source = true;
            }
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let source_changed = match (&mut source.value, &mut source.default_value, &value) {
                    (
                        RuntimeDataBindGraphValue::Number(source_value),
                        RuntimeDataBindGraphValue::Number(default_value),
                        RuntimeDataBindGraphValue::Number(value),
                    ) => {
                        let mut source_changed = false;
                        if *source_value != *value {
                            *source_value = *value;
                            source_changed = true;
                        }
                        if *default_value != *value {
                            *default_value = *value;
                            source_changed = true;
                        }
                        source_changed
                    }
                    (
                        RuntimeDataBindGraphValue::Boolean(source_value),
                        RuntimeDataBindGraphValue::Boolean(default_value),
                        RuntimeDataBindGraphValue::Boolean(value),
                    ) => {
                        let mut source_changed = false;
                        if *source_value != *value {
                            *source_value = *value;
                            source_changed = true;
                        }
                        if *default_value != *value {
                            *default_value = *value;
                            source_changed = true;
                        }
                        source_changed
                    }
                    _ => false,
                };
                if source_changed {
                    source.reset_formula_random_state_for_source_change();
                    if source.is_main_to_source()
                        && matches!(
                            source.converter.as_ref(),
                            Some(
                                RuntimeDataBindGraphConverter::Formula { .. }
                                    | RuntimeDataBindGraphConverter::Group(_)
                                    | RuntimeDataBindGraphConverter::Interpolator { .. }
                                    | RuntimeDataBindGraphConverter::Rounder { .. }
                                    | RuntimeDataBindGraphConverter::SystemOperationValue { .. }
                            )
                        )
                    {
                        source.source_to_target_dirty_after_target_to_source = true;
                    }
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        self.formula_random_source = formula_random_source;
        changed || applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_symbol_list_index_targets_to_sources(
        &mut self,
        integers: &[StateMachineBindableIntegerInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Integer { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_symbol_list_index_target_to_source() {
                continue;
            }
            let Some(value) = integers
                .iter()
                .find(|integer| integer.global_id == global_id)
                .map(|integer| integer.value)
            else {
                continue;
            };
            let RuntimeDataBindGraphValue::SymbolListIndex(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::SymbolListIndex(default_value) =
                &mut source.default_value
            else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_symbol_list_index_public_update_targets_to_sources(
        &mut self,
        integers: &[StateMachineBindableIntegerInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, u64)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Integer { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_symbol_list_index_target_to_source() {
                continue;
            }
            let Some(value) = integers
                .iter()
                .find(|integer| integer.global_id == global_id)
                .map(|integer| integer.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::SymbolListIndex(source_value) = &mut source.value
                else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                let RuntimeDataBindGraphValue::SymbolListIndex(default_value) =
                    &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_boolean_targets_to_sources(
        &mut self,
        booleans: &[StateMachineBindableBooleanInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Boolean { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            let Some(value) = booleans
                .iter()
                .find(|boolean| boolean.global_id == global_id)
                .map(|boolean| boolean.value)
            else {
                continue;
            };
            let Some(value) = source.boolean_target_to_source_value(value) else {
                continue;
            };
            let RuntimeDataBindGraphValue::Boolean(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Boolean(default_value) = &mut source.default_value
            else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_boolean_public_update_targets_to_sources(
        &mut self,
        booleans: &[StateMachineBindableBooleanInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, bool)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Boolean { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            let Some(value) = booleans
                .iter()
                .find(|boolean| boolean.global_id == global_id)
                .map(|boolean| boolean.value)
            else {
                continue;
            };
            let Some(value) = source.boolean_target_to_source_value(value) else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let (
                    RuntimeDataBindGraphValue::Boolean(source_value),
                    RuntimeDataBindGraphValue::Boolean(default_value),
                ) = (&mut source.value, &mut source.default_value)
                else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_string_targets_to_sources(
        &mut self,
        strings: &[StateMachineBindableStringInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::String { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            let Some(value) = strings
                .iter()
                .find(|string| string.global_id == global_id)
                .map(|string| string.value.as_slice())
            else {
                continue;
            };
            let Some(value) = source.string_target_to_source_value(value) else {
                continue;
            };
            if source.is_main_to_source()
                && source.uses_delayed_string_source_to_target_after_main_to_source()
            {
                source.source_to_target_dirty_after_immediate = true;
                changed = true;
            }
            let RuntimeDataBindGraphValue::String(value) = value else {
                continue;
            };
            let RuntimeDataBindGraphValue::String(source_value) = &mut source.value else {
                continue;
            };
            if source_value.as_slice() != value.as_slice() {
                *source_value = value.clone();
                changed = true;
            }
            let RuntimeDataBindGraphValue::String(default_value) = &mut source.default_value else {
                continue;
            };
            if default_value.as_slice() != value.as_slice() {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_string_public_update_targets_to_sources(
        &mut self,
        strings: &[StateMachineBindableStringInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut updates = Vec::<(Vec<u32>, RuntimeDataBindGraphValue)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::String { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            let Some(value) = strings
                .iter()
                .find(|string| string.global_id == global_id)
                .map(|string| string.value.as_slice())
            else {
                continue;
            };
            let Some(value) = source.string_target_to_source_value(value) else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let (
                    RuntimeDataBindGraphValue::String(source_value),
                    RuntimeDataBindGraphValue::String(default_value),
                    RuntimeDataBindGraphValue::String(value),
                ) = (&mut source.value, &mut source.default_value, &value)
                else {
                    continue;
                };
                if source_value.as_slice() != value.as_slice() {
                    *source_value = value.clone();
                    changed = true;
                }
                if default_value.as_slice() != value.as_slice() {
                    *default_value = value.clone();
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        changed || applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_color_targets_to_sources(
        &mut self,
        colors: &[StateMachineBindableColorInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Color { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_color_target_to_source() {
                continue;
            }
            let Some(value) = colors
                .iter()
                .find(|color| color.global_id == global_id)
                .map(|color| color.value)
            else {
                continue;
            };
            let RuntimeDataBindGraphValue::Color(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Color(default_value) = &mut source.default_value else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_color_public_update_targets_to_sources(
        &mut self,
        colors: &[StateMachineBindableColorInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, u32)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Color { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_color_target_to_source() {
                continue;
            }
            let Some(value) = colors
                .iter()
                .find(|color| color.global_id == global_id)
                .map(|color| color.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::Color(source_value) = &mut source.value else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                let RuntimeDataBindGraphValue::Color(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_enum_targets_to_sources(
        &mut self,
        enums: &[StateMachineBindableEnumInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Enum { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_enum_target_to_source() {
                continue;
            }
            let Some(value) = enums
                .iter()
                .find(|r#enum| r#enum.global_id == global_id)
                .map(|r#enum| r#enum.value)
            else {
                continue;
            };
            let RuntimeDataBindGraphValue::Enum(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Enum(default_value) = &mut source.default_value else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_enum_public_update_targets_to_sources(
        &mut self,
        enums: &[StateMachineBindableEnumInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, u64)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Enum { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_enum_target_to_source() {
                continue;
            }
            let Some(value) = enums
                .iter()
                .find(|r#enum| r#enum.global_id == global_id)
                .map(|r#enum| r#enum.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::Enum(source_value) = &mut source.value else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                let RuntimeDataBindGraphValue::Enum(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_asset_targets_to_sources(
        &mut self,
        assets: &[StateMachineBindableAssetInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Asset { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_asset_target_to_source() {
                continue;
            }
            let Some(value) = assets
                .iter()
                .find(|asset| asset.global_id == global_id)
                .map(|asset| asset.value)
            else {
                continue;
            };
            let RuntimeDataBindGraphValue::Asset(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Asset(default_value) = &mut source.default_value else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_asset_public_update_targets_to_sources(
        &mut self,
        assets: &[StateMachineBindableAssetInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, u64)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Asset { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_asset_target_to_source() {
                continue;
            }
            let Some(value) = assets
                .iter()
                .find(|asset| asset.global_id == global_id)
                .map(|asset| asset.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::Asset(source_value) = &mut source.value else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                let RuntimeDataBindGraphValue::Asset(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_artboard_targets_to_sources(
        &mut self,
        artboards: &[StateMachineBindableArtboardInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Artboard { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_artboard_target_to_source() {
                continue;
            }
            let Some(value) = artboards
                .iter()
                .find(|artboard| artboard.global_id == global_id)
                .map(|artboard| artboard.value)
            else {
                continue;
            };
            let RuntimeDataBindGraphValue::Artboard(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Artboard(default_value) = &mut source.default_value
            else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_artboard_public_update_targets_to_sources(
        &mut self,
        artboards: &[StateMachineBindableArtboardInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, u64)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Artboard { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_artboard_target_to_source() {
                continue;
            }
            let Some(value) = artboards
                .iter()
                .find(|artboard| artboard.global_id == global_id)
                .map(|artboard| artboard.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::Artboard(source_value) = &mut source.value else {
                    continue;
                };
                if *source_value != value {
                    *source_value = value;
                    changed = true;
                }
                let RuntimeDataBindGraphValue::Artboard(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_list_targets_to_sources(&mut self) -> bool {
        self.apply_default_view_model_list_targets_to_sources_with_options(false)
    }

    pub(crate) fn apply_default_view_model_list_public_update_targets_to_sources(
        &mut self,
    ) -> bool {
        self.apply_default_view_model_list_targets_to_sources_with_options(true)
    }

    pub(crate) fn apply_default_view_model_list_targets_to_sources_with_options(
        &mut self,
        include_deferred_main_to_target: bool,
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut consumed_target_to_source = false;
        let mut needs_source_to_target_noop = false;
        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::List { .. } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty = source.target_to_source_dirty
                || (include_deferred_main_to_target
                    && source.source_to_target_dirty_after_immediate);
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if include_deferred_main_to_target {
                source.source_to_target_dirty_after_immediate = false;
            }
            if !source.bound || !source.applies_target_to_source() {
                continue;
            }
            consumed_target_to_source = true;
            if source.applies_source_to_target()
                && (include_deferred_main_to_target
                    || !source.suppresses_explicit_list_target_reapply_after_formula())
            {
                source.source_to_target_dirty_after_target_to_source = true;
                needs_source_to_target_noop = true;
            }
        }

        if needs_source_to_target_noop {
            self.mark_default_view_model_bindings_dirty();
        }
        consumed_target_to_source
    }

    pub(crate) fn apply_default_view_model_trigger_targets_to_sources(
        &mut self,
        triggers: &[StateMachineBindableTriggerInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut changed = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Trigger { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_target_to_source = false;
            let Some(value) = triggers
                .iter()
                .find(|trigger| trigger.global_id == global_id)
                .map(|trigger| trigger.value)
            else {
                continue;
            };
            let Some(value) = source.trigger_target_to_source_value(value) else {
                continue;
            };
            let RuntimeDataBindGraphValue::Trigger(value) = value else {
                continue;
            };
            let RuntimeDataBindGraphValue::Trigger(source_value) = &mut source.value else {
                continue;
            };
            if *source_value != value {
                *source_value = value;
                changed = true;
            }
            let RuntimeDataBindGraphValue::Trigger(default_value) = &mut source.default_value
            else {
                continue;
            };
            if *default_value != value {
                *default_value = value;
                changed = true;
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_trigger_public_update_targets_to_sources(
        &mut self,
        triggers: &[StateMachineBindableTriggerInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, RuntimeDataBindGraphValue)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::Trigger { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            let Some(value) = triggers
                .iter()
                .find(|trigger| trigger.global_id == global_id)
                .map(|trigger| trigger.value)
            else {
                continue;
            };
            let Some(value) = source.trigger_target_to_source_value(value) else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let (
                    RuntimeDataBindGraphValue::Trigger(source_value),
                    RuntimeDataBindGraphValue::Trigger(default_value),
                    RuntimeDataBindGraphValue::Trigger(value),
                ) = (&mut source.value, &mut source.default_value, &value)
                else {
                    continue;
                };
                if *source_value != *value {
                    *source_value = *value;
                    changed = true;
                }
                if *default_value != *value {
                    *default_value = *value;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_view_model_targets_to_sources(
        &mut self,
        view_models: &[StateMachineBindableViewModelInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }
        let mut updates = Vec::<(Vec<u32>, RuntimeViewModelPointer)>::new();

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::ViewModel { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            if !source.bound || !source.supports_direct_view_model_target_to_source() {
                continue;
            }
            let Some(value) = view_models
                .iter()
                .find(|view_model| view_model.global_id == global_id)
                .map(|view_model| view_model.value)
            else {
                continue;
            };
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::ViewModel(source_value) = &mut source.value else {
                    continue;
                };
                let mut source_changed = false;
                if *source_value != value {
                    *source_value = value;
                    source_changed = true;
                }
                let RuntimeDataBindGraphValue::ViewModel(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    source_changed = true;
                }
                if source_changed {
                    source.source_to_target_dirty_after_target_to_source = true;
                    changed = true;
                }
            }
        }

        if changed {
            self.mark_default_view_model_bindings_dirty();
        }
        changed
    }

    pub(crate) fn apply_default_view_model_view_model_public_update_targets_to_sources(
        &mut self,
        view_models: &[StateMachineBindableViewModelInstance],
    ) -> bool {
        if !self.default_view_model_source_context_bound() {
            return false;
        }

        let mut updates = Vec::<(Vec<u32>, RuntimeViewModelPointer)>::new();
        let mut applied_target_to_source = false;

        for binding in self.default_view_model_bindings.clone() {
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            let RuntimeDataBindGraphTarget::ViewModel { global_id } = target.target else {
                continue;
            };
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            let target_to_source_dirty =
                source.target_to_source_dirty || source.source_to_target_dirty_after_immediate;
            if !target_to_source_dirty {
                continue;
            }
            source.target_to_source_dirty = false;
            source.source_to_target_dirty_after_immediate = false;
            if !source.bound || !source.supports_direct_view_model_target_to_source() {
                continue;
            }
            let Some(value) = view_models
                .iter()
                .find(|view_model| view_model.global_id == global_id)
                .map(|view_model| view_model.value)
            else {
                continue;
            };
            applied_target_to_source = true;
            source.source_to_target_dirty_after_target_to_source = true;
            updates.push((source.path.clone(), value));
        }

        let mut changed = false;
        for (path, value) in updates {
            for source in &mut self.sources {
                if !source.bound || source.path != path {
                    continue;
                }
                let RuntimeDataBindGraphValue::ViewModel(source_value) = &mut source.value else {
                    continue;
                };
                let mut source_changed = false;
                if *source_value != value {
                    *source_value = value;
                    source_changed = true;
                }
                let RuntimeDataBindGraphValue::ViewModel(default_value) = &mut source.default_value
                else {
                    continue;
                };
                if *default_value != value {
                    *default_value = value;
                    source_changed = true;
                }
                if source_changed {
                    source.source_to_target_dirty_after_target_to_source = true;
                    changed = true;
                }
            }
        }

        if changed || applied_target_to_source {
            self.mark_default_view_model_bindings_dirty();
        }
        applied_target_to_source
    }

    pub(crate) fn apply_default_view_model_bindings(
        &mut self,
        mut targets: RuntimeDataBindGraphTargetsMut<'_>,
        phase: RuntimeDataBindGraphApplyPhase,
    ) {
        if !self.default_view_model_context_bound() || !self.default_view_model_bindings_dirty {
            return;
        }
        let mut skipped_dirty_binding = false;
        let mut formula_random_source = std::mem::take(&mut self.formula_random_source);

        for binding in self.default_view_model_bindings.clone() {
            let Some(source) = self.sources.get_mut(binding.source.0) else {
                continue;
            };
            if !source.bound {
                continue;
            }
            if !source.applies_source_to_target() {
                continue;
            }
            if matches!(phase, RuntimeDataBindGraphApplyPhase::Immediate)
                && source.is_main_to_source()
                && !source.source_to_target_dirty_after_target_to_source
            {
                if source.source_to_target_dirty_after_immediate {
                    skipped_dirty_binding = true;
                }
                continue;
            }
            if matches!(phase, RuntimeDataBindGraphApplyPhase::PublicUpdate)
                && !source.source_to_target_dirty_after_target_to_source
            {
                skipped_dirty_binding = true;
                continue;
            }
            if matches!(phase, RuntimeDataBindGraphApplyPhase::Immediate)
                && source.source_to_target_dirty_after_immediate
            {
                skipped_dirty_binding = true;
                continue;
            }
            let Some(target) = self.targets.get(binding.target.0) else {
                continue;
            };
            if matches!(target.target, RuntimeDataBindGraphTarget::ViewModel { .. })
                && !targets.include_view_models
            {
                skipped_dirty_binding = true;
                continue;
            }
            if matches!(target.target, RuntimeDataBindGraphTarget::ViewModel { .. })
                && matches!(phase, RuntimeDataBindGraphApplyPhase::Immediate)
                && !source.source_to_target_dirty_after_target_to_source
            {
                skipped_dirty_binding = true;
                continue;
            }
            if source.should_skip_binding_for_phase(phase)
                && !source.source_to_target_dirty_after_immediate
            {
                skipped_dirty_binding = true;
                continue;
            }
            let Some(value) = source.converted_value(&mut formula_random_source) else {
                continue;
            };
            if matches!(phase, RuntimeDataBindGraphApplyPhase::Immediate)
                && !source.source_to_target_dirty_after_target_to_source
                && matches!(target.target, RuntimeDataBindGraphTarget::List { .. })
                && matches!(value, RuntimeDataBindGraphValue::Number(_))
            {
                skipped_dirty_binding = true;
                continue;
            }
            targets.apply_default_view_model_binding(&target.target, &value);
            source.source_to_target_dirty_after_immediate = false;
            source.source_to_target_dirty_after_target_to_source = false;
        }
        self.formula_random_source = formula_random_source;
        self.default_view_model_bindings_dirty = skipped_dirty_binding;
    }
}

impl RuntimeDataBindGraphSourceNode {
    fn applies_source_to_target(&self) -> bool {
        data_bind_flags_apply_source_to_target(self.flags)
    }

    fn applies_target_to_source(&self) -> bool {
        data_bind_flags_apply_target_to_source(self.flags)
    }

    fn is_main_to_source(&self) -> bool {
        self.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0
    }

    fn suppresses_explicit_list_target_reapply_after_formula(&self) -> bool {
        matches!(
            (&self.value, self.converter.as_ref()),
            (
                RuntimeDataBindGraphValue::List { .. },
                Some(RuntimeDataBindGraphConverter::Formula { .. })
            )
        )
    }

    fn number_target_to_source_value(
        &mut self,
        value: f32,
        formula_random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
    ) -> Option<RuntimeDataBindGraphValue> {
        if !self.bound || !self.applies_target_to_source() {
            return None;
        }
        if let (
            RuntimeDataBindGraphValue::List { item_count },
            Some(RuntimeDataBindGraphConverter::Formula { .. }),
        ) = (&self.value, self.converter.as_ref())
        {
            return Some(RuntimeDataBindGraphValue::List {
                item_count: *item_count,
            });
        }
        let converted = match self.converter.as_ref() {
            None => RuntimeDataBindGraphValue::Number(value),
            Some(converter) if self.is_main_to_source() => {
                self.converter_state.convert_value_with_formula_randoms(
                    converter,
                    &RuntimeDataBindGraphValue::Number(value),
                    formula_random_source,
                )?
            }
            Some(converter) => self
                .converter_state
                .reverse_convert_value_with_formula_randoms(
                    converter,
                    &RuntimeDataBindGraphValue::Number(value),
                    formula_random_source,
                )?,
        };
        match (&self.value, converted) {
            (RuntimeDataBindGraphValue::Number(_), RuntimeDataBindGraphValue::Number(value)) => {
                Some(RuntimeDataBindGraphValue::Number(value))
            }
            (RuntimeDataBindGraphValue::Boolean(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Boolean(*value))
            }
            (RuntimeDataBindGraphValue::String(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::String(value.clone()))
            }
            (RuntimeDataBindGraphValue::Color(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Color(*value))
            }
            (RuntimeDataBindGraphValue::Enum(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Enum(*value))
            }
            (RuntimeDataBindGraphValue::Asset(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Asset(*value))
            }
            (RuntimeDataBindGraphValue::Artboard(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Artboard(*value))
            }
            (RuntimeDataBindGraphValue::ViewModel(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_non_trigger_non_number_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::ViewModel(*value))
            }
            (RuntimeDataBindGraphValue::Trigger(value), RuntimeDataBindGraphValue::Number(_))
                if self.converter.as_ref().is_some_and(
                    runtime_data_bind_graph_converter_preserves_trigger_source_on_number_target_apply,
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Trigger(*value))
            }
            (
                RuntimeDataBindGraphValue::SymbolListIndex(value),
                RuntimeDataBindGraphValue::Number(_),
            ) if self.converter.as_ref().is_some_and(
                runtime_data_bind_graph_converter_preserves_symbol_list_index_source_on_number_target_apply,
            ) =>
            {
                Some(RuntimeDataBindGraphValue::SymbolListIndex(*value))
            }
            (
                RuntimeDataBindGraphValue::ListLength(value),
                RuntimeDataBindGraphValue::Number(_),
            ) if matches!(
                self.converter.as_ref(),
                Some(RuntimeDataBindGraphConverter::ListToLength)
            ) =>
            {
                Some(RuntimeDataBindGraphValue::ListLength(*value))
            }
            (
                RuntimeDataBindGraphValue::List { item_count },
                RuntimeDataBindGraphValue::Number(_),
            ) if matches!(
                self.converter.as_ref(),
                Some(RuntimeDataBindGraphConverter::Formula { .. })
            ) =>
            {
                Some(RuntimeDataBindGraphValue::List {
                    item_count: *item_count,
                })
            }
            (RuntimeDataBindGraphValue::Boolean(_), RuntimeDataBindGraphValue::Boolean(value)) => {
                Some(RuntimeDataBindGraphValue::Boolean(value))
            }
            _ => None,
        }
    }

    fn supports_direct_symbol_list_index_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::SymbolListIndex(_))
    }

    fn boolean_target_to_source_value(&self, value: bool) -> Option<bool> {
        if !self.bound
            || !self.applies_target_to_source()
            || !matches!(self.value, RuntimeDataBindGraphValue::Boolean(_))
        {
            return None;
        }
        let value = match self.converter.as_ref() {
            None => RuntimeDataBindGraphValue::Boolean(value),
            Some(converter) if self.is_main_to_source() => runtime_data_bind_graph_convert_value(
                converter,
                &RuntimeDataBindGraphValue::Boolean(value),
            )?,
            Some(converter) => runtime_data_bind_graph_reverse_convert_value(
                converter,
                &RuntimeDataBindGraphValue::Boolean(value),
            )?,
        };
        let RuntimeDataBindGraphValue::Boolean(value) = value else {
            return None;
        };
        Some(value)
    }

    fn string_target_to_source_value(&mut self, value: &[u8]) -> Option<RuntimeDataBindGraphValue> {
        if !self.bound || !self.applies_target_to_source() {
            return None;
        }
        if self.preserves_string_source_on_main_to_source_target_apply() {
            let RuntimeDataBindGraphValue::String(value) = &self.value else {
                return None;
            };
            return Some(RuntimeDataBindGraphValue::String(value.clone()));
        }
        let value = RuntimeDataBindGraphValue::String(value.to_vec());
        let converted = match self.converter.as_ref() {
            None => value,
            Some(converter) if self.is_main_to_source() => {
                self.converter_state.convert_value(converter, &value)?
            }
            Some(converter) => self
                .converter_state
                .reverse_convert_value(converter, &value)?,
        };
        match (&self.value, converted) {
            (RuntimeDataBindGraphValue::String(_), RuntimeDataBindGraphValue::String(value)) => {
                Some(RuntimeDataBindGraphValue::String(value))
            }
            (RuntimeDataBindGraphValue::Number(value), RuntimeDataBindGraphValue::String(_))
                if runtime_data_bind_graph_converter_starts_with_to_string(
                    self.converter.as_ref(),
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Number(*value))
            }
            (RuntimeDataBindGraphValue::Boolean(value), RuntimeDataBindGraphValue::String(_))
                if matches!(
                    self.converter.as_ref(),
                    Some(RuntimeDataBindGraphConverter::ToString { .. })
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Boolean(*value))
            }
            (RuntimeDataBindGraphValue::Trigger(value), RuntimeDataBindGraphValue::String(_))
                if matches!(
                    self.converter.as_ref(),
                    Some(RuntimeDataBindGraphConverter::ToString { .. })
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Trigger(*value))
            }
            (
                RuntimeDataBindGraphValue::SymbolListIndex(value),
                RuntimeDataBindGraphValue::String(_),
            ) if matches!(
                self.converter.as_ref(),
                Some(RuntimeDataBindGraphConverter::ToString { .. })
            ) =>
            {
                Some(RuntimeDataBindGraphValue::SymbolListIndex(*value))
            }
            (RuntimeDataBindGraphValue::Color(value), RuntimeDataBindGraphValue::String(_))
                if matches!(
                    self.converter.as_ref(),
                    Some(RuntimeDataBindGraphConverter::ToString { .. })
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Color(*value))
            }
            (RuntimeDataBindGraphValue::Enum(value), RuntimeDataBindGraphValue::String(_))
                if matches!(
                    self.converter.as_ref(),
                    Some(RuntimeDataBindGraphConverter::ToString { .. })
                ) =>
            {
                Some(RuntimeDataBindGraphValue::Enum(*value))
            }
            _ => None,
        }
    }

    fn uses_delayed_string_source_to_target_after_main_to_source(&self) -> bool {
        runtime_data_bind_graph_converter_starts_with_to_string(self.converter.as_ref())
            || self
                .converter
                .as_ref()
                .is_some_and(runtime_data_bind_graph_converter_preserves_string_source_on_main_to_source_target_apply)
    }

    fn preserves_string_source_on_main_to_source_target_apply(&self) -> bool {
        self.is_main_to_source()
            && self
                .converter
                .as_ref()
                .is_some_and(runtime_data_bind_graph_converter_preserves_string_source_on_main_to_source_target_apply)
    }

    fn supports_direct_color_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::Color(_))
    }

    fn supports_direct_enum_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::Enum(_))
    }

    fn supports_direct_asset_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::Asset(_))
    }

    fn supports_direct_artboard_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::Artboard(_))
    }

    fn trigger_target_to_source_value(&mut self, value: u64) -> Option<RuntimeDataBindGraphValue> {
        if !self.bound || !self.applies_target_to_source() {
            return None;
        }
        let converted = match self.converter.as_ref() {
            None => RuntimeDataBindGraphValue::Trigger(value),
            Some(converter) if self.is_main_to_source() => self
                .converter_state
                .convert_value(converter, &RuntimeDataBindGraphValue::Trigger(value))?,
            Some(converter) => self
                .converter_state
                .reverse_convert_value(converter, &RuntimeDataBindGraphValue::Trigger(value))?,
        };
        match (&self.value, converted) {
            (RuntimeDataBindGraphValue::Trigger(_), RuntimeDataBindGraphValue::Trigger(value)) => {
                Some(RuntimeDataBindGraphValue::Trigger(value))
            }
            _ => None,
        }
    }

    fn supports_direct_view_model_target_to_source(&self) -> bool {
        self.applies_target_to_source()
            && self.converter.is_none()
            && matches!(self.value, RuntimeDataBindGraphValue::ViewModel(_))
    }

    fn reset_converter_state(&mut self) {
        self.converter_state =
            RuntimeDataBindGraphConverterState::for_converter(self.converter.as_ref());
    }

    fn reset_formula_random_state(&mut self) {
        self.converter_state.reset_formula_randoms();
    }

    fn reset_formula_random_state_for_source_change(&mut self) {
        if self
            .converter
            .as_ref()
            .is_some_and(runtime_data_bind_graph_converter_contains_source_change_random)
        {
            self.reset_formula_random_state();
        }
    }

    fn advance_stateful_converter(
        &mut self,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        self.converter_state
            .advance_converter(self.converter.as_ref(), elapsed_seconds)
    }

    fn should_skip_binding_for_phase(&self, phase: RuntimeDataBindGraphApplyPhase) -> bool {
        if !self.converter_state.is_initialized_stateful() {
            return false;
        }
        match phase {
            RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance => true,
            RuntimeDataBindGraphApplyPhase::AfterStatefulAdvance { elapsed_positive } => {
                !elapsed_positive
            }
            RuntimeDataBindGraphApplyPhase::Immediate
            | RuntimeDataBindGraphApplyPhase::PublicUpdate => false,
        }
    }

    fn converted_value(
        &mut self,
        formula_random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
    ) -> Option<RuntimeDataBindGraphValue> {
        match self.converter.as_ref() {
            None => Some(self.value.clone()),
            Some(converter @ RuntimeDataBindGraphConverter::ListToLength)
                if self.is_main_to_source() =>
            {
                self.converter_state
                    .reverse_convert_value_with_formula_randoms(
                        converter,
                        &self.value,
                        formula_random_source,
                    )
            }
            Some(converter @ RuntimeDataBindGraphConverter::ToString { .. })
                if self.is_main_to_source() =>
            {
                self.converter_state
                    .reverse_convert_value_with_formula_randoms(
                        converter,
                        &self.value,
                        formula_random_source,
                    )
            }
            Some(converter @ RuntimeDataBindGraphConverter::Interpolator { .. })
                if self.is_main_to_source() =>
            {
                self.converter_state
                    .reverse_convert_value_with_formula_randoms(
                        converter,
                        &self.value,
                        formula_random_source,
                    )
            }
            Some(converter @ RuntimeDataBindGraphConverter::TriggerIncrement)
                if self.is_main_to_source() =>
            {
                self.converter_state
                    .reverse_convert_value_with_formula_randoms(
                        converter,
                        &self.value,
                        formula_random_source,
                    )
            }
            Some(
                converter @ (RuntimeDataBindGraphConverter::StringTrim { .. }
                | RuntimeDataBindGraphConverter::StringRemoveZeros
                | RuntimeDataBindGraphConverter::StringPad { .. }),
            ) if self.is_main_to_source() => self
                .converter_state
                .reverse_convert_value_with_formula_randoms(
                    converter,
                    &self.value,
                    formula_random_source,
                ),
            Some(converter @ RuntimeDataBindGraphConverter::Group(_))
                if self.is_main_to_source() =>
            {
                self.converter_state
                    .reverse_convert_value_with_formula_randoms(
                        converter,
                        &self.value,
                        formula_random_source,
                    )
            }
            Some(converter) => self.converter_state.convert_value_with_formula_randoms(
                converter,
                &self.value,
                formula_random_source,
            ),
        }
    }
}

impl RuntimeDataBindGraphTargetsMut<'_> {
    pub(crate) fn apply_default_view_model_binding(
        &mut self,
        target: &RuntimeDataBindGraphTarget,
        value: &RuntimeDataBindGraphValue,
    ) {
        match (target, value) {
            (
                RuntimeDataBindGraphTarget::Number { global_id },
                RuntimeDataBindGraphValue::Number(value),
            ) => {
                if let Some(target) = self
                    .numbers
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Integer { global_id },
                RuntimeDataBindGraphValue::SymbolListIndex(value),
            ) => {
                if let Some(target) = self
                    .integers
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Boolean { global_id },
                RuntimeDataBindGraphValue::Boolean(value),
            ) => {
                if let Some(target) = self
                    .booleans
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::String { global_id },
                RuntimeDataBindGraphValue::String(value),
            ) => {
                if let Some(target) = self
                    .strings
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Color { global_id },
                RuntimeDataBindGraphValue::Color(value),
            ) => {
                if let Some(target) = self
                    .colors
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Enum { global_id },
                RuntimeDataBindGraphValue::Enum(value),
            ) => {
                if let Some(target) = self
                    .enums
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Asset { global_id },
                RuntimeDataBindGraphValue::Asset(value),
            ) => {
                if let Some(target) = self
                    .assets
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::Artboard { global_id },
                RuntimeDataBindGraphValue::Artboard(value),
            ) => {
                if let Some(target) = self
                    .artboards
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (RuntimeDataBindGraphTarget::List { .. }, RuntimeDataBindGraphValue::List { .. }) => {
                // C++ only applies list values to DataBindListItemConsumer targets.
            }
            (
                RuntimeDataBindGraphTarget::List { global_id },
                RuntimeDataBindGraphValue::Number(value),
            ) => {
                if let Some(target) = self
                    .lists
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(value.floor().max(0.0) as usize);
                }
            }
            (
                RuntimeDataBindGraphTarget::Trigger { global_id },
                RuntimeDataBindGraphValue::Trigger(value),
            ) => {
                if let Some(target) = self
                    .triggers
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            (
                RuntimeDataBindGraphTarget::ViewModel { global_id },
                RuntimeDataBindGraphValue::ViewModel(value),
            ) => {
                if let Some(target) = self
                    .view_models
                    .iter_mut()
                    .find(|target| target.global_id == *global_id)
                {
                    target.set_value(*value);
                }
            }
            _ => {}
        }
    }
}
