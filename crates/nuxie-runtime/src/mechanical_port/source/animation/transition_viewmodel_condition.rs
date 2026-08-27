use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        artboard_property::ArtboardProperty, state_machine_instance::StateMachineInstance,
        state_machine_layer_instance::StateMachineLayerInstance,
        transition_comparator::TransitionComparator,
        transition_condition_op::TransitionConditionOp,
        transition_property_artboard_comparator::TransitionPropertyArtboardComparator,
        transition_property_component_comparator::TransitionPropertyComponentComparator,
        transition_property_viewmodel_comparator::TransitionPropertyViewModelComparator,
        transition_self_comparator::TransitionSelfComparator,
        transition_value_artboard_comparator::TransitionValueArtboardComparator,
        transition_value_asset_comparator::TransitionValueAssetComparator,
        transition_value_boolean_comparator::TransitionValueBooleanComparator,
        transition_value_color_comparator::TransitionValueColorComparator,
        transition_value_enum_comparator::TransitionValueEnumComparator,
        transition_value_number_comparator::TransitionValueNumberComparator,
        transition_value_string_comparator::TransitionValueStringComparator,
        transition_value_trigger_comparator::TransitionValueTriggerComparator,
    },
    core::Core,
    data_bind::{
        bindable_property::BindableProperty, bindable_property_artboard::BindablePropertyArtboard,
        bindable_property_asset::BindablePropertyAsset,
        bindable_property_boolean::BindablePropertyBoolean,
        bindable_property_color::BindablePropertyColor,
        bindable_property_enum::BindablePropertyEnum,
        bindable_property_integer::BindablePropertyInteger,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        bindable_property_trigger::BindablePropertyTrigger,
        bindable_property_viewmodel::BindablePropertyViewModel,
    },
    generated::{
        animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase,
        core_registry::CoreRegistry,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};

pub trait ConditionOperation {
    fn compare_numbers(&self, _a: f32, _b: f32) -> bool {
        false
    }
    fn compare_booleans(&self, _a: bool, _b: bool) -> bool {
        false
    }
    fn compare_strings(&self, _a: &str, _b: &str) -> bool {
        false
    }
    fn compare_ints(&self, _a: i32, _b: i32) -> bool {
        false
    }
    fn compare_u32s(&self, _a: u32, _b: u32) -> bool {
        false
    }
}

fn equal<T: PartialEq>(left: T, right: T) -> bool {
    left == right
}

fn not_equal<T: PartialEq>(left: T, right: T) -> bool {
    left != right
}

fn less_than_or_equal<T: PartialOrd>(left: T, right: T) -> bool {
    left <= right
}

fn less_than<T: PartialOrd>(left: T, right: T) -> bool {
    left < right
}

fn greater_than_or_equal<T: PartialOrd>(left: T, right: T) -> bool {
    left >= right
}

fn greater_than<T: PartialOrd>(left: T, right: T) -> bool {
    left > right
}

pub struct ConditionOperationEqual;
impl ConditionOperation for ConditionOperationEqual {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        equal(a, b)
    }
    fn compare_booleans(&self, a: bool, b: bool) -> bool {
        equal(a, b)
    }
    fn compare_strings(&self, a: &str, b: &str) -> bool {
        equal(a, b)
    }
    fn compare_ints(&self, a: i32, b: i32) -> bool {
        equal(a, b)
    }
    fn compare_u32s(&self, a: u32, b: u32) -> bool {
        equal(a, b)
    }
}
pub struct ConditionOperationNotEqual;
impl ConditionOperation for ConditionOperationNotEqual {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        not_equal(a, b)
    }
    fn compare_booleans(&self, a: bool, b: bool) -> bool {
        not_equal(a, b)
    }
    fn compare_strings(&self, a: &str, b: &str) -> bool {
        not_equal(a, b)
    }
    fn compare_ints(&self, a: i32, b: i32) -> bool {
        not_equal(a, b)
    }
    fn compare_u32s(&self, a: u32, b: u32) -> bool {
        not_equal(a, b)
    }
}
pub struct ConditionOperationLessThanOrEqual;
impl ConditionOperation for ConditionOperationLessThanOrEqual {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        less_than_or_equal(a, b)
    }
}
pub struct ConditionOperationLessThan;
impl ConditionOperation for ConditionOperationLessThan {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        less_than(a, b)
    }
}
pub struct ConditionOperationGreaterThanOrEqual;
impl ConditionOperation for ConditionOperationGreaterThanOrEqual {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        greater_than_or_equal(a, b)
    }
}
pub struct ConditionOperationGreaterThan;
impl ConditionOperation for ConditionOperationGreaterThan {
    fn compare_numbers(&self, a: f32, b: f32) -> bool {
        greater_than(a, b)
    }
}
pub struct ConditionOperationDefault;
impl ConditionOperation for ConditionOperationDefault {}

pub trait ConditionComparand {}
impl<T: ?Sized> ConditionComparand for T {}

pub trait ConditionComparandNumber: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> f32;
}
pub trait ConditionComparandBoolean: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> bool;
}
pub trait ConditionComparandString: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> String;
}
pub trait ConditionComparandColor: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> i32;
}
pub trait ConditionComparandUint32: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> u32;
}
pub trait ConditionComparandViewModel: ConditionComparand {
    fn value(&self, state_machine: &StateMachineInstance) -> Option<NonNull<ViewModelInstance>>;
}

