use super::{
    StateMachineBindableArtboardInstance, StateMachineBindableAssetInstance,
    StateMachineBindableBooleanInstance, StateMachineBindableColorInstance,
    StateMachineBindableEnumInstance, StateMachineBindableIntegerInstance,
    StateMachineBindableNumberInstance, StateMachineBindableStringInstance,
    StateMachineBindableTriggerInstance, StateMachineBindableViewModelInstance,
    StateMachineInputInstance, StateMachineViewModelTriggerInstance, bindable_artboard_value,
    bindable_asset_value, bindable_boolean_value, bindable_color_value, bindable_enum_value,
    bindable_integer_value, bindable_number_value, bindable_string_value,
    bindable_trigger_source_global_id, bindable_trigger_value, bindable_view_model_value,
};
use crate::ArtboardInstance;
use crate::components::TransformProperty;
use crate::properties::{
    property_key_for_name, runtime_object_bool_property_by_key,
    runtime_object_color_property_by_key, runtime_object_double_property_by_key,
    runtime_object_string_property_by_key, runtime_object_uint_property_by_key,
    transform_property_for_key,
};
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::{NoopScriptHost, ScriptMethod, ScriptValue};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{
    CoreRegistryFieldKind, core_registry_field_kind_by_property_key, object_supports_property,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) enum RuntimeTransitionCondition {
    Scripted {
        global_id: u32,
    },
    Bool {
        input_index: usize,
        op: TransitionConditionOp,
    },
    Number {
        input_index: usize,
        op: TransitionConditionOp,
        value: f32,
    },
    Trigger {
        input_index: usize,
    },
    ViewModelNumber {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: f32,
    },
    ViewModelIntegerNumber {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: f32,
    },
    ViewModelBoolean {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: bool,
    },
    ViewModelColor {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: u32,
    },
    ViewModelString {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: Vec<u8>,
    },
    ViewModelEnum {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: u64,
    },
    ViewModelAsset {
        bindable_global_id: u32,
        op: TransitionConditionOp,
        value: u64,
    },
    ViewModelNumberPair {
        left: RuntimeViewModelNumberValue,
        right: RuntimeViewModelNumberValue,
        op: TransitionConditionOp,
    },
    ViewModelBooleanPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelColorPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelStringPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelEnumPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelAssetPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelArtboardPair {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ViewModelTrigger {
        bindable_global_id: u32,
    },
    ViewModelPointer {
        left_bindable_global_id: u32,
        right_bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentNumber {
        component: RuntimeComponentNumberValue,
        op: TransitionConditionOp,
        value: f32,
    },
    ComponentNumberPair {
        left: RuntimeComponentNumberValue,
        right: RuntimeComponentNumberValue,
        op: TransitionConditionOp,
    },
    ComponentBoolean {
        component: RuntimeComponentBoolValue,
        op: TransitionConditionOp,
        value: bool,
    },
    ComponentBooleanPair {
        left: RuntimeComponentBoolValue,
        right: RuntimeComponentBoolValue,
        op: TransitionConditionOp,
    },
    ComponentString {
        component: RuntimeComponentStringValue,
        op: TransitionConditionOp,
        value: Vec<u8>,
    },
    ComponentStringPair {
        left: RuntimeComponentStringValue,
        right: RuntimeComponentStringValue,
        op: TransitionConditionOp,
    },
    ComponentColor {
        component: RuntimeComponentColorValue,
        op: TransitionConditionOp,
        value: u32,
    },
    ComponentColorPair {
        left: RuntimeComponentColorValue,
        right: RuntimeComponentColorValue,
        op: TransitionConditionOp,
    },
    ComponentUint {
        component: RuntimeComponentUintValue,
        op: TransitionConditionOp,
        value: u64,
    },
    ComponentUintPair {
        left: RuntimeComponentUintValue,
        right: RuntimeComponentUintValue,
        op: TransitionConditionOp,
    },
    ComponentViewModelNumber {
        component: RuntimeComponentNumberValue,
        view_model: RuntimeViewModelNumberValue,
        component_on_left: bool,
        op: TransitionConditionOp,
    },
    ComponentViewModelInteger {
        component: RuntimeComponentUintValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelBoolean {
        component: RuntimeComponentBoolValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelString {
        component: RuntimeComponentStringValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelColor {
        component: RuntimeComponentColorValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelEnum {
        component: RuntimeComponentUintValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelAsset {
        component: RuntimeComponentUintValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelTrigger {
        component: RuntimeComponentUintValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ComponentViewModelArtboard {
        component: RuntimeComponentUintValue,
        bindable_global_id: u32,
        op: TransitionConditionOp,
    },
    ArtboardComponentNumber {
        property_type: u64,
        op: TransitionConditionOp,
        component: RuntimeComponentNumberValue,
    },
    ArtboardNumber {
        property_type: u64,
        op: TransitionConditionOp,
        threshold: f32,
    },
}

impl RuntimeTransitionCondition {
    pub(super) fn from_object(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        object: &RuntimeObject,
    ) -> Option<Self> {
        match object.type_name {
            "ScriptedTransitionCondition" => Some(Self::Scripted {
                global_id: object.id,
            }),
            "TransitionBoolCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Bool {
                    input_index,
                    op: TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    ),
                })
            }
            "TransitionNumberCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Number {
                    input_index,
                    op: TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    ),
                    value: object.double_property("value").unwrap_or(0.0),
                })
            }
            "TransitionTriggerCondition" => {
                let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
                Some(Self::Trigger { input_index })
            }
            "TransitionViewModelCondition" | "TransitionArtboardCondition" => {
                let comparators = file.transition_view_model_condition_comparators(object)?;
                let left = comparators.left?;
                let right = comparators.right?;
                if left.type_name == "TransitionPropertyArtboardComparator"
                    && right.type_name == "TransitionValueNumberComparator"
                {
                    return Some(Self::ArtboardNumber {
                        property_type: left.uint_property("propertyType").unwrap_or(0),
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        threshold: right.double_property("value").unwrap_or(0.0),
                    });
                }
                if left.type_name == "TransitionPropertyArtboardComparator"
                    && right.type_name == "TransitionPropertyComponentComparator"
                {
                    return Self::from_artboard_component(file, graph, object, left, right);
                }
                if left.type_name == "TransitionPropertyComponentComparator"
                    && right.type_name == "TransitionPropertyComponentComparator"
                {
                    return Self::from_component_pair(file, graph, object, left, right);
                }
                if left.type_name == "TransitionPropertyComponentComparator"
                    && right.type_name == "TransitionPropertyViewModelComparator"
                {
                    return Self::from_component_viewmodel(file, graph, object, left, right, true);
                }
                if left.type_name == "TransitionPropertyViewModelComparator"
                    && right.type_name == "TransitionPropertyComponentComparator"
                {
                    return Self::from_component_viewmodel(file, graph, object, right, left, false);
                }
                if left.type_name == "TransitionPropertyComponentComparator" {
                    return Self::from_component_literal(file, graph, object, left, right);
                }
                if left.type_name != "TransitionPropertyViewModelComparator" {
                    return None;
                }
                if right.type_name == "TransitionValueTriggerComparator"
                    || right.type_name == "TransitionSelfComparator"
                {
                    let bindable = file.latest_bindable_property_for_object(left)?;
                    if bindable.type_name == "BindablePropertyTrigger" {
                        return Some(Self::ViewModelTrigger {
                            bindable_global_id: bindable.id,
                        });
                    }
                    return None;
                }
                if right.type_name == "TransitionPropertyViewModelComparator" {
                    let left_bindable = file.latest_bindable_property_for_object(left)?;
                    let right_bindable = file.latest_bindable_property_for_object(right)?;
                    let op = TransitionConditionOp::from_value(
                        object.uint_property("opValue").unwrap_or(0),
                    );
                    if left_bindable.type_name == "BindablePropertyNumber"
                        || left_bindable.type_name == "BindablePropertyInteger"
                    {
                        return Some(Self::ViewModelNumberPair {
                            left: RuntimeViewModelNumberValue::from_bindable(left_bindable)?,
                            right: RuntimeViewModelNumberValue::from_bindable(right_bindable)?,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyBoolean"
                        && right_bindable.type_name == "BindablePropertyBoolean"
                    {
                        return Some(Self::ViewModelBooleanPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyColor"
                        && right_bindable.type_name == "BindablePropertyColor"
                    {
                        return Some(Self::ViewModelColorPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyString"
                        && right_bindable.type_name == "BindablePropertyString"
                    {
                        return Some(Self::ViewModelStringPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyEnum"
                        && right_bindable.type_name == "BindablePropertyEnum"
                    {
                        return Some(Self::ViewModelEnumPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyAsset"
                        && right_bindable.type_name == "BindablePropertyAsset"
                    {
                        return Some(Self::ViewModelAssetPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyArtboard"
                        && right_bindable.type_name == "BindablePropertyArtboard"
                    {
                        return Some(Self::ViewModelArtboardPair {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    if left_bindable.type_name == "BindablePropertyViewModel"
                        && right_bindable.type_name == "BindablePropertyViewModel"
                    {
                        return Some(Self::ViewModelPointer {
                            left_bindable_global_id: left_bindable.id,
                            right_bindable_global_id: right_bindable.id,
                            op,
                        });
                    }
                    return None;
                }
                if right.type_name != "TransitionValueNumberComparator"
                    && right.type_name != "TransitionValueBooleanComparator"
                    && right.type_name != "TransitionValueColorComparator"
                    && right.type_name != "TransitionValueStringComparator"
                    && right.type_name != "TransitionValueEnumComparator"
                    && right.type_name != "TransitionValueAssetComparator"
                {
                    return None;
                }
                let bindable = file.latest_bindable_property_for_object(left)?;
                if bindable.type_name == "BindablePropertyNumber"
                    && right.type_name == "TransitionValueNumberComparator"
                {
                    return Some(Self::ViewModelNumber {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.double_property("value").unwrap_or(0.0),
                    });
                }
                if bindable.type_name == "BindablePropertyInteger"
                    && right.type_name == "TransitionValueNumberComparator"
                {
                    return Some(Self::ViewModelIntegerNumber {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.double_property("value").unwrap_or(0.0),
                    });
                }
                if bindable.type_name == "BindablePropertyBoolean"
                    && right.type_name == "TransitionValueBooleanComparator"
                {
                    return Some(Self::ViewModelBoolean {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.bool_property("value").unwrap_or(false),
                    });
                }
                if bindable.type_name == "BindablePropertyColor"
                    && right.type_name == "TransitionValueColorComparator"
                {
                    return Some(Self::ViewModelColor {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.color_property("value").unwrap_or(0),
                    });
                }
                if bindable.type_name == "BindablePropertyString"
                    && right.type_name == "TransitionValueStringComparator"
                {
                    return Some(Self::ViewModelString {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right
                            .string_property_bytes("value")
                            .unwrap_or_default()
                            .to_vec(),
                    });
                }
                if bindable.type_name == "BindablePropertyEnum"
                    && right.type_name == "TransitionValueEnumComparator"
                {
                    return Some(Self::ViewModelEnum {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.uint_property("value").unwrap_or(u64::from(u32::MAX)),
                    });
                }
                if bindable.type_name == "BindablePropertyAsset"
                    && right.type_name == "TransitionValueAssetComparator"
                {
                    return Some(Self::ViewModelAsset {
                        bindable_global_id: bindable.id,
                        op: TransitionConditionOp::from_value(
                            object.uint_property("opValue").unwrap_or(0),
                        ),
                        value: right.uint_property("value").unwrap_or(u64::from(u32::MAX)),
                    });
                }
                None
            }
            _ => None,
        }
    }

    fn from_component_viewmodel(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        condition: &RuntimeObject,
        component: &RuntimeObject,
        viewmodel: &RuntimeObject,
        component_on_left: bool,
    ) -> Option<Self> {
        let local_id = usize::try_from(component.uint_property("objectId")?).ok()?;
        let property_key = u16::try_from(component.uint_property("propertyKey")?).ok()?;
        let component_kind = RuntimeComponentComparandKind::from_property_key(property_key)?;
        let bindable = file.latest_bindable_property_for_object(viewmodel)?;
        let viewmodel_kind = RuntimeComponentComparandKind::from_bindable(bindable)?;
        if !component_kind.is_compatible_with(viewmodel_kind) {
            return None;
        }

        let op = TransitionConditionOp::from_value(condition.uint_property("opValue").unwrap_or(0));
        let source_object = component_source_object(file, graph, local_id);
        let supports_property = component_supports_property(source_object, property_key);

        match (component_kind, viewmodel_kind) {
            (
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
            ) if component_kind == RuntimeComponentComparandKind::NumberFromUint
                && viewmodel_kind == RuntimeComponentComparandKind::NumberFromUint =>
            {
                Some(Self::ComponentViewModelInteger {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
            ) => Some(Self::ComponentViewModelNumber {
                component: RuntimeComponentNumberValue::from_parts(
                    local_id,
                    property_key,
                    component_kind,
                    source_object,
                    supports_property,
                )?,
                view_model: RuntimeViewModelNumberValue::from_bindable(bindable)?,
                component_on_left,
                op,
            }),
            (RuntimeComponentComparandKind::Boolean, RuntimeComponentComparandKind::Boolean) => {
                Some(Self::ComponentViewModelBoolean {
                    component: RuntimeComponentBoolValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::String, RuntimeComponentComparandKind::String) => {
                Some(Self::ComponentViewModelString {
                    component: RuntimeComponentStringValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::Color, RuntimeComponentComparandKind::Color) => {
                Some(Self::ComponentViewModelColor {
                    component: RuntimeComponentColorValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::Enum, RuntimeComponentComparandKind::Enum) => {
                Some(Self::ComponentViewModelEnum {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::Asset, RuntimeComponentComparandKind::Asset) => {
                Some(Self::ComponentViewModelAsset {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::Trigger, RuntimeComponentComparandKind::Trigger) => {
                Some(Self::ComponentViewModelTrigger {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            (RuntimeComponentComparandKind::Artboard, RuntimeComponentComparandKind::Artboard) => {
                Some(Self::ComponentViewModelArtboard {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    bindable_global_id: bindable.id,
                    op,
                })
            }
            _ => None,
        }
    }

    fn from_artboard_component(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        condition: &RuntimeObject,
        left: &RuntimeObject,
        right: &RuntimeObject,
    ) -> Option<Self> {
        let local_id = usize::try_from(right.uint_property("objectId")?).ok()?;
        let property_key = u16::try_from(right.uint_property("propertyKey")?).ok()?;
        let kind = RuntimeComponentComparandKind::from_property_key(property_key)?;
        if !kind.is_number() {
            return None;
        }

        let source_object = component_source_object(file, graph, local_id);
        let supports_property = component_supports_property(source_object, property_key);
        Some(Self::ArtboardComponentNumber {
            property_type: left.uint_property("propertyType").unwrap_or(0),
            op: TransitionConditionOp::from_value(condition.uint_property("opValue").unwrap_or(0)),
            component: RuntimeComponentNumberValue::from_parts(
                local_id,
                property_key,
                kind,
                source_object,
                supports_property,
            )?,
        })
    }

    fn from_component_literal(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        condition: &RuntimeObject,
        left: &RuntimeObject,
        right: &RuntimeObject,
    ) -> Option<Self> {
        let local_id = usize::try_from(left.uint_property("objectId")?).ok()?;
        let property_key = u16::try_from(left.uint_property("propertyKey")?).ok()?;
        let kind = RuntimeComponentComparandKind::from_property_key(property_key)?;
        let op = TransitionConditionOp::from_value(condition.uint_property("opValue").unwrap_or(0));
        let source_object = graph
            .local_objects
            .iter()
            .find(|local| local.local_id == local_id)
            .and_then(|local| file.object(local.global_id as usize));
        let supports_property = source_object
            .is_some_and(|object| object_supports_property(object.type_key, property_key));

        match (kind, right.type_name) {
            (RuntimeComponentComparandKind::NumberDouble, "TransitionValueNumberComparator") => {
                Some(Self::ComponentNumber {
                    component: RuntimeComponentNumberValue::from_parts(
                        local_id,
                        property_key,
                        kind,
                        source_object,
                        supports_property,
                    )?,
                    op,
                    value: right.double_property("value").unwrap_or(0.0),
                })
            }
            (RuntimeComponentComparandKind::NumberFromUint, "TransitionValueNumberComparator") => {
                Some(Self::ComponentNumber {
                    component: RuntimeComponentNumberValue::from_parts(
                        local_id,
                        property_key,
                        kind,
                        source_object,
                        supports_property,
                    )?,
                    op,
                    value: right.double_property("value").unwrap_or(0.0),
                })
            }
            (RuntimeComponentComparandKind::Boolean, "TransitionValueBooleanComparator") => {
                Some(Self::ComponentBoolean {
                    component: RuntimeComponentBoolValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right.bool_property("value").unwrap_or(false),
                })
            }
            (RuntimeComponentComparandKind::String, "TransitionValueStringComparator") => {
                Some(Self::ComponentString {
                    component: RuntimeComponentStringValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right
                        .string_property_bytes("value")
                        .unwrap_or_default()
                        .to_vec(),
                })
            }
            (RuntimeComponentComparandKind::Color, "TransitionValueColorComparator") => {
                Some(Self::ComponentColor {
                    component: RuntimeComponentColorValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right.color_property("value").unwrap_or(0),
                })
            }
            (RuntimeComponentComparandKind::Enum, "TransitionValueEnumComparator") => {
                Some(Self::ComponentUint {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right.uint_property("value").unwrap_or(u64::from(u32::MAX)),
                })
            }
            (RuntimeComponentComparandKind::Trigger, "TransitionValueTriggerComparator") => {
                Some(Self::ComponentUint {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right.uint_property("value").unwrap_or(0),
                })
            }
            (RuntimeComponentComparandKind::Asset, "TransitionValueAssetComparator")
            | (RuntimeComponentComparandKind::Artboard, "TransitionValueArtboardComparator") => {
                Some(Self::ComponentUint {
                    component: RuntimeComponentUintValue::from_parts(
                        local_id,
                        property_key,
                        source_object,
                        supports_property,
                    ),
                    op,
                    value: right.uint_property("value").unwrap_or(u64::from(u32::MAX)),
                })
            }
            _ => None,
        }
    }

    fn from_component_pair(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        condition: &RuntimeObject,
        left: &RuntimeObject,
        right: &RuntimeObject,
    ) -> Option<Self> {
        let left_local_id = usize::try_from(left.uint_property("objectId")?).ok()?;
        let right_local_id = usize::try_from(right.uint_property("objectId")?).ok()?;
        let left_property_key = u16::try_from(left.uint_property("propertyKey")?).ok()?;
        let right_property_key = u16::try_from(right.uint_property("propertyKey")?).ok()?;
        let left_kind = RuntimeComponentComparandKind::from_property_key(left_property_key)?;
        let right_kind = RuntimeComponentComparandKind::from_property_key(right_property_key)?;
        if !left_kind.is_compatible_with(right_kind) {
            return None;
        }

        let op = TransitionConditionOp::from_value(condition.uint_property("opValue").unwrap_or(0));
        let left_source = component_source_object(file, graph, left_local_id);
        let right_source = component_source_object(file, graph, right_local_id);
        let left_supports = component_supports_property(left_source, left_property_key);
        let right_supports = component_supports_property(right_source, right_property_key);

        match (left_kind, right_kind) {
            (
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
            ) if left_kind == RuntimeComponentComparandKind::NumberFromUint
                && right_kind == RuntimeComponentComparandKind::NumberFromUint =>
            {
                Some(Self::ComponentUintPair {
                    left: RuntimeComponentUintValue::from_parts(
                        left_local_id,
                        left_property_key,
                        left_source,
                        left_supports,
                    ),
                    right: RuntimeComponentUintValue::from_parts(
                        right_local_id,
                        right_property_key,
                        right_source,
                        right_supports,
                    ),
                    op,
                })
            }
            (
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
                RuntimeComponentComparandKind::NumberDouble
                | RuntimeComponentComparandKind::NumberFromUint,
            ) => Some(Self::ComponentNumberPair {
                left: RuntimeComponentNumberValue::from_parts(
                    left_local_id,
                    left_property_key,
                    left_kind,
                    left_source,
                    left_supports,
                )?,
                right: RuntimeComponentNumberValue::from_parts(
                    right_local_id,
                    right_property_key,
                    right_kind,
                    right_source,
                    right_supports,
                )?,
                op,
            }),
            (RuntimeComponentComparandKind::Boolean, RuntimeComponentComparandKind::Boolean) => {
                Some(Self::ComponentBooleanPair {
                    left: RuntimeComponentBoolValue::from_parts(
                        left_local_id,
                        left_property_key,
                        left_source,
                        left_supports,
                    ),
                    right: RuntimeComponentBoolValue::from_parts(
                        right_local_id,
                        right_property_key,
                        right_source,
                        right_supports,
                    ),
                    op,
                })
            }
            (RuntimeComponentComparandKind::String, RuntimeComponentComparandKind::String) => {
                Some(Self::ComponentStringPair {
                    left: RuntimeComponentStringValue::from_parts(
                        left_local_id,
                        left_property_key,
                        left_source,
                        left_supports,
                    ),
                    right: RuntimeComponentStringValue::from_parts(
                        right_local_id,
                        right_property_key,
                        right_source,
                        right_supports,
                    ),
                    op,
                })
            }
            (RuntimeComponentComparandKind::Color, RuntimeComponentComparandKind::Color) => {
                Some(Self::ComponentColorPair {
                    left: RuntimeComponentColorValue::from_parts(
                        left_local_id,
                        left_property_key,
                        left_source,
                        left_supports,
                    ),
                    right: RuntimeComponentColorValue::from_parts(
                        right_local_id,
                        right_property_key,
                        right_source,
                        right_supports,
                    ),
                    op,
                })
            }
            (RuntimeComponentComparandKind::Enum, RuntimeComponentComparandKind::Enum)
            | (RuntimeComponentComparandKind::Trigger, RuntimeComponentComparandKind::Trigger)
            | (RuntimeComponentComparandKind::Asset, RuntimeComponentComparandKind::Asset)
            | (RuntimeComponentComparandKind::Artboard, RuntimeComponentComparandKind::Artboard) => {
                Some(Self::ComponentUintPair {
                    left: RuntimeComponentUintValue::from_parts(
                        left_local_id,
                        left_property_key,
                        left_source,
                        left_supports,
                    ),
                    right: RuntimeComponentUintValue::from_parts(
                        right_local_id,
                        right_property_key,
                        right_source,
                        right_supports,
                    ),
                    op,
                })
            }
            _ => None,
        }
    }

    pub(super) fn evaluate(
        &self,
        scripted_instances: &BTreeMap<u32, RuntimeScriptInstanceHandle>,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
        bindable_colors: &[StateMachineBindableColorInstance],
        bindable_strings: &[StateMachineBindableStringInstance],
        bindable_enums: &[StateMachineBindableEnumInstance],
        bindable_assets: &[StateMachineBindableAssetInstance],
        bindable_artboards: &[StateMachineBindableArtboardInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        bindable_view_models: &[StateMachineBindableViewModelInstance],
        bindable_booleans: &[StateMachineBindableBooleanInstance],
        view_model_triggers: &[StateMachineViewModelTriggerInstance],
        data_context_present: bool,
        data_context_view_model_bound: bool,
        layer_index: usize,
    ) -> bool {
        match self {
            Self::Scripted { global_id } => {
                evaluate_scripted_condition(*global_id, scripted_instances)
            }
            Self::Bool { input_index, op } => {
                let Some(value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::bool_value)
                else {
                    return true;
                };
                (value && *op == TransitionConditionOp::Equal)
                    || (!value && *op == TransitionConditionOp::NotEqual)
            }
            Self::Number {
                input_index,
                op,
                value,
            } => {
                let Some(input_value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::number_value)
                else {
                    return true;
                };
                op.compare(input_value, *value)
            }
            Self::Trigger { input_index } => {
                let Some(input) = inputs.get(*input_index) else {
                    return true;
                };
                input
                    .trigger_is_fireable_for_layer(layer_index)
                    .unwrap_or(true)
            }
            Self::ViewModelNumber {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_number_value(bindable_numbers, *bindable_global_id).unwrap_or(0.0);
                op.compare(input_value, *value)
            }
            Self::ViewModelIntegerNumber {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value = bindable_integer_value(bindable_integers, *bindable_global_id)
                    .unwrap_or(0) as f32;
                op.compare(input_value, *value)
            }
            Self::ViewModelBoolean {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_boolean_value(bindable_booleans, *bindable_global_id).unwrap_or(false);
                op.compare_bool(input_value, *value)
            }
            Self::ViewModelColor {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_color_value(bindable_colors, *bindable_global_id).unwrap_or(0);
                op.compare_u32_equal_only(input_value, *value)
            }
            Self::ViewModelString {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_string_value(bindable_strings, *bindable_global_id).unwrap_or(&[]);
                op.compare_bytes_equal_only(input_value, value)
            }
            Self::ViewModelEnum {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_enum_value(bindable_enums, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(input_value, *value)
            }
            Self::ViewModelAsset {
                bindable_global_id,
                op,
                value,
            } => {
                if !data_context_present {
                    return false;
                }
                let input_value =
                    bindable_asset_value(bindable_assets, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(input_value, *value)
            }
            Self::ViewModelNumberPair { left, right, op } => {
                if !data_context_present {
                    return false;
                }
                op.compare(
                    left.value(bindable_numbers, bindable_integers),
                    right.value(bindable_numbers, bindable_integers),
                )
            }
            Self::ViewModelBooleanPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left = bindable_boolean_value(bindable_booleans, *left_bindable_global_id)
                    .unwrap_or(false);
                let right = bindable_boolean_value(bindable_booleans, *right_bindable_global_id)
                    .unwrap_or(false);
                op.compare_bool(left, right)
            }
            Self::ViewModelColorPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left =
                    bindable_color_value(bindable_colors, *left_bindable_global_id).unwrap_or(0);
                let right =
                    bindable_color_value(bindable_colors, *right_bindable_global_id).unwrap_or(0);
                op.compare_u32_equal_only(left, right)
            }
            Self::ViewModelStringPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left = bindable_string_value(bindable_strings, *left_bindable_global_id)
                    .unwrap_or(&[]);
                let right = bindable_string_value(bindable_strings, *right_bindable_global_id)
                    .unwrap_or(&[]);
                op.compare_bytes_equal_only(left, right)
            }
            Self::ViewModelEnumPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left =
                    bindable_enum_value(bindable_enums, *left_bindable_global_id).unwrap_or(0);
                let right =
                    bindable_enum_value(bindable_enums, *right_bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(left, right)
            }
            Self::ViewModelAssetPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left =
                    bindable_asset_value(bindable_assets, *left_bindable_global_id).unwrap_or(0);
                let right =
                    bindable_asset_value(bindable_assets, *right_bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(left, right)
            }
            Self::ViewModelArtboardPair {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left = bindable_artboard_value(bindable_artboards, *left_bindable_global_id)
                    .unwrap_or(0);
                let right = bindable_artboard_value(bindable_artboards, *right_bindable_global_id)
                    .unwrap_or(0);
                op.compare_u64_equal_only(left, right)
            }
            Self::ViewModelTrigger { bindable_global_id } => {
                if !data_context_present || !data_context_view_model_bound {
                    return false;
                }
                let Some(trigger_global_id) =
                    bindable_trigger_source_global_id(bindable_triggers, *bindable_global_id)
                else {
                    return false;
                };
                view_model_triggers
                    .iter()
                    .find(|trigger| trigger.global_id() == trigger_global_id)
                    .is_some_and(|trigger| trigger.is_fireable_for_layer(layer_index))
            }
            Self::ViewModelPointer {
                left_bindable_global_id,
                right_bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let left = bindable_view_model_value(
                    bindable_view_models,
                    *left_bindable_global_id,
                    data_context_present,
                );
                let right = bindable_view_model_value(
                    bindable_view_models,
                    *right_bindable_global_id,
                    data_context_present,
                );
                op.compare_bool(left == right, true)
            }
            Self::ComponentNumber {
                component,
                op,
                value,
            } => op.compare(component.value(artboard), *value),
            Self::ComponentNumberPair { left, right, op } => {
                op.compare(left.value(artboard), right.value(artboard))
            }
            Self::ComponentBoolean {
                component,
                op,
                value,
            } => op.compare_bool(component.value(artboard), *value),
            Self::ComponentBooleanPair { left, right, op } => {
                op.compare_bool(left.value(artboard), right.value(artboard))
            }
            Self::ComponentString {
                component,
                op,
                value,
            } => op.compare_bytes_equal_only(component.value(artboard), value),
            Self::ComponentStringPair { left, right, op } => {
                op.compare_bytes_equal_only(left.value(artboard), right.value(artboard))
            }
            Self::ComponentColor {
                component,
                op,
                value,
            } => op.compare_u32_equal_only(component.value(artboard), *value),
            Self::ComponentColorPair { left, right, op } => {
                op.compare_u32_equal_only(left.value(artboard), right.value(artboard))
            }
            Self::ComponentUint {
                component,
                op,
                value,
            } => op.compare_u64_equal_only(component.value(artboard), *value),
            Self::ComponentUintPair { left, right, op } => {
                op.compare_u64_equal_only(left.value(artboard), right.value(artboard))
            }
            Self::ComponentViewModelNumber {
                component,
                view_model,
                component_on_left,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let component_value = component.value(artboard);
                let view_model_value = view_model.value(bindable_numbers, bindable_integers);
                if *component_on_left {
                    op.compare(component_value, view_model_value)
                } else {
                    op.compare(view_model_value, component_value)
                }
            }
            Self::ComponentViewModelInteger {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_integer_value(bindable_integers, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelBoolean {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_boolean_value(bindable_booleans, *bindable_global_id).unwrap_or(false);
                op.compare_bool(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelString {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_string_value(bindable_strings, *bindable_global_id).unwrap_or(&[]);
                op.compare_bytes_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelColor {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_color_value(bindable_colors, *bindable_global_id).unwrap_or(0);
                op.compare_u32_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelEnum {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_enum_value(bindable_enums, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelAsset {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_asset_value(bindable_assets, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelTrigger {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_trigger_value(bindable_triggers, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(component.value(artboard), view_model_value)
            }
            Self::ComponentViewModelArtboard {
                component,
                bindable_global_id,
                op,
            } => {
                if !data_context_present {
                    return false;
                }
                let view_model_value =
                    bindable_artboard_value(bindable_artboards, *bindable_global_id).unwrap_or(0);
                op.compare_u64_equal_only(component.value(artboard), view_model_value)
            }
            Self::ArtboardComponentNumber {
                property_type,
                op,
                component,
            } => op.compare(
                artboard.artboard_property_value(*property_type),
                component.value(artboard),
            ),
            Self::ArtboardNumber {
                property_type,
                op,
                threshold,
            } => op.compare(artboard.artboard_property_value(*property_type), *threshold),
        }
    }

    pub(super) fn use_input(
        &self,
        inputs: &mut [StateMachineInputInstance],
        bindable_triggers: &[StateMachineBindableTriggerInstance],
        view_model_triggers: &mut [StateMachineViewModelTriggerInstance],
        layer_index: usize,
    ) {
        match self {
            Self::Trigger { input_index } => {
                if let Some(input) = inputs.get_mut(*input_index) {
                    input.use_trigger_in_layer(layer_index);
                }
            }
            Self::ViewModelTrigger { bindable_global_id } => {
                let Some(trigger_global_id) =
                    bindable_trigger_source_global_id(bindable_triggers, *bindable_global_id)
                else {
                    return;
                };
                if let Some(trigger) = view_model_triggers
                    .iter_mut()
                    .find(|trigger| trigger.global_id() == trigger_global_id)
                {
                    trigger.use_in_layer(layer_index);
                }
            }
            _ => {}
        }
    }
}

fn evaluate_scripted_condition(
    global_id: u32,
    scripted_instances: &BTreeMap<u32, RuntimeScriptInstanceHandle>,
) -> bool {
    scripted_instances
        .get(&global_id)
        .and_then(|instance| {
            instance
                .borrow_mut()
                .call_method(ScriptMethod::Evaluate, &[], &mut NoopScriptHost)
                .ok()
        })
        .is_some_and(|value| value == ScriptValue::Bool(true))
}

#[cfg(test)]
mod scripted_tests {
    use super::*;
    use crate::{ScriptError, ScriptHost, ScriptInstance};

    struct ConditionScript {
        result: Result<ScriptValue, ScriptError>,
    }

    impl ScriptInstance for ConditionScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Evaluate)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Evaluate);
            self.result.clone()
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn instances_with(
        result: Result<ScriptValue, ScriptError>,
    ) -> BTreeMap<u32, RuntimeScriptInstanceHandle> {
        BTreeMap::from([(
            7,
            RuntimeScriptInstanceHandle::new(Box::new(ConditionScript { result })),
        )])
    }

    #[test]
    fn scripted_transition_requires_an_exact_true_boolean() {
        assert!(evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Bool(true)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Bool(false)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Number(1.0)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Err(ScriptError::new("evaluate failed")))
        ));
        assert!(!evaluate_scripted_condition(7, &BTreeMap::new()));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeComponentComparandKind {
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

impl RuntimeComponentComparandKind {
    fn from_property_key(property_key: u16) -> Option<Self> {
        match core_registry_field_kind_by_property_key(property_key)? {
            CoreRegistryFieldKind::Double => Some(Self::NumberDouble),
            CoreRegistryFieldKind::Bool => Some(Self::Boolean),
            CoreRegistryFieldKind::StringOrBytes => Some(Self::String),
            CoreRegistryFieldKind::Color => Some(Self::Color),
            CoreRegistryFieldKind::Uint => {
                if component_property_key_matches(
                    property_key,
                    &[
                        ("CustomPropertyEnum", "propertyValue"),
                        ("ViewModelInstanceEnum", "propertyValue"),
                    ],
                ) {
                    return Some(Self::Enum);
                }
                if component_property_key_matches(
                    property_key,
                    &[
                        ("CustomPropertyTrigger", "propertyValue"),
                        ("ViewModelInstanceTrigger", "propertyValue"),
                    ],
                ) {
                    return Some(Self::Trigger);
                }
                if property_key_for_name("ViewModelInstanceAsset", "propertyValue")
                    == Some(property_key)
                {
                    return Some(Self::Asset);
                }
                if property_key_for_name("ViewModelInstanceArtboard", "propertyValue")
                    == Some(property_key)
                {
                    return Some(Self::Artboard);
                }
                if property_key_for_name("ViewModelInstanceViewModel", "propertyValue")
                    == Some(property_key)
                {
                    return Some(Self::ViewModel);
                }
                Some(Self::NumberFromUint)
            }
        }
    }

    fn from_bindable(bindable: &RuntimeObject) -> Option<Self> {
        match bindable.type_name {
            "BindablePropertyNumber" => Some(Self::NumberDouble),
            "BindablePropertyInteger" => Some(Self::NumberFromUint),
            "BindablePropertyBoolean" => Some(Self::Boolean),
            "BindablePropertyString" => Some(Self::String),
            "BindablePropertyColor" => Some(Self::Color),
            "BindablePropertyEnum" => Some(Self::Enum),
            "BindablePropertyTrigger" => Some(Self::Trigger),
            "BindablePropertyAsset" => Some(Self::Asset),
            "BindablePropertyArtboard" => Some(Self::Artboard),
            "BindablePropertyViewModel" => Some(Self::ViewModel),
            _ => None,
        }
    }

    fn is_number(self) -> bool {
        matches!(self, Self::NumberDouble | Self::NumberFromUint)
    }

    fn is_compatible_with(self, other: Self) -> bool {
        (self.is_number() && other.is_number()) || self == other
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeComponentNumberValue {
    local_id: usize,
    transform_property: Option<TransformProperty>,
    uint_property: Option<u16>,
    source_value: f32,
}

impl RuntimeComponentNumberValue {
    fn from_parts(
        local_id: usize,
        property_key: u16,
        kind: RuntimeComponentComparandKind,
        source_object: Option<&RuntimeObject>,
        supports_property: bool,
    ) -> Option<Self> {
        match kind {
            RuntimeComponentComparandKind::NumberDouble => Some(Self {
                local_id,
                transform_property: supports_property
                    .then(|| transform_property_for_key(property_key))
                    .flatten(),
                uint_property: None,
                source_value: source_object
                    .filter(|_| supports_property)
                    .and_then(|object| runtime_object_double_property_by_key(object, property_key))
                    .unwrap_or(0.0),
            }),
            RuntimeComponentComparandKind::NumberFromUint => Some(Self {
                local_id,
                transform_property: None,
                uint_property: supports_property.then_some(property_key),
                source_value: runtime_component_uint_value(
                    source_object,
                    property_key,
                    supports_property,
                ) as f32,
            }),
            _ => None,
        }
    }

    fn value(self, artboard: &ArtboardInstance) -> f32 {
        if let Some(value) = self
            .transform_property
            .and_then(|property| artboard.transform_property(self.local_id, property))
        {
            return value;
        }
        if let Some(value) = self
            .uint_property
            .and_then(|property_key| artboard.uint_property(self.local_id, property_key))
        {
            return value as f32;
        }
        self.source_value
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeComponentUintValue {
    local_id: usize,
    property_key: Option<u16>,
    source_value: u64,
}

impl RuntimeComponentUintValue {
    fn from_parts(
        local_id: usize,
        property_key: u16,
        source_object: Option<&RuntimeObject>,
        supports_property: bool,
    ) -> Self {
        Self {
            local_id,
            property_key: supports_property.then_some(property_key),
            source_value: runtime_component_uint_value(
                source_object,
                property_key,
                supports_property,
            ),
        }
    }

    fn value(self, artboard: &ArtboardInstance) -> u64 {
        self.property_key
            .and_then(|property_key| artboard.uint_property(self.local_id, property_key))
            .unwrap_or(self.source_value)
    }
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeComponentStringValue {
    local_id: usize,
    property_key: Option<u16>,
    source_value: Vec<u8>,
}

impl RuntimeComponentStringValue {
    fn from_parts(
        local_id: usize,
        property_key: u16,
        source_object: Option<&RuntimeObject>,
        supports_property: bool,
    ) -> Self {
        Self {
            local_id,
            property_key: supports_property.then_some(property_key),
            source_value: runtime_component_string_value(
                source_object,
                property_key,
                supports_property,
            ),
        }
    }

    fn value<'a>(&'a self, artboard: &'a ArtboardInstance) -> &'a [u8] {
        self.property_key
            .and_then(|property_key| artboard.string_property(self.local_id, property_key))
            .unwrap_or(&self.source_value)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeComponentBoolValue {
    local_id: usize,
    property_key: Option<u16>,
    source_value: bool,
}

impl RuntimeComponentBoolValue {
    fn from_parts(
        local_id: usize,
        property_key: u16,
        source_object: Option<&RuntimeObject>,
        supports_property: bool,
    ) -> Self {
        Self {
            local_id,
            property_key: supports_property.then_some(property_key),
            source_value: source_object
                .filter(|_| supports_property)
                .and_then(|object| runtime_object_bool_property_by_key(object, property_key))
                .unwrap_or(false),
        }
    }

    fn value(self, artboard: &ArtboardInstance) -> bool {
        self.property_key
            .and_then(|property_key| artboard.bool_property(self.local_id, property_key))
            .unwrap_or(self.source_value)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeComponentColorValue {
    local_id: usize,
    property_key: Option<u16>,
    source_value: u32,
}

impl RuntimeComponentColorValue {
    fn from_parts(
        local_id: usize,
        property_key: u16,
        source_object: Option<&RuntimeObject>,
        supports_property: bool,
    ) -> Self {
        Self {
            local_id,
            property_key: supports_property.then_some(property_key),
            source_value: source_object
                .filter(|_| supports_property)
                .and_then(|object| runtime_object_color_property_by_key(object, property_key))
                .unwrap_or(0),
        }
    }

    fn value(self, artboard: &ArtboardInstance) -> u32 {
        self.property_key
            .and_then(|property_key| artboard.color_property(self.local_id, property_key))
            .unwrap_or(self.source_value)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RuntimeViewModelNumberValue {
    Number { bindable_global_id: u32 },
    Integer { bindable_global_id: u32 },
}

impl RuntimeViewModelNumberValue {
    fn from_bindable(bindable: &RuntimeObject) -> Option<Self> {
        match bindable.type_name {
            "BindablePropertyNumber" => Some(Self::Number {
                bindable_global_id: bindable.id,
            }),
            "BindablePropertyInteger" => Some(Self::Integer {
                bindable_global_id: bindable.id,
            }),
            _ => None,
        }
    }

    fn value(
        self,
        bindable_numbers: &[StateMachineBindableNumberInstance],
        bindable_integers: &[StateMachineBindableIntegerInstance],
    ) -> f32 {
        match self {
            Self::Number { bindable_global_id } => {
                bindable_number_value(bindable_numbers, bindable_global_id).unwrap_or(0.0)
            }
            Self::Integer { bindable_global_id } => {
                bindable_integer_value(bindable_integers, bindable_global_id).unwrap_or(0) as f32
            }
        }
    }
}

fn component_source_object<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> Option<&'a RuntimeObject> {
    graph
        .local_objects
        .iter()
        .find(|local| local.local_id == local_id)
        .and_then(|local| file.object(local.global_id as usize))
}

fn component_supports_property(source_object: Option<&RuntimeObject>, property_key: u16) -> bool {
    source_object.is_some_and(|object| object_supports_property(object.type_key, property_key))
}

fn component_property_key_matches(property_key: u16, properties: &[(&str, &str)]) -> bool {
    properties.iter().any(|(type_name, property_name)| {
        property_key_for_name(type_name, property_name) == Some(property_key)
    })
}

fn runtime_component_uint_value(
    source_object: Option<&RuntimeObject>,
    property_key: u16,
    supports_property: bool,
) -> u64 {
    source_object
        .filter(|_| supports_property)
        .and_then(|object| runtime_object_uint_property_by_key(object, property_key))
        .unwrap_or(0)
}

fn runtime_component_string_value(
    source_object: Option<&RuntimeObject>,
    property_key: u16,
    supports_property: bool,
) -> Vec<u8> {
    source_object
        .filter(|_| supports_property)
        .and_then(|object| runtime_object_string_property_by_key(object, property_key))
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransitionConditionOp {
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
}

impl TransitionConditionOp {
    fn from_value(value: u64) -> Self {
        match value {
            1 => Self::NotEqual,
            2 => Self::LessThanOrEqual,
            3 => Self::GreaterThanOrEqual,
            4 => Self::LessThan,
            5 => Self::GreaterThan,
            _ => Self::Equal,
        }
    }

    fn compare(self, input_value: f32, value: f32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            Self::LessThanOrEqual => input_value <= value,
            Self::GreaterThanOrEqual => input_value >= value,
            Self::LessThan => input_value < value,
            Self::GreaterThan => input_value > value,
        }
    }

    fn compare_bool(self, input_value: bool, value: bool) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    fn compare_u32_equal_only(self, input_value: u32, value: u32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    fn compare_bytes_equal_only(self, input_value: &[u8], value: &[u8]) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    fn compare_u64_equal_only(self, input_value: u64, value: u64) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }
}
