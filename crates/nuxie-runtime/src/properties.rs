use nuxie_binary::{FieldValue, RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{
    FieldKind, StoredFieldInitializer, definition_by_name, property_by_key_in_hierarchy,
};
use std::sync::OnceLock;

use crate::components::TransformProperty;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeArtboardDimensions {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) clip: bool,
}

impl RuntimeArtboardDimensions {
    pub(crate) fn from_object(object: Option<&RuntimeObject>) -> Self {
        let width = object
            .and_then(|object| object.double_property("width"))
            .unwrap_or(0.0);
        let height = object
            .and_then(|object| object.double_property("height"))
            .unwrap_or(0.0);
        let origin_x = object
            .and_then(|object| object.double_property("originX"))
            .unwrap_or(0.0);
        let origin_y = object
            .and_then(|object| object.double_property("originY"))
            .unwrap_or(0.0);
        let clip = object
            .and_then(|object| object.bool_property("clip"))
            .unwrap_or(true);
        Self {
            width,
            height,
            origin_x,
            origin_y,
            clip,
        }
    }
}

pub(crate) fn artboard_index_for_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Option<usize> {
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == graph.global_id)
}

fn runtime_object_property_value_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<&FieldValue> {
    object
        .properties
        .iter()
        .rev()
        .find(|property| property.key == property_key)
        .map(|property| &property.value)
}

pub(crate) fn runtime_object_explicit_double_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<f32> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_double)
}

pub(crate) fn runtime_object_explicit_uint_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u64> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_uint)
}

pub(crate) fn runtime_object_explicit_int_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<i32> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_int)
}

pub(crate) fn runtime_object_explicit_bool_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<bool> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_bool)
}

fn runtime_object_stored_field_initializer_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<StoredFieldInitializer> {
    let (_, property) = property_by_key_in_hierarchy(object.type_key, property_key)?;
    if object.type_name == "Artboard" && property.name == "clip" {
        return Some(StoredFieldInitializer::Bool(true));
    }
    (*property).stored_field_initializer()
}

pub(crate) fn runtime_object_double_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<f32> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_double();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Double(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_uint_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u64> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_uint();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Uint(value) => Some(u64::from(value)),
        _ => None,
    }
}

pub(crate) fn runtime_object_int_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<i32> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_int();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Int(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_bool_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<bool> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_bool();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Bool(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_color_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u32> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_color();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Color(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_string_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<Vec<u8>> {
    runtime_object_string_property_bytes_by_key(object, property_key).map(|value| value.to_vec())
}

fn runtime_object_string_property_bytes_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<&[u8]> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return match value {
            FieldValue::String(value) => Some(value.as_bytes()),
            FieldValue::Bytes(value) => Some(value.as_bytes()),
            _ => None,
        };
    }

    let (_, property) = property_by_key_in_hierarchy(object.type_key, property_key)?;
    match (*property).stored_field_initializer()? {
        StoredFieldInitializer::String(value) if property.runtime_type == FieldKind::String => {
            Some(value.as_bytes())
        }
        _ => None,
    }
}

pub(crate) fn transform_property_for_key(property_key: u16) -> Option<TransformProperty> {
    [
        TransformProperty::X.property_key_for_type("Node"),
        TransformProperty::Y.property_key_for_type("Node"),
        TransformProperty::X.property_key_for_type("RootBone"),
        TransformProperty::Y.property_key_for_type("RootBone"),
        TransformProperty::Rotation.property_key_for_type("TransformComponent"),
        TransformProperty::ScaleX.property_key_for_type("TransformComponent"),
        TransformProperty::ScaleY.property_key_for_type("TransformComponent"),
        TransformProperty::Opacity.property_key_for_type("TransformComponent"),
        TransformProperty::Opacity.property_key_for_type("Artboard"),
    ]
    .into_iter()
    .zip([
        TransformProperty::X,
        TransformProperty::Y,
        TransformProperty::X,
        TransformProperty::Y,
        TransformProperty::Rotation,
        TransformProperty::ScaleX,
        TransformProperty::ScaleY,
        TransformProperty::Opacity,
        TransformProperty::Opacity,
    ])
    .find_map(|(key, property)| (key == Some(property_key)).then_some(property))
}

pub(crate) fn solid_color_value_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "SolidColor", "colorValue")
}

pub(crate) fn shape_paint_is_visible_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "ShapePaint", "isVisible")
}