fn bindable_instance(
    machine: &StateMachineInstance,
    property: NonNull<BindableProperty>,
) -> Option<NonNull<BindableProperty>> {
    machine.bindable_property_instance(property)
}

pub struct ConditionComparandNumberBindable {
    property: NonNull<BindablePropertyNumber>,
}
impl ConditionComparandNumberBindable {
    pub fn new(property: NonNull<BindablePropertyNumber>) -> Self {
        Self { property }
    }
}
impl ConditionComparandNumber for ConditionComparandNumberBindable {
    fn value(&self, machine: &StateMachineInstance) -> f32 {
        bindable_instance(machine, self.property.cast())
            .and_then(|value| unsafe { value.as_ref() }.as_number())
            .map_or(0.0, |value| value.property_value())
    }
}
pub struct ConditionComparandArtboardProperty {
    property: NonNull<TransitionPropertyArtboardComparator>,
}
impl ConditionComparandArtboardProperty {
    pub fn new(property: NonNull<TransitionPropertyArtboardComparator>) -> Self {
        Self { property }
    }
}
impl ConditionComparandNumber for ConditionComparandArtboardProperty {
    fn value(&self, machine: &StateMachineInstance) -> f32 {
        let Some(artboard) = machine.artboard() else {
            return 0.0;
        };
        match ArtboardProperty::from_u32(unsafe { self.property.as_ref() }.base.property_type()) {
            Some(ArtboardProperty::Width) => unsafe { artboard.as_ref() }.layout_width(),
            Some(ArtboardProperty::Height) => unsafe { artboard.as_ref() }.layout_height(),
            Some(ArtboardProperty::Ratio) => {
                unsafe { artboard.as_ref() }.layout_width()
                    / unsafe { artboard.as_ref() }.layout_height()
            }
            _ => 0.0,
        }
    }
}
pub struct ConditionComparandNumberBindableInteger {
    property: NonNull<BindablePropertyInteger>,
}
impl ConditionComparandNumberBindableInteger {
    pub fn new(property: NonNull<BindablePropertyInteger>) -> Self {
        Self { property }
    }
}
impl ConditionComparandNumber for ConditionComparandNumberBindableInteger {
    fn value(&self, machine: &StateMachineInstance) -> f32 {
        bindable_instance(machine, self.property.cast())
            .and_then(|value| unsafe { value.as_ref() }.as_integer())
            .map_or(0.0, |value| value.property_value() as f32)
    }
}
pub struct ConditionComparandNumberValue {
    value: NonNull<TransitionValueNumberComparator>,
}
impl ConditionComparandNumberValue {
    pub fn new(value: NonNull<TransitionValueNumberComparator>) -> Self {
        Self { value }
    }
}
impl ConditionComparandNumber for ConditionComparandNumberValue {
    fn value(&self, _: &StateMachineInstance) -> f32 {
        unsafe { self.value.as_ref() }.base.value()
    }
}

macro_rules! bindable_comparand {
    ($name:ident, $trait_name:ident, $property:ty, $result:ty, $accessor:ident, $default:expr) => {
        pub struct $name {
            property: NonNull<$property>,
        }
        impl $name {
            pub fn new(property: NonNull<$property>) -> Self {
                Self { property }
            }
        }
        impl $trait_name for $name {
            fn value(&self, machine: &StateMachineInstance) -> $result {
                bindable_instance(machine, self.property.cast())
                    .and_then(|value| unsafe { value.as_ref() }.$accessor())
                    .map_or($default, |value| value.property_value())
            }
        }
    };
}
macro_rules! value_comparand {
    ($name:ident, $trait_name:ident, $value_type:ty, $result:ty, $default:expr) => {
        pub struct $name {
            value: NonNull<$value_type>,
        }
        impl $name {
            pub fn new(value: NonNull<$value_type>) -> Self {
                Self { value }
            }
        }
        impl $trait_name for $name {
            fn value(&self, _: &StateMachineInstance) -> $result {
                unsafe { self.value.as_ref() }.base.value().into()
            }
        }
    };
}

bindable_comparand!(
    ConditionComparandBooleanBindable,
    ConditionComparandBoolean,
    BindablePropertyBoolean,
    bool,
    as_boolean,
    false
);
value_comparand!(
    ConditionComparandBooleanValue,
    ConditionComparandBoolean,
    TransitionValueBooleanComparator,
    bool,
    false
);
bindable_comparand!(
    ConditionComparandStringBindable,
    ConditionComparandString,
    BindablePropertyString,
    String,
    as_string,
    String::new()
);
value_comparand!(
    ConditionComparandStringValue,
    ConditionComparandString,
    TransitionValueStringComparator,
    String,
    String::new()
);
bindable_comparand!(
    ConditionComparandColorBindable,
    ConditionComparandColor,
    BindablePropertyColor,
    i32,
    as_color,
    0
);
value_comparand!(
    ConditionComparandColorValue,
    ConditionComparandColor,
    TransitionValueColorComparator,
    i32,
    0
);
bindable_comparand!(
    ConditionComparandEnumBindable,
    ConditionComparandUint32,
    BindablePropertyEnum,
    u32,
    as_enum,
    0
);
value_comparand!(
    ConditionComparandEnumValue,
    ConditionComparandUint32,
    TransitionValueEnumComparator,
    u32,
    0
);
bindable_comparand!(
    ConditionComparandTriggerBindable,
    ConditionComparandUint32,
    BindablePropertyTrigger,
    u32,
    as_trigger,
    0
);
value_comparand!(
    ConditionComparandTriggerValue,
    ConditionComparandUint32,
    TransitionValueTriggerComparator,
    u32,
    0
);
bindable_comparand!(
    ConditionComparandIntegerBindable,
    ConditionComparandUint32,
    BindablePropertyInteger,
    u32,
    as_integer,
    0
);
bindable_comparand!(
    ConditionComparandAssetBindable,
    ConditionComparandUint32,
    BindablePropertyAsset,
    u32,
    as_asset,
    0
);
value_comparand!(
    ConditionComparandAssetValue,
    ConditionComparandUint32,
    TransitionValueAssetComparator,
    u32,
    0
);
bindable_comparand!(
    ConditionComparandArtboardBindable,
    ConditionComparandUint32,
    BindablePropertyArtboard,
    u32,
    as_artboard,
    0
);
value_comparand!(
    ConditionComparandArtboardValue,
    ConditionComparandUint32,
    TransitionValueArtboardComparator,
    u32,
    0
);

