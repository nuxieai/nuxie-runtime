use crate::mechanical_port::source::{
    animation::{
        artboard_property::ArtboardProperty,
        state_machine_instance::{
            RuntimeComparisonValue, RuntimeStateMachineLayerInstanceWeakHandle,
            StateMachineInstance,
        },
        transition_condition_op::TransitionConditionOp,
    },
    core::CoreHandle,
    data_bind::{
        bindable_property_artboard::BindablePropertyArtboard,
        bindable_property_asset::BindablePropertyAsset,
        bindable_property_boolean::BindablePropertyBoolean,
        bindable_property_color::BindablePropertyColor,
        bindable_property_enum::BindablePropertyEnum,
        bindable_property_integer::BindablePropertyInteger,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        bindable_property_viewmodel::BindablePropertyViewModel, data_bind_context::DataBindContext,
    },
    generated::{
        animation::{
            transition_property_artboard_comparator_base::TransitionPropertyArtboardComparatorBase,
            transition_property_component_comparator_base::TransitionPropertyComponentComparatorBase,
            transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
            transition_self_comparator_base::TransitionSelfComparatorBase,
            transition_value_artboard_comparator_base::TransitionValueArtboardComparatorBase,
            transition_value_asset_comparator_base::TransitionValueAssetComparatorBase,
            transition_value_boolean_comparator_base::TransitionValueBooleanComparatorBase,
            transition_value_color_comparator_base::TransitionValueColorComparatorBase,
            transition_value_enum_comparator_base::TransitionValueEnumComparatorBase,
            transition_value_number_comparator_base::TransitionValueNumberComparatorBase,
            transition_value_string_comparator_base::TransitionValueStringComparatorBase,
            transition_value_trigger_comparator_base::TransitionValueTriggerComparatorBase,
            transition_viewmodel_condition_base::TransitionViewModelConditionBase,
        },
        core_registry::CoreRegistry,
    },
    viewmodel::viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparandKind {
    NumberDouble,
    NumberFromUint,
    Boolean,
    String,
    Color,
    Enum,
    Trigger,
    Asset,
    Artboard,
    ViewModel,
}

impl ComparandKind {
    fn is_number(self) -> bool {
        matches!(self, Self::NumberDouble | Self::NumberFromUint)
    }

    fn compatible_with(self, other: Self) -> bool {
        (self.is_number() && other.is_number()) || self == other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparatorSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonShape {
    Number,
    Boolean,
    String,
    Color,
    Enum,
    Uint32,
    ViewModel,
}

impl ComparisonShape {
    fn from_kinds(left: ComparandKind, right: ComparandKind) -> Option<Self> {
        if !left.compatible_with(right) {
            return None;
        }
        if left.is_number() && right.is_number() {
            return Some(
                if left == ComparandKind::NumberFromUint && right == ComparandKind::NumberFromUint {
                    Self::Uint32
                } else {
                    Self::Number
                },
            );
        }
        match left {
            ComparandKind::Boolean => Some(Self::Boolean),
            ComparandKind::String => Some(Self::String),
            ComparandKind::Color => Some(Self::Color),
            ComparandKind::Enum => Some(Self::Enum),
            ComparandKind::Trigger | ComparandKind::Asset | ComparandKind::Artboard => {
                Some(Self::Uint32)
            }
            ComparandKind::ViewModel => Some(Self::ViewModel),
            ComparandKind::NumberDouble | ComparandKind::NumberFromUint => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ComparandRecipe {
    ArtboardProperty,
    ComponentProperty,
    ViewModelProperty {
        property: CoreHandle,
        kind: ComparandKind,
    },
    Literal(ComparandKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ComparisonRecipe {
    None,
    SelfChange(CoreHandle),
    Typed {
        shape: ComparisonShape,
        left: ComparandRecipe,
        right: ComparandRecipe,
    },
}

pub struct TransitionViewModelCondition {
    pub base: TransitionViewModelConditionBase,
    left_comparator: Option<CoreHandle>,
    right_comparator: Option<CoreHandle>,
    comparison: Option<ComparisonRecipe>,
}

impl Default for TransitionViewModelCondition {
    fn default() -> Self {
        Self {
            base: TransitionViewModelConditionBase::default(),
            left_comparator: None,
            right_comparator: None,
            comparison: None,
        }
    }
}

impl TransitionViewModelCondition {
    pub fn reads_pending_change(&self) -> bool {
        self.reads_component_trigger()
            || [self.left_comparator(), self.right_comparator()]
                .into_iter()
                .flatten()
                .any(|comparator| {
                    comparator.is_type_of(TransitionSelfComparatorBase::TYPE_KEY)
                        || comparator.is_type_of(TransitionValueTriggerComparatorBase::TYPE_KEY)
                        || (Self::is_viewmodel_property(&comparator)
                            && comparator
                                .with(|c| c.transition_comparator_bindable_property())
                                .flatten()
                                .is_some_and(|b| {
                                    Self::bindable_kind(&b) == Some(ComparandKind::Trigger)
                                }))
                })
    }
    pub fn reads_component_trigger(&self) -> bool {
        [self.left_comparator(), self.right_comparator()]
            .into_iter()
            .flatten()
            .any(|c| {
                c.is_type_of(TransitionPropertyComponentComparatorBase::TYPE_KEY)
                    && Self::component_kind(&c) == Some(ComparandKind::Trigger)
            })
    }
    pub fn left_comparator(&self) -> Option<CoreHandle> {
        self.left_comparator.clone()
    }

    pub fn right_comparator(&self) -> Option<CoreHandle> {
        self.right_comparator.clone()
    }

    pub fn set_comparator(&mut self, comparator: CoreHandle) {
        if self.left_comparator.is_none() {
            self.left_comparator = Some(comparator);
        } else {
            self.right_comparator = Some(comparator);
        }
    }

    pub fn op(&self) -> Option<TransitionConditionOp> {
        TransitionConditionOp::from_u32(self.base.op_value())
    }

    fn is_viewmodel_property(comparator: &CoreHandle) -> bool {
        comparator.is_type_of(TransitionPropertyViewModelComparatorBase::TYPE_KEY)
    }

    fn bindable_kind(bindable: &CoreHandle) -> Option<ComparandKind> {
        match bindable.core_type()? {
            crate::mechanical_port::source::generated::data_bind::bindable_property_number_base::BindablePropertyNumberBase::TYPE_KEY => Some(ComparandKind::NumberDouble),
            crate::mechanical_port::source::generated::data_bind::bindable_property_integer_base::BindablePropertyIntegerBase::TYPE_KEY => Some(ComparandKind::NumberFromUint),
            crate::mechanical_port::source::generated::data_bind::bindable_property_boolean_base::BindablePropertyBooleanBase::TYPE_KEY => Some(ComparandKind::Boolean),
            crate::mechanical_port::source::generated::data_bind::bindable_property_string_base::BindablePropertyStringBase::TYPE_KEY => Some(ComparandKind::String),
            crate::mechanical_port::source::generated::data_bind::bindable_property_color_base::BindablePropertyColorBase::TYPE_KEY => Some(ComparandKind::Color),
            crate::mechanical_port::source::generated::data_bind::bindable_property_enum_base::BindablePropertyEnumBase::TYPE_KEY => Some(ComparandKind::Enum),
            crate::mechanical_port::source::generated::data_bind::bindable_property_trigger_base::BindablePropertyTriggerBase::TYPE_KEY => Some(ComparandKind::Trigger),
            crate::mechanical_port::source::generated::data_bind::bindable_property_asset_base::BindablePropertyAssetBase::TYPE_KEY => Some(ComparandKind::Asset),
            crate::mechanical_port::source::generated::data_bind::bindable_property_artboard_base::BindablePropertyArtboardBase::TYPE_KEY => Some(ComparandKind::Artboard),
            crate::mechanical_port::source::generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase::TYPE_KEY => Some(ComparandKind::ViewModel),
            _ => None,
        }
    }

    fn viewmodel_kind(comparator: &CoreHandle) -> Option<ComparandKind> {
        let bindable = comparator
            .with(|comparator| comparator.transition_comparator_bindable_property())??;
        Self::bindable_kind(&bindable)
    }

    fn component_kind(comparator: &CoreHandle) -> Option<ComparandKind> {
        let property_key = comparator
            .with(|comparator| comparator.transition_comparator_component_property_key())??;
        let field_id = CoreRegistry::property_field_id(property_key as i32);
        if field_id < 0 {
            return None;
        }
        match field_id {
            crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::ID => Some(ComparandKind::NumberDouble),
            crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::ID => Some(ComparandKind::Boolean),
            crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::ID => Some(ComparandKind::String),
            crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::ID => Some(ComparandKind::Color),
            crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::ID => {
                if property_key == crate::mechanical_port::source::generated::custom_property_enum_base::CustomPropertyEnumBase::PROPERTY_VALUE_PROPERTY_KEY as u32
                    || property_key == crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_enum_base::ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY as u32
                {
                    Some(ComparandKind::Enum)
                } else if property_key == crate::mechanical_port::source::generated::custom_property_trigger_base::CustomPropertyTriggerBase::PROPERTY_VALUE_PROPERTY_KEY as u32
                    || property_key == crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY as u32
                {
                    Some(ComparandKind::Trigger)
                } else if property_key == crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY as u32 {
                    Some(ComparandKind::Asset)
                } else if property_key == crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase::PROPERTY_VALUE_PROPERTY_KEY as u32 {
                    Some(ComparandKind::Artboard)
                } else if property_key == crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase::PROPERTY_VALUE_PROPERTY_KEY as u32 {
                    Some(ComparandKind::ViewModel)
                } else {
                    Some(ComparandKind::NumberFromUint)
                }
            }
            _ => None,
        }
    }

    fn comparator_kind(comparator: &CoreHandle, side: ComparatorSide) -> Option<ComparandKind> {
        let core_type = comparator.core_type()?;
        if core_type == TransitionPropertyArtboardComparatorBase::TYPE_KEY {
            return (side == ComparatorSide::Left).then_some(ComparandKind::NumberDouble);
        }
        if core_type == TransitionPropertyComponentComparatorBase::TYPE_KEY {
            return Self::component_kind(comparator);
        }
        if core_type == TransitionPropertyViewModelComparatorBase::TYPE_KEY {
            return Self::viewmodel_kind(comparator);
        }
        if side != ComparatorSide::Right {
            return None;
        }
        match core_type {
            TransitionValueNumberComparatorBase::TYPE_KEY => Some(ComparandKind::NumberDouble),
            TransitionValueBooleanComparatorBase::TYPE_KEY => Some(ComparandKind::Boolean),
            TransitionValueStringComparatorBase::TYPE_KEY => Some(ComparandKind::String),
            TransitionValueColorComparatorBase::TYPE_KEY => Some(ComparandKind::Color),
            TransitionValueEnumComparatorBase::TYPE_KEY => Some(ComparandKind::Enum),
            TransitionValueAssetComparatorBase::TYPE_KEY => Some(ComparandKind::Asset),
            TransitionValueArtboardComparatorBase::TYPE_KEY => Some(ComparandKind::Artboard),
            TransitionValueTriggerComparatorBase::TYPE_KEY => Some(ComparandKind::Trigger),
            _ => None,
        }
    }

    fn comparand_recipe(
        comparator: &CoreHandle,
        side: ComparatorSide,
        kind: ComparandKind,
    ) -> Option<ComparandRecipe> {
        match comparator.core_type()? {
            TransitionPropertyArtboardComparatorBase::TYPE_KEY
                if side == ComparatorSide::Left && kind == ComparandKind::NumberDouble =>
            {
                Some(ComparandRecipe::ArtboardProperty)
            }
            TransitionPropertyComponentComparatorBase::TYPE_KEY => {
                (Self::component_kind(comparator) == Some(kind))
                    .then_some(ComparandRecipe::ComponentProperty)
            }
            TransitionPropertyViewModelComparatorBase::TYPE_KEY => {
                let property = comparator
                    .with(|comparator| comparator.transition_comparator_bindable_property())??;
                (Self::bindable_kind(&property) == Some(kind))
                    .then_some(ComparandRecipe::ViewModelProperty { property, kind })
            }
            _ if side == ComparatorSide::Right
                && Self::comparator_kind(comparator, side) == Some(kind) =>
            {
                Some(ComparandRecipe::Literal(kind))
            }
            _ => None,
        }
    }

    fn comparand_value(
        comparator: &CoreHandle,
        recipe: &ComparandRecipe,
        machine: &StateMachineInstance,
        layer: Option<&RuntimeStateMachineLayerInstanceWeakHandle>,
    ) -> Option<RuntimeComparisonValue> {
        match recipe {
            ComparandRecipe::ArtboardProperty => {
                let property_type = comparator.with(|comparator| {
                    comparator.transition_comparator_artboard_property_type()
                })??;
                let (width, height) = machine.artboard_layout_size()?;
                Some(RuntimeComparisonValue::Number(
                    match ArtboardProperty::from_u32(property_type) {
                        Some(ArtboardProperty::Width) => width,
                        Some(ArtboardProperty::Height) => height,
                        Some(ArtboardProperty::Ratio) => width / height,
                        _ => 0.0,
                    },
                ))
            }
            ComparandRecipe::ComponentProperty => {
                let (object_id, property_key) = comparator.with(|comparator| {
                    Some((
                        comparator.transition_comparator_component_object_id()?,
                        comparator.transition_comparator_component_property_key()?,
                    ))
                })??;
                if Self::component_kind(comparator) == Some(ComparandKind::Trigger) {
                    let sequence = machine.resolve_artboard_object(object_id).and_then(|target| target.with(|target| {
                        if let Some(value) = target.as_any().downcast_ref::<crate::source::custom_property_trigger::CustomPropertyTrigger>() { value.change_sequence() }
                        else if let Some(value) = target.as_any().downcast_ref::<crate::source::script_input_trigger::ScriptInputTrigger>() { value.base.base.change_sequence() }
                        else { target.as_view_model_instance_value().map_or(0, |v| v.change_sequence()) }
                    })).unwrap_or(0);
                    Some(RuntimeComparisonValue::Uint(u32::from(
                        machine.change_pending(sequence, layer),
                    )))
                } else {
                    machine.component_comparison_value(object_id, property_key)
                }
            }
            ComparandRecipe::ViewModelProperty { property, kind } => {
                let Some(property) = machine.bindable_property_instance(property) else {
                    return (*kind == ComparandKind::ViewModel)
                        .then_some(RuntimeComparisonValue::ViewModel(None));
                };
                match kind {
                    ComparandKind::NumberDouble => property
                        .with_downcast::<BindablePropertyNumber, _>(|property| {
                            RuntimeComparisonValue::Number(property.base.property_value())
                        }),
                    ComparandKind::NumberFromUint => property
                        .with_downcast::<BindablePropertyInteger, _>(|property| {
                            RuntimeComparisonValue::Uint(property.base.property_value())
                        }),
                    ComparandKind::Boolean => {
                        property.with_downcast::<BindablePropertyBoolean, _>(|property| {
                            RuntimeComparisonValue::Boolean(property.base.property_value())
                        })
                    }
                    ComparandKind::String => {
                        property.with_downcast::<BindablePropertyString, _>(|property| {
                            RuntimeComparisonValue::String(
                                property.base.property_value().to_owned(),
                            )
                        })
                    }
                    ComparandKind::Color => {
                        property.with_downcast::<BindablePropertyColor, _>(|property| {
                            RuntimeComparisonValue::Color(property.base.property_value())
                        })
                    }
                    ComparandKind::Enum => {
                        property.with_downcast::<BindablePropertyEnum, _>(|property| {
                            RuntimeComparisonValue::Uint(property.base.property_value())
                        })
                    }
                    ComparandKind::Trigger => {
                        let source =
                            machine
                                .bindable_data_bind_to_target(&property)
                                .and_then(|bind| {
                                    bind.with(|bind| {
                                        bind.as_data_bind().and_then(|bind| bind.source())
                                    })
                                    .flatten()
                                });
                        // A fire already used by this layer reads zero, like
                        // the self/fired condition. Component triggers above
                        // retain the plain change window.
                        let pending = source
                            .is_some_and(|source| machine.view_model_value_changed(&source, layer));
                        Some(RuntimeComparisonValue::Uint(u32::from(pending)))
                    }
                    ComparandKind::Asset => {
                        property.with_downcast::<BindablePropertyAsset, _>(|property| {
                            RuntimeComparisonValue::Uint(property.base.property_value())
                        })
                    }
                    ComparandKind::Artboard => property
                        .with_downcast::<BindablePropertyArtboard, _>(|property| {
                            RuntimeComparisonValue::Uint(property.base.property_value())
                        }),
                    ComparandKind::ViewModel => {
                        if let Some(bind) = machine.bindable_data_bind_to_target(&property) {
                            if bind
                                .with_downcast::<DataBindContext, _>(|bind| {
                                    bind.source_path_length() == 1
                                })
                                .unwrap_or(false)
                            {
                                return Some(RuntimeComparisonValue::ViewModel(
                                    machine.data_context().and_then(|context| {
                                        context.with_context(|context| {
                                            context.main_view_model_instance()
                                        })
                                    }),
                                ));
                            }
                            if let Some(referenced) = bind
                                .with(|bind| bind.as_data_bind().and_then(|bind| bind.source()))
                                .flatten()
                                .and_then(|source| {
                                    source.with_downcast::<ViewModelInstanceViewModel, _>(
                                        |source| source.reference_view_model_instance(),
                                    )
                                })
                                .flatten()
                            {
                                return Some(RuntimeComparisonValue::ViewModel(Some(referenced)));
                            }
                        }
                        property.with_downcast::<BindablePropertyViewModel, _>(|property| {
                            RuntimeComparisonValue::ViewModel(
                                property
                                    .view_model_instance_value()
                                    .or_else(|| property.view_model_instance()),
                            )
                        })
                    }
                }
            }
            ComparandRecipe::Literal(kind) => comparator.with(|comparator| match kind {
                ComparandKind::NumberDouble => comparator
                    .transition_comparator_number_value()
                    .map(RuntimeComparisonValue::Number),
                ComparandKind::Boolean => comparator
                    .transition_comparator_bool_value()
                    .map(RuntimeComparisonValue::Boolean),
                ComparandKind::String => comparator
                    .transition_comparator_string_value()
                    .map(RuntimeComparisonValue::String),
                ComparandKind::Color => comparator
                    .transition_comparator_color_value()
                    .map(RuntimeComparisonValue::Color),
                ComparandKind::NumberFromUint
                | ComparandKind::Enum
                | ComparandKind::Trigger
                | ComparandKind::Asset
                | ComparandKind::Artboard => comparator
                    .transition_comparator_uint_value()
                    .map(RuntimeComparisonValue::Uint),
                ComparandKind::ViewModel => None,
            })?,
        }
    }

    fn compare_values(
        &self,
        shape: ComparisonShape,
        left: RuntimeComparisonValue,
        right: RuntimeComparisonValue,
    ) -> bool {
        match shape {
            ComparisonShape::Number => {
                let number = |value| match value {
                    RuntimeComparisonValue::Number(value) => Some(value),
                    RuntimeComparisonValue::Uint(value) => Some(value as f32),
                    _ => None,
                };
                number(left)
                    .zip(number(right))
                    .is_some_and(|(left, right)| self.compare_number(left, right))
            }
            ComparisonShape::Boolean => match (left, right) {
                (RuntimeComparisonValue::Boolean(left), RuntimeComparisonValue::Boolean(right)) => {
                    self.compare_eq(left, right)
                }
                _ => false,
            },
            ComparisonShape::String => match (left, right) {
                (RuntimeComparisonValue::String(left), RuntimeComparisonValue::String(right)) => {
                    self.compare_eq(left, right)
                }
                _ => false,
            },
            ComparisonShape::Color => match (left, right) {
                (RuntimeComparisonValue::Color(left), RuntimeComparisonValue::Color(right)) => {
                    self.compare_eq(left, right)
                }
                _ => false,
            },
            ComparisonShape::Enum | ComparisonShape::Uint32 => match (left, right) {
                (RuntimeComparisonValue::Uint(left), RuntimeComparisonValue::Uint(right)) => {
                    self.compare_eq(left, right)
                }
                _ => false,
            },
            ComparisonShape::ViewModel => match (left, right) {
                (
                    RuntimeComparisonValue::ViewModel(left),
                    RuntimeComparisonValue::ViewModel(right),
                ) => self.compare_eq(left, right),
                _ => false,
            },
        }
    }

    fn compare_eq<T: PartialEq>(&self, left: T, right: T) -> bool {
        match self.op() {
            Some(TransitionConditionOp::Equal) => left == right,
            Some(TransitionConditionOp::NotEqual) => left != right,
            _ => false,
        }
    }

    fn compare_number(&self, left: f32, right: f32) -> bool {
        match self.op() {
            Some(TransitionConditionOp::Equal) => left == right,
            Some(TransitionConditionOp::NotEqual) => left != right,
            Some(TransitionConditionOp::LessThanOrEqual) => left <= right,
            Some(TransitionConditionOp::LessThan) => left < right,
            Some(TransitionConditionOp::GreaterThanOrEqual) => left >= right,
            Some(TransitionConditionOp::GreaterThan) => left > right,
            _ => false,
        }
    }

    fn can_evaluate(&self, machine: &StateMachineInstance) -> bool {
        let (Some(_), Some(_), Some(recipe)) = (
            &self.left_comparator,
            &self.right_comparator,
            self.comparison.as_ref(),
        ) else {
            return false;
        };
        let requires_data_context = match recipe {
            ComparisonRecipe::SelfChange(_) => true,
            ComparisonRecipe::Typed { left, right, .. } => matches!(
                (left, right),
                (ComparandRecipe::ViewModelProperty { .. }, _)
                    | (_, ComparandRecipe::ViewModelProperty { .. })
            ),
            ComparisonRecipe::None => false,
        };
        !requires_data_context || machine.data_context().is_some()
    }

    pub fn evaluate(
        &self,
        machine: &StateMachineInstance,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) -> bool {
        if !self.can_evaluate(machine) {
            return false;
        }
        let (Some(left), Some(right), Some(recipe)) = (
            &self.left_comparator,
            &self.right_comparator,
            self.comparison.as_ref(),
        ) else {
            return false;
        };
        match recipe {
            ComparisonRecipe::None => false,
            ComparisonRecipe::SelfChange(property) => {
                machine.bindable_source_changed_in_layer(property, layer)
            }
            ComparisonRecipe::Typed {
                shape,
                left: left_recipe,
                right: right_recipe,
            } => {
                let (Some(left), Some(right)) = (
                    Self::comparand_value(left, left_recipe, machine, layer.as_ref()),
                    Self::comparand_value(right, right_recipe, machine, layer.as_ref()),
                ) else {
                    return false;
                };
                self.compare_values(*shape, left, right)
            }
        }
    }

    pub fn use_in_layer(
        &self,
        machine: &StateMachineInstance,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        if let Some(left) = &self.left_comparator {
            let _ = left
                .with_mut(|left| left.transition_comparator_use_in_layer(machine, layer.clone()));
        }
        if let Some(right) = self
            .right_comparator
            .as_ref()
            .filter(|right| right.is_type_of(TransitionPropertyViewModelComparatorBase::TYPE_KEY))
        {
            let property = right
                .with(|right| right.transition_comparator_bindable_property())
                .flatten();
            super::transition_property_viewmodel_comparator::TransitionPropertyViewModelComparator::use_trigger_property_fire_in_layer(property.as_ref(),machine,layer);
        }
    }

    pub fn initialize(&mut self) {
        let (Some(left), Some(right)) = (&self.left_comparator, &self.right_comparator) else {
            return;
        };
        if right.is_type_of(TransitionSelfComparatorBase::TYPE_KEY) {
            self.comparison = Some(
                left.with(|left| left.transition_comparator_bindable_property())
                    .flatten()
                    .map(ComparisonRecipe::SelfChange)
                    .unwrap_or(ComparisonRecipe::None),
            );
            return;
        }

        let left_kind = Self::comparator_kind(left, ComparatorSide::Left);
        let right_kind = Self::comparator_kind(right, ComparatorSide::Right);
        if left_kind == Some(ComparandKind::Trigger)
            && right.is_type_of(TransitionValueTriggerComparatorBase::TYPE_KEY)
            && Self::is_viewmodel_property(left)
        {
            self.comparison = Some(
                left.with(|left| left.transition_comparator_bindable_property())
                    .flatten()
                    .map(ComparisonRecipe::SelfChange)
                    .unwrap_or(ComparisonRecipe::None),
            );
            return;
        }

        self.comparison = Some(match left_kind.zip(right_kind) {
            Some((left_kind, right_kind)) => {
                match (
                    ComparisonShape::from_kinds(left_kind, right_kind),
                    Self::comparand_recipe(left, ComparatorSide::Left, left_kind),
                    Self::comparand_recipe(right, ComparatorSide::Right, right_kind),
                ) {
                    (Some(shape), Some(left), Some(right)) => {
                        ComparisonRecipe::Typed { shape, left, right }
                    }
                    _ => ComparisonRecipe::None,
                }
            }
            None => ComparisonRecipe::None,
        });
    }
}
impl std::ops::Deref for TransitionViewModelCondition {
    type Target = TransitionViewModelConditionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionViewModelCondition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBaseCallbacks for TransitionViewModelCondition { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }

#[cfg(test)]
mod nullable_view_model_tests {
    use super::*;
    use crate::source::{core::CoreArena, viewmodel::viewmodel_instance::ViewModelInstance};

    #[test]
    fn view_model_comparands_compare_nullable_pointer_identity() {
        let arena = CoreArena::default();
        let first = arena.insert(ViewModelInstance::default());
        let second = arena.insert(ViewModelInstance::default());
        let cases = [
            (None, None, true),
            (None, Some(first.clone()), false),
            (Some(first.clone()), None, false),
            (Some(first.clone()), Some(first), true),
            (Some(second.clone()), Some(second.clone()), true),
            (
                Some(arena.insert(ViewModelInstance::default())),
                Some(second),
                false,
            ),
        ];
        let mut condition = TransitionViewModelCondition::default();
        for (left, right, equal) in cases {
            for (op, expected) in [
                (TransitionConditionOp::Equal, equal),
                (TransitionConditionOp::NotEqual, !equal),
            ] {
                condition.base.set_op_value_value(op as u32);
                assert_eq!(
                    condition.compare_values(
                        ComparisonShape::ViewModel,
                        RuntimeComparisonValue::ViewModel(left.clone()),
                        RuntimeComparisonValue::ViewModel(right.clone()),
                    ),
                    expected
                );
            }
        }
    }
}