pub(crate) fn solo_active_component_id_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Solo", "activeComponentId")
}

pub(crate) fn layout_component_style_display_value_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "LayoutComponentStyle", "displayValue")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeLayoutComputedProperty {
    LocalX,
    LocalY,
    WorldX,
    WorldY,
    RootX,
    RootY,
    Width,
    Height,
}

pub(crate) fn layout_computed_property_for_key(
    property_key: u16,
) -> Option<RuntimeLayoutComputedProperty> {
    [
        ("computedLocalX", RuntimeLayoutComputedProperty::LocalX),
        ("computedLocalY", RuntimeLayoutComputedProperty::LocalY),
        ("computedWorldX", RuntimeLayoutComputedProperty::WorldX),
        ("computedWorldY", RuntimeLayoutComputedProperty::WorldY),
        ("computedRootX", RuntimeLayoutComputedProperty::RootX),
        ("computedRootY", RuntimeLayoutComputedProperty::RootY),
        ("computedWidth", RuntimeLayoutComputedProperty::Width),
        ("computedHeight", RuntimeLayoutComputedProperty::Height),
    ]
    .into_iter()
    .find_map(|(property_name, property)| {
        (property_key_for_name("Node", property_name) == Some(property_key)).then_some(property)
    })
}

pub(crate) fn property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    let definition = definition_by_name(type_name)?;
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return Some(property.key.int);
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)?;
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return Some(property.key.int);
        }
    }

    None
}

pub(crate) fn cached_property_key_for_name(
    slot: &'static OnceLock<Option<u16>>,
    type_name: &'static str,
    property_name: &'static str,
) -> Option<u16> {
    *slot.get_or_init(|| property_key_for_name(type_name, property_name))
}