pub struct ConditionComparandViewModelBindable {
    property: NonNull<BindablePropertyViewModel>,
}

impl ConditionComparandViewModelBindable {
    pub fn new(property: NonNull<BindablePropertyViewModel>) -> Self {
        Self { property }
    }
}

impl ConditionComparandViewModel for ConditionComparandViewModelBindable {
    fn value(&self, machine: &StateMachineInstance) -> Option<NonNull<ViewModelInstance>> {
        let bindable_instance = bindable_instance(machine, self.property.cast())?;
        if let Some(data_bind) = machine.bindable_data_bind_to_target(Some(bindable_instance)) {
            if let Some(context) = unsafe { data_bind.as_ref() }.as_data_bind_context() {
                // A root-only source path marks "My ViewModel" for transition
                // comparands. Resolve it from the state machine data context.
                if context.source_path_ids().len() == 1 {
                    return machine.data_context().and_then(|context| {
                        unsafe { context.as_ref() }.main_view_model_instance()
                    });
                }
            }
            if let Some(source) = unsafe { data_bind.as_ref() }.source() {
                if let Some(source_view_model) =
                    unsafe { source.as_ref() }.as_view_model_instance_view_model()
                {
                    if let Some(referenced) = source_view_model.reference_view_model_instance() {
                        return Some(referenced);
                    }
                }
            }
        }
        let bindable_view_model = unsafe { bindable_instance.as_ref() }.as_view_model()?;
        if let Some(instance_value) = bindable_view_model.view_model_instance_value() {
            return Some(instance_value);
        }
        bindable_view_model.view_model_instance()
    }
}

fn resolve_component_target(
    machine: &StateMachineInstance,
    comparator: &TransitionPropertyComponentComparator,
) -> Option<NonNull<Core>> {
    let artboard = machine.artboard()?;
    let target = unsafe { artboard.as_ref() }.resolve(comparator.base.object_id())?;
    if !CoreRegistry::object_supports_property(
        unsafe { target.as_ref() },
        comparator.base.property_key(),
    ) {
        return None;
    }
    Some(target)
}

macro_rules! component_comparand {
    ($name:ident, $trait_name:ident, $result:ty, $default:expr, $getter:ident) => {
        pub struct $name {
            comparator: NonNull<TransitionPropertyComponentComparator>,
        }

        impl $name {
            pub fn new(comparator: NonNull<TransitionPropertyComponentComparator>) -> Self {
                Self { comparator }
            }
        }

        impl $trait_name for $name {
            fn value(&self, machine: &StateMachineInstance) -> $result {
                let comparator = unsafe { self.comparator.as_ref() };
                let Some(target) = resolve_component_target(machine, comparator) else {
                    return $default;
                };
                CoreRegistry::$getter(
                    unsafe { target.as_ref() },
                    comparator.base.property_key() as i32,
                )
            }
        }
    };
}

component_comparand!(
    ConditionComparandComponentCoreNumber,
    ConditionComparandNumber,
    f32,
    0.0,
    get_double
);

pub struct ConditionComparandComponentCoreUintAsNumber {
    comparator: NonNull<TransitionPropertyComponentComparator>,
}

impl ConditionComparandComponentCoreUintAsNumber {
    pub fn new(comparator: NonNull<TransitionPropertyComponentComparator>) -> Self {
        Self { comparator }
    }
}

impl ConditionComparandNumber for ConditionComparandComponentCoreUintAsNumber {
    fn value(&self, machine: &StateMachineInstance) -> f32 {
        let comparator = unsafe { self.comparator.as_ref() };
        let Some(target) = resolve_component_target(machine, comparator) else {
            return 0.0;
        };
        CoreRegistry::get_uint(
            unsafe { target.as_ref() },
            comparator.base.property_key() as i32,
        ) as f32
    }
}

component_comparand!(
    ConditionComparandComponentCoreBoolean,
    ConditionComparandBoolean,
    bool,
    false,
    get_bool
);
component_comparand!(
    ConditionComparandComponentCoreString,
    ConditionComparandString,
    String,
    String::new(),
    get_string
);
component_comparand!(
    ConditionComparandComponentCoreColor,
    ConditionComparandColor,
    i32,
    0,
    get_color
);
component_comparand!(
    ConditionComparandComponentCoreUint,
    ConditionComparandUint32,
    u32,
    0,
    get_uint
);

// The pinned header declares this comparand, but the pinned source supplies no
// constructor or value definition and never constructs it.
pub struct ConditionComparandComponentViewModel {
    comparator: NonNull<TransitionPropertyComponentComparator>,
}

pub trait ConditionComparison {
    fn operation(&self) -> Option<&dyn ConditionOperation> {
        None
    }

    fn compare(
        &self,
        machine: &StateMachineInstance,
        layer: Option<NonNull<StateMachineLayerInstance>>,
    ) -> bool;

    fn compare_numbers(&self, left: f32, right: f32) -> bool {
        self.operation()
            .is_some_and(|operation| operation.compare_numbers(left, right))
    }

    fn compare_strings(&self, left: &str, right: &str) -> bool {
        self.operation()
            .is_some_and(|operation| operation.compare_strings(left, right))
    }

    fn compare_booleans(&self, left: bool, right: bool) -> bool {
        self.operation()
            .is_some_and(|operation| operation.compare_booleans(left, right))
    }

    fn compare_colors(&self, left: i32, right: i32) -> bool {
        self.operation()
            .is_some_and(|operation| operation.compare_ints(left, right))
    }

    fn compare_u32s(&self, left: u32, right: u32) -> bool {
        self.operation()
            .is_some_and(|operation| operation.compare_u32s(left, right))
    }

    fn compare_pointers(
        &self,
        left: Option<NonNull<ViewModelInstance>>,
        right: Option<NonNull<ViewModelInstance>>,
    ) -> bool {
        self.compare_booleans(left == right, true)
    }
}

pub struct ConditionComparisonNone;

impl ConditionComparison for ConditionComparisonNone {
    fn compare(
        &self,
        _machine: &StateMachineInstance,
        _layer: Option<NonNull<StateMachineLayerInstance>>,
    ) -> bool {
        false
    }
}

pub struct ConditionComparisonSelf {
    bindable_property: NonNull<BindableProperty>,
}

impl ConditionComparisonSelf {
    pub fn new(bindable_property: NonNull<BindableProperty>) -> Self {
        Self { bindable_property }
    }
}

impl ConditionComparison for ConditionComparisonSelf {
    fn compare(
        &self,
        machine: &StateMachineInstance,
        layer: Option<NonNull<StateMachineLayerInstance>>,
    ) -> bool {
        let bindable_instance = machine.bindable_property_instance(self.bindable_property);
        let data_bind = machine.bindable_data_bind_to_target(bindable_instance);
        if let Some(data_bind) = data_bind {
            if let Some(source) = unsafe { data_bind.as_ref() }.source() {
                if unsafe { source.as_ref() }.has_changed()
                    && !unsafe { source.as_ref() }.is_used_in_layer(layer)
                {
                    return true;
                }
            }
        }
        false
    }
}

macro_rules! typed_comparison {
    ($name:ident, $comparand:ident, $operation_method:ident) => {
        pub struct $name {
            left_comparand: Box<dyn $comparand>,
            right_comparand: Box<dyn $comparand>,
            operation: Box<dyn ConditionOperation>,
        }

        impl $name {
            pub fn new(
                left: Box<dyn $comparand>,
                right: Box<dyn $comparand>,
                operation: Box<dyn ConditionOperation>,
            ) -> Self {
                Self {
                    left_comparand: left,
                    right_comparand: right,
                    operation,
                }
            }
        }

        impl ConditionComparison for $name {
            fn operation(&self) -> Option<&dyn ConditionOperation> {
                Some(self.operation.as_ref())
            }

            fn compare(
                &self,
                machine: &StateMachineInstance,
                _layer: Option<NonNull<StateMachineLayerInstance>>,
            ) -> bool {
                self.$operation_method(
                    self.left_comparand.value(machine),
                    self.right_comparand.value(machine),
                )
            }
        }
    };
}

typed_comparison!(
    ConditionComparisonNumber,
    ConditionComparandNumber,
    compare_numbers
);
typed_comparison!(
    ConditionComparisonBoolean,
    ConditionComparandBoolean,
    compare_booleans
);
typed_comparison!(
    ConditionComparisonString,
    ConditionComparandString,
    compare_strings
);
typed_comparison!(
    ConditionComparisonColor,
    ConditionComparandColor,
    compare_colors
);
typed_comparison!(
    ConditionComparisonEnum,
    ConditionComparandUint32,
    compare_u32s
);
typed_comparison!(
    ConditionComparisonUint32,
    ConditionComparandUint32,
    compare_u32s
);

pub struct ConditionComparisonViewModel {
    left_comparand: Box<dyn ConditionComparandViewModel>,
    right_comparand: Box<dyn ConditionComparandViewModel>,
    operation: Box<dyn ConditionOperation>,
}

impl ConditionComparisonViewModel {
    pub fn new(
        left: Box<dyn ConditionComparandViewModel>,
        right: Box<dyn ConditionComparandViewModel>,
        operation: Box<dyn ConditionOperation>,
    ) -> Self {
        Self {
            left_comparand: left,
            right_comparand: right,
            operation,
        }
    }
}

impl ConditionComparison for ConditionComparisonViewModel {
    fn operation(&self) -> Option<&dyn ConditionOperation> {
        Some(self.operation.as_ref())
    }