pub(crate) fn mix_value(current: f32, value: f32, mix: f32) -> f32 {
    if mix == 1.0 {
        value
    } else {
        current * (1.0 - mix) + value * mix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_bone_xy_keys_are_transform_properties() {
        let x = property_key_for_name("RootBone", "x").expect("RootBone.x key");
        let y = property_key_for_name("RootBone", "y").expect("RootBone.y key");

        assert_eq!(transform_property_for_key(x), Some(TransformProperty::X));
        assert_eq!(transform_property_for_key(y), Some(TransformProperty::Y));
    }

    /// Every property name the runtime resolves from a string literal --
    /// direct `property_key_for_name` calls, the cached `OnceLock` helpers,
    /// and the `cached_runtime_property_key!` /
    /// `cached_runtime_data_bind_property_key!` match tables. A rename in the
    /// generated schema makes resolution return `None`, which the call sites
    /// swallow silently (dirt routes stop firing, binds stop applying), so
    /// this table pins each pair to `Some`.
    ///
    /// Regenerate after adding call sites by grepping the crate for the
    /// resolver names above and collecting their literal
    /// `("Type", "property")` arguments.
    const STRINGLY_KEYED_PROPERTY_NAMES: &[(&str, &str)] = &[
        ("Artboard", "clip"),
        ("Artboard", "defaultStateMachineId"),
        ("Artboard", "opacity"),
        ("Artboard", "originX"),
        ("Artboard", "originY"),
        ("ArtboardComponentList", "listSource"),
        ("ArtboardComponentListOverride", "artboardId"),
        ("ArtboardComponentListOverride", "instanceHeight"),
        ("ArtboardComponentListOverride", "instanceHeightScaleType"),
        ("ArtboardComponentListOverride", "instanceHeightUnitsValue"),
        ("ArtboardComponentListOverride", "instanceWidth"),
        ("ArtboardComponentListOverride", "instanceWidthScaleType"),
        ("ArtboardComponentListOverride", "instanceWidthUnitsValue"),
        ("AudioEvent", "parentId"),
        ("Axis", "offset"),
        ("BindablePropertyArtboard", "propertyValue"),
        ("BindablePropertyAsset", "propertyValue"),
        ("BindablePropertyBoolean", "propertyValue"),
        ("BindablePropertyColor", "propertyValue"),
        ("BindablePropertyEnum", "propertyValue"),
        ("BindablePropertyInteger", "propertyValue"),
        ("BindablePropertyList", "propertyValue"),
        ("BindablePropertyNumber", "propertyValue"),
        ("BindablePropertyString", "propertyValue"),
        ("BindablePropertyTrigger", "propertyValue"),
        ("BindablePropertyViewModel", "propertyValue"),
        ("Bone", "length"),
        ("ClippingShape", "fillRule"),
        ("ClippingShape", "isVisible"),
        ("Component", "parentId"),
        ("ComponentOrigin", "originX"),
        ("ComponentOrigin", "originY"),
        ("Constraint", "strength"),
        ("CubicAsymmetricVertex", "distance"),
        ("CubicAsymmetricVertex", "inDistance"),
        ("CubicAsymmetricVertex", "inRotation"),
        ("CubicAsymmetricVertex", "outDistance"),
        ("CubicAsymmetricVertex", "outRotation"),
        ("CubicAsymmetricVertex", "radius"),
        ("CubicAsymmetricVertex", "rotation"),
        ("CubicAsymmetricVertex", "x"),
        ("CubicAsymmetricVertex", "y"),
        ("CubicDetachedVertex", "distance"),
        ("CubicDetachedVertex", "inDistance"),
        ("CubicDetachedVertex", "inRotation"),
        ("CubicDetachedVertex", "outDistance"),
        ("CubicDetachedVertex", "outRotation"),
        ("CubicDetachedVertex", "radius"),
        ("CubicDetachedVertex", "rotation"),
        ("CubicDetachedVertex", "x"),
        ("CubicDetachedVertex", "y"),
        ("CubicMirroredVertex", "distance"),
        ("CubicMirroredVertex", "inDistance"),
        ("CubicMirroredVertex", "inRotation"),
        ("CubicMirroredVertex", "outDistance"),
        ("CubicMirroredVertex", "outRotation"),
        ("CubicMirroredVertex", "radius"),
        ("CubicMirroredVertex", "rotation"),
        ("CubicMirroredVertex", "x"),
        ("CubicMirroredVertex", "y"),
        ("CubicWeight", "inIndices"),
        ("CubicWeight", "inValues"),
        ("CubicWeight", "outIndices"),
        ("CubicWeight", "outValues"),
        ("CustomPropertyBoolean", "propertyValue"),
        ("CustomPropertyColor", "propertyValue"),
        ("CustomPropertyEnum", "propertyValue"),
        ("CustomPropertyNumber", "propertyValue"),
        ("CustomPropertyString", "name"),
        ("CustomPropertyString", "propertyValue"),
        ("CustomPropertyTrigger", "fire"),
        ("CustomPropertyTrigger", "propertyValue"),
        ("Dash", "length"),
        ("Dash", "lengthIsPercentage"),
        ("DashPath", "offset"),
        ("DashPath", "offsetIsPercentage"),
        ("DataConverterInterpolator", "duration"),
        ("DataConverterNumberToList", "viewModelId"),
        ("DataConverterOperationValue", "operationValue"),
        ("DataConverterRangeMapper", "maxInput"),
        ("DataConverterRangeMapper", "maxOutput"),
        ("DataConverterRangeMapper", "minInput"),
        ("DataConverterRangeMapper", "minOutput"),
        ("DataConverterStringPad", "length"),
        ("DataConverterStringPad", "padType"),
        ("DataConverterStringPad", "text"),
        ("DataConverterStringTrim", "trimType"),
        ("DataConverterToString", "colorFormat"),
        ("DataConverterToString", "decimals"),
        ("DrawRules", "drawTargetId"),
        ("DrawTarget", "drawableId"),
        ("DrawTarget", "placementValue"),
        ("Drawable", "blendModeValue"),
        ("Drawable", "drawableFlags"),
        ("Ellipse", "height"),
        ("Ellipse", "isClosed"),
        ("Ellipse", "isHole"),
        ("Ellipse", "originX"),
        ("Ellipse", "originY"),
        ("Ellipse", "pathFlags"),
        ("Ellipse", "width"),
        ("Event", "name"),
        ("Feather", "inner"),
        ("Feather", "offsetX"),
        ("Feather", "offsetY"),
        ("Feather", "spaceValue"),
        ("Feather", "strength"),
        ("FileAssetContents", "bytes"),
        ("Fill", "fillRule"),
        ("FocusData", "edgeBehaviorValue"),
        ("FocusData", "focusFlags"),
        ("FocusData", "name"),
        ("FollowPathConstraint", "distance"),
        ("FollowPathConstraint", "offset"),
        ("FollowPathConstraint", "orient"),
        ("FormulaTokenValue", "operationValue"),
        ("GradientStop", "colorValue"),
        ("GradientStop", "position"),
        ("GridTrack", "collection"),
        ("Image", "alignmentX"),
        ("Image", "alignmentY"),
        ("Image", "assetId"),
        ("Image", "fit"),
        ("Image", "originX"),
        ("Image", "originY"),
        ("Joystick", "joystickFlags"),
        ("Joystick", "x"),
        ("Joystick", "y"),
        ("LayoutComponent", "clip"),
        ("LayoutComponent", "drawableFlags"),
        ("LayoutComponent", "fractionalHeight"),
        ("LayoutComponent", "fractionalWidth"),
        ("LayoutComponent", "height"),
        ("LayoutComponent", "styleId"),
        ("LayoutComponent", "width"),
        ("LayoutComponentStyle", "animationStyleType"),
        ("LayoutComponentStyle", "aspectRatio"),
        ("LayoutComponentStyle", "borderBottom"),
        ("LayoutComponentStyle", "borderBottomUnitsValue"),
        ("LayoutComponentStyle", "borderLeft"),
        ("LayoutComponentStyle", "borderLeftUnitsValue"),
        ("LayoutComponentStyle", "borderRight"),
        ("LayoutComponentStyle", "borderRightUnitsValue"),
        ("LayoutComponentStyle", "borderTop"),
        ("LayoutComponentStyle", "borderTopUnitsValue"),
        ("LayoutComponentStyle", "cornerRadiusBL"),
        ("LayoutComponentStyle", "cornerRadiusBR"),
        ("LayoutComponentStyle", "cornerRadiusTL"),
        ("LayoutComponentStyle", "cornerRadiusTR"),
        ("LayoutComponentStyle", "directionValue"),
        ("LayoutComponentStyle", "displayValue"),
        ("LayoutComponentStyle", "flexBasis"),
        ("LayoutComponentStyle", "flexBasisUnitsValue"),
        ("LayoutComponentStyle", "flexDirectionValue"),
        ("LayoutComponentStyle", "flexWrapValue"),
        ("LayoutComponentStyle", "gapHorizontal"),
        ("LayoutComponentStyle", "gapHorizontalUnitsValue"),
        ("LayoutComponentStyle", "gapVertical"),
        ("LayoutComponentStyle", "gapVerticalUnitsValue"),
        ("LayoutComponentStyle", "heightUnitsValue"),
        ("LayoutComponentStyle", "interpolationTime"),
        ("LayoutComponentStyle", "interpolationType"),
        ("LayoutComponentStyle", "interpolatorId"),
        ("LayoutComponentStyle", "intrinsicallySizedValue"),
        ("LayoutComponentStyle", "justifyItemsValue"),
        ("LayoutComponentStyle", "justifySelfValue"),
        ("LayoutComponentStyle", "layoutAlignmentType"),
        ("LayoutComponentStyle", "layoutHeightScaleType"),
        ("LayoutComponentStyle", "layoutTypeValue"),
        ("LayoutComponentStyle", "layoutWidthScaleType"),
        ("LayoutComponentStyle", "linkCornerRadius"),
        ("LayoutComponentStyle", "marginBottom"),
        ("LayoutComponentStyle", "marginBottomUnitsValue"),
        ("LayoutComponentStyle", "marginLeft"),
        ("LayoutComponentStyle", "marginLeftUnitsValue"),
        ("LayoutComponentStyle", "marginRight"),
        ("LayoutComponentStyle", "marginRightUnitsValue"),
        ("LayoutComponentStyle", "marginTop"),
        ("LayoutComponentStyle", "marginTopUnitsValue"),
        ("LayoutComponentStyle", "maxHeight"),
        ("LayoutComponentStyle", "maxHeightUnitsValue"),
        ("LayoutComponentStyle", "maxWidth"),
        ("LayoutComponentStyle", "maxWidthUnitsValue"),
        ("LayoutComponentStyle", "minHeight"),
        ("LayoutComponentStyle", "minHeightUnitsValue"),
        ("LayoutComponentStyle", "minWidth"),
        ("LayoutComponentStyle", "minWidthUnitsValue"),
        ("LayoutComponentStyle", "paddingBottom"),
        ("LayoutComponentStyle", "paddingBottomUnitsValue"),
        ("LayoutComponentStyle", "paddingLeft"),
        ("LayoutComponentStyle", "paddingLeftUnitsValue"),
        ("LayoutComponentStyle", "paddingRight"),
        ("LayoutComponentStyle", "paddingRightUnitsValue"),
        ("LayoutComponentStyle", "paddingTop"),
        ("LayoutComponentStyle", "paddingTopUnitsValue"),
        ("LayoutComponentStyle", "positionBottom"),
        ("LayoutComponentStyle", "positionBottomUnitsValue"),
        ("LayoutComponentStyle", "positionLeft"),
        ("LayoutComponentStyle", "positionLeftUnitsValue"),
        ("LayoutComponentStyle", "positionRight"),
        ("LayoutComponentStyle", "positionRightUnitsValue"),
        ("LayoutComponentStyle", "positionTop"),
        ("LayoutComponentStyle", "positionTopUnitsValue"),
        ("LayoutComponentStyle", "positionTypeValue"),
        ("LayoutComponentStyle", "widthUnitsValue"),
        ("LinearGradient", "endX"),
        ("LinearGradient", "endY"),
        ("LinearGradient", "opacity"),
        ("LinearGradient", "startX"),
        ("LinearGradient", "startY"),
        ("ListFollowPathConstraint", "distanceEnd"),
        ("ListFollowPathConstraint", "distanceOffset"),
        ("ListenerBoolChange", "inputId"),
        ("ListenerBoolChange", "value"),
        ("ListenerFireEvent", "eventId"),
        ("ListenerFireEvent", "flags"),
        ("MeshVertex", "u"),
        ("MeshVertex", "v"),
        ("NSlicedNode", "height"),
        ("NSlicedNode", "width"),
        ("NestedArtboard", "artboardId"),
        ("NestedArtboard", "isPaused"),
        ("NestedArtboard", "isStateful"),
        ("NestedArtboard", "quantize"),
        ("NestedArtboard", "speed"),
        ("NestedArtboardLayout", "instanceHeight"),
        ("NestedArtboardLayout", "instanceHeightScaleType"),
        ("NestedArtboardLayout", "instanceHeightUnitsValue"),
        ("NestedArtboardLayout", "instanceWidth"),
        ("NestedArtboardLayout", "instanceWidthScaleType"),
        ("NestedArtboardLayout", "instanceWidthUnitsValue"),
        ("NestedArtboardLeaf", "alignmentX"),
        ("NestedArtboardLeaf", "alignmentY"),
        ("NestedArtboardLeaf", "fit"),
        ("NestedBool", "nestedValue"),
        ("NestedInput", "inputId"),
        ("NestedLinearAnimation", "mix"),
        ("NestedNumber", "nestedValue"),
        ("NestedRemapAnimation", "time"),
        ("NestedSimpleAnimation", "isPlaying"),
        ("NestedSimpleAnimation", "speed"),
        ("NestedTrigger", "fire"),
        ("Node", "opacity"),
        ("Node", "scaleX"),
        ("Node", "scaleY"),
        ("Node", "x"),
        ("Node", "y"),
        ("OpenUrlEvent", "targetValue"),
        ("OpenUrlEvent", "url"),
        ("ParametricPath", "height"),
        ("ParametricPath", "width"),
        ("Path", "isClosed"),
        ("Path", "isHole"),
        ("Path", "pathFlags"),
        ("PointsPath", "isClosed"),
        ("PointsPath", "isHole"),
        ("PointsPath", "pathFlags"),
        ("Polygon", "cornerRadius"),
        ("Polygon", "height"),
        ("Polygon", "isClosed"),
        ("Polygon", "isHole"),
        ("Polygon", "originX"),
        ("Polygon", "originY"),
        ("Polygon", "pathFlags"),
        ("Polygon", "points"),
        ("Polygon", "width"),
        ("RadialGradient", "endX"),
        ("RadialGradient", "endY"),
        ("RadialGradient", "opacity"),
        ("RadialGradient", "startX"),
        ("RadialGradient", "startY"),
        ("Rectangle", "cornerRadiusBL"),
        ("Rectangle", "cornerRadiusBR"),
        ("Rectangle", "cornerRadiusTL"),
        ("Rectangle", "cornerRadiusTR"),
        ("Rectangle", "height"),
        ("Rectangle", "isClosed"),
        ("Rectangle", "isHole"),
        ("Rectangle", "linkCornerRadius"),
        ("Rectangle", "originX"),
        ("Rectangle", "originY"),
        ("Rectangle", "pathFlags"),
        ("Rectangle", "width"),
        ("RootBone", "x"),
        ("RootBone", "y"),
        ("ScriptInputArtboard", "artboardId"),
        ("ScriptInputBoolean", "propertyValue"),
        ("ScriptInputColor", "propertyValue"),
        ("ScriptInputNumber", "propertyValue"),
        ("ScriptInputString", "propertyValue"),
        ("ScriptInputTrigger", "propertyValue"),
        ("ScrollBarConstraint", "scrollConstraintId"),
        ("ScrollConstraint", "computedContentHeight"),
        ("ScrollConstraint", "computedContentWidth"),
        ("ScrollConstraint", "physicsId"),
        ("ScrollConstraint", "scrollIndex"),
        ("ScrollConstraint", "scrollOffsetX"),
        ("ScrollConstraint", "scrollOffsetY"),
        ("ScrollConstraint", "scrollPercentX"),
        ("ScrollConstraint", "scrollPercentY"),
        ("ScrollConstraint", "virtualize"),
        ("Shape", "length"),
        ("Shape", "opacity"),
        ("Shape", "scaleX"),
        ("ShapePaint", "blendModeValue"),
        ("ShapePaint", "isVisible"),
        ("SolidColor", "colorValue"),
        ("Solo", "activeComponentId"),
        ("Star", "cornerRadius"),
        ("Star", "height"),
        ("Star", "innerRadius"),
        ("Star", "isClosed"),
        ("Star", "isHole"),
        ("Star", "originX"),
        ("Star", "originY"),
        ("Star", "pathFlags"),
        ("Star", "points"),
        ("Star", "width"),
        ("StateMachineListener", "targetId"),
        ("StateMachineListenerSingle", "targetId"),
        ("StateTransition", "duration"),
        ("StraightVertex", "distance"),
        ("StraightVertex", "inDistance"),
        ("StraightVertex", "inRotation"),
        ("StraightVertex", "outDistance"),
        ("StraightVertex", "outRotation"),
        ("StraightVertex", "radius"),
        ("StraightVertex", "rotation"),
        ("StraightVertex", "x"),
        ("StraightVertex", "y"),
        ("Stroke", "cap"),
        ("Stroke", "join"),
        ("Stroke", "thickness"),
        ("Stroke", "transformAffectsStroke"),
        ("TargetedConstraint", "targetId"),
        ("Tendon", "boneId"),
        ("Text", "height"),
        ("Text", "originValue"),
        ("Text", "originX"),
        ("Text", "originY"),
        ("Text", "overflowValue"),
        ("Text", "paragraphSpacing"),
        ("Text", "sizingValue"),
        ("Text", "textRunListSource"),
        ("Text", "width"),
        ("TextFollowPathModifier", "targetId"),
        ("TextInput", "multiline"),
        ("TextInput", "selectionRadius"),
        ("TextInput", "text"),
        ("TextModifierGroup", "modifierFlags"),
        ("TextModifierRange", "typeValue"),
        ("TextStyle", "fontAssetId"),
        ("TextStyle", "fontSize"),
        ("TextStyle", "letterSpacing"),
        ("TextStyle", "lineHeight"),
        ("TextStyleAxis", "axisValue"),
        ("TextStyleAxis", "tag"),
        ("TextStyleFeature", "featureValue"),
        ("TextStyleFeature", "tag"),
        ("TextValueRun", "styleId"),
        ("TextValueRun", "text"),
        ("TextVariationModifier", "axisTag"),
        ("TextVariationModifier", "axisValue"),
        ("TransformComponent", "opacity"),
        ("TransformComponent", "rotation"),
        ("TransformComponent", "scaleX"),
        ("TransformComponent", "scaleY"),
        ("Triangle", "height"),
        ("Triangle", "isClosed"),
        ("Triangle", "isHole"),
        ("Triangle", "originX"),
        ("Triangle", "originY"),
        ("Triangle", "pathFlags"),
        ("Triangle", "width"),
        ("TrimPath", "end"),
        ("TrimPath", "modeValue"),
        ("TrimPath", "offset"),
        ("TrimPath", "start"),
        ("Vertex", "x"),
        ("Vertex", "y"),
        ("ViewModelInstance", "viewModelId"),
        ("ViewModelInstanceArtboard", "propertyValue"),
        ("ViewModelInstanceAsset", "propertyValue"),
        ("ViewModelInstanceAssetFont", "propertyValue"),
        ("ViewModelInstanceAssetImage", "propertyValue"),
        ("ViewModelInstanceBoolean", "propertyValue"),
        ("ViewModelInstanceColor", "propertyValue"),
        ("ViewModelInstanceEnum", "propertyValue"),
        ("ViewModelInstanceNumber", "propertyValue"),
        ("ViewModelInstanceString", "propertyValue"),
        ("ViewModelInstanceValue", "viewModelPropertyId"),
        ("ViewModelInstanceViewModel", "propertyValue"),
        ("Weight", "indices"),
        ("Weight", "values"),
        ("WorldTransformComponent", "opacity"),
        ("WorldTransformComponent", "parentId"),
    ];

    /// Match arms in `runtime_draw_property_key_for_name` that target
    /// properties the schema does not define for that type (dead arms from a
    /// combinatorial expansion over vertex/path types). They can never fire;
    /// this list ratchets them so a schema advance that makes one real forces
    /// a routing decision instead of silently changing behavior.
    const KNOWN_UNRESOLVED_PROPERTY_NAMES: &[(&str, &str)] = &[
        ("CubicAsymmetricVertex", "distance"),
        ("CubicAsymmetricVertex", "inRotation"),
        ("CubicAsymmetricVertex", "outRotation"),
        ("CubicAsymmetricVertex", "radius"),
        ("CubicDetachedVertex", "distance"),
        ("CubicDetachedVertex", "radius"),
        ("CubicDetachedVertex", "rotation"),
        ("CubicMirroredVertex", "inDistance"),
        ("CubicMirroredVertex", "inRotation"),
        ("CubicMirroredVertex", "outDistance"),
        ("CubicMirroredVertex", "outRotation"),
        ("CubicMirroredVertex", "radius"),
        ("Ellipse", "isClosed"),
        ("Path", "isClosed"),
        ("Polygon", "isClosed"),
        ("Rectangle", "isClosed"),
        ("Star", "isClosed"),
        ("StraightVertex", "distance"),
        ("StraightVertex", "inDistance"),
        ("StraightVertex", "inRotation"),
        ("StraightVertex", "outDistance"),
        ("StraightVertex", "outRotation"),
        ("StraightVertex", "rotation"),
        ("Triangle", "isClosed"),
    ];

    #[test]
    fn stringly_keyed_property_names_resolve() {
        let unresolved: Vec<String> = STRINGLY_KEYED_PROPERTY_NAMES
            .iter()
            .filter(|pair| !KNOWN_UNRESOLVED_PROPERTY_NAMES.contains(pair))
            .filter(|(type_name, property_name)| {
                property_key_for_name(type_name, property_name).is_none()
            })
            .map(|(type_name, property_name)| format!("{type_name}.{property_name}"))
            .collect();
        assert!(
            unresolved.is_empty(),
            "stringly-keyed property names no longer resolve against the schema: {unresolved:?}"
        );
    }

    #[test]
    fn known_unresolved_property_names_stay_unresolved() {
        let resolved: Vec<String> = KNOWN_UNRESOLVED_PROPERTY_NAMES
            .iter()
            .filter(|(type_name, property_name)| {
                property_key_for_name(type_name, property_name).is_some()
            })
            .map(|(type_name, property_name)| format!("{type_name}.{property_name}"))
            .collect();
        assert!(
            resolved.is_empty(),
            "schema now defines {resolved:?}; route them and move them out of \
             KNOWN_UNRESOLVED_PROPERTY_NAMES"
        );
    }

    #[test]
    fn layout_computed_property_names_resolve() {
        for name in [
            "computedLocalX",
            "computedLocalY",
            "computedWorldX",
            "computedWorldY",
            "computedRootX",
            "computedRootY",
            "computedWidth",
            "computedHeight",
        ] {
            let key = property_key_for_name("Node", name)
                .unwrap_or_else(|| panic!("Node.{name} does not resolve"));
            assert!(
                layout_computed_property_for_key(key).is_some(),
                "layout_computed_property_for_key misses Node.{name}"
            );
        }
    }
}