    fn compare(
        &self,
        machine: &StateMachineInstance,
        _layer: Option<NonNull<StateMachineLayerInstance>>,
    ) -> bool {
        self.compare_pointers(
            self.left_comparand.value(machine),
            self.right_comparand.value(machine),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentComparandKind {
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

fn describe_component_side(
    comparator: &TransitionPropertyComponentComparator,
) -> Option<ComponentComparandKind> {
    let field_id = CoreRegistry::property_field_id(comparator.base.property_key() as i32);
    if field_id < 0 {
        return None;
    }
    let property_key = comparator.base.property_key();
    match field_id as u16 {
        CoreRegistry::CORE_DOUBLE_TYPE_ID => Some(ComponentComparandKind::NumberDouble),
        CoreRegistry::CORE_BOOL_TYPE_ID => Some(ComponentComparandKind::Boolean),
        CoreRegistry::CORE_STRING_TYPE_ID => Some(ComponentComparandKind::String),
        CoreRegistry::CORE_COLOR_TYPE_ID => Some(ComponentComparandKind::Color),
        CoreRegistry::CORE_UINT_TYPE_ID => {
            if property_key == CoreRegistry::CUSTOM_PROPERTY_ENUM_VALUE_KEY
                || property_key == CoreRegistry::VIEW_MODEL_ENUM_VALUE_KEY
            {
                Some(ComponentComparandKind::Enum)
            } else if property_key == CoreRegistry::CUSTOM_PROPERTY_TRIGGER_VALUE_KEY
                || property_key == CoreRegistry::VIEW_MODEL_TRIGGER_VALUE_KEY
            {
                Some(ComponentComparandKind::Trigger)
            } else if property_key == CoreRegistry::VIEW_MODEL_ASSET_VALUE_KEY {
                Some(ComponentComparandKind::Asset)
            } else if property_key == CoreRegistry::VIEW_MODEL_ARTBOARD_VALUE_KEY {
                Some(ComponentComparandKind::Artboard)
            } else if property_key == CoreRegistry::VIEW_MODEL_VIEW_MODEL_VALUE_KEY {
                Some(ComponentComparandKind::ViewModel)
            } else {
                Some(ComponentComparandKind::NumberFromUint)
            }
        }
        _ => None,
    }
}

fn is_number_kind(kind: ComponentComparandKind) -> bool {
    matches!(
        kind,
        ComponentComparandKind::NumberDouble | ComponentComparandKind::NumberFromUint
    )
}

fn component_kinds_compatible(left: ComponentComparandKind, right: ComponentComparandKind) -> bool {
    if is_number_kind(left) && is_number_kind(right) {
        return true;
    }
    left == right
}

fn make_component_number_comparand(
    comparator: NonNull<TransitionPropertyComponentComparator>,
    kind: ComponentComparandKind,
) -> Box<dyn ConditionComparandNumber> {
    if kind == ComponentComparandKind::NumberDouble {
        return Box::new(ConditionComparandComponentCoreNumber::new(comparator));
    }
    Box::new(ConditionComparandComponentCoreUintAsNumber::new(comparator))
}

fn describe_view_model_bindable_kind(
    bindable: NonNull<BindableProperty>,
) -> Option<ComponentComparandKind> {
    let core_type = unsafe { bindable.as_ref() }.core_type();
    if core_type == BindablePropertyNumber::TYPE_KEY {
        Some(ComponentComparandKind::NumberDouble)
    } else if core_type == BindablePropertyInteger::TYPE_KEY {
        Some(ComponentComparandKind::NumberFromUint)
    } else if core_type == BindablePropertyBoolean::TYPE_KEY {
        Some(ComponentComparandKind::Boolean)
    } else if core_type == BindablePropertyString::TYPE_KEY {
        Some(ComponentComparandKind::String)
    } else if core_type == BindablePropertyColor::TYPE_KEY {
        Some(ComponentComparandKind::Color)
    } else if core_type == BindablePropertyEnum::TYPE_KEY {
        Some(ComponentComparandKind::Enum)
    } else if core_type == BindablePropertyTrigger::TYPE_KEY {
        Some(ComponentComparandKind::Trigger)
    } else if core_type == BindablePropertyAsset::TYPE_KEY {
        Some(ComponentComparandKind::Asset)
    } else if core_type == BindablePropertyArtboard::TYPE_KEY {
        Some(ComponentComparandKind::Artboard)
    } else if core_type == BindablePropertyViewModel::TYPE_KEY {
        Some(ComponentComparandKind::ViewModel)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparatorSide {
    Left,
    Right,
}

// Artboard properties and literal values only participate on the left / right
// respectively, matching the pinned initialize() support.
fn append_comparable_kinds(
    comparator: NonNull<TransitionComparator>,
    side: ComparatorSide,
    out: &mut Vec<ComponentComparandKind>,
) {
    let comparator_ref = unsafe { comparator.as_ref() };
    if comparator_ref.is_transition_property_artboard_comparator() {
        if side == ComparatorSide::Left {
            out.push(ComponentComparandKind::NumberDouble);
        }
        return;
    }
    if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
        if let Some(kind) = describe_component_side(unsafe { component.as_ref() }) {
            out.push(kind);
        }
        return;
    }
    if let Some(view_model) = comparator_ref.as_transition_property_viewmodel_comparator() {
        if let Some(bindable) = unsafe { view_model.as_ref() }.bindable_property() {
            if let Some(kind) = describe_view_model_bindable_kind(bindable) {
                out.push(kind);
            }
        }
        return;
    }
    if side != ComparatorSide::Right {
        return;
    }
    if comparator_ref.is_transition_value_number_comparator() {
        out.push(ComponentComparandKind::NumberDouble);
    } else if comparator_ref.is_transition_value_boolean_comparator() {
        out.push(ComponentComparandKind::Boolean);
    } else if comparator_ref.is_transition_value_string_comparator() {
        out.push(ComponentComparandKind::String);
    } else if comparator_ref.is_transition_value_color_comparator() {
        out.push(ComponentComparandKind::Color);
    } else if comparator_ref.is_transition_value_enum_comparator() {
        out.push(ComponentComparandKind::Enum);
    } else if comparator_ref.is_transition_value_asset_comparator() {
        out.push(ComponentComparandKind::Asset);
    } else if comparator_ref.is_transition_value_artboard_comparator() {
        out.push(ComponentComparandKind::Artboard);
    } else if comparator_ref.is_transition_value_trigger_comparator() {
        // Uint32 comparison against component triggers; VM trigger plus value
        // trigger is handled as ConditionComparisonSelf before intersection.
        out.push(ComponentComparandKind::Trigger);
    }
}

fn intersect_compatible_kinds(
    left_kinds: &[ComponentComparandKind],
    right_kinds: &[ComponentComparandKind],
) -> Option<(ComponentComparandKind, ComponentComparandKind)> {
    for &left in left_kinds {
        for &right in right_kinds {
            if component_kinds_compatible(left, right) {
                return Some((left, right));
            }
        }
    }
    None
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

fn resolve_comparison_shape(
    left: ComponentComparandKind,
    right: ComponentComparandKind,
) -> Option<ComparisonShape> {
    if !component_kinds_compatible(left, right) {
        return None;
    }
    if is_number_kind(left) && is_number_kind(right) {
        if left == ComponentComparandKind::NumberFromUint
            && right == ComponentComparandKind::NumberFromUint
        {
            return Some(ComparisonShape::Uint32);
        }
        return Some(ComparisonShape::Number);
    }
    if left != right {
        return None;
    }
    match left {
        ComponentComparandKind::Boolean => Some(ComparisonShape::Boolean),
        ComponentComparandKind::String => Some(ComparisonShape::String),
        ComponentComparandKind::Color => Some(ComparisonShape::Color),
        ComponentComparandKind::Enum => Some(ComparisonShape::Enum),
        ComponentComparandKind::Trigger
        | ComponentComparandKind::Asset
        | ComponentComparandKind::Artboard => Some(ComparisonShape::Uint32),
        ComponentComparandKind::ViewModel => Some(ComparisonShape::ViewModel),
        _ => None,
    }
}

#[derive(Default)]
struct ComparandSlot {
    number: Option<Box<dyn ConditionComparandNumber>>,
    boolean: Option<Box<dyn ConditionComparandBoolean>>,
    string: Option<Box<dyn ConditionComparandString>>,
    color: Option<Box<dyn ConditionComparandColor>>,
    uint32: Option<Box<dyn ConditionComparandUint32>>,
    view_model: Option<Box<dyn ConditionComparandViewModel>>,
}

fn clear_comparand_slot(slot: &mut ComparandSlot) {
    slot.number = None;
    slot.boolean = None;
    slot.string = None;
    slot.color = None;
    slot.uint32 = None;
    slot.view_model = None;
}

fn make_comparand(
    comparator: NonNull<TransitionComparator>,
    kind: ComponentComparandKind,
    shape: ComparisonShape,
    slot: &mut ComparandSlot,
) -> bool {
    let comparator_ref = unsafe { comparator.as_ref() };
    match shape {
        ComparisonShape::Number => {
            if let Some(property) = comparator_ref.as_transition_property_artboard_comparator() {
                slot.number = Some(Box::new(ConditionComparandArtboardProperty::new(property)));
                return true;
            }
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let bindable_ref = unsafe { bindable.as_ref() };
                if let Some(number) = bindable_ref.as_number() {
                    slot.number = Some(Box::new(ConditionComparandNumberBindable::new(number)));
                    return true;
                }
                if let Some(integer) = bindable_ref.as_integer() {
                    slot.number = Some(Box::new(ConditionComparandNumberBindableInteger::new(
                        integer,
                    )));
                    return true;
                }
                return false;
            }
            if let Some(value) = comparator_ref.as_transition_value_number_comparator() {
                slot.number = Some(Box::new(ConditionComparandNumberValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                if kind == ComponentComparandKind::NumberDouble {
                    slot.number = Some(Box::new(ConditionComparandComponentCoreNumber::new(
                        component,
                    )));
                    return true;
                }
                if kind == ComponentComparandKind::NumberFromUint {
                    slot.number = Some(make_component_number_comparand(component, kind));
                    return true;
                }
                return false;
            }
            false
        }
        ComparisonShape::Boolean => {
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let Some(boolean) = unsafe { bindable.as_ref() }.as_boolean() else {
                    return false;
                };
                slot.boolean = Some(Box::new(ConditionComparandBooleanBindable::new(boolean)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_boolean_comparator() {
                slot.boolean = Some(Box::new(ConditionComparandBooleanValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                slot.boolean = Some(Box::new(ConditionComparandComponentCoreBoolean::new(
                    component,
                )));
                return true;
            }
            false
        }
        ComparisonShape::String => {
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let Some(string) = unsafe { bindable.as_ref() }.as_string() else {
                    return false;
                };
                slot.string = Some(Box::new(ConditionComparandStringBindable::new(string)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_string_comparator() {
                slot.string = Some(Box::new(ConditionComparandStringValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                slot.string = Some(Box::new(ConditionComparandComponentCoreString::new(
                    component,
                )));
                return true;
            }
            false
        }
        ComparisonShape::Color => {
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let Some(color) = unsafe { bindable.as_ref() }.as_color() else {
                    return false;
                };
                slot.color = Some(Box::new(ConditionComparandColorBindable::new(color)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_color_comparator() {
                slot.color = Some(Box::new(ConditionComparandColorValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                slot.color = Some(Box::new(ConditionComparandComponentCoreColor::new(
                    component,
                )));
                return true;
            }
            false
        }
        ComparisonShape::Enum => {
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let Some(enumeration) = unsafe { bindable.as_ref() }.as_enum() else {
                    return false;
                };
                slot.uint32 = Some(Box::new(ConditionComparandEnumBindable::new(enumeration)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_enum_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandEnumValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandComponentCoreUint::new(
                    component,
                )));
                return true;
            }
            false
        }
        ComparisonShape::Uint32 => {
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let bindable_ref = unsafe { bindable.as_ref() };
                if kind == ComponentComparandKind::NumberFromUint {
                    if let Some(integer) = bindable_ref.as_integer() {
                        slot.uint32 =
                            Some(Box::new(ConditionComparandIntegerBindable::new(integer)));
                        return true;
                    }
                }
                if kind == ComponentComparandKind::Trigger {
                    if let Some(trigger) = bindable_ref.as_trigger() {
                        slot.uint32 =
                            Some(Box::new(ConditionComparandTriggerBindable::new(trigger)));
                        return true;
                    }
                }
                if kind == ComponentComparandKind::Asset {
                    if let Some(asset) = bindable_ref.as_asset() {
                        slot.uint32 = Some(Box::new(ConditionComparandAssetBindable::new(asset)));
                        return true;
                    }
                }
                if kind == ComponentComparandKind::Artboard {
                    if let Some(artboard) = bindable_ref.as_artboard() {
                        slot.uint32 =
                            Some(Box::new(ConditionComparandArtboardBindable::new(artboard)));
                        return true;
                    }
                }
                return false;
            }
            if let Some(value) = comparator_ref.as_transition_value_trigger_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandTriggerValue::new(value)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_asset_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandAssetValue::new(value)));
                return true;
            }
            if let Some(value) = comparator_ref.as_transition_value_artboard_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandArtboardValue::new(value)));
                return true;
            }
            if let Some(component) = comparator_ref.as_transition_property_component_comparator() {
                slot.uint32 = Some(Box::new(ConditionComparandComponentCoreUint::new(
                    component,
                )));
                return true;
            }
            false
        }
        ComparisonShape::ViewModel => {
            if kind != ComponentComparandKind::ViewModel {
                return false;
            }
            if let Some(property) = comparator_ref.as_transition_property_viewmodel_comparator() {
                let Some(bindable) = unsafe { property.as_ref() }.bindable_property() else {
                    return false;
                };
                let Some(view_model) = unsafe { bindable.as_ref() }.as_view_model() else {
                    return false;
                };
                slot.view_model = Some(Box::new(ConditionComparandViewModelBindable::new(
                    view_model,
                )));
                return true;
            }
            false
        }
    }
}

fn wrap_comparison(
    shape: ComparisonShape,
    left_slot: &mut ComparandSlot,
    right_slot: &mut ComparandSlot,
    operation: Box<dyn ConditionOperation>,
) -> Option<Box<dyn ConditionComparison>> {
    match shape {
        ComparisonShape::Number => Some(Box::new(ConditionComparisonNumber::new(
            left_slot.number.take()?,
            right_slot.number.take()?,
            operation,
        ))),
        ComparisonShape::Boolean => Some(Box::new(ConditionComparisonBoolean::new(
            left_slot.boolean.take()?,
            right_slot.boolean.take()?,
            operation,
        ))),
        ComparisonShape::String => Some(Box::new(ConditionComparisonString::new(
            left_slot.string.take()?,
            right_slot.string.take()?,
            operation,
        ))),
        ComparisonShape::Color => Some(Box::new(ConditionComparisonColor::new(
            left_slot.color.take()?,
            right_slot.color.take()?,
            operation,
        ))),
        ComparisonShape::Enum => Some(Box::new(ConditionComparisonEnum::new(
            left_slot.uint32.take()?,
            right_slot.uint32.take()?,
            operation,
        ))),
        ComparisonShape::Uint32 => Some(Box::new(ConditionComparisonUint32::new(
            left_slot.uint32.take()?,
            right_slot.uint32.take()?,
            operation,
        ))),
        ComparisonShape::ViewModel => Some(Box::new(ConditionComparisonViewModel::new(
            left_slot.view_model.take()?,
            right_slot.view_model.take()?,
            operation,
        ))),
    }
}

fn build_comparands_from_intersect(
    left: NonNull<TransitionComparator>,
    right: NonNull<TransitionComparator>,
    left_kind: ComponentComparandKind,
    right_kind: ComponentComparandKind,
    operation: Box<dyn ConditionOperation>,
) -> Option<Box<dyn ConditionComparison>> {
    let shape = resolve_comparison_shape(left_kind, right_kind)?;
    let mut left_slot = ComparandSlot::default();
    let mut right_slot = ComparandSlot::default();
    if !make_comparand(left, left_kind, shape, &mut left_slot)
        || !make_comparand(right, right_kind, shape, &mut right_slot)
    {
        clear_comparand_slot(&mut left_slot);
        clear_comparand_slot(&mut right_slot);
        return None;
    }
    let wrapped = wrap_comparison(shape, &mut left_slot, &mut right_slot, operation);
    if wrapped.is_none() {
        clear_comparand_slot(&mut left_slot);
        clear_comparand_slot(&mut right_slot);
    }
    wrapped
}

pub struct TransitionViewModelCondition {
    pub base: TransitionViewModelConditionBase,
    left_comparator: Option<Box<TransitionComparator>>,
    right_comparator: Option<Box<TransitionComparator>>,
    comparison: Option<Box<dyn ConditionComparison>>,
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

impl Drop for TransitionViewModelCondition {
    fn drop(&mut self) {
        self.left_comparator = None;
        self.right_comparator = None;
        self.comparison = None;
    }
}

impl TransitionViewModelCondition {
    pub fn left_comparator(&self) -> Option<&TransitionComparator> {
        self.left_comparator.as_deref()
    }

    pub fn right_comparator(&self) -> Option<&TransitionComparator> {
        self.right_comparator.as_deref()
    }

    pub fn comparator(&mut self, value: Box<TransitionComparator>) {
        if self.left_comparator.is_none() {
            self.left_comparator = Some(value);
        } else {
            self.right_comparator = Some(value);
        }
    }

    pub fn op(&self) -> TransitionConditionOp {
        TransitionConditionOp::from_u32(self.base.op_value())
    }

    pub fn operation(op: TransitionConditionOp) -> Box<dyn ConditionOperation> {
        match op {
            TransitionConditionOp::Equal => Box::new(ConditionOperationEqual),
            TransitionConditionOp::NotEqual => Box::new(ConditionOperationNotEqual),
            TransitionConditionOp::LessThanOrEqual => Box::new(ConditionOperationLessThanOrEqual),
            TransitionConditionOp::LessThan => Box::new(ConditionOperationLessThan),
            TransitionConditionOp::GreaterThanOrEqual => {
                Box::new(ConditionOperationGreaterThanOrEqual)
            }
            TransitionConditionOp::GreaterThan => Box::new(ConditionOperationGreaterThan),
            _ => Box::new(ConditionOperationDefault),
        }
    }

    fn can_evaluate(&self, machine: &StateMachineInstance) -> bool {
        let (Some(left), Some(right)) = (self.left_comparator(), self.right_comparator()) else {
            return false;
        };
        if machine.data_context().is_none()
            && (right.is_transition_property_viewmodel_comparator()
                || left.is_transition_property_viewmodel_comparator())
        {
            return false;
        }
        true
    }

    pub fn evaluate(
        &self,
        machine: &StateMachineInstance,
        layer: Option<NonNull<StateMachineLayerInstance>>,
    ) -> bool {
        if self.can_evaluate(machine) {
            if let Some(comparison) = self.comparison.as_ref() {
                return comparison.compare(machine, layer);
            }
        }
        false
    }

    pub fn use_in_layer(
        &self,
        machine: &StateMachineInstance,
        layer: Option<NonNull<StateMachineLayerInstance>>,
    ) {
        if let Some(left) = self.left_comparator() {
            left.use_in_layer(machine, layer);
        }
    }

    pub fn initialize(&mut self) {
        let Some(left) = self.left_comparator.as_deref_mut().map(NonNull::from) else {
            return;
        };
        let Some(right) = self.right_comparator.as_deref_mut().map(NonNull::from) else {
            return;
        };

        // Asymmetric: Self compares the left bindable against its data-bind
        // source, not a typed-comparand intersection.
        if unsafe { right.as_ref() }.is_transition_self_comparator() {
            if let Some(left_property) =
                unsafe { left.as_ref() }.as_transition_property_viewmodel_comparator()
            {
                if let Some(left_bindable) = unsafe { left_property.as_ref() }.bindable_property() {
                    self.comparison = Some(Box::new(ConditionComparisonSelf::new(left_bindable)));
                    return;
                }
            }
            self.comparison = Some(Box::new(ConditionComparisonNone));
            return;
        }

        // Asymmetric: a value trigger on the right uses Self with a VM trigger
        // on the left, rather than uint32 versus a value comparand.
        if let Some(left_property) =
            unsafe { left.as_ref() }.as_transition_property_viewmodel_comparator()
        {
            if unsafe { right.as_ref() }.is_transition_value_trigger_comparator() {
                if let Some(left_bindable) = unsafe { left_property.as_ref() }.bindable_property() {
                    if unsafe { left_bindable.as_ref() }.is_trigger() {
                        self.comparison =
                            Some(Box::new(ConditionComparisonSelf::new(left_bindable)));
                        return;
                    }
                }
            }
        }

        let mut left_kinds = Vec::new();
        let mut right_kinds = Vec::new();
        append_comparable_kinds(left, ComparatorSide::Left, &mut left_kinds);
        append_comparable_kinds(right, ComparatorSide::Right, &mut right_kinds);

        let Some((left_kind, right_kind)) = intersect_compatible_kinds(&left_kinds, &right_kinds)
        else {
            self.comparison = Some(Box::new(ConditionComparisonNone));
            return;
        };

        let operation = Self::operation(self.op());
        if let Some(comparison) =
            build_comparands_from_intersect(left, right, left_kind, right_kind, operation)
        {
            self.comparison = Some(comparison);
            return;
        }
        self.comparison = Some(Box::new(ConditionComparisonNone));
    }
}
